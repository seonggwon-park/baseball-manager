//! Where the simulation's batted balls gain or lose distance, in the average park.
//!
//! * Balls in play at 20-35 deg are flown again from their contact point, to the ground
//!   and without walls or fielders, with (a) their own spin, (b) their own spin without
//!   the gyro component and (c) the spin Nathan's Statcast regressions give for their
//!   launch angle and direction (bm-physics `examples/flight_check.rs` flies measured
//!   balls the same way): mean distances and spins by exit velocity.
//! * Hard line drives (15-20 deg, 105-115 mph) by direction: spins, both distances and
//!   the home-run share.
//! * Home-run share by the distance beyond the fence at the landing bearing.
//! * Optionally the balls at 15-45 deg and 85 mph and up as CSV for `flight_check`.
//!
//! cargo run --release -p bm-engine --example carry_check -- [games] [seed] [csv file]

use bm_engine::{play_game, Club, Hand, Level, MatchEnv, Observer, PitchEvent, PitchResult, Rng};
use bm_physics::units::{FT, MPH, RPM};
use bm_physics::{spin_from_components, velocity_from_angles, BallState, PathEnd};

#[derive(Default, Clone, Copy)]
struct Bin {
    n: f64,
    reported: f64,
    own: f64,
    no_gyro: f64,
    nathan: f64,
    back: f64,
    side: f64,
    gyro: f64,
    nathan_back: f64,
    nathan_side: f64,
    la: f64,
    pull: f64,
    hr: f64,
    ev: f64,
    start_y: f64,
    start_z: f64,
    nathan_std: f64,
}

struct Check {
    env: MatchEnv,
    bins: [Bin; 7],
    /// Balls in play in the format of the bm-physics `flight_check` example.
    csv: Vec<String>,
    /// [sector][margin bin] = (balls, home runs): margin = distance - fence at the landing bearing.
    margin: [[(f64, f64); 9]; 3],
    /// Line drives, launch angle 15-20 deg and exit velocity 105-115 mph, by pull angle
    /// (< -15, -15..0, 0..15, 15..30, >= 30): n, back, hook, |gyro|, own distance, Nathan distance, HR.
    ld: [[f64; 7]; 5],
}

impl Check {
    #[allow(clippy::too_many_arguments)]
    fn fly(&self, start: bm_physics::Vec3, ev: f64, th: f64, ph: f64, wb: f64, ws: f64, wg: f64) -> f64 {
        self.land(start, ev, th, ph, wb, ws, wg).horiz() / FT
    }

    #[allow(clippy::too_many_arguments)]
    fn land(&self, start: bm_physics::Vec3, ev: f64, th: f64, ph: f64, wb: f64, ws: f64, wg: f64) -> bm_physics::Vec3 {
        let mut s = BallState::new(start, velocity_from_angles(ev, th, ph), spin_from_components(th, ph, wb, ws, wg));
        self.env.batted_flight.advance_until(&mut s, 0.005, 20.0, |b| b.pos.z, |_| {});
        s.pos
    }
}

