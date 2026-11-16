//! The game world: players with their attributes, the conversion of those attributes
//! into the physical profiles the match engine works with, and the leagues and clubs.

pub mod attr;
pub mod calendar;
pub mod convert;
pub mod generate;
pub mod league;
pub mod names;
pub mod player;

pub use attr::{display, Attr, Attributes, Group, Hidden, Scale, ScaleAnchor, N_ATTR};
pub use calendar::{Date, Weekday};
pub use convert::Baseline;
pub use generate::{Squad, Talent};
pub use league::{Ballpark, Club, Country, League, Level, World};
pub use player::{Bats, HitStyle, PitchSkill, Pitching, Player, Position, Role, Side};
