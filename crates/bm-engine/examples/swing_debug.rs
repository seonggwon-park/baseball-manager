//! Diagnostic: outcome of swings at a fastball down the middle, by miss cause.
use bm_engine::batting::{perceive, swing, Miss, Swing};
use bm_engine::pitching::{expectation, selection_weights, throw_pitch, PitchChoice, PitcherState, MAX_PITCHES};
use bm_engine::*;
use bm_physics::units::MPH;

fn main() {
    let env = MatchEnv::neutral_mlb();
    let pit = PitcherProfile::league_average(Hand::Right);
    let mut st = PitcherState::new(&pit, &env);
    let mut rng = Rng::new(1);
    let level = Level::mlb();
    let b = {
        let mut b = BatterProfile::league_average(Hand::Right);
        b.barrel_sd = level.barrel_sd;
        b
    };
    let zone = StrikeZone::for_batter(b.height, &env.rules);
    let mut w = [0.0; MAX_PITCHES];
    let n = selection_weights(&pit, b.hand, 1, 1, &env.params, &mut w);
    let exp = expectation(&pit, &st, &w[..n], env.params.expect_mix);
    for (idx, name) in [(0usize, "FF"), (1, "SL"), (2, "CH")] {
        let mut c = [0.0f64; 9];
        let (mut la, mut ev, mut nb) = (0.0, 0.0, 0.0);
        let total = 40000;
        for _ in 0..total {
            st.pitch_count = 10;
            let th = throw_pitch(&pit, &mut st, &PitchChoice { idx, target_x: 0.0, target_z: zone.mid() }, 1.0, &env, &mut rng);
            let per = perceive(&env.params, &b, &zone, 1, 1, &th, &exp, env.ball.radius, &mut rng);
            match swing(&env, &b, &zone, 1, &th, &exp, &per, &mut rng) {
                Swing::Whiff { miss, .. } => {
                    c[match miss {
                        Miss::Early => 0,
                        Miss::Late => 1,
                        Miss::OffTheEnd => 2,
                        Miss::Jammed => 3,
                        Miss::Over => 4,
                        Miss::Under => 5,
                    }] += 1.0
                }
                Swing::FoulTip { .. } => c[6] += 1.0,
                Swing::FoulBack { .. } => c[7] += 1.0,
                Swing::Batted { info, .. } => {
                    c[8] += 1.0;
                    nb += 1.0;
                    la += info.launch_angle.to_degrees();
                    ev += info.exit_speed / MPH;
                }
                Swing::Take => {}
            }
        }
        let t = total as f64 / 100.0;
        println!(
            "{name} middle-middle: early {:.1} late {:.1} off-end {:.1} jammed {:.1} over {:.1} under {:.1} | tip {:.1} foul-back {:.1} batted {:.1} (LA {:.1}, EV {:.1})",
            c[0] / t, c[1] / t, c[2] / t, c[3] / t, c[4] / t, c[5] / t, c[6] / t, c[7] / t, c[8] / t, la / nb, ev / nb
        );
    }
}