impl Observer for Check {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.result != PitchResult::InPlay {
            return;
        }
        let (Some(info), Some(path)) = (e.batted, e.path) else { return };
        let la = info.launch_angle.to_degrees();
        let ev = info.exit_speed / MPH;
        if !info.bunt && la >= 15.0 {
            let hr = path.end == PathEnd::OverFence;
            // As Statcast reports it: home runs projected to field level, balls off the wall
            // where they hit it, others where they landed or were caught.
            let end = if hr && path.landing.is_none() {
                self.land(path.points[0].pos, info.exit_speed, info.launch_angle, info.spray, info.backspin, info.sidespin, info.gyrospin)
            } else if let Some(w) = path.wall {
                w.pos
            } else {
                path.landing.map_or_else(|| path.last().pos, |l| l.pos)
            };
            let d = end.horiz();
            let bearing = end.x.atan2(end.y);
            if d / FT >= 250.0 && bearing.abs() <= std::f64::consts::FRAC_PI_4 {
                let m = (d - self.env.park.fence_at(bearing).0) / FT;
                let deg = bearing.to_degrees().abs();
                let sec = if deg >= 33.75 {
                    0
                } else if deg >= 11.25 {
                    1
                } else {
                    2
                };
                let mb = ((m / 10.0).floor() as i64 + 4).clamp(0, 8) as usize;
                self.margin[sec][mb].0 += 1.0;
                if hr {
                    self.margin[sec][mb].1 += 1.0;
                }
            }
        }
        if !info.bunt && (15.0..45.0).contains(&la) && ev >= 85.0 {
            let sign = if e.batter_hand == Hand::Right { 1 } else { -1 };
            let hr = path.end == PathEnd::OverFence;
            let mut d = path.landing.map_or_else(|| path.last().pos.horiz(), |l| l.pos.horiz()) / FT;
            if hr && path.landing.is_none() {
                d = self.fly(path.points[0].pos, info.exit_speed, info.launch_angle, info.spray, info.backspin, info.sidespin, 0.0);
            }
            self.csv.push(format!("{ev:.1},{la:.1},{:.2},{sign},{d:.0},{}", info.spray.to_degrees(), if hr { "home_run" } else { "other" }));
        }
        if !info.bunt && (15.0..20.0).contains(&la) && (105.0..115.0).contains(&ev) {
            let sign = if e.batter_hand == Hand::Right { 1.0 } else { -1.0 };
            let spray = info.spray.to_degrees();
            let pull = -spray * sign;
            let wb = (-763.0 + 120.0 * la + 21.0 * spray * sign) * RPM;
            let ws = (-849.0 * sign - 94.0 * spray) * RPM;
            let start = path.points[0].pos;
            let own = self.fly(start, info.exit_speed, info.launch_angle, info.spray, info.backspin, info.sidespin, info.gyrospin);
            let nathan = self.fly(start, info.exit_speed, info.launch_angle, info.spray, wb, ws, 0.0);
            let k = if pull < -15.0 {
                0
            } else if pull < 0.0 {
                1
            } else if pull < 15.0 {
                2
            } else if pull < 30.0 {
                3
            } else {
                4
            };
            let r = &mut self.ld[k];
            r[0] += 1.0;
            r[1] += info.backspin / RPM;
            r[2] += -info.sidespin / RPM * e.batter_hand.pull_sign();
            r[3] += (info.gyrospin / RPM).abs();
            r[4] += own;
            r[5] += nathan;
            if path.end == PathEnd::OverFence && path.landing.is_none() {
                r[6] += 1.0;
            }
        }
        if info.bunt || !(20.0..35.0).contains(&la) || !(80.0..115.0).contains(&ev) {
            return;
        }
        let start = path.points[0].pos;
        let (th, ph) = (info.launch_angle, info.spray);
        let mut reported = path.landing.map_or_else(|| path.last().pos.horiz(), |l| l.pos.horiz()) / FT;
        let own = self.fly(start, info.exit_speed, th, ph, info.backspin, info.sidespin, info.gyrospin);
        if path.end == PathEnd::OverFence && path.landing.is_none() {
            reported = self.fly(start, info.exit_speed, th, ph, info.backspin, info.sidespin, 0.0);
        }
        let no_gyro = self.fly(start, info.exit_speed, th, ph, info.backspin, info.sidespin, 0.0);
        // Nathan's regressions, in the convention of bm-physics `examples/flight_check.rs`
        // (sign +1 for a right-handed batter, spray + toward right field).
        let sign = if e.batter_hand == Hand::Right { 1.0 } else { -1.0 };
        let spray = ph.to_degrees();
        let wb = (-763.0 + 120.0 * la + 21.0 * spray * sign) * RPM;
        let ws = (-849.0 * sign - 94.0 * spray) * RPM;
        let nathan = self.fly(start, info.exit_speed, th, ph, wb, ws, 0.0);
        let nathan_std = self.fly(bm_physics::Vec3::new(0.0, 0.6, 0.9), info.exit_speed, th, ph, wb, ws, 0.0);
        let b = &mut self.bins[(((ev - 80.0) / 5.0) as usize).min(6)];
        b.n += 1.0;
        b.reported += reported;
        b.own += own;
        b.no_gyro += no_gyro;
        b.nathan += nathan;
        b.back += info.backspin / RPM;
        b.side += (info.sidespin / RPM).abs();
        b.gyro += (info.gyrospin / RPM).abs();
        b.nathan_back += wb / RPM;
        b.nathan_side += (ws / RPM).abs();
        b.la += la;
        b.ev += ev;
        b.start_y += start.y;
        b.start_z += start.z;
        b.nathan_std += nathan_std;
        b.pull += -spray * sign;
        if path.end == PathEnd::OverFence {
            b.hr += 1.0;
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let games: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(2000);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(7);
    let csv_path = args.next();
    let env = MatchEnv::neutral_mlb();
    let level = Level::mlb();
    let mut rng = Rng::new(seed);
    let clubs: Vec<Club> = (0..30).map(|i| Club::random(&mut rng, 1000 * (i + 1), &level)).collect();
    let mut check = Check { env: env.clone(), bins: [Bin::default(); 7], csv: Vec::new(), margin: [[(0.0, 0.0); 9]; 3], ld: [[0.0; 7]; 5] };
    for _ in 0..games {
        let a = rng.below(clubs.len());
        let h = (a + 1 + rng.below(clubs.len() - 1)) % clubs.len();
        let away = clubs[a].team_for_game(rng.below(5), env.rules.dh);
        let home = clubs[h].team_for_game(rng.below(5), env.rules.dh);
        play_game(&env, &away, &home, &mut rng, &mut check);
    }
    println!("line drives 15-20 deg, 105-115 mph, by pull angle: n, backspin, hook spin, |gyro|, own-spin distance, Nathan-spin distance, HR");
    for (k, name) in ["< -15", "-15..0", "0..15", "15..30", ">= 30"].iter().enumerate() {
        let r = &check.ld[k];
        let n = r[0].max(1.0);
        println!(
            "  pull {name:>7}  n {:5.0}  back {:5.0}  hook {:6.0}  gyro {:5.0}  own {:5.1}  Nathan {:5.1}  HR {:.3}",
            r[0],
            r[1] / n,
            r[2] / n,
            r[3] / n,
            r[4] / n,
            r[5] / n,
            r[6] / n
        );
    }
    println!("home-run share by distance beyond the fence at the landing bearing (LA >= 15, distance >= 250 ft)");
    println!("margin ft        line          alley         centre");
    for mb in 0..9 {
        let lo = (mb as i64 - 4) * 10;
        print!("{lo:4}..{:<4}", lo + 10);
        for sector in &check.margin {
            let (n, h) = sector[mb];
            print!("   {n:6.0} {:.2}", h / n.max(1.0));
        }
        println!();
    }
    if let Some(path) = csv_path {
        std::fs::write(&path, check.csv.join("\n") + "\n").expect("write csv");
    }
    println!("launch angle 20-35 deg; distances in ft to the ground, spins in rpm (|side|, |gyro| are mean magnitudes)");
    println!("EV bin        n    LA   pull    HR  reported  own spin  no gyro  Nathan spin |  back  |side|  |gyro| | Nathan back |side|");
    for b in check.bins.iter().filter(|b| b.n > 0.0) {
        let m = |x: f64| x / b.n;
        println!(
            "  mean EV {:.2} mph, contact point y {:.2} m z {:.2} m; Nathan spin from (0, 0.6, 0.9): {:.1} ft",
            m(b.ev),
            m(b.start_y),
            m(b.start_z),
            m(b.nathan_std)
        );
    }
    for (i, b) in check.bins.iter().enumerate() {
        if b.n == 0.0 {
            continue;
        }
        let m = |x: f64| x / b.n;
        println!(
            "[{:3},{:3})  {:6.0}  {:4.1}  {:5.1}  {:.3}  {:8.1}  {:8.1}  {:7.1}  {:11.1} | {:5.0}  {:5.0}  {:6.0} | {:11.0} {:5.0}",
            80 + 5 * i,
            85 + 5 * i,
            b.n,
            m(b.la),
            m(b.pull),
            m(b.hr),
            m(b.reported),
            m(b.own),
            m(b.no_gyro),
            m(b.nathan),
            m(b.back),
            m(b.side),
            m(b.gyro),
            m(b.nathan_back),
            m(b.nathan_side)
        );
    }
}
