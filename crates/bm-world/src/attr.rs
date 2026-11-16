//! Player attributes.
//!
//! Every attribute is stored as a standard score: 0 is the average major-league
//! player of the relevant kind (position player for batting, running and fielding
//! attributes, pitcher for pitching attributes), 1 is one standard deviation of
//! that population, and positive is always better. The match engine never sees
//! these numbers; `convert` turns them into physical quantities (bat speed,
//! timing error, pitch velocity ...). The 1-20 and 20-80 ratings on the screen
//! are two ways of displaying the same standard score.

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

/// Number of scalar attributes shown to the user (the fortieth is the list of
/// pitch grades, one per pitch in a pitcher's repertoire).
pub const N_ATTR: usize = 39;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Attr {
    // ---- batting ----
    /// Barrel accuracy: how small the vertical miss between bat and ball is.
    Contact,
    /// Timing precision of the swing.
    Timing,
    /// Bat speed on a full swing.
    Power,
    /// Judging where the pitch will cross the plate, and laying off the ones outside the zone.
    Discipline,
    /// Picking up the pitch type out of the pitcher's hand.
    Recognition,
    /// Getting a piece of pitches the swing would otherwise miss; meeting the ball on the barrel's sweet spot.
    BatControl,
    Bunt,
    // ---- running ----
    /// Reads on the bases: how well he judges whether he can take the extra base.
    Baserunning,
    /// Leads and jumps when stealing.
    Stealing,
    // ---- fielding ----
    /// Sure hands: fewer fielding errors.
    Hands,
    /// First-step quickness after contact.
    Range,
    /// Reading the ball off the bat: routes, and the break on fly balls.
    Reads,
    ArmStrength,
    ArmAccuracy,
    /// Time from gathering the ball to releasing the throw.
    Footwork,
    // ---- catching ----
    Framing,
    Blocking,
    /// Pop time to second base.
    ThrowingOut,
    /// Game calling (not yet used by the match engine).
    GameCalling,
    // ---- pitching ----
    /// Fastball velocity.
    Velocity,
    /// Life on the main fastball: spin and the movement it produces.
    Stuff,
    /// Consistency of the release: the location error along the arm's swing (the big
    /// misses, high to the arm side or low to the glove side).
    Control,
    /// Consistency of the arm's path: the location error across the arm's swing
    /// (hitting the corners).
    Command,
    /// Hiding the ball, so that batters pick the pitch up later.
    Deception,
    /// Quickness to the plate with runners on.
    HoldRunners,
    // ---- mental ----
    /// Keeping precision when the game is on the line.
    Composure,
    /// Fewer lapses: badly misread pitches, pitches that get away, routine plays booted.
    Concentration,
    /// Decisions in the field and on the bases (used by the world layer; engine hook pending).
    BaseballIq,
    /// Adjusting to a pitch that was not what he expected.
    Anticipation,
    /// Drive to improve (development) and to compete.
    Determination,
    Leadership,
    Teamwork,
    /// Willingness to swing and to take the extra base.
    Aggression,
    // ---- physical ----
    /// Top running speed.
    Speed,
    /// Acceleration on the bases.
    Acceleration,
    /// Quickness in the field: burst in the first steps toward the ball.
    Agility,
    /// Keeping bat speed on swings that are fooled or cut short.
    Strength,
    /// Pitchers: pitches before fatigue sets in. Everyone: holding up over a season.
    Stamina,
    /// Resistance to injury and speed of recovery.
    Durability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Batting,
    Running,
    Fielding,
    Catching,
    Pitching,
    Mental,
    Physical,
}

impl Group {
    /// Stable identifier used as the translation key.
    pub fn key(self) -> &'static str {
        match self {
            Group::Batting => "batting",
            Group::Running => "running",
            Group::Fielding => "fielding",
            Group::Catching => "catching",
            Group::Pitching => "pitching",
            Group::Mental => "mental",
            Group::Physical => "physical",
        }
    }
}

