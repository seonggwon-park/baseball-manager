//! Full path of a batted ball: flight, bounces, roll, and interaction with the
//! outfield wall, sampled so that fielding logic can intercept it.

use crate::ball::BallSpec;
use crate::flight::{BallState, FlightModel};
use crate::ground::bounce;
use crate::park::{bearing, Park};
use crate::vec3::Vec3;
use std::f64::consts::FRAC_PI_4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathPoint {
    pub t: f64,
    pub pos: Vec3,
    pub vel: Vec3,
    /// False once the ball is rolling.
    pub airborne: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathEnd {
    /// The ball came to rest (or reached a wall while rolling).
    Stopped,
    /// The ball left the field over the fence between the foul lines.
    OverFence,
    /// The ball left the field of play in foul territory (stands, backstop).
    OutOfPlay,
    /// Simulation time limit reached.
    TimeLimit,
}

/// Sampled path of a batted ball. Reuse one instance to avoid allocation.
#[derive(Clone, Debug)]
pub struct BattedPath {
    pub points: Vec<PathPoint>,
    /// First contact with the ground, if any.
    pub landing: Option<PathPoint>,
    /// First contact with the outfield wall, if any.
    pub wall: Option<PathPoint>,
    pub end: PathEnd,
    /// Maximum height reached, m.
    pub apex: f64,
    /// Number of ground contacts before the path ended.
    pub bounces: u32,
    /// For a ball that reached the wall or the stands on the fly: where it would first have
    /// come down at field level had nothing stopped it (Statcast's projected distance).
    pub carry: Option<Vec3>,
}

impl Default for BattedPath {
    fn default() -> Self {
        BattedPath { points: Vec::with_capacity(128), landing: None, wall: None, end: PathEnd::Stopped, apex: 0.0, bounces: 0, carry: None }
    }
}

impl BattedPath {
    pub fn clear(&mut self) {
        self.points.clear();
        self.landing = None;
        self.wall = None;
        self.end = PathEnd::Stopped;
        self.apex = 0.0;
        self.bounces = 0;
        self.carry = None;
    }

    /// Where the flight first came down at field level, or would have without the wall or
    /// the stands: the point behind Statcast's projected distance.
    pub fn carry_point(&self) -> Vec3 {
        self.carry.or(self.landing.map(|l| l.pos)).unwrap_or_else(|| self.last().pos)
    }

    pub fn last(&self) -> PathPoint {
        *self.points.last().expect("path has at least the initial point")
    }

    /// True if the ball cleared the fence on the fly (a home run when fair).
    pub fn is_home_run(&self) -> bool {
        self.end == PathEnd::OverFence && self.landing.is_none()
    }

    /// True if the ball bounced over the fence (ground-rule double when fair).
    pub fn bounced_over(&self) -> bool {
        self.end == PathEnd::OverFence && self.landing.is_some()
    }

    /// Position at time `t` (linear interpolation between samples; clamped).
    pub fn position_at(&self, t: f64) -> Vec3 {
        let p = &self.points;
        if t <= p[0].t {
            return p[0].pos;
        }
        let n = p.len();
        if t >= p[n - 1].t {
            return p[n - 1].pos;
        }
        let i = p.partition_point(|q| q.t <= t).clamp(1, n - 1);
        let (a, b) = (&p[i - 1], &p[i]);
        let f = if b.t > a.t { (t - a.t) / (b.t - a.t) } else { 0.0 };
        a.pos.lerp(b.pos, f)
    }
}

/// Configuration for the batted-ball path simulation.
#[derive(Clone, Copy, Debug)]
pub struct BattedSim<'a> {
    pub model: &'a FlightModel,
    pub ball: &'a BallSpec,
    pub park: &'a Park,
    /// Integration step in flight, s.
    pub dt: f64,
    /// Sample spacing stored in the path, s.
    pub sample_dt: f64,
    /// Bounces with a rebound speed below this roll instead, m/s.
    pub roll_threshold: f64,
    /// Hard time limit, s.
    pub t_max: f64,
}

impl<'a> BattedSim<'a> {
    pub fn new(model: &'a FlightModel, ball: &'a BallSpec, park: &'a Park) -> Self {
        BattedSim { model, ball, park, dt: 0.01, sample_dt: 0.05, roll_threshold: 1.0, t_max: 25.0 }
    }

    /// Simulate from the state right after contact until the ball stops or leaves the field.
    pub fn run(&self, init: BallState, path: &mut BattedPath) {
        path.clear();
        let park = self.park;
        let min_b = park.min_boundary();
        let mut s = init;
        path.points.push(PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true });
        path.apex = s.pos.z;
        let every = ((self.sample_dt / self.dt).round() as usize).max(1);

