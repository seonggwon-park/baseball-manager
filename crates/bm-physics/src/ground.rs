//! Ball-ground interaction: bounce and roll (stage-1 model).
//!
//! The bounce is an impulse model against a massive plane, the same formalism as
//! the bat-ball collision (normal COR, grip-limited friction acting on the
//! contact-point velocity so that topspin skips and backspin checks), plus a
//! "turf drag" impulse that represents the ball ploughing into a soft surface.
//!
//! What is measured and what is assumed:
//! * Measured: Penn State's PennBounce tests (Brosnan & McNitt) fire baseballs at
//!   90 mph and 25 deg onto baseball surfaces and report the ratio of speed after
//!   to speed before the first bounce: about 0.38-0.48 on natural turfgrass and
//!   0.51-0.60 on skinned infields. The default surfaces below reproduce the
//!   middle of those ranges for a spinless ball (see the tests).
//! * Assumed: how that loss is split between the vertical COR and the horizontal
//!   terms, the decrease of the vertical COR with impact speed, and the rolling
//!   resistance. These are tuned at engine level against ground-ball outcomes and
//!   are listed as stage-2 refinements in `docs/physics-model.md`.

use crate::ball::BallSpec;
use crate::units::G;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface {
    /// Vertical COR at vanishing impact speed.
    pub cor0: f64,
    /// Decrease of the vertical COR per m/s of normal impact speed.
    pub cor_slope: f64,
    /// Lower bound of the vertical COR.
    pub cor_min: f64,
    /// Sliding friction coefficient (limits the grip impulse).
    pub friction: f64,
    /// Turf drag: extra horizontal impulse per unit normal impulse.
    pub drag: f64,
    /// Rolling resistance coefficient (deceleration = `rolling * g`).
    pub rolling: f64,
}

impl Surface {
    /// Natural turfgrass (outfield / infield grass).
    pub const fn grass() -> Self {
        Surface { cor0: 0.55, cor_slope: 0.015, cor_min: 0.20, friction: 0.50, drag: 0.43, rolling: 0.30 }
    }
    /// Skinned infield (dirt).
    pub const fn dirt() -> Self {
        Surface { cor0: 0.60, cor_slope: 0.015, cor_min: 0.25, friction: 0.50, drag: 0.21, rolling: 0.20 }
    }
    /// Infilled synthetic turf: between grass and dirt for pace, lower rolling resistance.
    pub const fn synthetic() -> Self {
        Surface { cor0: 0.58, cor_slope: 0.015, cor_min: 0.22, friction: 0.50, drag: 0.32, rolling: 0.22 }
    }

    #[inline]
    pub fn cor(&self, normal_speed: f64) -> f64 {
        (self.cor0 - self.cor_slope * normal_speed).max(self.cor_min)
    }

    /// Deceleration of a rolling ball, m/s^2.
    #[inline]
    pub fn rolling_decel(&self) -> f64 {
        self.rolling * G
    }
}

/// Velocity and spin after bouncing on a horizontal surface.
/// `vel.z` must be negative (descending).
pub fn bounce(ball: &BallSpec, surf: &Surface, vel: Vec3, spin: Vec3) -> (Vec3, Vec3) {
    let n = Vec3::Z;
    let m = ball.mass;
    let alpha = ball.inertia_factor;
    let r = ball.radius;
    let vn = -vel.z; // normal impact speed (> 0)
    if vn <= 0.0 {
        return (vel, spin);
    }
    let e = surf.cor(vn);
    let jn = (1.0 + e) * m * vn;

    // Grip-limited friction acting on the contact-point velocity.
    let u = vel - spin.cross(n) * r;
    let ut = Vec3::new(u.x, u.y, 0.0);
    let ut_n = ut.norm();
    let mut jt = Vec3::ZERO;
    if ut_n > 1e-9 {
        let grip = ut_n / ((1.0 / m) * (1.0 + 1.0 / alpha));
        let mag = grip.min(surf.friction * jn);
        jt = ut * (-mag / ut_n);
    }
    let mut v = vel + (n * jn + jt) / m;
    let w = spin - n.cross(jt) / (alpha * m * r);

    // Turf drag: opposes the horizontal motion, cannot reverse it.
    let vh = Vec3::new(v.x, v.y, 0.0);
    let vh_n = vh.norm();
    if vh_n > 1e-9 {
        let dv = (surf.drag * jn / m).min(vh_n);
        v -= vh * (dv / vh_n);
    }
    (v, w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{MPH, RPM};

    fn pennbounce_ratio(surf: &Surface) -> f64 {
        let ball = BallSpec::mlb();
        let a = 25.0f64.to_radians();
        let v = 90.0 * MPH;
        let (out, _) = bounce(&ball, surf, Vec3::new(0.0, v * a.cos(), -v * a.sin()), Vec3::ZERO);
        out.norm() / v
    }

    #[test]
    fn pace_matches_pennbounce_measurements() {
        let grass = pennbounce_ratio(&Surface::grass());
        let dirt = pennbounce_ratio(&Surface::dirt());
        let synth = pennbounce_ratio(&Surface::synthetic());
        assert!(grass > 0.38 && grass < 0.48, "grass pace {grass}");
        assert!(dirt > 0.51 && dirt < 0.60, "skinned infield pace {dirt}");
        assert!(synth > grass && synth < dirt, "synthetic pace {synth}");
    }

    #[test]
    fn topspin_skips_and_backspin_checks() {
        let ball = BallSpec::mlb();
        let s = Surface::grass();
        let vel = Vec3::new(0.0, 40.0, -5.0);
        // Ball moving +y: topspin is spin about -x, backspin about +x.
        let (top, _) = bounce(&ball, &s, vel, Vec3::new(-2000.0 * RPM, 0.0, 0.0));
        let (none, _) = bounce(&ball, &s, vel, Vec3::ZERO);
        let (back, _) = bounce(&ball, &s, vel, Vec3::new(2000.0 * RPM, 0.0, 0.0));
        assert!(top.y >= none.y && none.y >= back.y, "top {} none {} back {}", top.y, none.y, back.y);
        assert!(top.z > 0.0 && none.z > 0.0);
        // A shallow grounder keeps most of its speed.
        assert!(none.y > 0.75 * vel.y && none.y < vel.y);
    }

    #[test]
    fn steep_landing_loses_most_horizontal_speed_but_never_reverses() {
        let ball = BallSpec::mlb();
        let s = Surface::grass();
        let (out, _) = bounce(&ball, &s, Vec3::new(3.0, 15.0, -24.0), Vec3::new(250.0, 0.0, 0.0));
        assert!(out.y >= 0.0 && out.y < 8.0, "horizontal after steep landing {}", out.y);
        assert!(out.z > 2.0 && out.z < 9.0, "vertical after steep landing {}", out.z);
    }
}
