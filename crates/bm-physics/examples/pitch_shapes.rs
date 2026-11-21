//! Solve for the spin efficiency ("active spin") and spin tilt of each pitch type
//! so that the modelled movement matches the 2025 Statcast averages (measured from
//! a 30-day, 126k-pitch sample of public Baseball Savant data).
//!
//! Output: the hand-normalised parameters used by the engine's league-average arsenal.
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::*;

struct Target {
    name: &'static str,
    mph: f64,
    rpm: f64,
    pfx_arm_in: f64, // Statcast pfx_x toward the arm side, in
    pfx_z_in: f64,
    ext_ft: f64,
    rel_side_ft: f64,
    rel_h_ft: f64,
    plate_ratio: f64, // measured plate speed / release speed
}

fn main() {
    let targets = [
        Target {
            name: "FourSeam",
            mph: 94.5,
            rpm: 2321.0,
            pfx_arm_in: 7.8,
            pfx_z_in: 15.7,
            ext_ft: 6.52,
            rel_side_ft: 1.86,
            rel_h_ft: 5.83,
            plate_ratio: 0.9127,
        },
        Target {
            name: "Sinker",
            mph: 93.8,
            rpm: 2186.0,
            pfx_arm_in: 15.1,
            pfx_z_in: 7.2,
            ext_ft: 6.40,
            rel_side_ft: 2.01,
            rel_h_ft: 5.63,
            plate_ratio: 0.9146,
        },
        Target {
            name: "Cutter",
            mph: 89.9,
            rpm: 2413.0,
            pfx_arm_in: -2.0,
            pfx_z_in: 8.0,
            ext_ft: 6.37,
            rel_side_ft: 1.88,
            rel_h_ft: 5.84,
            plate_ratio: 0.9220,
        },
        Target {
            name: "Slider",
            mph: 86.3,
            rpm: 2426.0,
            pfx_arm_in: -4.0,
            pfx_z_in: 1.8,
            ext_ft: 6.43,
            rel_side_ft: 1.90,
            rel_h_ft: 5.83,
            plate_ratio: 0.9227,
        },
        Target {
            name: "Sweeper",
            mph: 82.5,
            rpm: 2603.0,
            pfx_arm_in: -13.5,
            pfx_z_in: 1.0,
            ext_ft: 6.32,
            rel_side_ft: 2.22,
            rel_h_ft: 5.52,
            plate_ratio: 0.9173,
        },
        Target {
            name: "Curve",
            mph: 79.9,
            rpm: 2600.0,
            pfx_arm_in: -9.1,
            pfx_z_in: -10.1,
            ext_ft: 6.37,
            rel_side_ft: 1.75,
            rel_h_ft: 5.96,
            plate_ratio: 0.9203,
        },
        Target {
            name: "Changeup",
            mph: 86.0,
            rpm: 1809.0,
            pfx_arm_in: 14.5,
            pfx_z_in: 4.4,
            ext_ft: 6.45,
            rel_side_ft: 1.99,
            rel_h_ft: 5.72,
            plate_ratio: 0.9179,
        },
        Target {
            name: "Splitter",
            mph: 86.4,
            rpm: 1320.0,
            pfx_arm_in: 10.8,
            pfx_z_in: 3.4,
            ext_ft: 6.54,
            rel_side_ft: 1.82,
            rel_h_ft: 5.80,
            plate_ratio: 0.9200,
        },
    ];
    // Average MLB game conditions: about 24 C, 55 % RH, 180 m elevation -> rho ~ 1.16 kg/m^3.
    let atm = Atmosphere::new(24.0, 760.0, 55.0, 180.0);
    let ball = BallSpec::mlb();
    let model = FlightModel::new(&ball, AeroModel::pitched_2025(), &atm, Wind::CALM);
    println!("air density {:.4} kg/m^3", atm.air_density());
    println!(
        "{:9} {:>7} {:>8} | {:>9} {:>9} | {:>8} {:>8} | {:>9} {:>9} | {:>6}",
        "pitch", "active", "tilt_deg", "pfx_arm", "pfx_z", "tgt_arm", "tgt_z", "v_ratio", "tgt_ratio", "time"
    );
    for t in &targets {
        // Right-hander: arm side is -x. Solve (active, tilt) by Gauss-Newton with finite differences.
        let base = |active: f64, tilt_arm: f64| PitchSpec {
            release_pos: Vec3::new(-t.rel_side_ft * FT, RUBBER_Y - t.ext_ft * FT, t.rel_h_ft * FT),
            speed: t.mph * MPH,
            spin_rate: t.rpm * RPM,
            spin_tilt: -tilt_arm,
            spin_active: active,
            gyro_sign: 1.0,
        };
        let eval = |active: f64, tilt_arm: f64| {
            let p = base(active, tilt_arm);
            let a = p.aim(&model, 0.0, 0.76, 0.002);
            let (mx, mz) = p.statcast_movement(&model, a, 0.002);
            (-mx / INCH, mz / INCH)
        };
        let mut active = 0.7;
        let mut tilt = t.pfx_arm_in.atan2(t.pfx_z_in);
        for _ in 0..40 {
            let (ax, az) = eval(active, tilt);
            let (ex, ez) = (t.pfx_arm_in - ax, t.pfx_z_in - az);
            if ex.abs() < 1e-3 && ez.abs() < 1e-3 {
                break;
            }
            let h = 1e-4;
            let (bx, bz) = eval(active + h, tilt);
            let (cx, cz) = eval(active, tilt + h);
            let (j11, j21) = ((bx - ax) / h, (bz - az) / h);
            let (j12, j22) = ((cx - ax) / h, (cz - az) / h);
            let det = j11 * j22 - j12 * j21;
            if det.abs() < 1e-12 {
                break;
            }
            active = (active + 0.7 * (j22 * ex - j12 * ez) / det).clamp(0.02, 1.0);
            tilt += 0.7 * (-j21 * ex + j11 * ez) / det;
        }
        let p = base(active, tilt);
        let a = p.aim(&model, 0.0, 0.76, 0.002);
        let f = p.fly(&model, a, 0.002);
        let (ax, az) = eval(active, tilt);
        println!(
            "{:9} {:7.3} {:8.1} | {:9.2} {:9.2} | {:8.1} {:8.1} | {:9.4} {:9.4} | {:6.3}",
            t.name,
            active,
            tilt.to_degrees(),
            ax,
            az,
            t.pfx_arm_in,
            t.pfx_z_in,
            f.plate_vel.norm() / p.speed,
            t.plate_ratio,
            f.time
        );
    }
}
