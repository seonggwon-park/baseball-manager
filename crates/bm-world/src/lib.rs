//! The game world: players with their attributes, the conversion of those attributes
//! into the physical profiles the match engine works with, and the leagues and clubs.

pub mod ability;
pub mod attr;
pub mod birthplace;
pub mod calendar;
pub mod career;
pub mod contract;
pub mod convert;
pub mod develop;
pub mod generate;
pub mod league;
pub mod market;
pub mod names;
pub mod people;
pub mod player;
pub mod populate;
pub mod schedule;
pub mod season;

pub use attr::{display, Attr, Attributes, Group, Hidden, Scale, ScaleAnchor, N_ATTR};
pub use birthplace::Birthplaces;
pub use calendar::{Date, Weekday};
pub use career::{new_world, offseason, organisations, Career, CareerTable, Offseason, Org, OrgKind, Origin};
pub use contract::{Budgets, Contract, ContractRules, Economy};
pub use convert::Baseline;
pub use develop::{develop, develop_idle, AgingTable, Closure, Potential};
pub use generate::{Squad, Talent};
pub use league::{Ballpark, Club, Country, League, Level, World};
pub use people::{AgeSpread, PeopleMaker, Person};
pub use player::{Bats, HitStyle, PitchSkill, Pitching, Player, Position, Role, Side};
pub use populate::{populate, populate_with, Away, LevelProfile, LevelTable, Population, Roster};
pub use season::{Fixture, Season, Standing};
