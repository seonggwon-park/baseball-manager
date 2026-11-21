//! Pitched-ball kinematics: release description, flight to the plate, aiming.

use crate::flight::{spin_from_components, BallState, FlightModel};
use crate::units::{G, INCH};
use crate::vec3::Vec3;
use std::f64::consts::PI;

/// y-coordinate of the front edge of home plate (17 in from the back tip).
/// Statcast reports `plate_x` / `plate_z` at this plane.
pub const PLATE_FRONT_Y: f64 = 17.0 * INCH;
/// Half-width of home plate.
pub const PLATE_HALF_WIDTH: f64 = 8.5 * INCH;
/// Distance from the back tip of home plate to the front of the pitching rubber (60 ft 6 in).
pub const RUBBER_Y: f64 = 60.5 * 0.3048;

/// Physical description of a pitch at release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchSpec {
    /// Release point, m.
    pub release_pos: Vec3,
    /// Release speed, m/s.
    pub speed: f64,
    /// Total spin rate, rad/s.
    pub spin_rate: f64,
    /// Direction of the Magnus force in the plane perpendicular to the velocity,
    /// measured from straight up toward +x (catcher's right). 0 = pure backspin
    /// ("rise"), pi = pure topspin (drop), +pi/2 = breaks toward +x.
    pub spin_tilt: f64,
    /// Fraction of the spin that is transverse to the velocity ("active spin"), 0..=1.
    pub spin_active: f64,
    /// Sign of the gyro component: +1 = spin vector along the direction of flight
    /// (clockwise seen from the pitcher, a right-hander's slider), -1 = opposite.
    pub gyro_sign: f64,
}

/// Release direction. `vert` is the vertical angle (+ up); `horiz` is the
/// horizontal angle away from the -y axis (+ toward +x, the catcher's right).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ReleaseAngles {
    pub vert: f64,
    pub horiz: f64,
}

/// Result of flying a pitch to the front of home plate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchFlight {
    /// Position when crossing the front plane of the plate (y = [`PLATE_FRONT_Y`]).
    pub plate_pos: Vec3,
    pub plate_vel: Vec3,
    pub spin: Vec3,
    /// Time from release to the plate, s.
    pub time: f64,
    pub release_vel: Vec3,
    /// Spin-induced horizontal movement, m (+x = catcher's right): deviation from a
    /// straight line in x over the whole flight.
    pub move_x: f64,
    /// Spin-induced vertical movement, m: deviation from a drag-free, spin-free
    /// parabola (the "induced vertical break" over the whole flight).
    pub move_z: f64,
}

impl PitchSpec {
    /// Velocity and spin vectors for the given release angles.
    pub fn initial_state(&self, a: ReleaseAngles) -> BallState {
        let (sv, cv) = a.vert.sin_cos();
        let (sh, ch) = a.horiz.sin_cos();
        let vel = Vec3::new(self.speed * cv * sh, -self.speed * cv * ch, self.speed * sv);
        // Nathan's phi: direction measured from +y toward +x. For a pitch phi = pi - horiz.
        let phi = PI - a.horiz;
        let active = self.spin_active.clamp(0.0, 1.0);
        let wt = self.spin_rate * active;
        let wg = self.gyro_sign * self.spin_rate * (1.0 - active * active).max(0.0).sqrt();
        let spin = spin_from_components(a.vert, phi, wt * self.spin_tilt.cos(), wt * self.spin_tilt.sin(), wg);
        BallState::new(self.release_pos, vel, spin)
    }

    /// Fly the pitch to the front plane of home plate.
    pub fn fly(&self, model: &FlightModel, a: ReleaseAngles, dt: f64) -> PitchFlight {
        let init = self.initial_state(a);
        let mut s = init;
        model.advance_until(&mut s, dt, 3.0, |b| b.pos.y - PLATE_FRONT_Y, |_| {});
        let t = s.t;
        PitchFlight {
            plate_pos: s.pos,
            plate_vel: s.vel,
            spin: s.spin,
            time: t,
            release_vel: init.vel,
            move_x: s.pos.x - (init.pos.x + init.vel.x * t),
            move_z: s.pos.z - (init.pos.z + init.vel.z * t - 0.5 * G * t * t),
        }
    }

