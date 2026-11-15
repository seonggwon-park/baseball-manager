//! Ball flight: gravity + drag + Magnus force, integrated with classical RK4.
//!
//! Equations of motion (Nathan, "Analysis of Baseball Trajectories", Eqs. 7-8):
//!
//! ```text
//! dv/dt = -K Cd |v| v + K Cl |v| (omega x v) / omega - g z
//! K = rho A / (2 m)
//! ```
//!
//! where `v` is the velocity relative to the air.

use crate::ball::{AeroModel, BallSpec, SpinMode};
use crate::env::{Atmosphere, Wind};
use crate::units::{G, MPH, RPM};
use crate::vec3::Vec3;

/// Reference speed for the spin-decay law (100 mph; Nathan uses 146.7 ft/s).
const V_REF: f64 = 100.0 * MPH;
const KRPM: f64 = 1000.0 * RPM;

/// Kinematic state of a ball in flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallState {
    pub pos: Vec3,
    pub vel: Vec3,
    /// Spin angular velocity vector, rad/s (fixed direction; magnitude may decay).
    pub spin: Vec3,
    pub t: f64,
}

impl BallState {
    pub fn new(pos: Vec3, vel: Vec3, spin: Vec3) -> Self {
        BallState { pos, vel, spin, t: 0.0 }
    }
}

/// Everything needed to evaluate accelerations for one ball in one environment.
#[derive(Clone, Copy, Debug)]
pub struct FlightModel {
    /// rho * A / (2 m), 1/m.
    pub k: f64,
    pub radius: f64,
    pub aero: AeroModel,
    pub wind: Wind,
    pub g: f64,
}

impl FlightModel {
    pub fn new(ball: &BallSpec, aero: AeroModel, atmosphere: &Atmosphere, wind: Wind) -> Self {
        FlightModel { k: 0.5 * atmosphere.air_density() * ball.cross_section() / ball.mass, radius: ball.radius, aero, wind, g: G }
    }

    /// Acceleration of the ball, m/s^2.
    #[inline]
    pub fn accel(&self, pos: Vec3, vel: Vec3, spin: Vec3) -> Vec3 {
        let vr = vel - self.wind.at(pos.z);
        let v2 = vr.norm_sq();
        if v2 < 1e-18 {
            return Vec3::new(0.0, 0.0, -self.g);
        }
        let v = v2.sqrt();
        let w2 = spin.norm_sq();
        let (cd, magnus) = if w2 > 1e-12 {
            let w = w2.sqrt();
            let wxv = spin.cross(vr);
            match self.aero.spin_mode {
                SpinMode::Total => {
                    let cl = self.aero.cl(self.radius * w / v);
                    (self.aero.cd(w), wxv * (self.k * cl * v / w))
                }
                SpinMode::Transverse => {
                    let wxv_n = wxv.norm();
                    let w_perp = wxv_n / v;
                    let cl = self.aero.cl(self.radius * w_perp / v);
                    let m = if wxv_n > 1e-12 { wxv * (self.k * cl * v2 / wxv_n) } else { Vec3::ZERO };
                    (self.aero.cd(w_perp), m)
                }
            }
        } else {
            (self.aero.cd0, Vec3::ZERO)
        };
        Vec3::new(0.0, 0.0, -self.g) + vr * (-self.k * cd * v) + magnus
    }

    /// One classical Runge-Kutta step of size `dt`.
    #[inline]
    pub fn step(&self, s: &mut BallState, dt: f64) {
        let h = 0.5 * dt;
        let a1 = self.accel(s.pos, s.vel, s.spin);
        let v2 = s.vel + a1 * h;
        let a2 = self.accel(s.pos + s.vel * h, v2, s.spin);
        let v3 = s.vel + a2 * h;
        let a3 = self.accel(s.pos + v2 * h, v3, s.spin);
        let v4 = s.vel + a3 * dt;
        let a4 = self.accel(s.pos + v3 * dt, v4, s.spin);
        s.pos += (s.vel + (v2 + v3) * 2.0 + v4) * (dt / 6.0);
        s.vel += (a1 + (a2 + a3) * 2.0 + a4) * (dt / 6.0);
        s.t += dt;
        if self.aero.spin_decay_tau.is_finite() {
            let v = s.vel.norm();
            s.spin = s.spin * (-dt * v / (self.aero.spin_decay_tau * V_REF)).exp();
        }
    }

