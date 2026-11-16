//! Random players, drawn in attribute space.
//!
//! The spreads and the positional differences are those of the population the
//! match engine was calibrated with, so that a league of generated major-leaguers
//! plays like the calibrated one.

use crate::attr::{Attr, Attributes, Hidden};
use crate::convert;
use crate::player::{Bats, HitStyle, PitchSkill, Pitching, Player, Position, Role, Side};
use bm_engine::{Club, PitchKind, Rng};
use bm_physics::units::{FT, MPH, RPM};

/// Talent level of a league relative to the major leagues, in standard scores.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Talent {
    /// Added to the batting attributes that decide results at the plate.
    pub hitting: f32,
    /// Added to the pitching attributes.
    pub pitching: f32,
    /// Added to fielding, catching and running attributes.
    pub fielding: f32,
    /// Added to the physical attributes.
    pub physical: f32,
}

impl Talent {
    pub const MAJOR: Talent = Talent { hitting: 0.0, pitching: 0.0, fielding: 0.0, physical: 0.0 };
}

fn z(rng: &mut Rng, mean: f64) -> f32 {
    rng.gauss_clamped(mean, 1.0, mean - 3.5, mean + 3.5) as f32
}

fn hidden(rng: &mut Rng) -> Hidden {
    let mut h = |sd: f64| rng.gauss_clamped(0.0, sd, -3.0, 3.0) as f32;
    Hidden {
        consistency: h(1.0),
        big_games: h(1.0),
        injury_resistance: h(1.0),
        versatility: h(1.0),
        adaptability: h(1.0),
        ambition: h(1.0),
        loyalty: h(1.0),
        pressure: h(1.0),
        professionalism: h(1.0),
        temperament: h(1.0),
        platoon: h(1.0),
        growth: h(1.0),
        potential: 0.0,
    }
}

fn hit_style(rng: &mut Rng) -> HitStyle {
    HitStyle {
        attack_angle: rng.gauss_clamped(9.5, 3.0, -2.0, 21.0).to_radians() as f32,
        attack_sd: 5.5f64.to_radians() as f32,
        barrel_aim: rng.gauss(0.0154, 0.003) as f32,
        timing_bias: rng.gauss(0.0002, 0.0015) as f32,
        bat_speed_sd: (2.8 * MPH) as f32,
    }
}

/// Mental and physical attributes common to everyone.
fn common(rng: &mut Rng, a: &mut Attributes, t: &Talent) {
    for attr in
        [Attr::Composure, Attr::Concentration, Attr::BaseballIq, Attr::Anticipation, Attr::Determination, Attr::Leadership, Attr::Teamwork, Attr::Aggression]
    {
        a[attr] = z(rng, 0.0);
    }
    for attr in [Attr::Strength, Attr::Stamina, Attr::Durability, Attr::Agility] {
        a[attr] = z(rng, t.physical as f64);
    }
}

/// (speed, arm strength) of the regulars at each position relative to the average
/// position player, in standard scores.
fn position_tools(pos: Option<Position>) -> (f64, f64) {
    match pos {
        Some(Position::Catcher) => (-1.14, 0.0),
        Some(Position::First) => (-0.71, -1.9),
        Some(Position::Second) => (0.14, -1.25),
        Some(Position::Third) => (-0.14, 0.0),
        Some(Position::Short) => (0.36, 0.16),
        Some(Position::Left) => (0.0, 0.31),
        Some(Position::Center) => (0.79, 0.94),
        Some(Position::Right) => (0.21, 1.56),
        _ => (0.0, -0.6),
    }
}

