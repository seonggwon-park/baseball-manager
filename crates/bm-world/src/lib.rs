//! The game world: players with their attributes, and the conversion of those
//! attributes into the physical profiles the match engine works with.

pub mod attr;
pub mod convert;
pub mod generate;
pub mod player;

pub use attr::{display, Attr, Attributes, Group, Hidden, Scale, ScaleAnchor, N_ATTR};
pub use convert::Baseline;
pub use generate::{Squad, Talent};
pub use player::{Bats, HitStyle, PitchSkill, Pitching, Player, Position, Role, Side};
