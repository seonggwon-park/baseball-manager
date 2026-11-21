//! Pitch-by-pitch baseball match engine built on `bm-physics`.

pub mod batting;
pub mod fielding;
pub mod game;
pub mod params;
pub mod pitching;
pub mod play;
pub mod population;
pub mod profile;
pub mod rng;
pub mod rules;

pub use batting::{Approach, BattedInfo, Miss, Swing, SwingShape};
pub use fielding::{Alignment, InfieldDepth, OutfieldDepth, Shift};
pub use game::{
    play_game, play_game_styled, Auto, BatLine, DefenseOrders, GameEvent, GameResult, GameSim, ManagerStyle, NoObserver, Observer, OffenseOrders, OrderError,
    PaEvent, PaOutcome, PitchEvent, PitchLine, PitchResult, Situation, StealOrder, Step, Substitution, Team, TeamBox, BATTED_DT, BATTED_SAMPLE_DT,
};
pub use params::ModelParams;
pub use pitching::{Intent, MatchEnv, PitchOrder, PitcherState, Tendency, Thrown};
pub use play::{PlayKind, PlayResult, Runner};
pub use population::{Club, Level};
pub use profile::{BatterProfile, CatcherProfile, FielderProfile, Hand, PitchDef, PitchGroup, PitchKind, PitcherProfile, Player};
pub use rng::Rng;
pub use rules::{Rules, StrikeZone, ZoneCall, ZonePreset, ZONE_BALL_RADIUS};