        loop {
            // ---- airborne segment ----
            let mut count = 0usize;
            let mut apex = path.apex;
            let points = &mut path.points;
            let event = |b: &BallState| {
                let d = b.pos.horiz();
                let wall = if d > min_b { park.boundary(bearing(b.pos)).0 - d } else { f64::INFINITY };
                wall.min(b.pos.z)
            };
            let hit = self.model.advance_until(&mut s, self.dt, self.t_max, event, |b| {
                count += 1;
                if b.pos.z > apex {
                    apex = b.pos.z;
                }
                if count % every == 0 {
                    points.push(PathPoint { t: b.t, pos: b.pos, vel: b.vel, airborne: true });
                }
            });
            path.apex = apex.max(s.pos.z);
            if !hit {
                path.points.push(PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true });
                path.end = PathEnd::TimeLimit;
                return;
            }

            let ang = bearing(s.pos);
            let (b_dist, b_height) = park.boundary(ang);
            let at_wall = s.pos.horiz() >= b_dist - 1e-3 && s.pos.z > 1e-3;
            if at_wall {
                let fair = ang.abs() <= FRAC_PI_4;
                if path.landing.is_none() && path.carry.is_none() {
                    let mut c = s;
                    self.model.advance_until(&mut c, self.dt, self.t_max, |b| b.pos.z, |_| {});
                    path.carry = Some(Vec3::new(c.pos.x, c.pos.y, 0.0));
                }
                if s.pos.z > b_height {
                    path.points.push(PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true });
                    path.end = if fair { PathEnd::OverFence } else { PathEnd::OutOfPlay };
                    return;
                }
                if !fair {
                    // Off the foul-territory stands / netting: dead ball.
                    path.points.push(PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true });
                    path.end = PathEnd::OutOfPlay;
                    return;
                }
                // Rebound off the outfield wall: reverse and damp the outward component.
                let r_hat = Vec3::new(s.pos.x, s.pos.y, 0.0).normalized();
                let v_out = s.vel.dot(r_hat);
                if v_out > 0.0 {
                    s.vel -= r_hat * ((1.0 + park.wall_cor) * v_out);
                    let tang = s.vel - r_hat * s.vel.dot(r_hat) - Vec3::new(0.0, 0.0, s.vel.z);
                    s.vel -= tang * 0.3;
                }
                s.spin = s.spin * 0.3;
                s.pos -= r_hat * 0.02;
                let p = PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true };
                if path.wall.is_none() {
                    path.wall = Some(p);
                }
                path.points.push(p);
                continue;
            }

            // ---- ground contact ----
            s.pos.z = 0.0;
            let touch = PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true };
            if path.landing.is_none() {
                path.landing = Some(touch);
            }
            path.bounces += 1;
            let surf = park.surface_at(s.pos);
            let (v, w) = bounce(self.ball, surf, s.vel, s.spin);
            s.vel = v;
            s.spin = w;
            if v.z >= self.roll_threshold && path.bounces < 12 {
                path.points.push(PathPoint { t: s.t, pos: s.pos, vel: s.vel, airborne: true });
                s.pos.z = 1e-6;
                continue;
            }

            // ---- rolling ----
            let mut speed = (v.x * v.x + v.y * v.y).sqrt();
            let dir = if speed > 1e-9 { Vec3::new(v.x / speed, v.y / speed, 0.0) } else { Vec3::ZERO };
            let mut pos = Vec3::new(s.pos.x, s.pos.y, 0.0);
            let mut t = s.t;
            path.points.push(PathPoint { t, pos, vel: dir * speed, airborne: false });
            while speed > 0.05 && t < self.t_max {
                // Rolling resistance plus air drag (which dominates while the ball is still fast).
                let a = park.surface_at(pos).rolling_decel() + self.model.k * self.model.aero.cd0 * speed * speed;
                let step = self.sample_dt.min(speed / a);
                pos += dir * (speed * step - 0.5 * a * step * step);
                speed = (speed - a * step).max(0.0);
                t += step;
                let d = pos.horiz();
                if d > min_b {
                    let ang = bearing(pos);
                    let (bd, _) = park.boundary(ang);
                    if d >= bd {
                        pos = pos * ((bd - 0.05) / d);
                        path.points.push(PathPoint { t, pos, vel: Vec3::ZERO, airborne: false });
                        if ang.abs() > FRAC_PI_4 {
                            path.end = PathEnd::OutOfPlay;
                        } else {
                            path.end = PathEnd::Stopped;
                            if path.wall.is_none() {
                                path.wall = Some(PathPoint { t, pos, vel: Vec3::ZERO, airborne: false });
                            }
                        }
                        return;
                    }
                }
                path.points.push(PathPoint { t, pos, vel: dir * speed, airborne: false });
            }
            path.end = if t >= self.t_max { PathEnd::TimeLimit } else { PathEnd::Stopped };
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ball::AeroModel;
    use crate::env::{Atmosphere, Wind};
    use crate::flight::{spin_from_components, velocity_from_angles};
    use crate::units::{FT, MPH, RPM};

    fn setup() -> (FlightModel, BallSpec, Park) {
        let ball = BallSpec::mlb();
        let model = FlightModel::new(&ball, AeroModel::nathan_batted(), &Atmosphere::from_imperial(72.0, 29.92, 50.0, 0.0), Wind::CALM);
        (model, ball, Park::mlb_average())
    }

    fn hit(ev_mph: f64, la_deg: f64, spray_deg: f64, backspin_rpm: f64) -> BallState {
        let th = la_deg.to_radians();
        let ph = spray_deg.to_radians();
        BallState::new(Vec3::new(0.0, 0.6, 0.9), velocity_from_angles(ev_mph * MPH, th, ph), spin_from_components(th, ph, backspin_rpm * RPM, 0.0, 0.0))
    }

    #[test]
    fn crushed_fly_ball_is_a_home_run() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        BattedSim::new(&m, &b, &p).run(hit(108.0, 27.0, 5.0, 2400.0), &mut path);
        assert!(path.is_home_run(), "end {:?} landing {:?}", path.end, path.landing);
        assert!(path.apex > 20.0);
    }

    #[test]
    fn warning_track_fly_ball_stays_in() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        BattedSim::new(&m, &b, &p).run(hit(95.0, 30.0, 0.0, 2500.0), &mut path);
        let land = path.landing.expect("lands in play");
        let d = land.pos.horiz() / FT;
        assert!(d > 330.0 && d < 395.0, "carry {d} ft");
        assert!(!path.is_home_run());
        assert_eq!(path.end, PathEnd::Stopped);
        // Samples are ordered in time and the path is continuous.
        for w in path.points.windows(2) {
            assert!(w[1].t >= w[0].t);
            assert!((w[1].pos - w[0].pos).norm() < 4.0, "gap {}", (w[1].pos - w[0].pos).norm());
        }
    }

    #[test]
    fn drive_off_the_wall_rebounds_into_play() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        let sim = BattedSim::new(&m, &b, &p);
        // Between a ball that lands short and one that clears the fence there is a
        // window of launch angles that strike the wall on the fly.
        let mut wall_hits = 0;
        let mut la = 12.0;
        while la <= 30.0 {
            sim.run(hit(106.0, la, 0.0, 1200.0 + 60.0 * la), &mut path);
            if let Some(wp) = path.wall {
                if wp.airborne && path.landing.is_none_or(|l| l.t > wp.t) {
                    wall_hits += 1;
                    assert_eq!(path.end, PathEnd::Stopped);
                    assert!(wp.pos.z > 0.0 && wp.pos.z <= 8.5 * FT + 1e-6);
                    let end = path.last();
                    let (bd, _) = p.boundary(bearing(end.pos));
                    assert!(end.pos.horiz() < bd, "ball must stay inside the fence");
                    // It comes back toward the infield after the carom.
                    assert!(end.pos.horiz() < wp.pos.horiz());
                }
            }
            la += 0.25;
        }
        assert!(wall_hits >= 1, "no launch angle produced a wall ball");
    }

    #[test]
    fn ground_ball_bounces_then_rolls_and_slows_down() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        BattedSim::new(&m, &b, &p).run(hit(95.0, -8.0, 10.0, -1500.0), &mut path);
        let land = path.landing.unwrap();
        assert!(land.t < 0.2 && land.pos.horiz() < 12.0, "first bounce at {} s, {} m", land.t, land.pos.horiz());
        assert!(path.bounces >= 2);
        assert_eq!(path.end, PathEnd::Stopped);
        // Time to reach shortstop depth (about 44 m from home): roughly 1.2 to 2.2 s.
        let t_ss = path.points.iter().find(|q| q.pos.horiz() >= 44.0).map(|q| q.t).expect("reaches the infield dirt");
        assert!(t_ss > 1.1 && t_ss < 2.4, "time to 44 m: {t_ss}");
        let stop = path.last().pos.horiz();
        assert!(stop > 50.0 && stop < 115.0, "roll-out distance {stop} m");
    }

    #[test]
    fn foul_pop_into_the_stands_is_out_of_play() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        BattedSim::new(&m, &b, &p).run(hit(85.0, 40.0, 75.0, 3000.0), &mut path);
        assert_eq!(path.end, PathEnd::OutOfPlay);
    }

    #[test]
    fn interpolation_is_consistent() {
        let (m, b, p) = setup();
        let mut path = BattedPath::default();
        BattedSim::new(&m, &b, &p).run(hit(100.0, 20.0, 0.0, 2000.0), &mut path);
        let mid = path.points[path.points.len() / 2];
        assert!((path.position_at(mid.t) - mid.pos).norm() < 1e-9);
        assert_eq!(path.position_at(-1.0), path.points[0].pos);
        assert_eq!(path.position_at(1e9), path.last().pos);
    }
}
