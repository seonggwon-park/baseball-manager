//! Attributes to the physical profiles the match engine works with.
//!
//! Each attribute moves one physical quantity (a few move two that belong
//! together). The size of one standard deviation is the spread of that quantity
//! among major-league players: measured where pitch or bat tracking publishes it,
//! otherwise the spread the engine was calibrated with. `docs/attributes.md` lists
//! the source of every number.

use crate::attr::Attr;
use crate::player::{Bats, Player, Position};
use bm_engine::profile::{AVERAGE_BARREL_SD, AVERAGE_BAT_SPEED, AVERAGE_ERROR_RATIO, NO_POSITION};
use bm_engine::{BatterProfile, CatcherProfile, FielderProfile, Hand, PitchDef, PitchGroup, PitchKind, PitcherProfile};
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::BatSpec;

/// Between-player standard deviations of the physical quantities (major leagues).
pub mod spread {
    /// Bat speed, mph (Statcast bat tracking 2025, qualified hitters).
    pub const BAT_SPEED_MPH: f64 = 2.6;
    /// Relative spread of the barrel error and of the timing error.
    pub const BARREL: f64 = 0.19;
    pub const TIMING: f64 = 0.15;
    /// Probability of picking up the pitch type.
    pub const EYE: f64 = 0.07;
    /// Relative spread of the error in judging the pitch's location.
    pub const PERCEPTION: f64 = 0.14;
    /// Shift of the swing/take boundary, m.
    pub const CHASE: f64 = 0.026;
    pub const AGGRESSION: f64 = 0.055;
    pub const REACH: f64 = 0.10;
    pub const FOUL_SKILL: f64 = 0.15;
    pub const ADJUST: f64 = 0.15;
    pub const LAPSE: f64 = 0.20;
    pub const SPEED_LOSS: f64 = 0.15;
    /// Relative bat weight (0.8 oz on a 31.5 oz bat; major-league bats run from about 30 to 34 oz).
    pub const BAT_WEIGHT: f64 = 0.025;
    pub const JUDGEMENT: f64 = 0.25;
    pub const BUNT: f64 = 0.18;
    pub const STEAL: f64 = 0.38;
    /// Sprint speed, ft/s (Statcast sprint speed).
    pub const SPRINT_FT: f64 = 1.35;
    /// Running acceleration, m/s^2.
    pub const ACCEL: f64 = 0.43;
    /// Fielding: first-step time (s), burst (m/s^2), route efficiency, error multipliers,
    /// arm strength (mph), exchange time (s).
    pub const REACTION: f64 = 0.03;
    pub const BURST: f64 = 0.15;
    pub const ROUTE: f64 = 0.016;
    pub const READ_DELAY: f64 = 0.035;
    pub const HANDS: f64 = 0.20;
    pub const ARM_MPH: f64 = 5.9;
    pub const ARM_ACCURACY: f64 = 0.20;
    pub const TRANSFER: f64 = 0.06;
    /// Catching: pop time (s), blocking multiplier, framing as a shift of the called zone (m).
    /// Exchange time of a catcher (s), and what one mph of arm strength takes off the
    /// throw's flight to second base (s).
    pub const EXCHANGE: f64 = 0.040;
    pub const POP_PER_MPH: f64 = 0.0158;
    pub const BLOCKING: f64 = 0.15;
    pub const FRAMING: f64 = 0.0023;
    /// Pitching: fastball velocity (mph), relative location error on each axis of the
    /// error ellipse, deception as a change of the batter's pick-up probability,
    /// time to the plate (s), pitches of stamina.
    pub const VELOCITY_MPH: f64 = 2.2;
    pub const LOCATION: f64 = 0.05;
    pub const DECEPTION: f64 = 0.03;
    pub const TIME_TO_PLATE: f64 = 0.08;
    pub const STAMINA: f64 = 16.0;
    pub const WILDNESS: f64 = 0.15;
    /// Extra precision on a secondary pitch per point of its grade.
    pub const PITCH_COMMAND: f64 = 0.06;
    /// Change of the batter's pick-up probability on a secondary pitch per point of its grade.
    pub const PITCH_DISGUISE: f64 = 0.05;
    /// Change of the swing and location errors at full pressure, per point of composure.
    pub const PRESSURE: f64 = 0.04;
    /// Platoon: change of the pick-up probability against same-sided pitchers.
    pub const PLATOON_EYE: f64 = 0.02;
    /// Concentration's share in fielding errors.
    pub const FIELD_LAPSE: f64 = 0.05;
}