    /// Integrate until `event(state)` crosses from positive to non-positive or
    /// `t_max` is reached. On a crossing the state is refined onto the event
    /// surface (two secant iterations on the last step) and `true` is returned.
    /// `record` is called after every completed full step that did not end the flight.
    pub fn advance_until<E, R>(&self, s: &mut BallState, dt: f64, t_max: f64, event: E, mut record: R) -> bool
    where
        E: Fn(&BallState) -> f64,
        R: FnMut(&BallState),
    {
        let mut g_prev = event(s);
        if g_prev <= 0.0 {
            return true;
        }
        while s.t < t_max {
            let prev = *s;
            self.step(s, dt);
            let g = event(s);
            if g <= 0.0 {
                let f1 = (g_prev / (g_prev - g)).clamp(0.0, 1.0);
                let mut cur = prev;
                self.step(&mut cur, dt * f1);
                let g1 = event(&cur);
                if g1.abs() > 1e-7 && (g_prev - g1).abs() > 1e-12 {
                    let f2 = (f1 * g_prev / (g_prev - g1)).clamp(0.0, 1.0);
                    cur = prev;
                    self.step(&mut cur, dt * f2);
                }
                *s = cur;
                return true;
            }
            record(s);
            g_prev = g;
        }
        false
    }
}

/// Spin vector from backspin / sidespin / gyrospin components defined relative
/// to the velocity direction (Nathan, "Analysis of Baseball Trajectories", Eq. 4).
///
/// * `theta`: vertical angle of the velocity (rad, + up).
/// * `phi`: horizontal direction of the velocity, measured from +y toward +x
///   (0 = toward second base, pi = a pitch toward home plate).
/// * `wb`: backspin (lift), `ws`: sidespin, `wg`: gyrospin (along the velocity).
///
/// Sign conventions (same as the spreadsheet): positive `wb` produces an upward
/// Magnus force for both pitched and batted balls. Positive `ws` deflects a
/// pitch toward +x (the pitcher's left) and a batted ball toward -x (left field).
#[inline]
pub fn spin_from_components(theta: f64, phi: f64, wb: f64, ws: f64, wg: f64) -> Vec3 {
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    Vec3::new(wb * cp - ws * st * sp + wg * ct * sp, -wb * sp - ws * st * cp + wg * ct * cp, ws * ct + wg * st)
}

/// Velocity vector from speed, vertical angle `theta` and horizontal direction `phi`
/// (same angle conventions as [`spin_from_components`]).
#[inline]
pub fn velocity_from_angles(speed: f64, theta: f64, phi: f64) -> Vec3 {
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    Vec3::new(speed * ct * sp, speed * ct * cp, speed * st)
}

