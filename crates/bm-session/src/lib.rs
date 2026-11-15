//! Match session for the interface: one match, driven by commands, reported as
//! serialisable views. The desktop shell and the development server both sit on
//! [`App::call`].

pub mod api;
pub mod names;
pub mod session;
pub mod view;

pub use api::App;
pub use session::{AdvanceRequest, ApiError, Control, Delegate, Mode, NewMatch, RuleSet, Session, SubRequest};
