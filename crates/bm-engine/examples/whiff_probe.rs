//! Diagnostic: swings of the average batter against a pitcher with a single pitch,
//! by height of the pitch and by cause of the miss.
//!
//! cargo run --release -p bm-engine --example whiff_probe -- [pitches] [name=value ...]

use bm_engine::batting::{respond, Swing};
use bm_engine::pitching::{choose_pitch, expectation, selection_weights, throw_pitch, PitcherState, MAX_PITCHES};
use bm_engine::{BatterProfile, Hand, MatchEnv, Miss, PitchKind, PitcherProfile, Rng, StrikeZone};

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(200_000);
    let mut env = MatchEnv::neutral_mlb();
    let mut batter = BatterProfile::league_average(Hand::Right);
    for a in args {
        let (k, v) = a.split_once('=').expect("name=value");
        let v: f64 = v.parse().expect("number");
        let p = &mut env.params;
        match k {
            "height_bias" => p.height_bias = v,
            "ride_bias" => p.ride_bias = v,
            "ride_confusion" => p.ride_confusion = v,
            "tracking_residual" => p.tracking_residual = v,
            "noise_residual" => p.noise_residual = v,
            "arc_scale" => {
                p.arc_scale = v;
                p.arc_scale_early = v;
            }
            "barrel_outside_gain" => {
                p.barrel_outside_gain = v;
                p.barrel_outside_gain2 = v * 10.0;
            }
            "piece_rate" => p.piece_rate = v,
            "aim" => batter.barrel_aim = v,
            "barrel_sd" => batter.barrel_sd = v,
            "timing_sd" => batter.timing_sd = v,
            "attack_sd" => batter.attack_sd = v,
            "attack" => batter.attack_angle = v,
            "arc_early" => p.arc_scale_early = v,
            "arc_late" => p.arc_scale = v,
            "direction_sd" => p.direction_sd = v,
            "bat_tilt_sd" => p.bat_tilt_sd = v,
            "partial" => {
                p.partial_swing_rate = v;
                p.partial_fooled_rate = v;
            }
            "misread_rate" => p.misread_rate = v,
            "timing_speed_exp" => p.timing_speed_exp = v,
            "barrel_speed_exp" => p.barrel_speed_exp = v,
            "perception_sd" => batter.perception_sd = v,
            "reach_sd" => batter.reach_sd = v,
            "eye" => batter.eye = v,
            _ => panic!("unknown parameter {k}"),
        }
    }
    println!("kind  swing%  whiff/swing   early  late  off-end jammed  over  under | foul  in play  LA   | whiff by height (ft) [1.5,2) [2,2.5) [2.5,3) [3,3.5) [3.5,4)");
    for kind in [PitchKind::FourSeam, PitchKind::Sinker, PitchKind::Cutter, PitchKind::Slider, PitchKind::Curve, PitchKind::Changeup] {
        for hand in [Hand::Right] {
            let mut pitcher = PitcherProfile::league_average(hand);
            pitcher.pitches.retain(|d| d.kind == PitchKind::FourSeam);
            pitcher.pitches[0].kind = kind;
            pitcher.pitches[0].shape = kind.league_average();
            let mut st = PitcherState::new(&pitcher, &env);
            let zone = StrikeZone::for_batter(batter.height, &env.rules);
            let mut rng = Rng::new(17);
            let mut w = [0.0; MAX_PITCHES];
            let (mut swings, mut foul, mut bip, mut la) = (0.0, 0.0, 0.0, 0.0);
            let mut miss = [0.0; 6];
            let mut by_h = [[0.0; 2]; 5];
            let mut by_d = [[0.0; 2]; 5];
            let (mut dsum, mut dsq) = (0.0, 0.0);
            let mut early_miss = [0.0; 6];
            let mut early_n = 0.0;
            for i in 0..n {
                let (balls, strikes) = (i % 3, i % 2);
                let k = selection_weights(&pitcher, batter.hand, balls, strikes, &env.params, &mut w);
                let exp = expectation(&pitcher, &st, &w[..k], env.params.expect_mix);
                let choice = choose_pitch(&pitcher, batter.hand, &zone, balls, strikes, &w[..k], &env.params, &mut rng);
                st.pitch_count = 30;
                let thrown = throw_pitch(&pitcher, &mut st, &choice, 1.0, &env, &mut rng);
                let z = thrown.flight.plate_pos.z / 0.3048;
                let hb = if (1.5..4.0).contains(&z) { Some(((z - 1.5) / 0.5) as usize) } else { None };
                let r = respond(&env, &batter, &zone, balls, strikes, &thrown, &exp, &mut rng);
                let shape = match &r {
                    Swing::Take => None,
                    Swing::Whiff { shape, .. } | Swing::FoulTip { shape, .. } | Swing::FoulBack { shape } => Some(*shape),
                    Swing::Batted { info, .. } => Some(info.shape),
                };
                if let Some(sh) = shape {
                    swings += 1.0;
                    if let Some(h) = hb {
                        by_h[h][0] += 1.0;
                    }
                    let d = sh.depth / 0.0254;
                    dsum += d;
                    dsq += d * d;
                    let c = if d < -10.0 {
                        0
                    } else if d < -5.0 {
                        1
                    } else if d < 5.0 {
                        2
                    } else if d < 10.0 {
                        3
                    } else {
                        4
                    };
                    by_d[c][0] += 1.0;
                    if matches!(r, Swing::Whiff { .. }) {
                        by_d[c][1] += 1.0;
                    }
                    if (5.0..10.0).contains(&d) {
                        early_n += 1.0;
                        if let Swing::Whiff { miss: m, .. } = r {
                            early_miss[m as usize] += 1.0;
                        }
                    }
                }
                match r {
                    Swing::Take => {}
                    Swing::Whiff { miss: m, .. } => {
                        miss[match m {
                            Miss::Early => 0,
                            Miss::Late => 1,
                            Miss::OffTheEnd => 2,
                            Miss::Jammed => 3,
                            Miss::Over => 4,
                            Miss::Under => 5,
                        }] += 1.0;
                        if let Some(h) = hb {
                            by_h[h][1] += 1.0;
                        }
                    }
                    Swing::FoulTip { .. } | Swing::FoulBack { .. } => foul += 1.0,
                    Swing::Batted { info, .. } => {
                        bip += 1.0;
                        la += info.launch_angle.to_degrees();
                    }
                }
            }
            let p = |x: f64| 100.0 * x / swings;
            let wh: f64 = miss.iter().sum();
            println!(
                "{} {}  {:5.1}   {:5.1}        {:5.1} {:5.1} {:5.1}  {:5.1}  {:5.1} {:5.1}  | {:5.1} {:5.1}  {:5.1} | {} | depth {:+5.1} sd {:4.1}; share/whiff by depth {}",
                kind.code(),
                if hand == Hand::Right { 'R' } else { 'L' },
                100.0 * swings / n as f64,
                p(wh),
                p(miss[0]),
                p(miss[1]),
                p(miss[2]),
                p(miss[3]),
                p(miss[4]),
                p(miss[5]),
                p(foul),
                p(bip),
                la / bip,
                by_h.iter().map(|b| format!("{:5.1}", 100.0 * b[1] / b[0])).collect::<Vec<_>>().join(" "),
                dsum / swings,
                (dsq / swings - (dsum / swings).powi(2)).sqrt(),
                by_d.iter().map(|b| format!("{:4.1}:{:4.1}", 100.0 * b[0] / swings, 100.0 * b[1] / b[0])).collect::<Vec<_>>().join(" ")
            );
            println!(
                "      5-10 in early: misses % of those swings: early {:.1} late {:.1} off-end {:.1} jammed {:.1} over {:.1} under {:.1}",
                100.0 * early_miss[0] / early_n,
                100.0 * early_miss[1] / early_n,
                100.0 * early_miss[2] / early_n,
                100.0 * early_miss[3] / early_n,
                100.0 * early_miss[4] / early_n,
                100.0 * early_miss[5] / early_n
            );
        }
    }
}
