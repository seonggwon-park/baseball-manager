//! Ball properties and aerodynamic coefficient models.
//!
//! Aerodynamics follow A. M. Nathan's Trajectory Calculator (University of
//! Illinois, spreadsheet version June 2026; method in "Analysis of Baseball
//! Trajectories", 2017):
//!
//! ```text
//! Cd = cd0 + cdspin * (spin / 1000 rpm)
//! Cl = cl2 * S / (cl0 + cl1 * S),   S = r * omega / v
//! ```
//!
//! * Batted balls: `cd0 = 0.3008`, `cdspin = 0.0292`, `cl0 = 0.583`,
//!   `cl1 = 2.333`, `cl2 = 1.12`, fitted to 2016 Statcast fly balls at
//!   Tropicana Field with the *total* spin ("flag = 0").
//! * Pitched balls: same lift model, `cd0 = 0.28`, evaluated with the spin
//!   component *transverse* to the velocity ("flag = 1").

use crate::units::{INCH, MPH, OZ, RPM};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallSpec {
    /// Mass, kg.
    pub mass: f64,
    /// Radius, m.
    pub radius: f64,
    /// Moment of inertia factor: I = alpha * m * r^2 (0.4 = uniform sphere).
    pub inertia_factor: f64,
    /// Ball-on-rigid-wall coefficient of restitution at `cor_ref_speed`.
    pub cor_ref: f64,
    /// Reference impact speed for `cor_ref`, m/s.
    pub cor_ref_speed: f64,
    /// Change in COR per m/s of impact speed (negative).
    pub cor_slope: f64,
}

impl BallSpec {
    /// MLB ball: 5.125 oz, 9.125 in circumference (the values Nathan's
    /// calculator uses; the rules allow 5-5.25 oz and 9-9.25 in).
    ///
    /// COR against a rigid wall: 0.55 at 60 mph (MLB's specification is
    /// 0.546 +/- 0.032 at 58 mph), falling linearly with impact speed.
    ///
    /// The slope is an effective value, not a laboratory one. Laboratory data fall
    /// faster: Nathan (Am. J. Phys. 71, 134 (2003), Sec. II B) quotes a roughly
    /// linear drop "from about 0.55 at 60 mph to about 0.45 at 160 mph", and Nathan,
    /// Smith, Faber & Russell (Am. J. Phys. 79, 575 (2011)) measure a ball-bat COR
    /// of 0.489 +/- 0.009 for 110 mph balls on wood bats. With the laboratory slope
    /// the best possible exit velocity for a given swing comes out 4-5 mph below what
    /// Statcast's bat tracking implies: its squared-up model takes the maximum as
    /// `1.23 * bat speed + 0.23 * pitch speed at the plate`, a collision efficiency
    /// of 0.23 relative to the bat speed it measures about 6 in from the tip.
    ///
    /// The slope used here gives e = 0.51 at a closing speed of 157 mph, which is a
    /// collision efficiency of 0.23 at the sweet spot of the bat in `BatSpec::mlb_wood`:
    /// the best swings of the simulation then reach Statcast's squared-up maximum, as
    /// the best swings in the tracking data do by definition. With the measured
    /// distribution of bat speeds the whole distribution of exit velocity over that
    /// maximum follows 2025 Statcast (`docs/validation`). Whether the gap to the
    /// laboratory value comes from today's ball or from how bat speed is measured is
    /// not resolved here.
    pub fn mlb() -> Self {
        BallSpec {
            mass: 5.125 * OZ,
            radius: 9.125 * INCH / (2.0 * PI),
            inertia_factor: 0.4,
            cor_ref: 0.55,
            cor_ref_speed: 60.0 * MPH,
            cor_slope: -0.04 / (97.0 * MPH),
        }
    }

    pub fn cross_section(&self) -> f64 {
        PI * self.radius * self.radius
    }

    pub fn inertia(&self) -> f64 {
        self.inertia_factor * self.mass * self.radius * self.radius
    }

    /// Ball-wall COR at a given normal impact speed (m/s), clamped to a sane range.
    #[inline]
    pub fn cor_at(&self, impact_speed: f64) -> f64 {
        (self.cor_ref + self.cor_slope * (impact_speed - self.cor_ref_speed)).clamp(0.30, 0.62)
    }
}

/// Which spin enters the drag and lift coefficients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinMode {
    /// Total spin magnitude ("flag = 0"): used for the batted-ball fit.
    Total,
    /// Spin component perpendicular to the velocity ("flag = 1"): used for pitches.
    Transverse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AeroModel {
    pub cd0: f64,
    /// Added drag coefficient per 1000 rpm of spin.
    pub cd_spin: f64,
    pub cl0: f64,
    pub cl1: f64,
    pub cl2: f64,
    pub spin_mode: SpinMode,
    /// Spin decay time constant in seconds at 100 mph; `f64::INFINITY` disables decay.
    /// spin(t) follows d(omega)/dt = -omega * v / (tau * 100 mph).
    pub spin_decay_tau: f64,
}

