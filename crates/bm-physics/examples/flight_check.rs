//! Check the batted-ball flight model against measured fly balls.
//!
//! Reads `exit_speed_mph,launch_angle_deg,spray_deg,batter_sign,distance_ft,...`
//! lines (Statcast batted balls) from the file given as the first argument, flies
//! each ball with Nathan's spin model (backspin = -763 + 120 LA + 21 spray * sign,
//! sidespin = -849 sign - 94 spray) and compares the projected distance.
//! Optional arguments: air density (kg/m^3) and cd0.
use bm_physics::units::{FT, MPH, RPM};
use bm_physics::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[1]).expect("read csv");
    let rho: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1.155);
    let cd0: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.3008);
    let ball = BallSpec::mlb();
    let mut aero = AeroModel::nathan_batted();
    aero.cd0 = cd0;
    let model = FlightModel { k: 0.5 * rho * ball.cross_section() / ball.mass, radius: ball.radius, aero, wind: Wind::CALM, g: units::G };
    let mut bins = vec![(0.0f64, 0.0f64, 0.0f64, 0.0f64); 8];
    for line in text.lines() {
        let f: Vec<&str> = line.split(',').collect();
        let (ev, la, spray, sign, dist): (f64, f64, f64, f64, f64) =
            (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap());
        let (th, ph) = (la.to_radians(), spray.to_radians());
        let wb = -763.0 + 120.0 * la + 21.0 * spray * sign;
        let ws = -849.0 * sign - 94.0 * spray;
        let mut s = BallState::new(Vec3::new(0.0, 0.6, 0.9), velocity_from_angles(ev * MPH, th, ph), spin_from_components(th, ph, wb * RPM, ws * RPM, 0.0));
        model.advance_until(&mut s, 0.005, 20.0, |b| b.pos.z, |_| {});
        let sim = s.pos.horiz() / FT;
        let i = (((ev - 85.0) / 5.0) as usize).min(7);
        bins[i].0 += 1.0;
        bins[i].1 += sim;
        bins[i].2 += dist;
        bins[i].3 += (sim - dist).powi(2);
    }
    println!("rho = {rho}, cd0 = {cd0}");
    println!("EV bin      n    model   measured   diff    rms");
    let (mut n, mut a, mut b) = (0.0, 0.0, 0.0);
    for (i, (c, m, d, sq)) in bins.iter().enumerate() {
        if *c > 0.0 {
            println!("[{:3},{:3}) {:5.0}  {:6.1}   {:6.1}  {:+6.1}  {:5.1}", 85 + 5 * i, 90 + 5 * i, c, m / c, d / c, (m - d) / c, (sq / c).sqrt());
        }
        n += c;
        a += m;
        b += d;
    }
    println!("all        {:5.0}  {:6.1}   {:6.1}  {:+6.1}", n, a / n, b / n, (a - b) / n);
}
