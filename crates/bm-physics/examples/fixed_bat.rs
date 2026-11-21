//! The collision model against the fixed-bat experiment of Nakashima, Morishita, Yagi &
//! Sakurai, "Effect of vertical bat angle at impact on batted ball spin in baseball
//! hitting", Jpn. J. Biomech. Sports Exerc. 24, 8-18 (2020).
//!
//! Balls with 22 rps of backspin were fired at 88 m/s onto a bat clamped in a frame,
//! either level or with its head 40 deg below its grip, turned 0 or +-7.5 deg in the
//! horizontal plane. Over balls launched 0-60 deg upward they found side spin rising
//! with the horizontal angle of the batted ball by 1.077 rps/deg (level bat) and
//! 1.120 rps/deg (tilted), and backspin rising with the vertical angle by 0.693 and
//! 0.860 rps/deg. A ball sliding along a bat turned away from square picks up side
//! spin from friction along the bat's length, so the first pair of slopes measures that
//! friction directly.
//!
//! cargo run --release -p bm-physics --example fixed_bat

use bm_physics::units::RPM;
use bm_physics::{collide, launch_angle, spin_components, spray_angle, BallSpec, BatSpec, Contact, Vec3};

fn fit(points: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = points.len() as f64;
    let (sx, sy) = points.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    let (mx, my) = (sx / n, sy / n);
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in points {
        sxx += (x - mx) * (x - mx);
        sxy += (x - mx) * (y - my);
        syy += (y - my) * (y - my);
    }
    let b = sxy / sxx;
    (b, my - b * mx, sxy / (sxx * syy).sqrt())
}

fn main() {
    let ball = BallSpec::mlb();
    // The clamped bamboo bat returned the balls at 16.0 +- 3.1 m/s: much less than a
    // free wooden bat would at this speed. `bpf` scales the normal COR to match.
    for bpf in [1.0, 0.75, 0.55] {
        println!("normal COR scaled by {bpf}:");
        run(&ball, bpf);
    }
    println!("measured: level 1.077 rps/deg (side), 0.693 rps/deg (back); 40 deg down 1.120 and 0.860; exit speed 16.0 m/s");
}

fn run(ball: &BallSpec, bpf: f64) {
    // A bat clamped along its length: no recoil.
    let mut bat = BatSpec::mlb_wood().weighted(1.0e6);
    bat.bpf = bpf;
    let speed = 88.0;
    // The machine throws toward the bat (-y) with backspin.
    let ball_vel = Vec3::new(0.0, -speed, 0.0);
    let ball_spin = Vec3::new(-22.1 * 60.0 * RPM, 0.0, 0.0);
    let rps = |w: f64| w / (2.0 * std::f64::consts::PI);
    for (label, tilt_deg) in [("level bat", 0.0f64), ("head 40 deg down", 40.0)] {
        let mut side = Vec::new();
        let mut back = Vec::new();
        let mut exit = 0.0;
        for yaw_deg in [-7.5f64, 0.0, 7.5] {
            let (tilt, yaw) = (tilt_deg.to_radians(), yaw_deg.to_radians());
            // Tip toward +x (a right-handed batter's outside), dropped by the tilt and
            // turned toward the pitcher by the yaw (+ = facing the pull side).
            let axis = Vec3::new(tilt.cos() * yaw.cos(), tilt.cos() * yaw.sin(), -tilt.sin());
            let reach = ball.radius + bat.radius_at(bat.sweet_spot_from_tip);
            for i in 0..=60 {
                let offset = reach * (-0.95 + 1.9 * i as f64 / 60.0);
                for along in [0.10, 0.125, 0.15, 0.175, 0.20] {
                    let c = Contact { ball_vel, ball_spin, bat_axis: axis, bat_vel: Vec3::ZERO, offset, dist_from_tip: along };
                    let Ok(r) = collide(ball, &bat, &c) else { continue };
                    if r.vel.y <= 0.0 {
                        continue;
                    }
                    let la = launch_angle(r.vel).to_degrees();
                    if !(0.0..=60.0).contains(&la) {
                        continue;
                    }
                    // Horizontal angle + toward -x (the pull side of a right-handed
                    // batter, counter-clockwise seen from above), as in the paper.
                    let horiz = -spray_angle(r.vel).to_degrees();
                    let (wb, ws, _) = spin_components(r.vel, r.spin);
                    exit += r.vel.norm();
                    side.push((horiz, rps(ws)));
                    back.push((la, rps(wb)));
                }
            }
        }
        let (bs, as_, rs) = fit(&side);
        let (bb, ab, rb) = fit(&back);
        println!(
            "  {label}: side spin = {bs:.3} * horizontal angle {as_:+.2} rps (r {rs:.3}); backspin = {bb:.3} * vertical angle {ab:+.2} rps (r {rb:.3}); exit {:.1} m/s; {} balls",
            exit / side.len() as f64,
            side.len()
        );
    }
}
