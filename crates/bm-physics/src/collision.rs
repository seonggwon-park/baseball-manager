//! Bat-ball collision: three-dimensional impulse model.
//!
//! The model is the rigid-body formalism of Cross & Nathan (Am. J. Phys. 74,
//! 896 (2006)) and Kensrud, Nathan & Smith (Am. J. Phys. 85, 503 (2017)),
//! written in impulse form so that it holds for an arbitrary bat orientation:
//!
//! * Normal direction `n` (line of centres): the relative normal velocity is
//!   reversed with the coefficient of restitution `e_y`; the bat recoils with
//!   effective inverse mass `1/M + b^2/I0`.
//! * Tangential directions: the relative velocity of the contact point is
//!   reversed with the tangential coefficient `e_x` ("the ball grips the bat"),
//!   the ball responding with `(1/m)(1 + 1/alpha)` and the bat with
//!   `1/M + b^2/I0 + R^2/Iz` around the barrel and `1/M + R^2/I0` along it.
//!   For a ball at rest this reduces to their Eqs. 4-7.
//! * The tangential impulse cannot exceed `mu` times the normal impulse (gross slip).
//!
//! The geometry is specified by the impact parameter `offset`: the distance by
//! which the ball's centre misses the bat axis, measured perpendicular to both the
//! bat axis and the relative velocity (positive = ball above the bat, i.e. the
//! bat undercuts the ball, giving loft and backspin).

use crate::ball::BallSpec;
use crate::bat::BatSpec;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// Ball velocity just before impact, m/s.
    pub ball_vel: Vec3,
    /// Ball spin just before impact, rad/s.
    pub ball_spin: Vec3,
    /// Unit vector along the bat from knob to tip.
    pub bat_axis: Vec3,
    /// Velocity of the bat (its axis) at the impact location, m/s.
    pub bat_vel: Vec3,
    /// Impact parameter, m (see module docs).
    pub offset: f64,
    /// Impact location along the bat, m from the tip.
    pub dist_from_tip: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactResult {
    /// Ball velocity after impact, m/s.
    pub vel: Vec3,
    /// Ball spin after impact, rad/s.
    pub spin: Vec3,
    /// Unit normal at the contact point (bat axis -> ball centre).
    pub normal: Vec3,
    /// Effective normal COR used.
    pub cor: f64,
    /// True if the friction limit was reached (gross slip).
    pub slipped: bool,
    /// Normal closing speed at the contact point, m/s.
    pub closing_speed: f64,
}

/// Outcome when the bat does not strike the ball.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoContact {
    /// The impact parameter exceeds the sum of the radii.
    Miss,
    /// The surfaces are not approaching each other.
    NotClosing,
}

