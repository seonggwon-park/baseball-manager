//! Diagnostic: outcome distribution of specific batted balls with the bases empty.
use bm_engine::fielding::standard_alignment;
use bm_engine::play::{resolve, PlayInput};
use bm_engine::*;
use bm_physics::units::{FT, MPH, RPM};
use bm_physics::{spin_from_components, velocity_from_angles, BallState, BattedPath, BattedSim, Vec3};
use std::collections::BTreeMap;

fn main() {
    let env = MatchEnv::neutral_mlb();
    let fielders = [FielderProfile::league_average(); 9];
    let positions = standard_alignment();
    let batter = Runner { player: 0, sprint: 27.0 * FT, accel: 4.9, aggr: 0.5, pitcher: 0, earned: true };
    println!("  EV   LA spray | land(ft) hang  end(ft) | outcomes");
    let args: Vec<f64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let mut cases: Vec<(f64, f64, f64)> = vec![
        (100.0, 12.0, 13.0),
        (103.0, 18.0, -13.0),
        (98.0, 22.0, 13.0),
        (104.0, 20.0, 0.0),
        (95.0, 8.0, 20.0),
        (104.0, 24.0, 35.0),
        (70.0, 28.0, 10.0),
        (75.0, 22.0, -20.0),
        (99.0, 30.0, 13.0),
        (101.0, 24.0, -13.0),
    ];
    if args.len() >= 2 {
        cases = (-9..=9).map(|i| (args[0], args[1], 5.0 * i as f64)).collect();
    }
    for (ev, la, spray) in cases {
        let (th, ph): (f64, f64) = (f64::to_radians(la), f64::to_radians(spray));
        let state = BallState::new(
            Vec3::new(0.0, 0.9, 0.85),
            velocity_from_angles(ev * MPH, th, ph),
            spin_from_components(th, ph, (-763.0 + 120.0 * la) * RPM, 0.0, 0.0),
        );
        let mut path = BattedPath::default();
        BattedSim::new(&env.batted_flight, &env.ball, &env.park).run(state, &mut path);
        let info = BattedInfo {
            exit_speed: ev * MPH,
            launch_angle: th,
            spray: ph,
            backspin: 0.0,
            sidespin: 0.0,
            bat_speed: 32.0,
            attack_angle: 0.17,
            bunt: false,
            shape: Default::default(),
        };
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for seed in 0..400 {
            let inp = PlayInput {
                env: &env,
                path: &path,
                info: &info,
                fielders: &fielders,
                positions: &positions,
                bases: [None; 3],
                batter,
                box_delay: 0.25,
                running: [0.0; 3],
                infield_in: false,
                outs: 0,
            };
            let r = resolve(&inp, &mut Rng::new(seed));
            *counts.entry(format!("{:?}", r.kind)).or_default() += 1;
        }
        let land = path.landing.map(|l| (l.pos.horiz() / FT, l.t)).unwrap_or((0.0, 0.0));
        println!("{:5.0} {:4.0} {:5.0} | {:7.0} {:5.2} {:7.0} | {:?}", ev, la, spray, land.0, land.1, path.last().pos.horiz() / FT, counts);
    }
}
