//! Random players, drawn in attribute space.
//!
//! The spreads and the positional differences are those of the population the
//! match engine was calibrated with, so that a league of generated major-leaguers
//! plays like the calibrated one.

use crate::attr::{Attr, Attributes, Hidden};
use crate::convert;
use crate::player::{Bats, HitStyle, PitchSkill, Pitching, Player, Position, Role, Side};
use bm_engine::{BatterProfile, Club, Hand, PitchKind, Rng};
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
    /// Added to power on top of `hitting`: raw power and the rest of the bat need not be
    /// equally far from the majors.
    pub power: f32,
    /// Added to control and command on top of `pitching`: minor-league pitchers walk far
    /// more batters than their strikeout rates alone would suggest.
    pub command: f32,
    /// Added to velocity on top of `pitching`: minor-league pitchers throw nearly as hard as
    /// major-leaguers while the rest of their game lags.
    pub velocity: f32,
}

impl Talent {
    pub const MAJOR: Talent = Talent { hitting: 0.0, pitching: 0.0, fielding: 0.0, physical: 0.0, power: 0.0, command: 0.0, velocity: 0.0 };
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
    // Statcast 2025, qualified hitters: attack angle 9.8 deg with a spread of 3.5 deg.
    let lift = rng.gauss_clamped(0.0, 1.0, -3.0, 3.0);
    // Hitters who swing up also aim under the ball: lift is one habit seen in both.
    let aim = shape::LIFT_AIM * lift + (1.0 - shape::LIFT_AIM * shape::LIFT_AIM).sqrt() * rng.gauss(0.0, 1.0);
    let avg = BatterProfile::league_average(Hand::Right);
    HitStyle {
        attack_angle: (9.5 + shape::ATTACK_ANGLE_DEG * lift).to_radians() as f32,
        attack_sd: 5.5f64.to_radians() as f32,
        barrel_aim: (avg.barrel_aim + shape::BARREL_AIM * aim) as f32,
        timing_bias: rng.gauss(avg.timing_bias, shape::TIMING_BIAS) as f32,
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

/// Speed and arm strength of the regulars at each position, as (mean, spread) in
/// standard scores of all position players. Statcast 2025: sprint speed of players
/// with 30 or more competitive runs (catchers 26.2 ft/s, centre fielders 28.7; 1.35
/// over everyone) and arm strength of fielders with 50 or more throws (first basemen
/// 76.8 mph, right fielders 89.3; 5.9 over everyone).
fn position_tools(pos: Option<Position>) -> ((f64, f64), (f64, f64)) {
    match pos {
        Some(Position::Catcher) => ((-0.93, 0.90), (0.25, 0.51)),
        Some(Position::First) => ((-0.67, 0.91), (-1.20, 0.97)),
        Some(Position::Second) => ((0.27, 0.76), (-0.64, 0.72)),
        Some(Position::Third) => ((-0.10, 0.78), (-0.09, 0.65)),
        Some(Position::Short) => ((0.33, 0.81), (-0.01, 0.59)),
        Some(Position::Left) => ((0.27, 0.64), (0.40, 0.51)),
        Some(Position::Center) => ((0.99, 0.63), (0.66, 0.73)),
        Some(Position::Right) => ((0.27, 0.77), (0.92, 0.56)),
        _ => ((-0.58, 0.89), (-0.44, 1.0)),
    }
}

/// How players differ from one another beyond the spread of each attribute.
pub mod shape {
    /// Bat speed is bought with accuracy: share of a hitter's bat-speed advantage that
    /// comes off his contact, timing and bat control (Statcast 2025, qualified hitters:
    /// bat speed against in-zone contact -0.63, against whiffs per swing +0.69, against
    /// squared-up contact -0.67).
    pub const POWER_CONTACT: f64 = 0.80;
    pub const POWER_TIMING: f64 = 0.60;
    pub const POWER_BAT_CONTROL: f64 = 0.50;
    /// Between-hitter spreads of the swing's shape: attack angle (deg), aim under the
    /// ball (m), timing bias (s).
    pub const ATTACK_ANGLE_DEG: f64 = 3.5;
    pub const BARREL_AIM: f64 = 0.010;
    /// Correlation of the two.
    pub const LIFT_AIM: f64 = 0.35;
    pub const TIMING_BIAS: f64 = 0.0027;
    /// Where pitchers like to work: between-pitcher spread of the intended height, m, for
    /// the pitcher as a whole and for each pitch on top of that (2025 Statcast sample: the
    /// average height of a pitcher's four-seamers spreads by 6.0 cm, sliders 8.2 cm).
    pub const AIM_HEIGHT: f64 = 0.045;
    pub const AIM_HEIGHT_PITCH: f64 = 0.04;
    /// A catcher's exchange against his arm strength: the quick ones tend to have the
    /// weaker arms (pop time spreads less than exchange and arm would if independent).
    pub const EXCHANGE_ARM: f64 = -0.4;
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
    a[Attr::Power] = z(rng, hit + t.power as f64);
    let swing = a[Attr::Power] as f64 - hit - t.power as f64;
    let mut traded = |rho: f64| (hit - rho * swing + (1.0 - rho * rho).sqrt() * rng.gauss_clamped(0.0, 1.0, -3.5, 3.5)) as f32;
    a[Attr::Contact] = traded(shape::POWER_CONTACT);
    a[Attr::Timing] = traded(shape::POWER_TIMING);
    a[Attr::BatControl] = traded(shape::POWER_BAT_CONTROL);
    for attr in [Attr::Discipline, Attr::Recognition] {
        a[attr] = z(rng, hit);
    }
    a[Attr::Bunt] = z(rng, 0.0);
    let ((speed, speed_sd), (arm, arm_sd)) = position_tools(pos);
    let run = t.physical as f64;
    a[Attr::Speed] = rng.gauss_clamped(speed + run, speed_sd, speed + run - 3.0, speed + run + 3.0) as f32;
    // Quick players accelerate well and steal: the three go together.
    let sp = a[Attr::Speed] as f64;
    a[Attr::Acceleration] = (0.81 * sp + 0.58 * rng.gauss(0.0, 1.0)) as f32;
    a[Attr::Stealing] = (0.92 * sp + 0.39 * rng.gauss(0.0, 1.0) + t.fielding as f64) as f32;
    a[Attr::Baserunning] = z(rng, t.fielding as f64);
    let fld = t.fielding as f64;
    for attr in [Attr::Hands, Attr::Range, Attr::Reads, Attr::ArmAccuracy, Attr::Footwork] {
        a[attr] = z(rng, fld);
    }
    let arm_dev = rng.gauss_clamped(0.0, 1.0, -3.0, 3.0);
    a[Attr::ArmStrength] = (arm + run + arm_sd * arm_dev) as f32;
    let catching = if pos == Some(Position::Catcher) { fld } else { fld - 3.0 };
    for attr in [Attr::Framing, Attr::Blocking, Attr::GameCalling] {
        a[attr] = z(rng, catching);
    }
    let rho = shape::EXCHANGE_ARM;
    a[Attr::ThrowingOut] = (catching + rho * arm_dev + (1.0 - rho * rho).sqrt() * rng.gauss_clamped(0.0, 1.0, -3.0, 3.0)) as f32;
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

fn pitch(rng: &mut Rng, kind: PitchKind, weight: f64, t: &Talent, works: f64) -> PitchSkill {
    PitchSkill {
        height: (works + rng.gauss(0.0, shape::AIM_HEIGHT_PITCH)) as f32,
        kind,
        grade: z(rng, t.pitching as f64),
        speed: rng.gauss(0.0, 1.0 * MPH) as f32,
        spin: rng.gauss(0.0, 80.0 * RPM) as f32,
        active: rng.gauss(0.0, 0.05) as f32,
        tilt: rng.gauss(0.0, 6.0f64.to_radians()) as f32,
        usage: (weight * rng.gauss(0.0, 0.30).exp()) as f32,
    }
}

pub fn pitcher(rng: &mut Rng, id: u32, role: Role, t: &Talent) -> Player {
    let mut a = Attributes::default();
    common(rng, &mut a, t);
    let pit = t.pitching as f64;
    // Relievers are selected for velocity; starters for the stamina to go deep.
    a[Attr::Velocity] = z(rng, pit + t.velocity as f64 + if role == Role::Reliever { 0.27 } else { 0.0 });
    for attr in [Attr::Stuff, Attr::Control, Attr::Command, Attr::Deception] {
        let command = if matches!(attr, Attr::Control | Attr::Command) { t.command as f64 } else { 0.0 };
        a[attr] = z(rng, pit + command);
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
    a[Attr::ArmStrength] = z(rng, 0.6);
    for attr in [Attr::Framing, Attr::Blocking, Attr::ThrowingOut, Attr::GameCalling] {
        a[attr] = z(rng, -3.0);
    }

    // Some pitchers live at the top of the zone, some at the knees.
    let works = rng.gauss(0.0, shape::AIM_HEIGHT);
    let mut pitches: Vec<PitchSkill> = Vec::with_capacity(6);
    for &(kind, prob, weight) in &ARSENAL {
        if pitches.len() < 6 && rng.chance(prob) {
            pitches.push(pitch(rng, kind, weight, t, works));
        }
    }
    // Everybody has a fastball and at least two other offerings.
    if !pitches.iter().any(|s| matches!(s.kind, PitchKind::FourSeam | PitchKind::Sinker)) {
        pitches.insert(0, pitch(rng, PitchKind::FourSeam, ARSENAL[0].2, t, works));
    }
    let mut guard = 0;
    while pitches.len() < 3 && guard < 50 {
        guard += 1;
        let (kind, _, weight) = ARSENAL[2 + rng.below(6)];
        if !pitches.iter().any(|s| s.kind == kind) {
            pitches.push(pitch(rng, kind, weight, t, works));
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

    /// Replace `starters` of the rotation (from the top), the relievers named by
    /// `relievers` (from the top) and the regular hitters at the lineup slots `hitters`
    /// with players of `other`, at the same positions: foreign players in a league that
    /// limits how many of them a club may use.
    pub fn import_from(&mut self, other: &Squad, starters: usize, relievers: usize, hitters: &[usize]) {
        for i in 0..starters.min(self.starters.len()).min(other.starters.len()) {
            self.starters[i] = other.starters[i].clone();
        }
        for i in 0..relievers.min(self.relievers.len()).min(other.relievers.len()) {
            self.relievers[i] = other.relievers[i].clone();
        }
        for &i in hitters {
            if i < self.hitters.len() && i < other.hitters.len() {
                self.hitters[i] = other.hitters[i].clone();
            }
        }
    }

    /// The squad as the match engine's club.
    pub fn to_engine(&self) -> Club {
        self.to_engine_with(&convert::Baseline::default())
    }

    pub fn to_engine_with(&self, base: &convert::Baseline) -> Club {
        let field = |v: &[(Player, Option<Position>)]| v.iter().map(|(p, pos)| convert::engine_player_with(p, *pos, false, base)).collect();
        Club {
            hitters: field(&self.hitters),
            starters: self.starters.iter().map(|p| convert::engine_player_with(p, Some(Position::Pitcher), false, base)).collect(),
            relievers: self.relievers.iter().map(|p| convert::engine_player_with(p, Some(Position::Pitcher), true, base)).collect(),
            bench: field(&self.bench),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean_sd(xs: &[f64]) -> (f64, f64) {
        let m = xs.iter().sum::<f64>() / xs.len() as f64;
        (m, (xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / xs.len() as f64).sqrt())
    }

    fn corr(a: &[f64], b: &[f64]) -> f64 {
        let ((ma, sa), (mb, sb)) = (mean_sd(a), mean_sd(b));
        a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum::<f64>() / (a.len() as f64 * sa * sb)
    }

    /// The generated major-leaguers must spread the way the measured ones do.
    #[test]
    fn generated_major_leaguers_spread_like_the_measured_ones() {
        use bm_physics::units::{FT, MPH};
        let mut rng = Rng::new(11);
        let squads: Vec<Squad> = (0..150).map(|i| Squad::random(&mut rng, 1000 * (i + 1), &Talent::MAJOR)).collect();
        let hitters: Vec<(&Player, Option<Position>)> = squads.iter().flat_map(|s| s.hitters.iter().map(|(p, pos)| (p, *pos))).collect();
        let profile = |p: &Player| convert::batter(p);

        // Bat speed: 2.6 mph between hitters around the league average (Statcast: 2.64).
        let speed: Vec<f64> = hitters.iter().map(|(p, _)| profile(p).bat_speed / MPH).collect();
        let (m, sd) = mean_sd(&speed);
        assert!((m - bm_engine::profile::AVERAGE_BAT_SPEED / MPH).abs() < 0.25 && (sd - 2.6).abs() < 0.25, "bat speed {m} +- {sd}");
        // Bat speed is bought with accuracy.
        let power: Vec<f64> = hitters.iter().map(|(p, _)| p.attrs.z(Attr::Power)).collect();
        let contact: Vec<f64> = hitters.iter().map(|(p, _)| p.attrs.z(Attr::Contact)).collect();
        let c = corr(&power, &contact);
        assert!((c + shape::POWER_CONTACT).abs() < 0.06, "power against contact {c}");
        let (_, sd) = mean_sd(&contact);
        assert!((sd - 1.0).abs() < 0.08, "contact keeps a unit spread: {sd}");

        // Speed and arm by position (Statcast: catchers 26.2 ft/s, centre fielders 28.7;
        // first basemen throw 76.8 mph, right fielders 89.3).
        let at = |pos: Position, f: &dyn Fn(&Player) -> f64| -> (f64, f64) {
            mean_sd(&hitters.iter().filter(|(_, q)| *q == Some(pos)).map(|(p, _)| f(p)).collect::<Vec<_>>())
        };
        let sprint = |p: &Player| profile(p).sprint / FT;
        let (c_speed, c_sd) = at(Position::Catcher, &sprint);
        let (cf_speed, cf_sd) = at(Position::Center, &sprint);
        assert!((cf_speed - c_speed - 2.6).abs() < 0.4, "centre fielders {cf_speed}, catchers {c_speed}");
        assert!((c_sd - 1.22).abs() < 0.25 && (cf_sd - 0.85).abs() < 0.2, "spreads {c_sd} {cf_sd}");
        let arm = |pos: Position| at(pos, &|p: &Player| convert::fielder(p, Some(pos)).arm / MPH).0;
        assert!((arm(Position::Right) - arm(Position::First) - 12.5).abs() < 1.5, "right {} first {}", arm(Position::Right), arm(Position::First));
        // Pop time spreads less than exchange and arm would on their own (Statcast: 0.048 s).
        let pops: Vec<f64> = hitters.iter().filter(|(_, q)| *q == Some(Position::Catcher)).map(|(p, _)| convert::catcher(p).pop_time).collect();
        let (m, sd) = mean_sd(&pops);
        assert!((m - 1.95).abs() < 0.03 && (sd - 0.048).abs() < 0.012, "pop time {m} +- {sd}");

        // Pitchers: fastball velocity (Statcast: four-seamers of regular starters spread by
        // 2.1 mph), and the spin axis following the arm slot.
        let arms: Vec<&Player> = squads.iter().flat_map(|s| s.starters.iter()).collect();
        let four: Vec<(f64, f64, f64)> = arms
            .iter()
            .filter_map(|p| {
                let prof = convert::pitcher(p, false)?;
                let ff = prof.pitches.iter().find(|d| d.kind == PitchKind::FourSeam)?;
                Some((ff.shape.speed / MPH, ff.shape.tilt.to_degrees(), p.pitching.as_ref()?.arm_angle.to_degrees() as f64))
            })
            .collect();
        let (m, sd) = mean_sd(&four.iter().map(|f| f.0).collect::<Vec<_>>());
        assert!((m - 94.0).abs() < 0.5 && (sd - 2.1).abs() < 0.35, "four-seam velocity {m} +- {sd}");
        let c = corr(&four.iter().map(|f| f.2).collect::<Vec<_>>(), &four.iter().map(|f| f.1).collect::<Vec<_>>());
        assert!(c < -0.7, "a higher arm slot should stand the spin axis up: {c}");
        // Where pitchers like to work: the height of a pitcher's fastballs spreads by about 6 cm.
        let heights: Vec<f64> = arms.iter().filter_map(|p| p.pitching.as_ref()?.pitches.first().map(|k| k.height as f64)).collect();
        let (_, sd) = mean_sd(&heights);
        assert!((sd - 0.06).abs() < 0.012, "aim height spread {sd}");
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