impl Attr {
    pub const ALL: [Attr; N_ATTR] = [
        Attr::Contact,
        Attr::Timing,
        Attr::Power,
        Attr::Discipline,
        Attr::Recognition,
        Attr::BatControl,
        Attr::Bunt,
        Attr::Baserunning,
        Attr::Stealing,
        Attr::Hands,
        Attr::Range,
        Attr::Reads,
        Attr::ArmStrength,
        Attr::ArmAccuracy,
        Attr::Footwork,
        Attr::Framing,
        Attr::Blocking,
        Attr::ThrowingOut,
        Attr::GameCalling,
        Attr::Velocity,
        Attr::Stuff,
        Attr::Control,
        Attr::Command,
        Attr::Deception,
        Attr::HoldRunners,
        Attr::Composure,
        Attr::Concentration,
        Attr::BaseballIq,
        Attr::Anticipation,
        Attr::Determination,
        Attr::Leadership,
        Attr::Teamwork,
        Attr::Aggression,
        Attr::Speed,
        Attr::Acceleration,
        Attr::Agility,
        Attr::Strength,
        Attr::Stamina,
        Attr::Durability,
    ];

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn group(self) -> Group {
        use Attr::*;
        match self {
            Contact | Timing | Power | Discipline | Recognition | BatControl | Bunt => Group::Batting,
            Baserunning | Stealing => Group::Running,
            Hands | Range | Reads | ArmStrength | ArmAccuracy | Footwork => Group::Fielding,
            Framing | Blocking | ThrowingOut | GameCalling => Group::Catching,
            Velocity | Stuff | Control | Command | Deception | HoldRunners => Group::Pitching,
            Composure | Concentration | BaseballIq | Anticipation | Determination | Leadership | Teamwork | Aggression => Group::Mental,
            Speed | Acceleration | Agility | Strength | Stamina | Durability => Group::Physical,
        }
    }

    /// Stable identifier used in data files and as the translation key.
    pub fn key(self) -> &'static str {
        use Attr::*;
        match self {
            Contact => "contact",
            Timing => "timing",
            Power => "power",
            Discipline => "discipline",
            Recognition => "recognition",
            BatControl => "bat_control",
            Bunt => "bunt",
            Baserunning => "baserunning",
            Stealing => "stealing",
            Hands => "hands",
            Range => "range",
            Reads => "reads",
            ArmStrength => "arm_strength",
            ArmAccuracy => "arm_accuracy",
            Footwork => "footwork",
            Framing => "framing",
            Blocking => "blocking",
            ThrowingOut => "throwing_out",
            GameCalling => "game_calling",
            Velocity => "velocity",
            Stuff => "stuff",
            Control => "control",
            Command => "command",
            Deception => "deception",
            HoldRunners => "hold_runners",
            Composure => "composure",
            Concentration => "concentration",
            BaseballIq => "baseball_iq",
            Anticipation => "anticipation",
            Determination => "determination",
            Leadership => "leadership",
            Teamwork => "teamwork",
            Aggression => "aggression",
            Speed => "speed",
            Acceleration => "acceleration",
            Agility => "agility",
            Strength => "strength",
            Stamina => "stamina",
            Durability => "durability",
        }
    }
}

/// The scalar attributes of a player, as standard scores. Serialised as a map from
/// the attribute's key to its value, so that saved games survive attributes being
/// added or reordered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attributes(pub [f32; N_ATTR]);

impl Serialize for Attributes {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(Some(N_ATTR))?;
        for a in Attr::ALL {
            m.serialize_entry(a.key(), &self.0[a.index()])?;
        }
        m.end()
    }
}

impl<'de> Deserialize<'de> for Attributes {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let map: std::collections::BTreeMap<String, f32> = Deserialize::deserialize(d)?;
        let mut out = Attributes::default();
        for (key, value) in map {
            let a = Attr::ALL.into_iter().find(|a| a.key() == key).ok_or_else(|| serde::de::Error::custom(format!("unknown attribute {key:?}")))?;
            out[a] = value;
        }
        Ok(out)
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes([0.0; N_ATTR])
    }
}

impl Index<Attr> for Attributes {
    type Output = f32;
    #[inline]
    fn index(&self, a: Attr) -> &f32 {
        &self.0[a.index()]
    }
}

impl IndexMut<Attr> for Attributes {
    #[inline]
    fn index_mut(&mut self, a: Attr) -> &mut f32 {
        &mut self.0[a.index()]
    }
}

impl Attributes {
    /// Value as `f64`, for the conversion to physical quantities.
    #[inline]
    pub fn z(&self, a: Attr) -> f64 {
        self.0[a.index()] as f64
    }
}

