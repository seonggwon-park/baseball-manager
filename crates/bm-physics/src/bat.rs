//! Bat properties: inertia (for the recoil factors) and the effective
//! coefficient of restitution along the barrel.
//!
//! Sources:
//! * Recoil factors and the collision-efficiency formalism: A. M. Nathan,
//!   "Characterizing the performance of baseball bats", Am. J. Phys. 71, 134 (2003);
//!   R. Cross and A. M. Nathan, "Scattering of a baseball by a bat",
//!   Am. J. Phys. 74, 896 (2006).
//! * Effect of bending vibrations on the exit speed away from the sweet spot:
//!   A. M. Nathan, "Dynamics of the baseball-bat collision", Am. J. Phys. 68, 979 (2000).

use crate::ball::BallSpec;
use crate::units::{INCH, OZ};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatSpec {
    /// Mass, kg.
    pub mass: f64,
    /// Overall length, m.
    pub length: f64,
    /// Distance of the centre of mass from the knob, m.
    pub cm_from_knob: f64,
    /// Moment of inertia about the centre of mass, axis perpendicular to the bat (I0), kg m^2.
    pub inertia_cm: f64,
    /// Moment of inertia about the long axis (Iz), kg m^2.
    pub inertia_long: f64,
    /// Barrel radius, m.
    pub barrel_radius: f64,
    /// Handle radius, m.
    pub handle_radius: f64,
    /// Length of the full-diameter barrel measured from the tip, m.
    pub barrel_length: f64,
    /// Distance from the tip at which the taper reaches the handle radius, m.
    pub taper_end: f64,
    /// Location of maximum effective COR ("sweet spot"), m from the tip.
    pub sweet_spot_from_tip: f64,
    /// Quadratic fall-off of the effective COR away from the sweet spot, 1/m^2:
    /// `e_eff = e0 * max(floor, 1 - k * dz^2)`.
    pub cor_falloff: f64,
    /// Lower bound of the fall-off factor.
    pub cor_floor: f64,
    /// Bat performance factor e/e0 at the sweet spot (about 1 for wood).
    pub bpf: f64,
    /// Empirical loss of normal COR for off-centre impacts:
    /// `e *= 1 - cor_obliquity * sin^2(psi)`, where `psi` is the angle between the
    /// line of centres and the approach direction.
    pub cor_obliquity: f64,
    /// Tangential coefficient of restitution of the ball-bat contact (e_x).
    pub tangential_cor: f64,
    /// Coefficient of friction limiting the tangential impulse (gross slip).
    pub friction: f64,
}

impl BatSpec {
    /// A typical MLB wood bat: 34 in, 31.5 oz, I0 = 2578 oz in^2, Iz = 18.0 oz in^2,
    /// centre of mass 22.7 in from the knob, 2.625 in barrel (the wood bat used as
    /// the worked example by Cross & Nathan 2006, Sec. IV B).
    ///
    /// * Sweet spot 5.5 in from the tip: Nathan (2000) finds the exit speed peaked
    ///   12-13 cm from the barrel end, where the effective COR equals the
    ///   rigid-bat value.
    /// * `cor_falloff = 30 /m^2`: in the same calculation the exit speed 11.5 cm
    ///   inside the sweet spot is about 25 % below the rigid-bat value, which
    ///   corresponds to `e_eff ~ 0.30` against `e0 = 0.50` there.
    /// * `tangential_cor = 0.20`: measured values fall with impact speed, from
    ///   `e_x = 0.405 +/- 0.010` for a bat swung at 63-88 mph into a ball at rest
    ///   (Kensrud, Nathan & Smith, Am. J. Phys. 85, 503 (2017)) to 0.30 +/- 0.02
    ///   for a ball fired at up to 120 mph onto a fixed wood cylinder (Nathan et
    ///   al., Procedia Engineering 34, 182 (2012)). Game collisions close at about
    ///   157 mph; continuing that trend gives roughly 0.2. Lower values bring the
    ///   backspin gained per degree of launch angle closer to Nathan's Statcast fit
    ///   but carry the ball further: at 0.0 the league hits 39 % more home runs.
    /// * `cor_obliquity = 0.55`: not from the literature. The rigid-body model keeps
    ///   too much exit speed on topped and skied balls; this term is fitted together with
    ///   the swing constants to the 2025 Statcast comparisons of `bm validate` (it was
    ///   0.45 before the vertical swing geometry was rebuilt from bat tracking).
    /// * `friction = 0.15`: impulse ratio at which gross slip sets in in the
    ///   high-speed experiment of Nathan et al. (2012); consistent with the lower
    ///   limit of 0.15 reported by Kensrud et al. (2017).
    pub fn mlb_wood() -> Self {
        BatSpec {
            mass: 31.5 * OZ,
            length: 34.0 * INCH,
            cm_from_knob: 22.7 * INCH,
            inertia_cm: 2578.0 * OZ * INCH * INCH,
            inertia_long: 18.0 * OZ * INCH * INCH,
            barrel_radius: 0.5 * 2.625 * INCH,
            handle_radius: 0.5 * 1.0 * INCH,
            barrel_length: 10.0 * INCH,
            taper_end: 22.0 * INCH,
            sweet_spot_from_tip: 5.5 * INCH,
            cor_falloff: 30.0,
            cor_floor: 0.25,
            bpf: 1.0,
            cor_obliquity: 0.55826,
            tangential_cor: 0.20,
            friction: 0.15,
        }
    }

