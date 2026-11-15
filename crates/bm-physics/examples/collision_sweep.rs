//! Sweep the bat-ball impact parameter under game-like conditions and print the
//! resulting exit velocity, launch angle, spin and carry distance.
//!
//! Conditions follow Kensrud, Nathan & Smith (2017), Sec. V B: 85 mph pitch at the
//! plate descending at 6 deg with 2000 rpm backspin; bat speed 73 mph at the impact
//! point with a 6 deg attack angle.
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::*;

fn main() {
    let ball = BallSpec::mlb();
    let bat = BatSpec::mlb_wood();
    let args: Vec<f64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let ex = args.first().copied().unwrap_or(bat.tangential_cor);
    let mu = args.get(1).copied().unwrap_or(bat.friction);
    let bat = BatSpec { tangential_cor: ex, friction: mu, ..bat };
    let d = bat.sweet_spot_from_tip;
    let descent = 6.0f64.to_radians();
    let attack = 6.0f64.to_radians();
    let ball_vel = Vec3::new(0.0, -85.0 * MPH * descent.cos(), -85.0 * MPH * descent.sin());
    let ball_spin = Vec3::new(-2000.0 * RPM, 0.0, 0.0);
    let bat_vel = Vec3::new(0.0, 73.0 * MPH * attack.cos(), 73.0 * MPH * attack.sin());
    let model = FlightModel::new(&ball, AeroModel::nathan_batted(), &Atmosphere::from_imperial(72.0, 29.92, 50.0, 0.0), Wind::CALM);
    println!("e_x = {ex}, mu = {mu}");
    println!("offset(in)  EV(mph)  LA(deg)  backspin(rpm)  statcast_fit(rpm)  slipped  carry(ft)  hang(s)");
    let mut off = -1.6;
    while off <= 2.41 {
        let c = Contact { ball_vel, ball_spin, bat_axis: Vec3::X, bat_vel, offset: off * INCH, dist_from_tip: d };
        match collide(&ball, &bat, &c) {
            Ok(r) => {
                let la = launch_angle(r.vel).to_degrees();
                let (wb, _, _) = spin_components(r.vel, r.spin);
                let fit = -763.0 + 120.0 * la;
                let mut s = BallState::new(Vec3::new(0.0, 0.6, 0.9), r.vel, r.spin);
                model.advance_until(&mut s, 0.005, 30.0, |b| b.pos.z, |_| {});
                println!(
                    "{:9.2}  {:7.1}  {:7.1}  {:13.0}  {:17.0}  {:7}  {:9.0}  {:7.2}",
                    off,
                    r.vel.norm() / MPH,
                    la,
                    wb / RPM,
                    fit,
                    r.slipped,
                    s.pos.horiz() / FT,
                    s.t
                );
            }
            Err(e) => println!("{off:9.2}  {e:?}"),
        }
        off += 0.2;
    }
}