/// Drag coefficient helper for a given spin magnitude in rad/s (per-1000-rpm law).
#[inline]
pub fn cd_for_spin(aero: &AeroModel, spin: f64) -> f64 {
    aero.cd0 + aero.cd_spin * spin / KRPM
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{FT, INCH};

    fn batted_model(flag_transverse: bool) -> FlightModel {
        let mut aero = AeroModel::nathan_batted();
        if flag_transverse {
            aero.spin_mode = SpinMode::Transverse;
        }
        FlightModel::new(&BallSpec::mlb(), aero, &Atmosphere::from_imperial(75.0, 29.92, 50.0, 0.0), Wind::CALM)
    }

    fn fly_to_ground(model: &FlightModel, s: &mut BallState) {
        let hit = model.advance_until(s, 0.002, 30.0, |b| b.pos.z, |_| {});
        assert!(hit);
    }

    /// Oracle: Nathan's spreadsheet, sheet "BattedBallTrajectory-2" as distributed
    /// (100 mph, 30 deg, direction 0, 2837 rpm backspin, 75 F, sea level, flag = 1):
    /// distance 407.25 ft, hang time 5.7127 s.
    #[test]
    fn batted_ball_matches_nathan_sheet2() {
        let m = batted_model(true);
        let theta = 30.0f64.to_radians();
        let vel = velocity_from_angles(100.0 * MPH, theta, 0.0);
        let spin = spin_from_components(theta, 0.0, 2837.0 * RPM, 0.0, 0.0);
        let mut s = BallState::new(Vec3::new(0.0, 2.0 * FT, 6.0 * FT), vel, spin);
        fly_to_ground(&m, &mut s);
        let dist_ft = s.pos.horiz() / FT;
        // The spreadsheet uses a first-order scheme with dt = 0.01 s and 1.467 ft/s per mph;
        // RK4 agrees to well under half a percent.
        assert!((dist_ft - 407.25).abs() < 1.5, "distance {dist_ft}");
        assert!((s.t - 5.7127).abs() < 0.03, "hang time {}", s.t);
        assert!(s.pos.x.abs() < 1e-9);
    }

    /// Oracle: sheet "BattedBallTrajectory-1" (100 mph, 30 deg, direction 0, LHB:
    /// backspin 2837 rpm, sidespin +849 rpm, flag = 0): distance 402.33 ft,
    /// hang time 5.623 s, bearing -5.685 deg (toward left field).
    #[test]
    fn batted_ball_with_sidespin_matches_nathan_sheet1() {
        let m = batted_model(false);
        let theta = 30.0f64.to_radians();
        let vel = velocity_from_angles(100.0 * MPH, theta, 0.0);
        let spin = spin_from_components(theta, 0.0, 2837.0 * RPM, 849.0 * RPM, 0.0);
        let mut s = BallState::new(Vec3::new(0.0, 2.0 * FT, 6.0 * FT), vel, spin);
        fly_to_ground(&m, &mut s);
        let dist_ft = s.pos.horiz() / FT;
        let bearing = s.pos.x.atan2(s.pos.y).to_degrees();
        assert!((dist_ft - 402.33).abs() < 1.5, "distance {dist_ft}");
        assert!((s.t - 5.623).abs() < 0.03, "hang time {}", s.t);
        assert!((bearing - (-5.685)).abs() < 0.1, "bearing {bearing}");
    }

    /// Oracle: sheet "PitchedBallTrajectory" (90 mph, release angle -3 deg, 2000 rpm
    /// backspin, 1500 rpm sidespin, from (0, 55, 6) ft, 70 F, 15 ft elevation):
    /// plate speed 81.998 mph, px 1.1092 ft, pz 1.8349 ft, flight time 0.42711 s.
    #[test]
    fn pitch_matches_nathan_pitched_sheet() {
        let m = FlightModel::new(&BallSpec::mlb(), AeroModel::nathan_pitched(), &Atmosphere::from_imperial(70.0, 29.92, 50.0, 15.0), Wind::CALM);
        let theta = (-3.0f64).to_radians();
        let phi = std::f64::consts::PI;
        let vel = velocity_from_angles(90.0 * MPH, theta, phi);
        let spin = spin_from_components(theta, phi, 2000.0 * RPM, 1500.0 * RPM, 0.0);
        let mut s = BallState::new(Vec3::new(0.0, 55.0 * FT, 6.0 * FT), vel, spin);
        let plate_y = 17.0 * INCH;
        let hit = m.advance_until(&mut s, 0.001, 2.0, |b| b.pos.y - plate_y, |_| {});
        assert!(hit);
        assert!((s.pos.y - plate_y).abs() < 1e-6);
        assert!((s.vel.norm() / MPH - 81.998).abs() < 0.08, "vf {}", s.vel.norm() / MPH);
        assert!((s.pos.x / FT - 1.1092).abs() < 0.01, "px {}", s.pos.x / FT);
        assert!((s.pos.z / FT - 1.8349).abs() < 0.01, "pz {}", s.pos.z / FT);
        assert!((s.t - 0.42711).abs() < 0.0006, "t {}", s.t);
    }

    #[test]
    fn rk4_is_converged_at_production_step_sizes() {
        // Pitch: dt = 4 ms vs 0.25 ms reference.
        let m = FlightModel::new(&BallSpec::mlb(), AeroModel::nathan_pitched(), &Atmosphere::default(), Wind::CALM);
        let theta = (-2.0f64).to_radians();
        let phi = std::f64::consts::PI;
        let init = BallState::new(
            Vec3::new(-0.6, 16.6, 1.8),
            velocity_from_angles(95.0 * MPH, theta, phi),
            spin_from_components(theta, phi, 2100.0 * RPM, -900.0 * RPM, 300.0 * RPM),
        );
        let run = |dt: f64| {
            let mut s = init;
            m.advance_until(&mut s, dt, 2.0, |b| b.pos.y - 0.4318, |_| {});
            s
        };
        let fine = run(0.00025);
        let coarse = run(0.004);
        assert!((fine.pos - coarse.pos).norm() < 1e-5, "pitch position error {}", (fine.pos - coarse.pos).norm());

        // Fly ball: dt = 10 ms vs 0.5 ms reference.
        let mb = batted_model(false);
        let th = 28.0f64.to_radians();
        let init = BallState::new(
            Vec3::new(0.0, 0.6, 0.9),
            velocity_from_angles(103.0 * MPH, th, 0.3),
            spin_from_components(th, 0.3, 2400.0 * RPM, -700.0 * RPM, 0.0),
        );
        let runb = |dt: f64| {
            let mut s = init;
            mb.advance_until(&mut s, dt, 30.0, |b| b.pos.z, |_| {});
            s
        };
        let fine = runb(0.0005);
        let coarse = runb(0.010);
        assert!((fine.pos - coarse.pos).norm() < 2e-3, "fly ball position error {}", (fine.pos - coarse.pos).norm());
    }

    #[test]
    fn thin_air_carries_farther_and_headwind_shortens() {
        let ball = BallSpec::mlb();
        let th = 28.0f64.to_radians();
        let init = BallState::new(Vec3::new(0.0, 0.6, 0.9), velocity_from_angles(103.0 * MPH, th, 0.0), spin_from_components(th, 0.0, 2500.0 * RPM, 0.0, 0.0));
        let dist = |atm: Atmosphere, wind: Wind| {
            let m = FlightModel::new(&ball, AeroModel::nathan_batted(), &atm, wind);
            let mut s = init;
            m.advance_until(&mut s, 0.005, 30.0, |b| b.pos.z, |_| {});
            s.pos.horiz() / FT
        };
        let sea = dist(Atmosphere::from_imperial(70.0, 29.92, 50.0, 0.0), Wind::CALM);
        let denver = dist(Atmosphere::from_imperial(70.0, 29.92, 50.0, 5200.0), Wind::CALM);
        let head = dist(Atmosphere::from_imperial(70.0, 29.92, 50.0, 0.0), Wind::toward(10.0 * MPH, std::f64::consts::PI));
        let tail = dist(Atmosphere::from_imperial(70.0, 29.92, 50.0, 0.0), Wind::toward(10.0 * MPH, 0.0));
        assert!(denver > sea + 15.0, "sea {sea} denver {denver}");
        assert!(head < sea - 15.0 && tail > sea + 15.0, "head {head} sea {sea} tail {tail}");
    }
}