/// Major-league averages of the quantities that are not defined in the engine's
/// own league-average profiles.
pub mod mean {
    /// Arm strength of a position player on a full-effort throw, mph.
    pub const ARM_MPH: f64 = 85.9;
    /// Location error of the average pitcher on a fastball, root mean square over the two axes, in.
    pub const COMMAND_IN: f64 = 6.45;
    /// Pitches before fatigue for a pitcher of average stamina (starters sit near +2, relievers near -2).
    pub const STAMINA: f64 = 60.0;
    /// Fastball velocity of a starter with an average velocity attribute relative to the
    /// league-average pitch shapes, mph, and what the same pitcher gains in short relief.
    pub const STARTER_VELOCITY_MPH: f64 = -0.6;
    pub const RELIEF_GAIN_MPH: f64 = 0.9;
    /// Height the pitch geometry is scaled to, m, and the arm length and shoulder height
    /// that reproduce the average release point at the average arm angle.
    /// Average arm angle, degrees (Statcast 2025: 39.9 over 380 pitchers).
    pub const ARM_ANGLE: f64 = 40.0;
    pub const PITCHER_HEIGHT: f64 = 1.88;
    pub const ARM_LENGTH: f64 = 0.816;
    pub const SHOULDER_HEIGHT: f64 = 1.25;
}

/// What a major-leaguer with every attribute at zero is: the league averages of the
/// physical quantities the attributes move. These are the values fitted so that a
/// league of generated players reproduces the 2025 MLB totals.
#[derive(Clone, Debug, PartialEq)]
pub struct Baseline {
    /// Bat speed, m/s.
    pub bat_speed: f64,
    /// Timing error (s), barrel error (m), error of the contact point along the bat (m).
    pub timing_sd: f64,
    pub barrel_sd: f64,
    pub reach_sd: f64,
    /// Probability of picking up the pitch type; error in judging the location (m).
    pub eye: f64,
    pub perception_sd: f64,
    /// Shift of every hitter's timing, s (+ = late).
    pub timing_bias: f64,
    /// Shift of every hitter's swing/take boundary (m) and of his aim under the ball (m).
    pub chase: f64,
    pub aim: f64,
    /// Stealing skill, 0..=1.
    pub steal: f64,
    /// Location error of a fastball, root mean square over the two axes, m.
    pub command: f64,
    /// Fastball velocity of a starter relative to the league-average pitch shapes, m/s.
    pub velocity: f64,
}

impl Default for Baseline {
    fn default() -> Self {
        let avg = BatterProfile::league_average(Hand::Right);
        Baseline {
            bat_speed: AVERAGE_BAT_SPEED,
            timing_sd: avg.timing_sd,
            barrel_sd: AVERAGE_BARREL_SD,
            reach_sd: avg.reach_sd,
            eye: avg.eye,
            perception_sd: avg.perception_sd,
            chase: avg.chase,
            timing_bias: 0.0,
            aim: 0.0,
            steal: 0.375,
            command: mean::COMMAND_IN * INCH,
            velocity: mean::STARTER_VELOCITY_MPH * MPH,
        }
    }
}

/// `exp(-s z)` scaled so that its mean over a standard normal `z` is 1: an error
/// that shrinks by the same ratio with every point of the attribute.
#[inline]
fn shrink(z: f64, s: f64) -> f64 {
    (-s * z - 0.5 * s * s).exp()
}

/// The player as a batter and runner.
pub fn batter(p: &Player) -> BatterProfile {
    batter_with(p, &Baseline::default())
}