    /// The same bat made heavier or lighter by `factor`, the mass spread evenly along it.
    pub fn weighted(mut self, factor: f64) -> Self {
        self.mass *= factor;
        self.inertia_cm *= factor;
        self.inertia_long *= factor;
        self
    }

    /// Bat radius at a distance `d` from the tip, m.
    #[inline]
    pub fn radius_at(&self, d: f64) -> f64 {
        if d <= self.barrel_length {
            self.barrel_radius
        } else if d >= self.taper_end {
            self.handle_radius
        } else {
            let f = (d - self.barrel_length) / (self.taper_end - self.barrel_length);
            self.barrel_radius + f * (self.handle_radius - self.barrel_radius)
        }
    }

    /// Signed distance of an impact point from the centre of mass along the bat
    /// (positive toward the tip), m.
    #[inline]
    pub fn lever_arm(&self, dist_from_tip: f64) -> f64 {
        (self.length - dist_from_tip) - self.cm_from_knob
    }

    /// Normal recoil factor r_y = m (1/M + b^2/I0) (Cross & Nathan 2006, Eq. 5).
    #[inline]
    pub fn recoil_normal(&self, ball: &BallSpec, dist_from_tip: f64) -> f64 {
        let b = self.lever_arm(dist_from_tip);
        ball.mass * (1.0 / self.mass + b * b / self.inertia_cm)
    }

    /// Effective ball-bat COR at an impact location for a given normal closing speed.
    #[inline]
    pub fn cor(&self, ball: &BallSpec, dist_from_tip: f64, normal_speed: f64) -> f64 {
        let dz = dist_from_tip - self.sweet_spot_from_tip;
        let f = (1.0 - self.cor_falloff * dz * dz).max(self.cor_floor);
        self.bpf * ball.cor_at(normal_speed) * f
    }

    /// Collision efficiency q = (e - r) / (1 + r) (Nathan 2003, Eq. 6):
    /// `v_exit = q * v_pitch + (1 + q) * v_bat` for a head-on impact.
    pub fn collision_efficiency(&self, ball: &BallSpec, dist_from_tip: f64, normal_speed: f64) -> f64 {
        let r = self.recoil_normal(ball, dist_from_tip);
        (self.cor(ball, dist_from_tip, normal_speed) - r) / (1.0 + r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::MPH;

    #[test]
    fn recoil_and_efficiency_match_published_values() {
        let ball = BallSpec::mlb();
        let bat = BatSpec::mlb_wood();
        // Impact 6 in from the barrel end: b = 28 - 22.7 = 5.3 in.
        let r = bat.recoil_normal(&ball, 6.0 * INCH);
        let expect = 5.125 * (1.0 / 31.5 + 5.3 * 5.3 / 2578.0);
        assert!((r - expect).abs() < 1e-9, "r {r} expect {expect}");
        // Nathan (2003): "for a typical collision in the sweet spot zone e ~ 0.5 and r ~ 0.25,
        // so that e_A ~ 0.2". Nathan et al. (2011) measure 0.214-0.227 on 31 oz wood bats;
        // Statcast's squared-up model uses 0.23 as the upper limit.
        let q = bat.collision_efficiency(&ball, 5.5 * INCH, 150.0 * MPH);
        assert!(q > 0.17 && q < 0.24, "collision efficiency {q}");
    }

    #[test]
    fn cor_falls_off_away_from_sweet_spot() {
        let ball = BallSpec::mlb();
        let bat = BatSpec::mlb_wood();
        let v = 150.0 * MPH;
        let sweet = bat.cor(&ball, bat.sweet_spot_from_tip, v);
        let inside = bat.cor(&ball, bat.sweet_spot_from_tip + 0.115, v);
        let tip = bat.cor(&ball, 0.01, v);
        assert!((sweet - ball.cor_at(v)).abs() < 1e-12);
        assert!((inside / sweet - 0.60).abs() < 0.02, "ratio {}", inside / sweet);
        assert!(tip < sweet);
    }

    #[test]
    fn radius_profile() {
        let bat = BatSpec::mlb_wood();
        assert!((bat.radius_at(0.05) - 0.0333).abs() < 1e-3);
        assert!(bat.radius_at(0.4) < bat.radius_at(0.2));
        assert!((bat.radius_at(0.8) - 0.0127).abs() < 1e-4);
    }
}
