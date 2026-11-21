//! Projected distance of measured batted balls with the engine's batted-ball model, ball by
//! ball (no fences): Nathan's spin regressions as in `flight_check`, `AeroModel::batted_2025`,
//! and the air of each ball's park and game.
//!
//! Reads `exit_speed_mph,launch_angle_deg,spray_deg,batter_sign,distance_ft,temp_c,elevation_m`
//! lines (an optional eighth and ninth column: wind x and y in m/s, x toward right field, y
//! toward centre field) and prints each line with the model's distance (ft) and descent angle (deg) appended.
use bm_physics::units::{FT, MPH, RPM};
use bm_physics::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[1]).expect("read csv");
    let ball = BallSpec::mlb();
    // Optional second argument: a multiplier on the batted ball's drag (cd0).
    let drag: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut aero = AeroModel::batted_2025();
    aero.cd0 *= drag;
    for line in text.lines() {
        let f: Vec<f64> = line.split(',').map(|x| x.trim().parse().unwrap_or(f64::NAN)).collect();
        if f.len() < 7 || f[..7].iter().any(|x| !x.is_finite()) {
            continue;
        }
        let (ev, la, spray, sign, temp, elevation) = (f[0], f[1], f[2], f[3], f[5], f[6]);
        let wind =
            if f.len() >= 9 && f[7].is_finite() && f[8].is_finite() { Wind { velocity: Vec3::new(f[7], f[8], 0.0), min_height: 0.0 } } else { Wind::CALM };
        let model = FlightModel::new(&ball, aero, &Atmosphere::new(temp, 760.0, 55.0, elevation), wind);
        let (th, ph) = (la.to_radians(), spray.to_radians());
        let wb = -763.0 + 120.0 * la + 21.0 * spray * sign;
        let ws = -849.0 * sign - 94.0 * spray;
        let mut s = BallState::new(Vec3::new(0.0, 0.6, 0.9), velocity_from_angles(ev * MPH, th, ph), spin_from_components(th, ph, wb * RPM, ws * RPM, 0.0));
        model.advance_until(&mut s, 0.005, 20.0, |b| b.pos.z, |_| {});
        let descent = (-s.vel.z).atan2(s.vel.horiz()).to_degrees();
        println!("{line},{:.1},{descent:.1}", s.pos.horiz() / FT);
    }
}
