//! Fielder and runner kinematics, defensive alignment, and the two basic
//! fielding questions: can a ball in the air be caught, and who gets to a ball
//! on the ground first.

use crate::params::ModelParams;
use crate::profile::FielderProfile;
use bm_physics::units::FT;
use bm_physics::{BattedPath, Vec3};

pub const P: usize = 0;
pub const C: usize = 1;
pub const B1: usize = 2;
pub const B2: usize = 3;
pub const B3: usize = 4;
pub const SS: usize = 5;
pub const LF: usize = 6;
pub const CF: usize = 7;
pub const RF: usize = 8;

pub const POSITION_CODES: [&str; 9] = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

/// Standard starting positions (m). Infield and outfield depths follow the
/// average starting depths published on Baseball Savant's positioning
/// leaderboards (1B ~110 ft, 2B/SS ~147 ft, 3B ~118 ft, LF ~297 ft, CF ~321 ft,
/// RF ~294 ft from home plate).
pub fn standard_alignment() -> [Vec3; 9] {
    let polar = |r_ft: f64, deg: f64| {
        let a = deg.to_radians();
        Vec3::new(r_ft * FT * a.sin(), r_ft * FT * a.cos(), 0.0)
    };
    [
        Vec3::new(0.0, 17.2, 0.0), // pitcher after the follow-through
        Vec3::new(0.0, -1.2, 0.0), // catcher
        polar(110.0, 34.0),
        polar(147.0, 13.0),
        polar(118.0, -31.0),
        polar(147.0, -13.0),
        polar(297.0, -27.0),
        polar(321.0, 0.0),
        polar(294.0, 27.0),
    ]
}

/// Time to cover `dist` from rest with constant acceleration up to a top speed.
#[inline]
pub fn move_time(dist: f64, accel: f64, vmax: f64) -> f64 {
    if dist <= 0.0 {
        return 0.0;
    }
    let d_acc = vmax * vmax / (2.0 * accel);
    if dist <= d_acc {
        (2.0 * dist / accel).sqrt()
    } else {
        vmax / accel + (dist - d_acc) / vmax
    }
}

/// Flight time of a throw over `dist` released at `speed`, with drag:
/// dv/dx = -k v  =>  t = (exp(k d) - 1) / (k v0).
#[inline]
pub fn throw_time(dist: f64, speed: f64, drag: f64) -> f64 {
    ((drag * dist).exp() - 1.0) / (drag * speed)
}

/// Time for a fielder to reach a point `dist` away (reaction, route, acceleration).
#[inline]
pub fn arrival_time(f: &FielderProfile, dist: f64) -> f64 {
    f.reaction + move_time(dist / f.route, f.accel, f.speed)
}

/// Time for a fielder to get under a ball in the air `dist` away: he first has to
/// read it, and builds speed more slowly while tracking it.
#[inline]
pub fn fly_arrival_time(f: &FielderProfile, dist: f64) -> f64 {
    f.reaction + f.read_delay + move_time(dist / f.route, f.fly_accel, f.speed)
}

#[inline]
fn ground_dist(a: Vec3, b: Vec3) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CatchChance {
    pub fielder: usize,
    /// Ball arrival minus fielder arrival at the best point, s (positive = fielder waits).
    pub margin: f64,
    pub t: f64,
    pub pos: Vec3,
}