/// Collide the ball with the bat. Returns the post-impact ball velocity and spin.
pub fn collide(ball: &BallSpec, bat: &BatSpec, c: &Contact) -> Result<ContactResult, NoContact> {
    let a = c.bat_axis.normalized();
    let rb = ball.radius;
    let rbat = bat.radius_at(c.dist_from_tip);
    let reach = rb + rbat;
    if c.offset.abs() >= reach {
        return Err(NoContact::Miss);
    }

    // Relative velocity of the ball with respect to the bat, projected on the plane
    // perpendicular to the bat axis.
    let v_rel = c.ball_vel - c.bat_vel;
    let w = v_rel - a * v_rel.dot(a);
    let w_n = w.norm();
    if w_n < 1e-9 {
        return Err(NoContact::NotClosing);
    }
    let w_hat = w / w_n;
    // In-plane direction perpendicular to the approach, chosen to point upward.
    let mut p = a.cross(w_hat);
    if p.z < 0.0 {
        p = -p;
    }
    let sin_psi = c.offset / reach;
    let cos_psi = (1.0 - sin_psi * sin_psi).sqrt();
    let n = (p * sin_psi - w_hat * cos_psi).normalized();

    // Relative velocity of the contact points (ball surface minus bat surface).
    let u = c.ball_vel - c.ball_spin.cross(n) * rb - c.bat_vel;
    let u_n = u.dot(n);
    if u_n >= 0.0 {
        return Err(NoContact::NotClosing);
    }
    let u_t = u - n * u_n;
    let circ = n.cross(a); // unit: n is perpendicular to a
    let u_a = u_t.dot(a);
    let u_c = u_t.dot(circ);

    let m = ball.mass;
    let alpha = ball.inertia_factor;
    let b = bat.lever_arm(c.dist_from_tip);
    let inv_bat_n = 1.0 / bat.mass + b * b / bat.inertia_cm;
    let k_n = 1.0 / m + inv_bat_n;
    let ball_t = (1.0 / m) * (1.0 + 1.0 / alpha);
    let k_c = ball_t + inv_bat_n + rbat * rbat / bat.inertia_long;
    let k_a = ball_t + 1.0 / bat.mass + rbat * rbat / bat.inertia_cm;

    let e_y = bat.cor(ball, c.dist_from_tip, -u_n) * (1.0 - bat.cor_obliquity * sin_psi * sin_psi);
    let e_x = bat.tangential_cor;
    let j_n = -(1.0 + e_y) * u_n / k_n;
    let mut j_c = -(1.0 + e_x) * u_c / k_c;
    let mut j_a = -(1.0 + e_x) * u_a / k_a;
    let j_t = (j_c * j_c + j_a * j_a).sqrt();
    let limit = bat.friction * j_n;
    let slipped = j_t > limit;
    if slipped {
        let s = limit / j_t;
        j_c *= s;
        j_a *= s;
    }

    let j = n * j_n + circ * j_c + a * j_a;
    let vel = c.ball_vel + j / m;
    let spin = c.ball_spin - n.cross(j) / (alpha * m * rb);
    Ok(ContactResult { vel, spin, normal: n, cor: e_y, slipped, closing_speed: -u_n })
}

/// Vertical launch angle of a velocity vector, rad.
#[inline]
pub fn launch_angle(vel: Vec3) -> f64 {
    vel.z.atan2(vel.horiz())
}

/// Horizontal spray angle, rad: 0 toward second base, positive toward first base / right field.
#[inline]
pub fn spray_angle(vel: Vec3) -> f64 {
    vel.x.atan2(vel.y)
}