/// Attributes the user never sees directly (scouts and coaches describe them in words).
/// Standard scores except where noted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hidden {
    /// How little his form varies from day to day.
    pub consistency: f32,
    /// Performance in games that decide something: the post-season, pennant deciders.
    pub big_games: f32,
    /// Positive = gets hurt less often.
    pub injury_resistance: f32,
    /// How quickly he learns a new position or role.
    pub versatility: f32,
    /// Settling in a new league, country or club.
    pub adaptability: f32,
    pub ambition: f32,
    pub loyalty: f32,
    /// Handling a pennant race, a contract year, the expectations of a big club.
    pub pressure: f32,
    pub professionalism: f32,
    /// Positive = even-tempered.
    pub temperament: f32,
    /// Positive = smaller than usual disadvantage against same-sided opponents.
    pub platoon: f32,
    /// Positive = develops early and declines early; negative = late bloomer who lasts.
    pub growth: f32,
    /// Potential ability, on the scale of [`crate::ability`] (not a standard score).
    pub potential: f32,
}

/// How ratings are displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    /// 1-20, the default.
    OneToTwenty,
    /// The scouting scale: 50 is the major-league average and 10 points are one
    /// standard deviation, shown in steps of 5.
    TwentyToEighty,
}

/// Anchors of the 1-20 display. The scouting scale needs none: it is defined by
/// the standard score itself.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaleAnchor {
    /// Rating shown for the average major-league player.
    pub average: f32,
    /// Rating points per standard deviation.
    pub per_sd: f32,
}

impl Default for ScaleAnchor {
    /// 20 is three standard deviations above the major-league average (the "80" of the
    /// scouting scale); the average major-leaguer is a 13, which leaves the lower part
    /// of the scale to the minor leagues.
    fn default() -> Self {
        ScaleAnchor { average: 13.0, per_sd: 7.0 / 3.0 }
    }
}

impl ScaleAnchor {
    /// The 1-20 scale as a straight relabelling of 20-80 (the average major-leaguer is 10.5).
    pub fn linear_with_scouting() -> Self {
        ScaleAnchor { average: 10.5, per_sd: 19.0 / 6.0 }
    }
}

/// Rating shown for a standard score.
pub fn display(z: f32, scale: Scale, anchor: ScaleAnchor) -> u8 {
    match scale {
        Scale::OneToTwenty => (anchor.average + anchor.per_sd * z).round().clamp(1.0, 20.0) as u8,
        Scale::TwentyToEighty => ((50.0 + 10.0 * z) / 5.0).round().clamp(4.0, 16.0) as u8 * 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_table_is_consistent() {
        for (i, a) in Attr::ALL.iter().enumerate() {
            assert_eq!(a.index(), i, "{:?} is out of order", a);
        }
        let mut keys: Vec<&str> = Attr::ALL.iter().map(|a| a.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), N_ATTR, "attribute keys must be unique");
        let count = |g: Group| Attr::ALL.iter().filter(|a| a.group() == g).count();
        assert_eq!(
            [
                count(Group::Batting),
                count(Group::Running),
                count(Group::Fielding),
                count(Group::Catching),
                count(Group::Pitching),
                count(Group::Mental),
                count(Group::Physical)
            ],
            [7, 2, 6, 4, 6, 8, 6]
        );
    }

    #[test]
    fn display_scales() {
        let a = ScaleAnchor::default();
        assert_eq!(display(0.0, Scale::OneToTwenty, a), 13);
        assert_eq!(display(3.0, Scale::OneToTwenty, a), 20);
        assert_eq!(display(5.0, Scale::OneToTwenty, a), 20);
        assert_eq!(display(-9.0, Scale::OneToTwenty, a), 1);
        assert_eq!(display(0.0, Scale::TwentyToEighty, a), 50);
        assert_eq!(display(1.0, Scale::TwentyToEighty, a), 60);
        assert_eq!(display(0.26, Scale::TwentyToEighty, a), 55);
        assert_eq!(display(-4.0, Scale::TwentyToEighty, a), 20);
        assert_eq!(display(4.0, Scale::TwentyToEighty, a), 80);
        let lin = ScaleAnchor::linear_with_scouting();
        assert_eq!(display(3.0, Scale::OneToTwenty, lin), 20);
        assert_eq!(display(-3.0, Scale::OneToTwenty, lin), 1);
    }
}
