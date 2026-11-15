//! Physics core of Baseball Manager.
//!
//! All quantities are SI. Coordinates: origin at the back tip of home plate,
//! `x` toward the catcher's right, `y` toward the pitcher, `z` up.
//!
//! The sources for every model and constant are listed in `docs/physics-model.md`.

pub mod ball;
pub mod bat;
pub mod batted;
pub mod collision;
pub mod env;
pub mod flight;
pub mod ground;
pub mod park;
pub mod pitch;
pub mod units;
pub mod vec3;

pub use ball::{AeroModel, BallSpec, SpinMode};
pub use bat::BatSpec;
pub use batted::{BattedPath, BattedSim, PathEnd, PathPoint};
pub use collision::{collide, launch_angle, spin_components, spray_angle, Contact, ContactResult, NoContact};
pub use env::{Atmosphere, Wind};
pub use flight::{spin_from_components, velocity_from_angles, BallState, FlightModel};
pub use ground::{bounce, Surface};
pub use park::{base_position, bearing, is_fair, FencePoint, Park, SurfaceKind, BASE_PATH, MOUND_Y};
pub use pitch::{AimModel, PitchFlight, PitchSpec, ReleaseAngles, PLATE_FRONT_Y, PLATE_HALF_WIDTH, RUBBER_Y};
pub use vec3::Vec3;