pub fn batter_with(p: &Player, base: &Baseline) -> BatterProfile {
    use spread::*;
    let z = |a: Attr| p.attrs.z(a);
    let avg = BatterProfile::league_average(Hand::Right);
    BatterProfile {
        // A switch hitter is described from the left side; the engine turns him around.
        hand: if p.bats == Bats::Right { Hand::Right } else { Hand::Left },
        switch: p.bats == Bats::Switch,
        height: p.height as f64,
        bat_speed: base.bat_speed + BAT_SPEED_MPH * MPH * z(Attr::Power),
        bat_speed_sd: p.hit.bat_speed_sd as f64,
        attack_angle: p.hit.attack_angle as f64,
        attack_sd: p.hit.attack_sd as f64,
        timing_sd: base.timing_sd * shrink(z(Attr::Timing), TIMING),
        barrel_sd: base.barrel_sd * shrink(z(Attr::Contact), BARREL),
        barrel_aim: p.hit.barrel_aim as f64 + base.aim,
        reach_sd: base.reach_sd * shrink(z(Attr::BatControl), REACH),
        eye: (base.eye + EYE * z(Attr::Recognition)).clamp(0.15, 0.97),
        perception_sd: base.perception_sd * shrink(z(Attr::Discipline), PERCEPTION),
        chase: base.chase + CHASE * (-0.9 * z(Attr::Discipline) + 0.44 * z(Attr::Aggression)),
        aggression: (1.0 + AGGRESSION * z(Attr::Aggression)).clamp(0.75, 1.25),
        timing_bias: p.hit.timing_bias as f64 + base.timing_bias,
        sprint: (27.0 + SPRINT_FT * z(Attr::Speed)).clamp(20.0, 31.5) * FT,
        accel: (avg.accel + ACCEL * z(Attr::Acceleration)).clamp(3.4, 6.4),
        baserunning: (0.5 + 0.15 * z(Attr::Aggression)).clamp(0.0, 1.0),
        steal: (base.steal + STEAL * z(Attr::Stealing)).clamp(0.0, 1.0),
        bunt: (0.5 + BUNT * z(Attr::Bunt)).clamp(0.05, 1.0),
        foul_skill: (FOUL_SKILL * z(Attr::BatControl)).exp(),
        adjust: (ADJUST * z(Attr::Anticipation)).exp(),
        lapse: shrink(z(Attr::Concentration), LAPSE),
        speed_loss: shrink(z(Attr::Strength), SPEED_LOSS),
        judgement: shrink(z(Attr::Baserunning), JUDGEMENT),
        eye_same: PLATOON_EYE * p.hidden.platoon as f64,
        pressure: -PRESSURE * z(Attr::Composure),
        // A stronger hitter swings a heavier bat at the same speed: more of the bat's
        // momentum goes into the ball.
        bat: BatSpec::mlb_wood().weighted((1.0 + BAT_WEIGHT * z(Attr::Strength)).clamp(0.85, 1.15)),
    }
}

/// The player as a fielder at `pos` (`None` = not in the field: a designated hitter).
/// A position he does not know costs him his first step, his routes and his hands.
pub fn fielder(p: &Player, pos: Option<Position>) -> FielderProfile {
    use spread::*;
    let z = |a: Attr| p.attrs.z(a);
    let lost = pos.map_or(0.0, |q| 1.0 - p.familiarity(q)).clamp(0.0, 1.0);
    let infield = pos.is_some_and(Position::is_infield);
    let middle = matches!(pos, Some(Position::Second | Position::Short));
    let outfield = pos.is_some_and(Position::is_outfield);
    let battery = matches!(pos, Some(Position::Pitcher | Position::Catcher));
    let focus = shrink(z(Attr::Concentration), FIELD_LAPSE);
    let sprint_ft = (27.0 + SPRINT_FT * z(Attr::Speed)).clamp(20.0, 31.5);

    // Infielders read the ball off the bat and move with a quick first step; the
    // middle infielders are the rangiest. Outfielders build speed more slowly while
    // they track a ball in the air.
    let mut reaction = 0.30 - REACTION * z(Attr::Range) + 0.10 * lost;
    let mut accel = 4.6 + BURST * z(Attr::Agility);
    if infield {
        reaction -= 0.08;
        accel += 0.8;
    }
    if middle {
        reaction -= 0.03;
        accel += 0.5;
    }
    let accel = accel.clamp(3.2, 7.2);
    FielderProfile {
        reaction: reaction.clamp(0.10, 0.60),
        accel,
        speed: (sprint_ft + 0.5) * FT,
        read_delay: if outfield { (0.01 - READ_DELAY * z(Attr::Reads)).clamp(0.0, 0.25) + 0.10 * lost } else { 0.0 },
        fly_accel: if outfield {
            accel - 0.6
        } else if infield {
            accel
        } else {
            4.6
        },
        route: (0.95 + ROUTE * z(Attr::Reads) - 0.04 * lost).clamp(0.80, 0.995),
        hands: shrink(z(Attr::Hands), HANDS) * focus * (1.0 + 1.5 * lost),
        // Pitchers and catchers rarely get to set their feet on the throws they make as fielders.
        arm: ((mean::ARM_MPH + ARM_MPH * z(Attr::ArmStrength)).clamp(60.0, 105.0) - if battery { 5.0 } else { 0.0 }) * MPH,
        transfer: (0.80 - TRANSFER * z(Attr::Footwork)).clamp(0.58, 1.15) + if outfield { 0.05 } else { 0.0 } + 0.15 * lost,
        arm_accuracy: shrink(z(Attr::ArmAccuracy), ARM_ACCURACY) * focus * (1.0 + lost),
    }
}