    /// Fly the pitch and also report its state at an arbitrary plane `y = y_plane`
    /// (e.g. the contact point in front of the plate). Returns `(state_at_plane, flight_to_plate)`.
    pub fn fly_with_plane(&self, model: &FlightModel, a: ReleaseAngles, dt: f64, y_plane: f64) -> (BallState, PitchFlight) {
        let init = self.initial_state(a);
        let mut s = init;
        let mut at_plane = init;
        if y_plane > PLATE_FRONT_Y {
            model.advance_until(&mut s, dt, 3.0, |b| b.pos.y - y_plane, |_| {});
            at_plane = s;
        }
        model.advance_until(&mut s, dt, 3.0, |b| b.pos.y - PLATE_FRONT_Y, |_| {});
        if y_plane <= PLATE_FRONT_Y {
            at_plane = s;
            if y_plane < PLATE_FRONT_Y {
                model.advance_until(&mut at_plane, dt, 3.0, |b| b.pos.y - y_plane, |_| {});
            }
        }
        let t = s.t;
        let flight = PitchFlight {
            plate_pos: s.pos,
            plate_vel: s.vel,
            spin: s.spin,
            time: t,
            release_vel: init.vel,
            move_x: s.pos.x - (init.pos.x + init.vel.x * t),
            move_z: s.pos.z - (init.pos.z + init.vel.z * t - 0.5 * G * t * t),
        };
        (at_plane, flight)
    }

    /// Spin-induced movement measured the way Statcast reports `pfx_x` / `pfx_z`:
    /// deviation over the last stretch of the flight, from `y = 50 ft` to the front of
    /// the plate, relative to a spin-free pitch. Returns `(pfx_x, pfx_z)` in metres.
    pub fn statcast_movement(&self, model: &FlightModel, a: ReleaseAngles, dt: f64) -> (f64, f64) {
        let y50 = 50.0 * 0.3048;
        let mut s = self.initial_state(a);
        if s.pos.y > y50 {
            model.advance_until(&mut s, dt, 3.0, |b| b.pos.y - y50, |_| {});
        }
        let start = s;
        model.advance_until(&mut s, dt, 3.0, |b| b.pos.y - PLATE_FRONT_Y, |_| {});
        let t = s.t - start.t;
        (s.pos.x - (start.pos.x + start.vel.x * t), s.pos.z - (start.pos.z + start.vel.z * t - 0.5 * G * t * t))
    }

    /// Release angles that send the pitch through `(target_x, target_z)` at the
    /// front of the plate. Fixed-point iteration on the nearly diagonal Jacobian
    /// (location changes by about one flight distance per radian).
    pub fn aim(&self, model: &FlightModel, target_x: f64, target_z: f64, dt: f64) -> ReleaseAngles {
        let dist = (self.release_pos.y - PLATE_FRONT_Y).max(1.0);
        let mut a = ReleaseAngles { vert: ((target_z - self.release_pos.z) / dist).atan(), horiz: ((target_x - self.release_pos.x) / dist).atan() };
        for _ in 0..6 {
            let f = self.fly(model, a, dt);
            let ex = target_x - f.plate_pos.x;
            let ez = target_z - f.plate_pos.z;
            if ex.abs() < 1e-5 && ez.abs() < 1e-5 {
                break;
            }
            a.horiz += ex / dist;
            a.vert += ez / dist;
        }
        a
    }
}

/// Linearised aiming model for one pitch type of one pitcher in one environment:
/// lets the engine convert an intended plate location into release angles without
/// re-solving the trajectory for every pitch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimModel {
    pub base: ReleaseAngles,
    pub base_x: f64,
    pub base_z: f64,
    /// Plate displacement per radian of release angle (m/rad).
    pub dx_dh: f64,
    pub dz_dv: f64,
    /// Flight time of the reference pitch, s.
    pub time: f64,
    /// Spin-induced movement of the reference pitch, m.
    pub move_x: f64,
    pub move_z: f64,
    /// Speed at the plate of the reference pitch, m/s.
    pub plate_speed: f64,
}

impl AimModel {
    pub fn build(spec: &PitchSpec, model: &FlightModel, ref_x: f64, ref_z: f64, dt: f64) -> Self {
        let base = spec.aim(model, ref_x, ref_z, dt);
        let f0 = spec.fly(model, base, dt);
        let d = 0.01;
        let fh = spec.fly(model, ReleaseAngles { vert: base.vert, horiz: base.horiz + d }, dt);
        let fv = spec.fly(model, ReleaseAngles { vert: base.vert + d, horiz: base.horiz }, dt);
        AimModel {
            base,
            base_x: f0.plate_pos.x,
            base_z: f0.plate_pos.z,
            dx_dh: (fh.plate_pos.x - f0.plate_pos.x) / d,
            dz_dv: (fv.plate_pos.z - f0.plate_pos.z) / d,
            time: f0.time,
            move_x: f0.move_x,
            move_z: f0.move_z,
            plate_speed: f0.plate_vel.norm(),
        }
    }