/// 1000 rpm in rad/s.
const KRPM: f64 = 1000.0 * RPM;

impl AeroModel {
    /// Nathan's batted-ball parameters (fit to 2016 Statcast data).
    pub const fn nathan_batted() -> Self {
        AeroModel { cd0: 0.3008, cd_spin: 0.0292, cl0: 0.583, cl1: 2.333, cl2: 1.12, spin_mode: SpinMode::Total, spin_decay_tau: f64::INFINITY }
    }

    /// Nathan's pitched-ball parameters.
    pub const fn nathan_pitched() -> Self {
        AeroModel { cd0: 0.28, cd_spin: 0.0292, cl0: 0.583, cl1: 2.333, cl2: 1.12, spin_mode: SpinMode::Transverse, spin_decay_tau: f64::INFINITY }
    }

    /// Pitched-ball parameters used by the engine: the lift model above with the
    /// transverse spin, and the batted-ball `cd0 = 0.3008`.
    ///
    /// Nathan's pitched-ball sheet ships with `cd0 = 0.28`. Against 2025 Statcast
    /// pitch tracking (plate speed / release speed for eight pitch types, 126k
    /// pitches) that value under-predicts the speed loss by about 0.5 % of the
    /// release speed, while 0.30 reproduces it to within 0.3 % for every pitch type
    /// at the average game-time air density (see `examples/pitch_shapes.rs`).
    pub const fn pitched_2025() -> Self {
        AeroModel { cd0: 0.3008, cd_spin: 0.0292, cl0: 0.583, cl1: 2.333, cl2: 1.12, spin_mode: SpinMode::Transverse, spin_decay_tau: f64::INFINITY }
    }

    /// Batted-ball parameters used by the engine: Nathan's model with `cd0` refitted
    /// to 2025 data.
    ///
    /// Flying 5,870 Statcast fly balls of the 2025 season (launch angle 15-45 deg,
    /// exit velocity >= 85 mph) one by one with the 2016 constants and Nathan's
    /// spin model over-predicts the measured distance by 11 ft on average at the
    /// mean game-time air density (1.155 kg/m^3); `cd0 = 0.331` removes the bias
    /// (`examples/flight_check.rs`). MLB reported higher drag on the 2025 ball.
    pub const fn batted_2025() -> Self {
        AeroModel { cd0: 0.331, cd_spin: 0.0292, cl0: 0.583, cl1: 2.333, cl2: 1.12, spin_mode: SpinMode::Total, spin_decay_tau: f64::INFINITY }
    }

    /// Drag coefficient for a spin (rad/s) chosen according to `spin_mode`.
    #[inline]
    pub fn cd(&self, spin: f64) -> f64 {
        self.cd0 + self.cd_spin * spin / KRPM
    }

    /// Lift coefficient for spin factor `s = r * omega / v`.
    #[inline]
    pub fn cl(&self, s: f64) -> f64 {
        self.cl2 * s / (self.cl0 + self.cl1 * s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mlb_ball_dimensions() {
        let b = BallSpec::mlb();
        assert!((b.mass - 0.145_291).abs() < 1e-5);
        assert!((b.radius - 0.036_888).abs() < 1e-5);
        // K = 0.5 * A / m = 0.07182 ft^2/lb in Nathan's units
        let k_imperial = 0.5 * b.cross_section() / b.mass / (0.3048f64.powi(2) / 0.453_592_37);
        assert!((k_imperial - 0.07182).abs() < 2e-5, "{k_imperial}");
    }

    #[test]
    fn cor_speed_dependence() {
        let b = BallSpec::mlb();
        assert!((b.cor_at(60.0 * MPH) - 0.55).abs() < 1e-12);
        assert!((b.cor_at(157.0 * MPH) - 0.51).abs() < 1e-12);
        // Collision efficiency 0.23 at the sweet spot, as in Statcast's squared-up maximum.
        let bat = crate::bat::BatSpec::mlb_wood();
        let q = bat.collision_efficiency(&b, bat.sweet_spot_from_tip, 157.0 * MPH);
        assert!((q - 0.23).abs() < 0.01, "{q}");
    }

    #[test]
    fn nathan_coefficients() {
        let a = AeroModel::nathan_batted();
        // BattedBallTrajectory-2: 2837 rpm -> cd = 0.3836404
        assert!((a.cd(2837.0 * RPM) - 0.383_640_4).abs() < 1e-9);
        // Lift saturates below cl2/cl1
        assert!(a.cl(10.0) < 1.12 / 2.333);
        assert!((a.cl(0.2) - 1.12 * 0.2 / (0.583 + 2.333 * 0.2)).abs() < 1e-12);
    }
}