/// The player behind the plate.
pub fn catcher(p: &Player) -> CatcherProfile {
    use spread::*;
    let z = |a: Attr| p.attrs.z(a);
    let lost = 1.0 - p.familiarity(Position::Catcher);
    CatcherProfile {
        // Pop time is the exchange plus the flight of the throw: the first is the catcher's
        // own skill, the second his arm (Statcast 2025: exchange 0.656 s with a spread of
        // 0.040 s, arm 82.4 mph with a spread of 3.0 mph, pop time 1.96 s).
        pop_time: (1.97 - EXCHANGE * z(Attr::ThrowingOut) - POP_PER_MPH * ARM_MPH * z(Attr::ArmStrength)).clamp(1.70, 2.45) + 0.15 * lost,
        blocking: shrink(z(Attr::Blocking), BLOCKING) * (1.0 + lost),
        framing: FRAMING * z(Attr::Framing) - 0.004 * lost,
    }
}

/// What makes each pitch type better, per point of its grade:
/// (release speed in mph, spin rate in rpm, active-spin fraction, spin tilt in degrees).
pub fn quality_direction(kind: PitchKind) -> (f64, f64, f64, f64) {
    match kind {
        PitchKind::FourSeam => (0.0, 90.0, 0.03, 0.0),
        PitchKind::Sinker => (0.0, 40.0, 0.02, 5.0),
        PitchKind::Cutter => (0.7, 110.0, 0.0, 0.0),
        PitchKind::Slider => (0.6, 120.0, 0.0, 0.0),
        PitchKind::Sweeper => (0.0, 120.0, 0.05, 0.0),
        PitchKind::Curve => (0.0, 140.0, 0.04, 0.0),
        PitchKind::Changeup => (-0.5, -100.0, 0.02, 0.0),
        PitchKind::Splitter => (-0.3, -120.0, 0.0, 0.0),
    }
}

/// How much of a change in arm slot shows up in the spin axis of each pitch type:
/// fully for the pitches thrown with the fastball's arm action, half for the ones
/// spun off the side of the ball.
fn slot_share(kind: PitchKind) -> f64 {
    match kind.group() {
        PitchGroup::Fastball if kind != PitchKind::Cutter => 1.0,
        PitchGroup::Offspeed => 1.0,
        _ => 0.5,
    }
}

/// Usage against same-sided and opposite-sided batters relative to the pitcher's
/// overall liking for the pitch (2025 Statcast usage splits).
pub fn platoon_usage(kind: PitchKind) -> (f64, f64) {
    match kind {
        PitchKind::FourSeam => (0.89, 1.09),
        PitchKind::Sinker => (1.28, 0.70),
        PitchKind::Cutter => (0.97, 1.08),
        PitchKind::Slider => (1.34, 0.74),
        PitchKind::Sweeper => (1.43, 0.65),
        PitchKind::Curve => (0.69, 1.21),
        PitchKind::Changeup => (0.48, 1.43),
        PitchKind::Splitter => (0.76, 1.37),
    }
}

/// Index of the pitch that carries the `Stuff` attribute: the fastball he throws most.
pub fn main_fastball(pitches: &[crate::player::PitchSkill]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, s) in pitches.iter().enumerate() {
        if matches!(s.kind, PitchKind::FourSeam | PitchKind::Sinker) && best.is_none_or(|b| s.usage > pitches[b].usage) {
            best = Some(i);
        }
    }
    best
}