    /// Release angles for an intended plate location.
    #[inline]
    pub fn angles_for(&self, target_x: f64, target_z: f64) -> ReleaseAngles {
        ReleaseAngles { vert: self.base.vert + (target_z - self.base_z) / self.dz_dv, horiz: self.base.horiz + (target_x - self.base_x) / self.dx_dh }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ball::{AeroModel, BallSpec};
    use crate::env::{Atmosphere, Wind};
    use crate::units::{FT, MPH, RPM};

    fn model() -> FlightModel {
        FlightModel::new(&BallSpec::mlb(), AeroModel::nathan_pitched(), &Atmosphere::from_imperial(72.0, 29.92, 50.0, 500.0), Wind::CALM)
    }

    /// Average 2025 MLB four-seam fastball from a right-hander (Statcast sample:
    /// 94.5 mph, 2321 rpm, extension 6.52 ft, release height 5.83 ft, 1.86 ft to the arm side).
    fn four_seam() -> PitchSpec {
        PitchSpec {
            release_pos: Vec3::new(-1.86 * FT, RUBBER_Y - 6.52 * FT, 5.83 * FT),
            speed: 94.5 * MPH,
            spin_rate: 2321.0 * RPM,
            spin_tilt: (-26.0f64).to_radians(), // arm-side run for a right-hander is toward -x
            spin_active: 0.88,
            gyro_sign: -1.0,
        }
    }

    #[test]
    fn aim_hits_the_target() {
        let m = model();
        let p = four_seam();
        for &(x, z) in &[(0.0, 0.75), (0.25, 0.5), (-0.3, 1.1), (0.5, 0.2)] {
            let a = p.aim(&m, x, z, 0.004);
            let f = p.fly(&m, a, 0.004);
            assert!((f.plate_pos.x - x).abs() < 1e-4 && (f.plate_pos.z - z).abs() < 1e-4, "{:?}", f.plate_pos);
            assert!((f.plate_pos.y - PLATE_FRONT_Y).abs() < 1e-6);
        }
    }

    #[test]
    fn linear_aim_model_is_accurate_across_the_zone() {
        let m = model();
        let p = four_seam();
        let am = AimModel::build(&p, &m, 0.0, 0.75, 0.004);
        for &(x, z) in &[(0.35, 0.3), (-0.4, 1.2), (0.0, 0.0), (0.6, 1.0)] {
            let f = p.fly(&m, am.angles_for(x, z), 0.004);
            let err = ((f.plate_pos.x - x).powi(2) + (f.plate_pos.z - z).powi(2)).sqrt();
            assert!(err < 0.004, "aim error {err} m at ({x},{z})");
        }
    }

    /// The modelled average fastball should look like the measured one:
    /// 2025 Statcast four-seamers lose 8.7 % of their speed by the plate
    /// (plate/release = 0.913), ride 15.7 in and run 7.8 in to the arm side.
    #[test]
    fn four_seam_matches_statcast_averages() {
        let m = model();
        let p = four_seam();
        let a = p.aim(&m, 0.0, 0.85, 0.004);
        let f = p.fly(&m, a, 0.004);
        let ratio = f.plate_vel.norm() / p.speed;
        assert!((ratio - 0.913).abs() < 0.006, "plate/release speed ratio {ratio}");
        assert!(f.time > 0.36 && f.time < 0.42, "flight time {}", f.time);
        let ivb_in = f.move_z / INCH;
        let run_in = -f.move_x / INCH;
        assert!((ivb_in - 15.7).abs() < 3.0, "induced vertical break {ivb_in} in");
        assert!((run_in - 7.8).abs() < 3.0, "arm-side run {run_in} in");
    }

    #[test]
    fn spin_direction_conventions() {
        let m = model();
        let base = four_seam();
        let fly = |tilt_deg: f64, active: f64| {
            let p = PitchSpec { spin_tilt: tilt_deg.to_radians(), spin_active: active, ..base };
            p.fly(&m, ReleaseAngles { vert: -0.03, horiz: 0.03 }, 0.004)
        };
        let back = fly(0.0, 1.0);
        let top = fly(180.0, 1.0);
        let right = fly(90.0, 1.0);
        let left = fly(-90.0, 1.0);
        let gyro = fly(0.0, 0.0);
        assert!(back.move_z > 0.25 && top.move_z < -0.25, "back {} top {}", back.move_z, top.move_z);
        assert!(right.move_x > 0.25 && left.move_x < -0.25, "right {} left {}", right.move_x, left.move_x);
        // A pure gyro ball only picks up a little movement as gravity bends the path away
        // from the spin axis (and `move_z` contains the small effect of drag on the fall).
        assert!(gyro.move_x.abs() < 0.08 && gyro.move_z.abs() < 0.08, "gyro ball moved {} {}", gyro.move_x, gyro.move_z);
        // With the spin along the flight direction (a right-hander's "bullet" spin) that
        // residual movement is toward +x, the right-hander's glove side.
        let rh_gyro = PitchSpec { spin_active: 0.0, gyro_sign: 1.0, ..base }.fly(&m, ReleaseAngles { vert: -0.03, horiz: 0.03 }, 0.004);
        assert!(rh_gyro.move_x > 0.0 && gyro.move_x < 0.0);
        // Topspin drops more than backspin (same release).
        assert!(top.plate_pos.z < back.plate_pos.z - 0.5);
    }
}