/// Positions a player picks up from his own.
fn familiarity(rng: &mut Rng, primary: Option<Position>) -> [u8; 9] {
    use Position::*;
    let mut f = [1u8; 9];
    let related: &[(Position, f64)] = match primary {
        Some(Catcher) => &[(First, 9.0)],
        Some(First) => &[(Left, 8.0), (Third, 6.0)],
        Some(Second) => &[(Short, 13.0), (Third, 12.0), (First, 9.0)],
        Some(Third) => &[(First, 13.0), (Second, 11.0), (Short, 9.0)],
        Some(Short) => &[(Second, 16.0), (Third, 15.0), (First, 9.0)],
        Some(Left) => &[(Right, 15.0), (First, 10.0), (Center, 9.0)],
        Some(Center) => &[(Left, 17.0), (Right, 17.0)],
        Some(Right) => &[(Left, 16.0), (Center, 11.0), (First, 8.0)],
        Some(Pitcher) => &[],
        None => &[(First, 12.0), (Left, 10.0)],
    };
    for &(pos, level) in related {
        f[pos.index()] = rng.gauss_clamped(level, 3.0, 1.0, 19.0).round() as u8;
    }
    if let Some(pos) = primary {
        f[pos.index()] = 20;
    }
    f
}

/// A position player. `pos` = `None` makes a designated hitter.
pub fn position_player(rng: &mut Rng, id: u32, pos: Option<Position>, t: &Talent) -> Player {
    let mut a = Attributes::default();
    common(rng, &mut a, t);
    let hit = t.hitting as f64;
    for attr in [Attr::Contact, Attr::Timing, Attr::Power, Attr::Discipline, Attr::Recognition, Attr::BatControl] {
        a[attr] = z(rng, hit);
    }
    a[Attr::Bunt] = z(rng, 0.0);
    let (speed, arm) = position_tools(pos);
    let run = t.physical as f64;
    a[Attr::Speed] = z(rng, speed + run);
    // Quick players accelerate well and steal: the three go together.
    let sp = a[Attr::Speed] as f64;
    a[Attr::Acceleration] = (0.81 * sp + 0.58 * rng.gauss(0.0, 1.0)) as f32;
    a[Attr::Stealing] = (0.92 * sp + 0.39 * rng.gauss(0.0, 1.0) + t.fielding as f64) as f32;
    a[Attr::Baserunning] = z(rng, t.fielding as f64);
    let fld = t.fielding as f64;
    for attr in [Attr::Hands, Attr::Range, Attr::Reads, Attr::ArmAccuracy, Attr::Footwork] {
        a[attr] = z(rng, fld);
    }
    a[Attr::ArmStrength] = rng.gauss_clamped(arm + run, 0.78, arm - 3.0, arm + 3.0) as f32;
    let catching = if pos == Some(Position::Catcher) { fld } else { fld - 3.0 };
    for attr in [Attr::Framing, Attr::Blocking, Attr::ThrowingOut, Attr::GameCalling] {
        a[attr] = z(rng, catching);
    }
    let side = rng.f64();
    let bats = if side < 0.56 {
        Bats::Right
    } else if side < 0.87 {
        Bats::Left
    } else {
        Bats::Switch
    };
    // Catchers and the infielders other than the first baseman throw right-handed.
    let throws = match pos {
        Some(Position::Catcher | Position::Second | Position::Third | Position::Short) => Side::Right,
        _ if bats == Bats::Left && rng.chance(0.6) => Side::Left,
        _ => Side::Right,
    };
    let height = rng.gauss_clamped(1.855, 0.055, 1.68, 2.03) as f32;
    Player {
        id,
        height,
        weight: (rng.gauss_clamped(27.0, 1.8, 22.0, 33.0) as f32) * height * height,
        bats,
        throws,
        position: pos.unwrap_or(Position::First),
        positions: familiarity(rng, pos),
        attrs: a,
        hidden: hidden(rng),
        hit: hit_style(rng),
        pitching: None,
    }
}

/// (kind, probability of having it, usage weight when present): the repertoires of
/// major-league pitchers in the 2025 Statcast sample.
const ARSENAL: [(PitchKind, f64, f64); 8] = [
    (PitchKind::FourSeam, 0.75, 38.0),
    (PitchKind::Sinker, 0.55, 28.0),
    (PitchKind::Cutter, 0.30, 22.0),
    (PitchKind::Slider, 0.55, 28.0),
    (PitchKind::Sweeper, 0.30, 29.0),
    (PitchKind::Curve, 0.40, 26.0),
    (PitchKind::Changeup, 0.55, 19.5),
    (PitchKind::Splitter, 0.14, 20.0),
];

