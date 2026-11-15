//! Physical profiles of players as the match engine sees them.
//!
//! The engine does not know about 1-20 ratings; the world layer converts player
//! attributes into these physical quantities (bat speed, timing precision, sprint
//! speed, pitch velocity and spin, command ...). Everything is SI.

use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::BatSpec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// x-direction of a pitcher's arm side (+x is the first-base side).
    #[inline]
    pub fn arm_sign(self) -> f64 {
        match self {
            Hand::Right => -1.0,
            Hand::Left => 1.0,
        }
    }
    /// x-direction of "away from the batter" for a batter of this handedness.
    #[inline]
    pub fn outside_sign(self) -> f64 {
        match self {
            Hand::Right => 1.0,
            Hand::Left => -1.0,
        }
    }
    /// x-direction of the batter's pull field.
    #[inline]
    pub fn pull_sign(self) -> f64 {
        -self.outside_sign()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PitchKind {
    FourSeam,
    Sinker,
    Cutter,
    Slider,
    Sweeper,
    Curve,
    Changeup,
    Splitter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchGroup {
    Fastball,
    Breaking,
    Offspeed,
}

impl PitchKind {
    pub const ALL: [PitchKind; 8] = [
        PitchKind::FourSeam,
        PitchKind::Sinker,
        PitchKind::Cutter,
        PitchKind::Slider,
        PitchKind::Sweeper,
        PitchKind::Curve,
        PitchKind::Changeup,
        PitchKind::Splitter,
    ];

    pub fn group(self) -> PitchGroup {
        match self {
            PitchKind::FourSeam | PitchKind::Sinker | PitchKind::Cutter => PitchGroup::Fastball,
            PitchKind::Slider | PitchKind::Sweeper | PitchKind::Curve => PitchGroup::Breaking,
            PitchKind::Changeup | PitchKind::Splitter => PitchGroup::Offspeed,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn code(self) -> &'static str {
        match self {
            PitchKind::FourSeam => "FF",
            PitchKind::Sinker => "SI",
            PitchKind::Cutter => "FC",
            PitchKind::Slider => "SL",
            PitchKind::Sweeper => "ST",
            PitchKind::Curve => "CU",
            PitchKind::Changeup => "CH",
            PitchKind::Splitter => "FS",
        }
    }

    /// League-average shape of the pitch (2025 MLB, Statcast sample), hand-normalised.
    /// The spin efficiency and tilt were solved so that the flight model reproduces
    /// the measured average movement (`bm-physics/examples/pitch_shapes.rs`).
    pub fn league_average(self) -> PitchShape {
        // (mph, rpm, active spin fraction, tilt toward the arm side in degrees)
        let (mph, rpm, active, tilt): (f64, f64, f64, f64) = match self {
            PitchKind::FourSeam => (94.5, 2321.0, 0.868, 26.0),
            PitchKind::Sinker => (93.8, 2186.0, 0.867, 67.0),
            PitchKind::Cutter => (89.9, 2413.0, 0.249, -16.9),
            PitchKind::Slider => (86.3, 2426.0, 0.112, -83.0),
            PitchKind::Sweeper => (82.5, 2603.0, 0.543, -92.2),
            PitchKind::Curve => (79.9, 2600.0, 0.575, -143.5),
            PitchKind::Changeup => (86.0, 1809.0, 0.830, 76.5),
            PitchKind::Splitter => (86.4, 1320.0, 0.732, 77.4),
        };
        PitchShape { speed: mph * MPH, spin_rate: rpm * RPM, active, tilt: tilt.to_radians() }
    }

    /// Typical intended location: (x toward the outside edge in m, height as a
    /// fraction of the strike zone from its bottom), from the mean plate locations
    /// in the Statcast sample.
    pub fn typical_target(self) -> (f64, f64) {
        match self {
            PitchKind::FourSeam => (0.14 * FT, 0.69),
            PitchKind::Sinker => (-0.03 * FT, 0.42),
            PitchKind::Cutter => (0.07 * FT, 0.51),
            PitchKind::Slider => (0.27 * FT, 0.18),
            PitchKind::Sweeper => (0.41 * FT, 0.22),
            PitchKind::Curve => (0.13 * FT, 0.14),
            PitchKind::Changeup => (0.35 * FT, 0.125),
            PitchKind::Splitter => (0.24 * FT, 0.09),
        }
    }
}

/// Speed and spin of a pitch, hand-normalised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchShape {
    /// Release speed, m/s.
    pub speed: f64,
    /// Spin rate, rad/s.
    pub spin_rate: f64,
    /// Active (transverse) spin fraction, 0..=1.
    pub active: f64,
    /// Direction of the Magnus force measured from straight up toward the arm side, rad.
    pub tilt: f64,
}

/// One pitch in a pitcher's arsenal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchDef {
    pub kind: PitchKind,
    pub shape: PitchShape,
    /// Usage weight against same-handed batters.
    pub usage_same: f64,
    /// Usage weight against opposite-handed batters.
    pub usage_opp: f64,
    /// Command: standard deviation of the plate location error per axis, m.
    pub command: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PitcherProfile {
    pub hand: Hand,
    /// Release point: distance toward the arm side of the rubber's centre line, m.
    pub release_side: f64,
    /// Release height, m.
    pub release_height: f64,
    /// Extension in front of the rubber, m.
    pub extension: f64,
    pub pitches: Vec<PitchDef>,
    /// Pitch-to-pitch scatter of release speed, m/s.
    pub speed_sd: f64,
    /// Pitch-to-pitch scatter of spin rate, rad/s.
    pub spin_sd: f64,
    /// Pitch-to-pitch scatter of spin tilt, rad.
    pub tilt_sd: f64,
    /// Number of pitches before fatigue starts to cost velocity and command.
    pub stamina: f64,
    /// Time from first movement to the plate with runners on, s.
    pub time_to_plate: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatterProfile {
    pub hand: Hand,
    /// Standing height, m (sets the strike zone).
    pub height: f64,
    /// Mean bat speed at the sweet spot on a full swing, m/s.
    pub bat_speed: f64,
    /// Swing-to-swing scatter of bat speed, m/s.
    pub bat_speed_sd: f64,
    /// Pitch of the swing plane toward the pitcher: the attack angle of a swing that
    /// meets the ball square (pulled contact is steeper, opposite-field contact flatter), rad.
    pub attack_angle: f64,
    pub attack_sd: f64,
    /// Timing precision: SD of the timing error on a recognised pitch, s.
    pub timing_sd: f64,
    /// Barrel precision: SD of the vertical bat-ball offset on a recognised pitch, m.
    pub barrel_sd: f64,
    /// Where the batter aims the barrel relative to the centre of the ball, m
    /// (positive = under the ball, for loft).
    pub barrel_aim: f64,
    /// SD of the contact position along the bat, m.
    pub reach_sd: f64,
    /// Probability of picking up the pitch type out of the hand (0..1).
    pub eye: f64,
    /// SD of the perceived plate location, m.
    pub perception_sd: f64,
    /// Shift of the swing/take boundary: positive expands the zone (chases more), m.
    pub chase: f64,
    /// Multiplier on the willingness to swing at hittable pitches (1 = average).
    pub aggression: f64,
    /// Systematic timing bias, s (negative = early, pulls the ball).
    pub timing_bias: f64,
    /// Top running speed, m/s.
    pub sprint: f64,
    /// Running acceleration, m/s^2.
    pub accel: f64,
    /// Base-running aggressiveness, 0..=1.
    pub baserunning: f64,
    /// Inclination and ability to steal, 0..=1.
    pub steal: f64,
    /// Bunting skill, 0..=1 (0.5 = average position player).
    pub bunt: f64,
    pub bat: BatSpec,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FielderProfile {
    /// Time from contact to the first step, s.
    pub reaction: f64,
    /// Acceleration, m/s^2.
    pub accel: f64,
    /// Top speed, m/s.
    pub speed: f64,
    /// Extra time needed to read a ball in the air before breaking on it, s.
    pub read_delay: f64,
    /// Acceleration while chasing a ball in the air (long strides, eyes on the ball), m/s^2.
    pub fly_accel: f64,
    /// Route efficiency (1 = straight line to the ball).
    pub route: f64,
    /// Sure-handedness: multiplier on the error probability (1 = average, lower is better).
    pub hands: f64,
    /// Throwing speed at release on a full-effort throw, m/s.
    pub arm: f64,
    /// Time from fielding the ball to releasing a throw, s.
    pub transfer: f64,
    /// Multiplier on the probability of a wild throw (1 = average).
    pub arm_accuracy: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CatcherProfile {
    /// Pop time to second base, s.
    pub pop_time: f64,
    /// Multiplier on the chance that a pitch in the dirt gets by (1 = average).
    pub blocking: f64,
    /// Called-strike gain at the edges under a human umpire (0 = average), m of zone shift.
    pub framing: f64,
}

/// Position code for a player without a defensive position (designated hitter, utility).
pub const NO_POSITION: usize = 9;

/// A player as used by the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u32,
    /// Primary position: an index into the defensive alignment (0 = P ... 8 = RF) or `NO_POSITION`.
    pub position: usize,
    pub batter: BatterProfile,
    pub fielder: FielderProfile,
    pub pitcher: Option<PitcherProfile>,
    pub catcher: CatcherProfile,
}

impl PitcherProfile {
    /// League-average right-hander with a four-pitch mix (used in tests and as a
    /// template for population generation).
    pub fn league_average(hand: Hand) -> Self {
        let mk = |kind: PitchKind, same: f64, opp: f64, cmd_in: f64| PitchDef {
            kind,
            shape: kind.league_average(),
            usage_same: same,
            usage_opp: opp,
            command: cmd_in * INCH,
        };
        PitcherProfile {
            hand,
            release_side: 1.9 * FT,
            release_height: 5.8 * FT,
            extension: 6.45 * FT,
            pitches: vec![
                mk(PitchKind::FourSeam, 0.48, 0.50, 6.6),
                mk(PitchKind::Slider, 0.34, 0.18, 7.6),
                mk(PitchKind::Changeup, 0.08, 0.22, 7.4),
                mk(PitchKind::Curve, 0.10, 0.10, 8.0),
            ],
            speed_sd: 1.05 * MPH,
            spin_sd: 85.0 * RPM,
            tilt_sd: 4.5f64.to_radians(),
            stamina: 95.0,
            time_to_plate: 1.38,
        }
    }
}

impl BatterProfile {
    pub fn league_average(hand: Hand) -> Self {
        BatterProfile {
            hand,
            height: 1.855,
            bat_speed: 71.9 * MPH,
            bat_speed_sd: 2.8 * MPH,
            attack_angle: 9.5f64.to_radians(),
            attack_sd: 2.5f64.to_radians(),
            timing_sd: 0.0175,
            barrel_sd: 0.019,
            barrel_aim: 0.0155,
            reach_sd: 0.06,
            eye: 0.62,
            perception_sd: 0.065,
            chase: 0.0,
            aggression: 1.0,
            timing_bias: -0.006,
            sprint: 27.0 * FT,
            accel: 4.9,
            baserunning: 0.5,
            steal: 0.3,
            bunt: 0.5,
            bat: BatSpec::mlb_wood(),
        }
    }

    /// Rough offensive value used by the AI manager to compare hitters: bat speed,
    /// barrel control and pitch recognition (about 1.0 for an average hitter).
    pub fn quality(&self) -> f64 {
        self.bat_speed / (71.9 * MPH) - 0.6 * self.barrel_sd / 0.034 + 0.5 * self.eye + 0.03
    }
}

impl FielderProfile {
    pub fn league_average() -> Self {
        FielderProfile {
            reaction: 0.30,
            accel: 4.6,
            speed: 27.5 * FT,
            read_delay: 0.0,
            fly_accel: 4.6,
            route: 0.95,
            hands: 1.0,
            arm: 84.0 * MPH,
            transfer: 0.80,
            arm_accuracy: 1.0,
        }
    }
}

impl CatcherProfile {
    pub fn league_average() -> Self {
        CatcherProfile { pop_time: 1.97, blocking: 1.0, framing: 0.0 }
    }
}

impl Player {
    pub fn league_average(id: u32, bats: Hand) -> Self {
        Player {
            id,
            position: NO_POSITION,
            batter: BatterProfile::league_average(bats),
            fielder: FielderProfile::league_average(),
            pitcher: None,
            catcher: CatcherProfile::league_average(),
        }
    }

    pub fn league_average_pitcher(id: u32, throws: Hand) -> Self {
        let mut p = Player::league_average(id, throws);
        p.position = 0;
        p.pitcher = Some(PitcherProfile::league_average(throws));
        // Pitchers hit and field like non-specialists.
        p.batter.bat_speed *= 0.93;
        p.batter.barrel_sd *= 1.45;
        p.batter.timing_sd *= 1.35;
        p.batter.eye = 0.55;
        p
    }
}
