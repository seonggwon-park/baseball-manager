//! A player as the game world knows him: attributes, body, and the physical
//! "signature" of his swing and of each of his pitches.

use crate::attr::{Attributes, Hidden};
use bm_engine::{Hand, PitchKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn hand(self) -> Hand {
        match self {
            Side::Left => Hand::Left,
            Side::Right => Hand::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bats {
    Left,
    Right,
    Switch,
}

/// Fielding positions in the order of the engine's defensive alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Position {
    #[serde(rename = "P")]
    Pitcher,
    #[serde(rename = "C")]
    Catcher,
    #[serde(rename = "1B")]
    First,
    #[serde(rename = "2B")]
    Second,
    #[serde(rename = "3B")]
    Third,
    #[serde(rename = "SS")]
    Short,
    #[serde(rename = "LF")]
    Left,
    #[serde(rename = "CF")]
    Center,
    #[serde(rename = "RF")]
    Right,
}

impl Position {
    pub const ALL: [Position; 9] = [
        Position::Pitcher,
        Position::Catcher,
        Position::First,
        Position::Second,
        Position::Third,
        Position::Short,
        Position::Left,
        Position::Center,
        Position::Right,
    ];

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Position> {
        Position::ALL.get(i).copied()
    }

    pub fn code(self) -> &'static str {
        ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"][self.index()]
    }

    pub fn is_infield(self) -> bool {
        matches!(self, Position::First | Position::Second | Position::Third | Position::Short)
    }

    pub fn is_outfield(self) -> bool {
        matches!(self, Position::Left | Position::Center | Position::Right)
    }
}

/// How well a player knows a position, 0..=20. A natural at the position is 20; at
/// 10 he can fill in; below 5 he is lost.
pub type Familiarity = u8;

/// The shape of a hitter's swing. These are tendencies rather than abilities: they
/// decide what kind of hitter he is (fly balls or grounders, pull or all fields).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HitStyle {
    /// Upward pitch of the swing plane, rad.
    pub attack_angle: f32,
    /// Swing-to-swing scatter of the attack angle, rad.
    pub attack_sd: f32,
    /// How far under the centre of the ball he aims the barrel, m.
    pub barrel_aim: f32,
    /// Systematic timing bias, s (negative = early, pulls the ball).
    pub timing_bias: f32,
    /// Swing-to-swing scatter of bat speed, m/s.
    pub bat_speed_sd: f32,
}

/// One pitch of a repertoire.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchSkill {
    #[serde(with = "pitch_code")]
    pub kind: PitchKind,
    /// Quality of the pitch as a standard score (0 = the average major-league pitch of
    /// its type). For the main fastball the pitcher's `Stuff` attribute is used instead.
    pub grade: f32,
    /// Personal shape of the pitch: deviations from the league-average pitch of its
    /// type that make it his own without making it better or worse.
    /// Release speed, m/s; spin rate, rad/s; active-spin fraction; spin tilt, rad.
    pub speed: f32,
    pub spin: f32,
    pub active: f32,
    pub tilt: f32,
    /// How much he likes to throw it (relative weight, before the batter's side is taken into account).
    pub usage: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Starter,
    Reliever,
}

/// Everything about a player as a pitcher that is not an attribute.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pitching {
    pub role: Role,
    /// Arm angle at release, rad (0 = sidearm, positive = over the top).
    pub arm_angle: f32,
    /// Extension in front of the rubber, m.
    pub extension: f32,
    /// Pitch-to-pitch scatter of release speed (m/s), spin rate (rad/s) and spin tilt (rad).
    pub speed_sd: f32,
    pub spin_sd: f32,
    pub tilt_sd: f32,
    pub pitches: Vec<PitchSkill>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    /// Height, m.
    pub height: f32,
    /// Weight, kg.
    pub weight: f32,
    pub bats: Bats,
    pub throws: Side,
    /// Primary position.
    pub position: Position,
    /// Familiarity with each position, in the order of [`Position::ALL`].
    pub positions: [Familiarity; 9],
    pub attrs: Attributes,
    pub hidden: Hidden,
    pub hit: HitStyle,
    /// `None` for a position player who does not pitch.
    pub pitching: Option<Pitching>,
}

impl Player {
    pub fn is_pitcher(&self) -> bool {
        self.position == Position::Pitcher
    }

    pub fn familiarity(&self, pos: Position) -> f64 {
        self.positions[pos.index()] as f64 / 20.0
    }
}

/// Serialise a pitch type as its two-letter code ("FF", "SL" ...).
mod pitch_code {
    use bm_engine::PitchKind;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(k: &PitchKind, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(k.code())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<PitchKind, D::Error> {
        let code = String::deserialize(d)?;
        PitchKind::ALL.into_iter().find(|k| k.code() == code).ok_or_else(|| serde::de::Error::custom(format!("unknown pitch type {code:?}")))
    }
}