/// The player as a pitcher, or `None` if he has no repertoire. `relief` = used in
/// short stints, where the same arm throws harder.
pub fn pitcher(p: &Player, relief: bool) -> Option<PitcherProfile> {
    pitcher_with(p, relief, &Baseline::default())
}

pub fn pitcher_with(p: &Player, relief: bool, base: &Baseline) -> Option<PitcherProfile> {
    use spread::*;
    let pit = p.pitching.as_ref()?;
    if pit.pitches.is_empty() {
        return None;
    }
    let z = |a: Attr| p.attrs.z(a);
    let fb_delta = base.velocity + (VELOCITY_MPH * z(Attr::Velocity) + if relief { mean::RELIEF_GAIN_MPH } else { 0.0 }) * MPH;

    // Location error: the long axis of the ellipse is the control attribute, the short one command.
    let r2 = AVERAGE_ERROR_RATIO * AVERAGE_ERROR_RATIO;
    let along = base.command * (2.0 * r2 / (1.0 + r2)).sqrt() * shrink(z(Attr::Control), LOCATION);
    let across = base.command * (2.0 / (1.0 + r2)).sqrt() * shrink(z(Attr::Command), LOCATION);
    let rms = ((along * along + across * across) / 2.0).sqrt();

    let main = main_fastball(&pit.pitches);
    let slot_tilt = mean::ARM_ANGLE.to_radians() - pit.arm_angle as f64;
    let pitches = pit
        .pitches
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let base = s.kind.league_average();
            let q = if Some(i) == main { z(Attr::Stuff) } else { s.grade as f64 };
            let (d_mph, d_rpm, d_active, d_tilt) = quality_direction(s.kind);
            let shape = bm_engine::profile::PitchShape {
                speed: base.speed + 0.9 * fb_delta + s.speed as f64 + q * d_mph * MPH,
                spin_rate: (base.spin_rate + s.spin as f64 + q * d_rpm * RPM).max(600.0 * RPM),
                active: (base.active + s.active as f64 + q * d_active).clamp(0.03, 0.99),
                // The spin axis follows the arm: a higher slot gives a fastball more ride, a
                // lower one more run (Statcast 2025: arm angle and induced rise correlate at 0.64).
                tilt: base.tilt + slot_tilt * slot_share(s.kind) + s.tilt as f64 + q * d_tilt.to_radians(),
            };
            let group = match s.kind.group() {
                PitchGroup::Fastball => 1.0,
                PitchGroup::Breaking => 1.12,
                PitchGroup::Offspeed => 1.10,
            };
            let feel = if Some(i) == main { 1.0 } else { (-PITCH_COMMAND * s.grade as f64).exp() };
            let (same, opp) = platoon_usage(s.kind);
            let disguise = if Some(i) == main { 0.0 } else { PITCH_DISGUISE * s.grade as f64 };
            PitchDef {
                kind: s.kind,
                shape,
                usage_same: s.usage as f64 * same,
                usage_opp: s.usage as f64 * opp,
                command: rms * group * feel,
                disguise,
                aim_height: s.height as f64,
            }
        })
        .collect();

    let scale = p.height as f64 / mean::PITCHER_HEIGHT;
    let (sin, cos) = (pit.arm_angle as f64).sin_cos();
    Some(PitcherProfile {
        hand: p.throws.hand(),
        release_side: (mean::ARM_LENGTH * scale * cos - 0.05).max(0.02),
        release_height: (mean::SHOULDER_HEIGHT * scale + mean::ARM_LENGTH * scale * sin).max(0.3),
        extension: pit.extension as f64,
        pitches,
        speed_sd: pit.speed_sd as f64,
        spin_sd: pit.spin_sd as f64,
        tilt_sd: pit.tilt_sd as f64,
        stamina: (mean::STAMINA + STAMINA * z(Attr::Stamina)).clamp(10.0, 130.0),
        time_to_plate: (1.38 - TIME_TO_PLATE * z(Attr::HoldRunners)).clamp(1.05, 1.80),
        error_ratio: along / across,
        wildness: shrink(z(Attr::Concentration), WILDNESS),
        deception: DECEPTION * z(Attr::Deception),
        pressure: -PRESSURE * z(Attr::Composure),
    })
}

