//! A reinforcement-learning environment on the match engine.
//!
//! The agent manages one club's offence: before every pitch to one of its hitters it
//! chooses a batting approach (hit away, sacrifice bunt, bunt for a hit, hit and run,
//! take) and a steal order (the runner's own call, hold, go). The engine's AI managers run
//! everything else. The interface follows Gymnasium's (`reset`, `step` with `terminated`
//! and `truncated`, an action mask), so that the Python binding in `python/` is thin.
//!
//! Results learned here are about the engine: a policy that exploits a flaw of the model
//! is not a lesson about baseball (docs/rl.md).

pub mod env;
pub mod league;
pub mod obs;
pub mod we;

pub use env::{action_name, coerce, decode, encode, mask, EnvConfig, Info, ManagerEnv, Reward, StepResult, APPROACHES, N_ACTIONS, STEALS};
pub use league::{Fixture, League};
pub use obs::{observe, NAMES as OBS_NAMES, N_OBS};
pub use we::WinExpectancy;
