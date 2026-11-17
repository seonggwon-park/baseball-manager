//! Fly a list of batted balls the same way, whatever produced them, for comparisons that
//! should differ only in the outcome (`tools/statcast/catch_check.py`).
//!
//! Reads `exit_speed_mph,launch_angle_deg,direction_deg,batter_sign` lines (direction + toward
//! first base, batter_sign +1 for a right-handed batter) and writes
//! `land_x_m,land_y_m,hang_s,apex_m` for each: the engine's batted-ball drag and lift
//! (`AeroModel::batted_2025`), Nathan's Statcast spin regressions (as in `flight_check`),
//! air at the engine's game average (1.155 kg/m^3), from 0.9 m above the plate.
//!
//! cargo run --release -p bm-physics --example fly_list -- IN.csv > OUT.csv
use bm_physics::units::{MPH, RPM};
use bm_physics::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[1]).expect("read csv");
    let ball = BallSpec::mlb();
    let model =
        FlightModel { k: 0.5 * 1.155 * ball.cross_section() / ball.mass, radius: ball.radius, aero: AeroModel::batted_2025(), wind: Wind::CALM, g: units::G };
    println!("land_x,land_y,hang,apex");
    for line in text.lines() {
        let f: Vec<f64> = line.split(',').map(|s| s.trim().parse().unwrap_or(f64::NAN)).collect();
        if f.len() < 4 || f.iter().take(4).any(|v| !v.is_finite()) {
            println!("NaN,NaN,NaN,NaN");
            continue;
        }
        let (ev, la, dir, sign) = (f[0], f[1], f[2], f[3]);
        let (th, ph) = (la.to_radians(), dir.to_radians());
        let wb = -763.0 + 120.0 * la + 21.0 * dir * sign;
        let ws = -849.0 * sign - 94.0 * dir;
        let mut s = BallState::new(Vec3::new(0.0, 0.6, 0.9), velocity_from_angles(ev * MPH, th, ph), spin_from_components(th, ph, wb * RPM, ws * RPM, 0.0));
        let mut apex: f64 = 0.0;
        model.advance_until(&mut s, 0.005, 20.0, |b| b.pos.z, |b| apex = apex.max(b.pos.z));
        println!("{:.3},{:.3},{:.3},{:.3}", s.pos.x, s.pos.y, s.t, apex);
    }
}