/// Decompose a spin vector into (backspin, sidespin, gyrospin) relative to the
/// direction of flight; the inverse of [`crate::flight::spin_from_components`].
pub fn spin_components(vel: Vec3, spin: Vec3) -> (f64, f64, f64) {
    let theta = launch_angle(vel);
    let phi = spray_angle(vel);
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    let b_hat = Vec3::new(cp, -sp, 0.0);
    let s_hat = Vec3::new(-st * sp, -st * cp, ct);
    let g_hat = Vec3::new(ct * sp, ct * cp, st);
    (spin.dot(b_hat), spin.dot(s_hat), spin.dot(g_hat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{INCH, MPH, RPM};

    fn level_bat() -> Vec3 {
        // Bat along +x (tip toward first base side), swung toward +y.
        Vec3::X
    }

    #[test]
    fn head_on_collision_matches_collision_efficiency() {
        let ball = BallSpec::mlb();
        let bat = BatSpec::mlb_wood();
        let d = bat.sweet_spot_from_tip;
        let v_pitch = 85.0 * MPH;
        let v_bat = 72.0 * MPH;
        let c = Contact {
            ball_vel: Vec3::new(0.0, -v_pitch, 0.0),
            ball_spin: Vec3::ZERO,
            bat_axis: level_bat(),
            bat_vel: Vec3::new(0.0, v_bat, 0.0),
            offset: 0.0,
            dist_from_tip: d,
        };
        let r = collide(&ball, &bat, &c).unwrap();
        let q = bat.collision_efficiency(&ball, d, v_pitch + v_bat);
        let expect = q * v_pitch + (1.0 + q) * v_bat;
        assert!((r.vel.y - expect).abs() < 1e-9, "exit {} expect {}", r.vel.y, expect);
        assert!(r.vel.x.abs() < 1e-9 && r.vel.z.abs() < 1e-9);
        // Statcast: maximum exit velocity ~ 1.23 * bat speed + 0.21 * pitch speed (at the plate).
        let ev = r.vel.norm() / MPH;
        assert!(ev > 98.0 && ev < 108.0, "exit velocity {ev} mph");
        assert!(r.spin.norm() < 1e-9);
    }

    #[test]
    fn stationary_ball_reproduces_kensrud_equations() {
        // Kensrud et al. (2017), Eqs. 4-9, for a bat striking a ball at rest.
        let ball = BallSpec::mlb();
        let mut bat = BatSpec::mlb_wood();
        bat.friction = 10.0; // no gross slip in this check
        let d = 6.0 * INCH;
        let v_bat = 77.0 * MPH;
        let psi = 20.0f64.to_radians();
        let reach = ball.radius + bat.radius_at(d);
        let c = Contact {
            ball_vel: Vec3::ZERO,
            ball_spin: Vec3::ZERO,
            bat_axis: level_bat(),
            bat_vel: Vec3::new(0.0, v_bat, 0.0),
            offset: reach * psi.sin(),
            dist_from_tip: d,
        };
        let r = collide(&ball, &bat, &c).unwrap();
        let (m, alpha, rb, rr) = (ball.mass, ball.inertia_factor, ball.radius, bat.radius_at(d));
        let b = bat.lever_arm(d);
        let ry = m * (1.0 / bat.mass + b * b / bat.inertia_cm);
        let rx = m * alpha / (1.0 + alpha) * (1.0 / bat.mass + b * b / bat.inertia_cm + rr * rr / bat.inertia_long);
        let ey = r.cor;
        let ex = bat.tangential_cor;
        let v_n = (1.0 + ey) / (1.0 + ry) * v_bat * psi.cos();
        let omega = (1.0 + ex) * v_bat * psi.sin() / (rb * (1.0 + rx) * (1.0 + alpha));
        let v_t = (1.0 + ex) * alpha * v_bat * psi.sin() / ((1.0 + rx) * (1.0 + alpha));
        let n = r.normal;
        let got_n = r.vel.dot(n);
        let got_t = (r.vel - n * got_n).norm();
        assert!((got_n - v_n).abs() < 1e-9, "normal {got_n} vs {v_n}");
        assert!((got_t - v_t).abs() < 1e-9, "tangential {got_t} vs {v_t}");
        assert!((r.spin.norm() - omega).abs() < 1e-9, "spin {} vs {}", r.spin.norm(), omega);
        // Undercutting the ball gives backspin and loft.
        let (wb, _, _) = spin_components(r.vel, r.spin);
        assert!(wb > 0.0 && launch_angle(r.vel) > 0.0);
    }

    /// Game-like conditions used by Kensrud et al. (2017), Sec. V B: 85 mph pitch
    /// descending at 6 deg with 2000 rpm backspin, bat at 73 mph with a 6 deg attack
    /// angle, e_x = 0.4. They report topspin for a head-on impact, backspin roughly
    /// linear in the offset, and a ~400 ft fly ball at intermediate offsets.
    #[test]
    fn game_conditions_give_realistic_launch_and_spin() {
        let ball = BallSpec::mlb();
        let bat = BatSpec { tangential_cor: 0.40, ..BatSpec::mlb_wood() };
        let d = bat.sweet_spot_from_tip;
        let descent = 6.0f64.to_radians();
        let attack = 6.0f64.to_radians();
        let v_pitch = 85.0 * MPH;
        let v_bat = 73.0 * MPH;
        let ball_vel = Vec3::new(0.0, -v_pitch * descent.cos(), -v_pitch * descent.sin());
        // Backspin on a pitch travelling toward -y is a spin vector along -x.
        let ball_spin = Vec3::new(-2000.0 * RPM, 0.0, 0.0);
        let bat_vel = Vec3::new(0.0, v_bat * attack.cos(), v_bat * attack.sin());
        let run = |offset_in: f64| {
            let c = Contact { ball_vel, ball_spin, bat_axis: level_bat(), bat_vel, offset: offset_in * INCH, dist_from_tip: d };
            let r = collide(&ball, &bat, &c).unwrap();
            let (wb, ws, _) = spin_components(r.vel, r.spin);
            (r.vel.norm() / MPH, launch_angle(r.vel).to_degrees(), wb / RPM, ws / RPM)
        };
        let (ev0, la0, wb0, _) = run(0.0);
        // The line of centres points 6 deg upward, but the pitch's backspin drags the ball
        // down across the barrel, so a centred hit leaves a few degrees lower.
        assert!(la0 > 1.0 && la0 < 6.0, "squared-up launch angle {la0}");
        assert!(wb0 < -200.0 && wb0 > -1200.0, "head-on spin {wb0} rpm should be modest topspin");
        assert!(ev0 > 100.0 && ev0 < 110.0, "head-on exit velocity {ev0}");
        let (ev1, la1, wb1, ws1) = run(0.75);
        assert!(la1 > 18.0 && la1 < 30.0, "launch angle at 0.75 in offset: {la1}");
        assert!(wb1 > 2500.0 && wb1 < 4500.0, "backspin at 0.75 in offset: {wb1}");
        assert!(ev1 < ev0 && ev1 > 0.93 * ev0, "exit velocity drops slowly with offset: {ev1} vs {ev0}");
        assert!(ws1.abs() < 1.0);
        // Backspin is linear in the offset while the ball grips the bat ...
        let (_, _, wa, _) = run(0.2);
        let (_, _, wb_, _) = run(0.4);
        let (_, _, wc, _) = run(0.6);
        assert!(((wc - wb_) / (wb_ - wa) - 1.0).abs() < 0.02, "spin not linear in offset");
        // ... and saturates near 5000 rpm once gross slip sets in (Nathan et al. 2012).
        let (_, la2, wb2, _) = run(1.5);
        assert!(la2 > la1 && wb2 > 4000.0 && wb2 < 6500.0, "saturated spin {wb2}");
        // Hitting the top half gives a topspin grounder.
        let (_, lam, wbm, _) = run(-0.75);
        assert!(lam < -10.0 && wbm < wb0, "topped ball: la {lam}, spin {wbm}");
    }

    #[test]
    fn misses_and_degenerate_contacts_are_rejected() {
        let ball = BallSpec::mlb();
        let bat = BatSpec::mlb_wood();
        let mut c = Contact {
            ball_vel: Vec3::new(0.0, -38.0, 0.0),
            ball_spin: Vec3::ZERO,
            bat_axis: level_bat(),
            bat_vel: Vec3::new(0.0, 32.0, 0.0),
            offset: 0.08,
            dist_from_tip: 0.14,
        };
        assert_eq!(collide(&ball, &bat, &c), Err(NoContact::Miss));
        // No relative motion across the bat: nothing to resolve.
        c.offset = 0.0;
        c.bat_vel = c.ball_vel;
        assert_eq!(collide(&ball, &bat, &c), Err(NoContact::NotClosing));
    }

    #[test]
    fn collision_never_creates_energy() {
        let ball = BallSpec::mlb();
        let bat = BatSpec::mlb_wood();
        for i in 0..40 {
            let off = -0.065 + 0.13 * (i as f64) / 39.0;
            for &d in &[0.03, 0.14, 0.30] {
                let c = Contact {
                    ball_vel: Vec3::new(1.0, -39.0, -4.0),
                    ball_spin: Vec3::new(-220.0, 30.0, 60.0),
                    bat_axis: Vec3::new(0.94, 0.2, -0.28).normalized(),
                    bat_vel: Vec3::ZERO,
                    offset: off,
                    dist_from_tip: d,
                };
                if let Ok(r) = collide(&ball, &bat, &c) {
                    // In the frame where the bat is initially at rest the ball cannot gain energy.
                    let ke = |v: Vec3, w: Vec3| 0.5 * ball.mass * v.norm_sq() + 0.5 * ball.inertia() * w.norm_sq();
                    assert!(ke(r.vel, r.spin) <= ke(c.ball_vel, c.ball_spin) * (1.0 + 1e-9), "energy gain at offset {off} d {d}");
                    assert!(r.vel.is_finite() && r.spin.is_finite());
                }
            }
        }
    }
}