/// The player as the engine uses him at `pos` (`None` = designated hitter or bench).
pub fn engine_player(p: &Player, pos: Option<Position>, relief: bool) -> bm_engine::Player {
    engine_player_with(p, pos, relief, &Baseline::default())
}

pub fn engine_player_with(p: &Player, pos: Option<Position>, relief: bool, base: &Baseline) -> bm_engine::Player {
    bm_engine::Player {
        id: p.id,
        position: pos.map_or(NO_POSITION, Position::index),
        batter: batter_with(p, base),
        fielder: fielder(p, pos),
        pitcher: pitcher_with(p, relief, base),
        catcher: catcher(p),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::{Attributes, Hidden};
    use crate::player::{HitStyle, PitchSkill, Pitching, Role, Side};

    /// A player with every attribute at the major-league average.
    fn average(pitcher: bool) -> Player {
        let avg = BatterProfile::league_average(Hand::Right);
        Player {
            id: 1,
            height: if pitcher { 1.88 } else { 1.855 },
            weight: 95.0,
            bats: Bats::Right,
            throws: Side::Right,
            position: if pitcher { Position::Pitcher } else { Position::Short },
            positions: [20; 9],
            attrs: Attributes::default(),
            hidden: Hidden::default(),
            hit: HitStyle {
                attack_angle: avg.attack_angle as f32,
                attack_sd: avg.attack_sd as f32,
                barrel_aim: avg.barrel_aim as f32,
                timing_bias: avg.timing_bias as f32,
                bat_speed_sd: avg.bat_speed_sd as f32,
            },
            pitching: pitcher.then(|| Pitching {
                role: Role::Starter,
                arm_angle: 40.5f32.to_radians(),
                extension: (6.45 * FT) as f32,
                speed_sd: (1.05 * MPH) as f32,
                spin_sd: (85.0 * RPM) as f32,
                tilt_sd: 4.5f32.to_radians(),
                pitches: [(PitchKind::FourSeam, 0.5), (PitchKind::Slider, 0.3), (PitchKind::Changeup, 0.2)]
                    .into_iter()
                    .map(|(kind, usage)| PitchSkill { kind, grade: 0.0, speed: 0.0, spin: 0.0, active: 0.0, tilt: 0.0, height: 0.0, usage })
                    .collect(),
            }),
        }
    }

    fn close(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * b.abs().max(1e-9)
    }

    #[test]
    fn average_attributes_give_the_league_average_batter() {
        let b = batter(&average(false));
        let avg = BatterProfile::league_average(Hand::Right);
        assert!(close(b.bat_speed, avg.bat_speed, 1e-6));
        // Errors are scaled so that their mean over the population is the league average,
        // which puts the player at zero a little under it (the median of a skewed spread).
        assert!(close(b.timing_sd, avg.timing_sd, 0.02) && b.timing_sd < avg.timing_sd, "{} vs {}", b.timing_sd, avg.timing_sd);
        assert!(close(b.barrel_sd, avg.barrel_sd, 0.02) && b.barrel_sd < avg.barrel_sd);
        assert!(close(b.perception_sd, avg.perception_sd, 0.02));
        assert!(close(b.reach_sd, avg.reach_sd, 0.02));
        assert!(close(b.eye, avg.eye, 1e-6) && close(b.sprint, avg.sprint, 1e-6) && close(b.accel, avg.accel, 1e-6));
        assert_eq!((b.chase, b.aggression, b.baserunning, b.bunt), (avg.chase, 1.0, 0.5, 0.5));
        assert_eq!((b.foul_skill, b.adjust, b.eye_same, b.pressure), (1.0, 1.0, 0.0, 0.0));
        assert!(close(b.lapse, 1.0, 0.03) && close(b.speed_loss, 1.0, 0.02) && close(b.judgement, 1.0, 0.04));
        assert!(!b.switch);
    }

    #[test]
    fn average_attributes_give_the_league_average_fielder_and_pitcher() {
        let f = fielder(&average(false), None);
        let avg = FielderProfile::league_average();
        assert!(close(f.reaction, avg.reaction, 1e-6) && close(f.accel, avg.accel, 1e-6) && close(f.route, avg.route, 1e-6));
        assert!(close(f.speed, avg.speed, 1e-6) && close(f.transfer, avg.transfer, 1e-6));
        assert!(close(f.hands, 1.0, 0.03) && close(f.arm_accuracy, 1.0, 0.03));
        let c = catcher(&average(false));
        assert!(close(c.pop_time, 1.97, 1e-6) && close(c.blocking, 1.0, 0.02) && c.framing == 0.0);

        let p = pitcher(&average(true), false).expect("has a repertoire");
        let avg = PitcherProfile::league_average(Hand::Right);
        assert!(close(p.release_side, avg.release_side, 0.03), "{} vs {}", p.release_side, avg.release_side);
        assert!(close(p.release_height, avg.release_height, 0.03), "{} vs {}", p.release_height, avg.release_height);
        assert!(close(p.error_ratio, AVERAGE_ERROR_RATIO, 1e-9));
        assert!(close(p.pitches[0].command, mean::COMMAND_IN * INCH, 0.012), "{}", p.pitches[0].command / INCH);
        assert!(close(p.pitches[0].shape.spin_rate, PitchKind::FourSeam.league_average().spin_rate, 1e-9));
        assert!(close(p.stamina, 60.0, 1e-9) && close(p.time_to_plate, 1.38, 1e-9));
        assert_eq!((p.deception, p.pressure), (0.0, 0.0));
        // The same arm throws harder out of the bullpen.
        let r = pitcher(&average(true), true).unwrap();
        assert!(close((r.pitches[0].shape.speed - p.pitches[0].shape.speed) / MPH, 0.9 * mean::RELIEF_GAIN_MPH, 1e-6));
        assert!(pitcher(&average(false), false).is_none());
    }

    /// Every attribute that reaches the engine must move its physical quantity in the
    /// direction that helps the player.
    #[test]
    fn better_attributes_are_better_physically() {
        let base = average(false);
        let up = |a: Attr| {
            let mut p = base.clone();
            p.attrs[a] = 1.0;
            p
        };
        let b0 = batter(&base);
        assert!(batter(&up(Attr::Contact)).barrel_sd < b0.barrel_sd);
        assert!(batter(&up(Attr::Timing)).timing_sd < b0.timing_sd);
        assert!(close((batter(&up(Attr::Power)).bat_speed - b0.bat_speed) / MPH, spread::BAT_SPEED_MPH, 1e-9));
        assert!(batter(&up(Attr::Discipline)).perception_sd < b0.perception_sd && batter(&up(Attr::Discipline)).chase < b0.chase);
        assert!(batter(&up(Attr::Recognition)).eye > b0.eye);
        assert!(batter(&up(Attr::BatControl)).foul_skill > 1.0 && batter(&up(Attr::BatControl)).reach_sd < b0.reach_sd);
        assert!(batter(&up(Attr::Bunt)).bunt > b0.bunt && batter(&up(Attr::Stealing)).steal > b0.steal);
        assert!(batter(&up(Attr::Baserunning)).judgement < b0.judgement);
        assert!(batter(&up(Attr::Speed)).sprint > b0.sprint && batter(&up(Attr::Acceleration)).accel > b0.accel);
        assert!(batter(&up(Attr::Anticipation)).adjust > 1.0 && batter(&up(Attr::Concentration)).lapse < b0.lapse);
        assert!(batter(&up(Attr::Strength)).speed_loss < b0.speed_loss && batter(&up(Attr::Composure)).pressure < 0.0);
        assert!(batter(&up(Attr::Strength)).bat.mass > b0.bat.mass && batter(&up(Attr::Strength)).bat.inertia_cm > b0.bat.inertia_cm);
        assert!(batter(&up(Attr::Aggression)).aggression > 1.0 && batter(&up(Attr::Aggression)).chase > b0.chase);

        let short = Some(Position::Short);
        let f0 = fielder(&base, short);
        assert!(fielder(&up(Attr::Hands), short).hands < f0.hands);
        assert!(fielder(&up(Attr::Range), short).reaction < f0.reaction);
        assert!(fielder(&up(Attr::Reads), short).route > f0.route);
        assert!(fielder(&up(Attr::Reads), Some(Position::Center)).read_delay < fielder(&base, Some(Position::Center)).read_delay);
        assert!(fielder(&up(Attr::ArmStrength), short).arm > f0.arm && fielder(&up(Attr::ArmAccuracy), short).arm_accuracy < f0.arm_accuracy);
        assert!(fielder(&up(Attr::Footwork), short).transfer < f0.transfer && fielder(&up(Attr::Agility), short).accel > f0.accel);
        assert!(fielder(&up(Attr::Speed), short).speed > f0.speed);
        let c0 = catcher(&base);
        assert!(catcher(&up(Attr::Framing)).framing > c0.framing && catcher(&up(Attr::Blocking)).blocking < c0.blocking);
        assert!(catcher(&up(Attr::ThrowingOut)).pop_time < c0.pop_time);

        let arm = average(true);
        let p0 = pitcher(&arm, false).unwrap();
        let pup = |a: Attr| {
            let mut p = arm.clone();
            p.attrs[a] = 1.0;
            pitcher(&p, false).unwrap()
        };
        assert!(close((pup(Attr::Velocity).pitches[0].shape.speed - p0.pitches[0].shape.speed) / MPH, 0.9 * spread::VELOCITY_MPH, 1e-6));
        // Stuff is the main fastball's own grade; it leaves the other pitches alone.
        assert!(pup(Attr::Stuff).pitches[0].shape.spin_rate > p0.pitches[0].shape.spin_rate);
        assert_eq!(pup(Attr::Stuff).pitches[1].shape, p0.pitches[1].shape);
        // Control shortens the long axis of the error ellipse, command the short one.
        let axes = |p: &PitcherProfile| {
            let (c, r2) = (p.pitches[0].command, p.error_ratio * p.error_ratio);
            (c * (2.0 * r2 / (1.0 + r2)).sqrt(), c * (2.0 / (1.0 + r2)).sqrt())
        };
        let (a0, c0) = axes(&p0);
        let (a1, c1) = axes(&pup(Attr::Control));
        assert!(a1 < 0.97 * a0 && close(c1, c0, 1e-9));
        let (a2, c2) = axes(&pup(Attr::Command));
        assert!(c2 < 0.97 * c0 && close(a2, a0, 1e-9));
        assert!(pup(Attr::Deception).deception > 0.0 && pup(Attr::HoldRunners).time_to_plate < p0.time_to_plate);
        assert!(pup(Attr::Stamina).stamina > p0.stamina && pup(Attr::Concentration).wildness < p0.wildness && pup(Attr::Composure).pressure < 0.0);
        // A better secondary pitch moves more and is located better.
        let mut better = arm.clone();
        better.pitching.as_mut().unwrap().pitches[1].grade = 1.0;
        let pb = pitcher(&better, false).unwrap();
        assert!(pb.pitches[1].shape.spin_rate > p0.pitches[1].shape.spin_rate && pb.pitches[1].command < p0.pitches[1].command);
        assert!(pb.pitches[1].disguise > 0.0 && p0.pitches[1].disguise == 0.0);
        assert_eq!(pb.pitches[0], p0.pitches[0]);
    }

    #[test]
    fn an_unfamiliar_position_costs_range_and_hands() {
        let mut p = average(false);
        let home = fielder(&p, Some(Position::Short));
        p.positions[Position::Short.index()] = 6;
        let lost = fielder(&p, Some(Position::Short));
        assert!(lost.reaction > home.reaction + 0.05 && lost.route < home.route && lost.hands > 1.5 * home.hands && lost.transfer > home.transfer);
        // A switch hitter is handed to the engine as one.
        p.bats = Bats::Switch;
        let b = batter(&p);
        assert!(b.switch && b.side_against(Hand::Left) == Hand::Right && b.side_against(Hand::Right) == Hand::Left);
    }

    #[test]
    fn players_survive_a_round_trip_through_json() {
        let mut p = average(true);
        p.attrs[Attr::Velocity] = 1.25;
        p.hidden.potential = 142.0;
        let text = serde_json::to_string(&p).unwrap();
        assert!(text.contains("\"velocity\":1.25") && text.contains("\"FF\""));
        let back: Player = serde_json::from_str(&text).unwrap();
        assert_eq!(back, p);
    }
}