fn pitch(rng: &mut Rng, kind: PitchKind, weight: f64, t: &Talent) -> PitchSkill {
    PitchSkill {
        kind,
        grade: z(rng, t.pitching as f64),
        speed: rng.gauss(0.0, 1.0 * MPH) as f32,
        spin: rng.gauss(0.0, 100.0 * RPM) as f32,
        active: rng.gauss(0.0, 0.05) as f32,
        tilt: rng.gauss(0.0, 8.0f64.to_radians()) as f32,
        usage: (weight * rng.gauss(0.0, 0.30).exp()) as f32,
    }
}

pub fn pitcher(rng: &mut Rng, id: u32, role: Role, t: &Talent) -> Player {
    let mut a = Attributes::default();
    common(rng, &mut a, t);
    let pit = t.pitching as f64;
    // Relievers are selected for velocity; starters for the stamina to go deep.
    a[Attr::Velocity] = z(rng, pit + if role == Role::Reliever { 0.27 } else { 0.0 });
    for attr in [Attr::Stuff, Attr::Control, Attr::Command, Attr::Deception] {
        a[attr] = z(rng, pit);
    }
    a[Attr::HoldRunners] = z(rng, 0.0);
    a[Attr::Stamina] = match role {
        Role::Starter => rng.gauss_clamped(2.0, 0.44, 0.6, 3.3) as f32,
        Role::Reliever => rng.gauss_clamped(-2.0, 0.31, -2.9, -0.9) as f32,
    };
    // At the plate and on the bases a pitcher is a non-specialist.
    a[Attr::Power] = z(rng, -2.0);
    a[Attr::Contact] = z(rng, -3.1);
    a[Attr::Timing] = z(rng, -3.0);
    a[Attr::Recognition] = z(rng, -1.0);
    for attr in [Attr::Discipline, Attr::BatControl, Attr::Bunt, Attr::Baserunning] {
        a[attr] = z(rng, -1.0);
    }
    a[Attr::Speed] = z(rng, -0.5);
    let sp = a[Attr::Speed] as f64;
    a[Attr::Acceleration] = (0.81 * sp + 0.58 * rng.gauss(0.0, 1.0)) as f32;
    a[Attr::Stealing] = (0.92 * sp + 0.39 * rng.gauss(0.0, 1.0) - 1.0) as f32;
    for attr in [Attr::Hands, Attr::Range, Attr::Reads, Attr::ArmAccuracy, Attr::Footwork] {
        a[attr] = z(rng, t.fielding as f64);
    }
    a[Attr::ArmStrength] = z(rng, 1.0);
    for attr in [Attr::Framing, Attr::Blocking, Attr::ThrowingOut, Attr::GameCalling] {
        a[attr] = z(rng, -3.0);
    }

    let mut pitches: Vec<PitchSkill> = Vec::with_capacity(6);
    for &(kind, prob, weight) in &ARSENAL {
        if pitches.len() < 6 && rng.chance(prob) {
            pitches.push(pitch(rng, kind, weight, t));
        }
    }
    // Everybody has a fastball and at least two other offerings.
    if !pitches.iter().any(|s| matches!(s.kind, PitchKind::FourSeam | PitchKind::Sinker)) {
        pitches.insert(0, pitch(rng, PitchKind::FourSeam, ARSENAL[0].2, t));
    }
    let mut guard = 0;
    while pitches.len() < 3 && guard < 50 {
        guard += 1;
        let (kind, _, weight) = ARSENAL[2 + rng.below(6)];
        if !pitches.iter().any(|s| s.kind == kind) {
            pitches.push(pitch(rng, kind, weight, t));
        }
    }

    let throws = if rng.chance(0.73) { Side::Right } else { Side::Left };
    let height = rng.gauss_clamped(1.88, 0.055, 1.70, 2.08) as f32;
    Player {
        id,
        height,
        weight: (rng.gauss_clamped(27.0, 1.8, 22.0, 33.0) as f32) * height * height,
        bats: if throws == Side::Right { Bats::Right } else { Bats::Left },
        throws,
        position: Position::Pitcher,
        positions: familiarity(rng, Some(Position::Pitcher)),
        attrs: a,
        hidden: hidden(rng),
        hit: hit_style(rng),
        pitching: Some(Pitching {
            role,
            arm_angle: rng.gauss_clamped(40.0, 11.6, -15.0, 72.0).to_radians() as f32,
            extension: (rng.gauss_clamped(6.45, 0.3, 5.5, 7.4) * FT) as f32,
            speed_sd: (rng.gauss_clamped(1.05, 0.12, 0.7, 1.5) * MPH) as f32,
            spin_sd: (rng.gauss_clamped(85.0, 10.0, 55.0, 130.0) * RPM) as f32,
            tilt_sd: rng.gauss_clamped(4.5, 0.6, 2.5, 7.0).to_radians() as f32,
            pitches,
        }),
    }
}

