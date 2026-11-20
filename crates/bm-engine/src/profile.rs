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
    #[inline]
    pub fn opposite(self) -> Hand {
        match self {
            Hand::Right => Hand::Left,
            Hand::Left => Hand::Right,
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

    /// Typical intended location against a batter on the pitcher's own side
    /// (`same_side`) or the other side: (x toward the outside edge in m, height as a
    /// fraction of the strike zone from its bottom). The sideways values are the mean
    /// plate locations in the 2025 Statcast sample by the batter's side: pitches that
    /// move to the glove side go away from same-side batters and in on opposite-side
    /// ones (cutters +0.46 and -0.19 ft, sliders +0.54 and -0.13 ft), and sinkers,
    /// changeups and splitters the other way round. The heights put the pitches at the
    /// sample's mean plate heights (FF 2.87, SI 2.38, FC 2.53, SL 1.93, ST 2.01, CU 1.85,
    /// CH 1.83, FS 1.77 ft) once the count's intent and the pitcher's command have been
    /// applied: sinkers, breaking balls and offspeed pitches are buried in pitchers'
    /// counts, so they are aimed a little above those means on average.
    pub fn typical_target(self, same_side: bool) -> (f64, f64) {
        let (same, opposite, height) = match self {
            PitchKind::FourSeam => (0.161, 0.123, 0.69),
            PitchKind::Sinker => (-0.161, 0.165, 0.459),
            PitchKind::Cutter => (0.458, -0.194, 0.51),
            PitchKind::Slider => (0.543, -0.127, 0.217),
            PitchKind::Sweeper => (0.642, -0.008, 0.27),
            PitchKind::Curve => (0.338, 0.040, 0.203),
            PitchKind::Changeup => (-0.050, 0.449, 0.174),
            PitchKind::Splitter => (-0.059, 0.357, 0.14),
        };
        let x = if same_side { same } else { opposite };
        (x * FT, height)
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
    /// Command: root-mean-square of the plate location error over its two axes, m
    /// (see [`PitcherProfile::error_ratio`] for how it is split between them).
    pub command: f64,
    /// How late batters pick this pitch up (arm speed that sells a change-up, a
    /// breaking ball that looks like the fastball out of the hand): subtracted from
    /// their chance of recognising it (0 = average).
    pub disguise: f64,
    /// Where he likes to throw it: added to the intended height, m (0 = the usual
    /// spot for the pitch type, positive = higher).
    pub aim_height: f64,
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
    /// Shape of the location error. Pitches scatter in an ellipse whose long axis lies
    /// along the arm's swing (released early the ball sails high to the arm side, late
    /// it is pulled low to the glove side); this is the ratio of the error along that
    /// axis to the error across it.
    pub error_ratio: f64,
    /// Multiplier on how often a pitch gets away from him altogether (1 = average).
    pub wildness: f64,
    /// How well he hides the ball: subtracted from the batter's chance of picking up
    /// the pitch type out of the hand (0 = average).
    pub deception: f64,
    /// Relative change of the command error at full pressure (0 = unaffected,
    /// positive = worse when the game is on the line).
    pub pressure: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatterProfile {
    /// Side of the plate he bats from (against a right-hander, for a switch hitter).
    pub hand: Hand,
    /// Bats from the side opposite to the pitcher's throwing hand.
    pub switch: bool,
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
    /// Bat control: multiplier on the chance of getting a piece of a pitch the swing
    /// would otherwise miss narrowly (1 = average).
    pub foul_skill: f64,
    /// Adjustment to a pitch he did not pick up out of the hand: multiplier on how much
    /// of it he still recognises on the way (1 = average).
    pub adjust: f64,
    /// Multiplier on how often he badly misreads a pitch's location (1 = average).
    pub lapse: f64,
    /// Multiplier on the bat speed lost on swings he is fooled on or cuts short
    /// (1 = average, lower = stronger).
    pub speed_loss: f64,
    /// Multiplier on the error of his reads on the bases (1 = average, lower is better).
    pub judgement: f64,
    /// Added to `eye` against pitchers who throw from his own side (0 = average).
    pub eye_same: f64,
    /// Relative change of the timing and barrel errors at full pressure (0 = unaffected,
    /// positive = worse when the game is on the line).
    pub pressure: f64,
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

/// Ratio of the location error along the arm's swing to the error across it for the
/// average pitcher. With this value the engine reproduces the ellipse of four-seam
/// fastball locations in the 2025 Statcast sample (25.9 cm along, 19.1 cm across,
/// intended locations included).
pub const AVERAGE_ERROR_RATIO: f64 = 1.6;

impl PitcherProfile {
    /// Arm angle at release, rad: 0 is sidearm, positive is over the top. Estimated
    /// from the release point as pitch tracking does (ball relative to the shoulder).
    pub fn arm_angle(&self) -> f64 {
        (self.release_height - 1.25).atan2((self.release_side + 0.05).max(0.05))
    }

    /// Direction of the long axis of the location error at the plate, as the angle
    /// above horizontal of the line that runs from low on the glove side to high on
    /// the arm side. Measured on four-seam fastballs in the 2025 Statcast sample: 34 deg
    /// for sidearm deliveries rising to 72 deg over the top, about 22 deg + 0.85 x arm angle.
    pub fn error_axis(&self) -> f64 {
        (22.0f64.to_radians() + 0.85 * self.arm_angle()).clamp(0.0, 1.5)
    }

    /// League-average right-hander with a four-pitch mix (used in tests and as a
    /// template for population generation).
    pub fn league_average(hand: Hand) -> Self {
        let mk = |kind: PitchKind, same: f64, opp: f64, cmd_in: f64| PitchDef {
            kind,
            shape: kind.league_average(),
            usage_same: same,
            usage_opp: opp,
            command: cmd_in * INCH,
            disguise: 0.0,
            aim_height: 0.0,
        };
        PitcherProfile {
            hand,
            release_side: 1.9 * FT,
            release_height: 5.8 * FT,
            extension: 6.45 * FT,
            pitches: vec![
                mk(PitchKind::FourSeam, 0.48, 0.50, 6.45),
                mk(PitchKind::Slider, 0.34, 0.18, 7.43),
                mk(PitchKind::Changeup, 0.08, 0.22, 7.23),
                mk(PitchKind::Curve, 0.10, 0.10, 7.82),
            ],
            speed_sd: 1.05 * MPH,
            spin_sd: 85.0 * RPM,
            tilt_sd: 4.5f64.to_radians(),
            stamina: 95.0,
            time_to_plate: 1.38,
            error_ratio: AVERAGE_ERROR_RATIO,
            wildness: 1.0,
            deception: 0.0,
            pressure: 0.0,
        }
    }
}

/// Bat speed, m/s, and barrel error, m, of the average hitter of the generated MLB population.
pub const AVERAGE_BAT_SPEED: f64 = 75.49 * MPH;
pub const AVERAGE_BARREL_SD: f64 = 0.025219;

impl BatterProfile {
    pub fn league_average(hand: Hand) -> Self {
        BatterProfile {
            hand,
            switch: false,
            height: 1.855,
            bat_speed: AVERAGE_BAT_SPEED,
            bat_speed_sd: 2.8 * MPH,
            attack_angle: 9.5f64.to_radians(),
            attack_sd: 3.9f64.to_radians(),
            timing_sd: 0.01078,
            barrel_sd: AVERAGE_BARREL_SD,
            barrel_aim: 0.035013,
            reach_sd: 0.050000,
            eye: 0.613,
            perception_sd: 0.026608,
            chase: -0.011,
            aggression: 1.0,
            timing_bias: 0.0026,
            sprint: 27.0 * FT,
            accel: 4.9,
            baserunning: 0.5,
            steal: 0.3,
            bunt: 0.5,
            foul_skill: 1.0,
            adjust: 1.0,
            lapse: 1.0,
            speed_loss: 1.0,
            judgement: 1.0,
            eye_same: 0.0,
            pressure: 0.0,
            bat: BatSpec::mlb_wood(),
        }
    }

    /// Side of the plate he takes against a pitcher who throws with `pitcher`.
    pub fn side_against(&self, pitcher: Hand) -> Hand {
        if self.switch {
            pitcher.opposite()
        } else {
            self.hand
        }
    }

    /// The batter as he is against a given pitcher with `pressure` (0..=1) on the
    /// plate appearance: deception and the pitcher's side change what he picks up,
    /// pressure changes how precisely he swings.
    pub fn facing(&self, pitcher: &PitcherProfile, pressure: f64) -> BatterProfile {
        let mut b = self.clone();
        b.hand = self.side_against(pitcher.hand);
        let same = if pitcher.hand == b.hand { self.eye_same } else { 0.0 };
        b.eye = (self.eye - pitcher.deception + same).clamp(0.05, 0.99);
        let noise = (1.0 + pressure * self.pressure).max(0.5);
        b.timing_sd *= noise;
        b.barrel_sd *= noise;
        b
    }

    /// Rough offensive value used by the AI manager to compare hitters: bat speed,
    /// barrel control and pitch recognition (about 1.0 for an average hitter).
    pub fn quality(&self) -> f64 {
        self.bat_speed / AVERAGE_BAT_SPEED - 0.335 * self.barrel_sd / AVERAGE_BARREL_SD + 0.5 * self.eye + 0.03
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