/// Best opportunity to catch the ball before it lands (or hits the wall).
pub fn best_catch(path: &BattedPath, t_end: f64, fielders: &[FielderProfile; 9], positions: &[Vec3; 9], params: &ModelParams) -> Option<CatchChance> {
    let mut best: Option<CatchChance> = None;
    for (i, (f, start)) in fielders.iter().zip(positions).enumerate() {
        // Arm's reach; the pitcher is still finishing his delivery.
        let reach = if i == P { 0.8 } else { 1.0 };
        let extra_reaction = if i == P { 0.10 } else { 0.0 };
        for pt in &path.points {
            if pt.t > t_end + 1e-9 {
                break;
            }
            if !pt.airborne || pt.pos.z > params.catch_height || pt.t < 0.05 {
                continue;
            }
            let d = (ground_dist(*start, pt.pos) - reach).max(0.0);
            let margin = pt.t - (fly_arrival_time(f, d) + extra_reaction);
            if best.is_none_or(|b| margin > b.margin) {
                best = Some(CatchChance { fielder: i, margin, t: pt.t, pos: pt.pos });
            }
        }
    }
    best
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub fielder: usize,
    pub t: f64,
    pub pos: Vec3,
    /// Horizontal speed of the ball when fielded, m/s.
    pub ball_speed: f64,
    /// 0 = waited comfortably, 1 = barely got there.
    pub stretch: f64,
}

/// Who fields a ball that is on the ground (bouncing or rolling), and when.
pub fn best_pickup(path: &BattedPath, t_start: f64, fielders: &[FielderProfile; 9], positions: &[Vec3; 9], params: &ModelParams) -> Pickup {
    let last = path.last();
    let mut best = Pickup { fielder: CF, t: f64::INFINITY, pos: last.pos, ball_speed: 0.0, stretch: 0.0 };
    // After a ball in the air comes down, the fielders who were running under it have
    // to redirect to wherever the bounce takes it.
    let redirect = if t_start > 1.2 { params.redirect_delay } else { 0.0 };
    for (i, (f, start)) in fielders.iter().zip(positions).enumerate() {
        // The pitcher is still finishing his delivery and only stops balls hit close to him.
        let reach = if i == P {
            0.7
        } else if i <= SS {
            params.infield_reach
        } else {
            0.8
        };
        let extra_reaction = if i == P { 0.15 } else { redirect };
        let mut found = false;
        for pt in &path.points {
            if pt.t < t_start - 1e-9 || pt.pos.z > params.field_height {
                continue;
            }
            if pt.t >= best.t {
                break;
            }
            let d = (ground_dist(*start, pt.pos) - reach).max(0.0);
            let need = arrival_time(f, d) + extra_reaction;
            if need <= pt.t {
                let stretch = 1.0 - ((pt.t - need) / 0.25).clamp(0.0, 1.0);
                best = Pickup { fielder: i, t: pt.t, pos: pt.pos, ball_speed: pt.vel.horiz(), stretch };
                found = true;
                break;
            }
        }
        if !found {
            // Chase it down where it stops.
            let t = (arrival_time(f, ground_dist(*start, last.pos)) + extra_reaction).max(last.t);
            if t < best.t {
                best = Pickup { fielder: i, t, pos: last.pos, ball_speed: 0.0, stretch: 0.0 };
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use bm_physics::units::MPH;

    #[test]
    fn kinematics() {
        // 90 ft at 27 ft/s top speed with 4.3 m/s^2: about 4.3 s from a standing start.
        let t = move_time(90.0 * FT, 4.3, 27.0 * FT);
        assert!(t > 4.1 && t < 4.5, "{t}");
        assert!((move_time(2.0, 4.0, 8.0) - 1.0).abs() < 1e-12);
        // An 85 mph throw across the diamond (about 127 ft) takes roughly 1.1 s.
        let th = throw_time(127.0 * FT, 85.0 * MPH, 0.0061);
        assert!(th > 1.0 && th < 1.2, "{th}");
        assert!(throw_time(30.0, 38.0, 0.0061) > 30.0 / 38.0);
    }

    #[test]
    fn alignment_is_sensible() {
        let a = standard_alignment();
        assert!(a[SS].x < 0.0 && a[B2].x > 0.0 && a[B3].x < a[SS].x && a[B1].x > a[B2].x);
        assert!(a[CF].y > 95.0 && a[LF].x < 0.0 && a[RF].x > 0.0);
        for p in &a[2..] {
            assert!(p.x.abs() <= p.y, "fielder in foul territory");
        }
    }
}
