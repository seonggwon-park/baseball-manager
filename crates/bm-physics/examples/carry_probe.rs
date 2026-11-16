//! Distance of single batted balls under the engine's batted-ball aerodynamics, for a few spins.
use bm_physics::units::{FT, MPH, RPM};
use bm_physics::*;
fn main() {
    let ball = BallSpec::mlb();
    let atm = Atmosphere::new(24.0, 760.0, 55.0, 180.0);
    for (name, aero) in [("total", AeroModel { spin_mode: SpinMode::Total, ..AeroModel::batted_2025() }), ("transverse", AeroModel::batted_2025())] {
        let model = FlightModel::new(&ball, aero, &atm, Wind::CALM);
        println!("{name}:");
        for &(ev, la) in &[(108.0, 17.0), (108.0, 22.0), (108.0, 27.0), (108.0, 32.0), (108.0, 42.0)] {
            let (th, ph) = (f64::to_radians(la), 0.0f64);
            let nb = -763.0 + 120.0 * la;
            let sb = -1339.0 + 143.0 * la;
            print!("  EV {ev} LA {la}: ");
            for &(wb, ws, wg) in &[(nb, -849.0, 0.0), (sb, 133.0, 0.0), (sb, 133.0, 1580.0), (sb, 1500.0, 1580.0)] {
                let mut s = BallState::new(
                    Vec3::new(0.0, 0.9, 0.8),
                    velocity_from_angles(ev * MPH, th, ph),
                    spin_from_components(th, ph, wb * RPM, ws * RPM, wg * RPM),
                );
                let mut apex = 0.0f64;
                model.advance_until(&mut s, 0.005, 20.0, |b| b.pos.z, |b| apex = apex.max(b.pos.z));
                print!("[b {wb:.0} s {ws:.0} g {wg:.0}] {:.0} ft (t {:.1} s)  ", s.pos.horiz() / FT, s.t);
            }
            println!();
        }
    }
}