/// A club's players for one game day: the regulars at the eight positions and the
/// designated hitter, four reserves, five starters and eight relievers.
#[derive(Clone, Debug)]
pub struct Squad {
    pub hitters: Vec<(Player, Option<Position>)>,
    pub bench: Vec<(Player, Option<Position>)>,
    pub starters: Vec<Player>,
    pub relievers: Vec<Player>,
}

impl Squad {
    pub fn random(rng: &mut Rng, id_base: u32, t: &Talent) -> Squad {
        use Position::*;
        let mut hitters: Vec<(Player, Option<Position>)> = [Catcher, First, Second, Third, Short, Left, Center, Right]
            .into_iter()
            .enumerate()
            .map(|(i, pos)| (position_player(rng, id_base + i as u32, Some(pos), t), Some(pos)))
            .collect();
        hitters.push((position_player(rng, id_base + 8, None, t), None));
        let starters = (0..5).map(|i| pitcher(rng, id_base + 20 + i, Role::Starter, t)).collect();
        let relievers = (0..8).map(|i| pitcher(rng, id_base + 30 + i, Role::Reliever, t)).collect();
        // Reserves are a little short of the regulars with the bat.
        let reserve = Talent { hitting: t.hitting - 0.48, ..*t };
        let bench = [Catcher, Short, Center, Third]
            .into_iter()
            .enumerate()
            .map(|(i, pos)| (position_player(rng, id_base + 50 + i as u32, Some(pos), &reserve), Some(pos)))
            .collect();
        Squad { hitters, bench, starters, relievers }
    }

    /// The squad as the match engine's club.
    pub fn to_engine(&self) -> Club {
        let field = |v: &[(Player, Option<Position>)]| v.iter().map(|(p, pos)| convert::engine_player(p, *pos, false)).collect();
        Club {
            hitters: field(&self.hitters),
            starters: self.starters.iter().map(|p| convert::engine_player(p, Some(Position::Pitcher), false)).collect(),
            relievers: self.relievers.iter().map(|p| convert::engine_player(p, Some(Position::Pitcher), true)).collect(),
            bench: field(&self.bench),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bm_engine::{BatterProfile, Hand, Level};

    /// The attribute population must describe the same league as the population the
    /// engine was calibrated with: same averages and spreads of the physical profiles.
    #[test]
    fn generated_major_leaguers_match_the_calibrated_population() {
        let mut rng = Rng::new(11);
        let squads: Vec<Squad> = (0..120).map(|i| Squad::random(&mut rng, 1000 * (i + 1), &Talent::MAJOR)).collect();
        let world: Vec<Club> = squads.iter().map(Squad::to_engine).collect();
        let mut rng = Rng::new(12);
        let level = Level::mlb();
        let engine: Vec<Club> = (0..120).map(|i| Club::random(&mut rng, 1000 * (i + 1), &level)).collect();

        let stat = |clubs: &[Club], f: &dyn Fn(&bm_engine::Player) -> f64, pitchers: bool| {
            let xs: Vec<f64> = clubs
                .iter()
                .flat_map(|c| if pitchers { c.starters.iter().chain(c.relievers.iter()).collect::<Vec<_>>() } else { c.hitters.iter().collect() })
                .map(f)
                .collect();
            let m = xs.iter().sum::<f64>() / xs.len() as f64;
            (m, (xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / xs.len() as f64).sqrt())
        };
        let check = |name: &str, f: &dyn Fn(&bm_engine::Player) -> f64, pitchers: bool, tol_mean: f64, tol_sd: f64| {
            let (mw, sw) = stat(&world, f, pitchers);
            let (me, se) = stat(&engine, f, pitchers);
            assert!((mw - me).abs() <= tol_mean * me.abs().max(1e-9), "{name}: mean {mw} vs {me}");
            assert!((sw - se).abs() <= tol_sd * se.max(1e-9), "{name}: sd {sw} vs {se}");
        };
        check("bat speed", &|p| p.batter.bat_speed, false, 0.004, 0.12);
        check("timing sd", &|p| p.batter.timing_sd, false, 0.015, 0.15);
        check("barrel sd", &|p| p.batter.barrel_sd, false, 0.015, 0.15);
        check("eye", &|p| p.batter.eye, false, 0.015, 0.15);
        check("perception", &|p| p.batter.perception_sd, false, 0.015, 0.15);
        check("sprint", &|p| p.batter.sprint, false, 0.006, 0.15);
        check("accel", &|p| p.batter.accel, false, 0.01, 0.15);
        check("steal", &|p| p.batter.steal, false, 0.08, 0.15);
        check("attack angle", &|p| p.batter.attack_angle, false, 0.03, 0.15);
        check("reaction", &|p| p.fielder.reaction, false, 0.02, 0.15);
        check("fielding accel", &|p| p.fielder.accel, false, 0.01, 0.15);
        check("arm", &|p| p.fielder.arm, false, 0.01, 0.2);
        check("hands", &|p| p.fielder.hands, false, 0.03, 0.2);
        check("fastball", &|p| p.pitcher.as_ref().unwrap().pitches[0].shape.speed, true, 0.004, 0.15);
        check("command", &|p| p.pitcher.as_ref().unwrap().pitches[0].command, true, 0.02, 0.25);
        check("stamina", &|p| p.pitcher.as_ref().unwrap().stamina, true, 0.03, 0.1);
        check("repertoire", &|p| p.pitcher.as_ref().unwrap().pitches.len() as f64, true, 0.03, 0.15);
        check("release side", &|p| p.pitcher.as_ref().unwrap().release_side, true, 0.05, 0.4);
        check("release height", &|p| p.pitcher.as_ref().unwrap().release_height, true, 0.02, 0.4);
        check("pitcher's bat", &|p| p.batter.bat_speed, true, 0.006, 0.2);
        check("pitcher's barrel", &|p| p.batter.barrel_sd, true, 0.03, 0.25);
        // Left-handed batters and switch hitters together are as common as the engine's left-handers.
        let left = |clubs: &[Club]| stat(clubs, &|p: &bm_engine::Player| (p.batter.hand == Hand::Left) as u8 as f64, false).0;
        assert!((left(&world) - left(&engine)).abs() < 0.05);
        let _ = BatterProfile::league_average(Hand::Right);
    }

    #[test]
    fn squads_have_the_expected_shape() {
        let mut rng = Rng::new(5);
        let s = Squad::random(&mut rng, 1000, &Talent::MAJOR);
        assert_eq!((s.hitters.len(), s.bench.len(), s.starters.len(), s.relievers.len()), (9, 4, 5, 8));
        for (p, pos) in s.hitters.iter().chain(&s.bench) {
            if let Some(pos) = pos {
                assert_eq!(p.positions[pos.index()], 20);
            }
            assert!(p.pitching.is_none());
        }
        for p in s.starters.iter().chain(&s.relievers) {
            let pit = p.pitching.as_ref().unwrap();
            assert!(pit.pitches.len() >= 3 && pit.pitches.len() <= 6);
            assert!(convert::main_fastball(&pit.pitches).is_some());
        }
        // Starters carry the stamina, relievers the velocity.
        let mean = |v: &[Player], a: Attr| v.iter().map(|p| p.attrs[a]).sum::<f32>() / v.len() as f32;
        assert!(mean(&s.starters, Attr::Stamina) > 1.2 && mean(&s.relievers, Attr::Stamina) < -1.2);
        let club = s.to_engine();
        assert!(club.relievers.iter().all(|p| p.pitcher.is_some()) && club.hitters.iter().all(|p| p.pitcher.is_none()));
    }
}
