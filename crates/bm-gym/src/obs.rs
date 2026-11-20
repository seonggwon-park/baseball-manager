//! The observation: what the manager of the batting side can see before a pitch, as a
//! fixed-length vector of numbers of order one.
//!
//! Player features are centred on the league average and scaled by about one spread of
//! the generated population, so that an average player reads near zero.

use crate::we::{bases, WinExpectancy};
use bm_engine::fielding::C;
use bm_engine::profile::{AVERAGE_BARREL_SD, AVERAGE_BAT_SPEED};
use bm_engine::{BatterProfile, GameSim, Hand, Situation};
use bm_physics::units::{FT, MPH};

/// Names of the observation's entries, in order.
pub const NAMES: [&str; N_OBS] = [
    "inning",
    "bottom",
    "home",
    "outs_0",
    "outs_1",
    "outs_2",
    "on_first",
    "on_second",
    "on_third",
    "balls",
    "strikes",
    "lead",
    "win_expectancy",
    "pitch_count",
    "batter_quality",
    "batter_speed",
    "batter_steal",
    "batter_bunt",
    "batter_left",
    "batter_eye",
    "batter_barrel",
    "batter_bat_speed",
    "on_deck_quality",
    "first_speed",
    "first_steal",
    "second_speed",
    "second_steal",
    "third_speed",
    "third_steal",
    "pitcher_velocity",
    "pitcher_time_to_plate",
    "pitcher_left",
    "pitcher_fatigue",
    "pitcher_deception",
    "catcher_pop_time",
];

pub const N_OBS: usize = 35;

/// Offensive value relative to an average hitter (`BatterProfile::quality`, about 1.0),
/// in tenths.
fn quality(b: &BatterProfile) -> f32 {
    ((b.quality() - 1.0) / 0.1) as f32
}

/// Running speed: (top speed in ft/s − 27) / 1.5, the scale the AI manager uses.
fn speed(b: &BatterProfile) -> f32 {
    ((b.sprint / FT - 27.0) / 1.5) as f32
}

/// The observation for the manager of `team` (who is batting when it is asked).
pub fn observe(sim: &GameSim, team: usize, we: &WinExpectancy) -> [f32; N_OBS] {
    let s: Situation = sim.situation();
    let mut o = [0.0f32; N_OBS];
    let bat = s.batting;
    let fld = s.fielding;
    let bat_team = sim.team(bat);
    let fld_team = sim.team(fld);
    o[0] = ((s.inning as f32 - 1.0) / 8.0).min(1.5);
    o[1] = f32::from(!s.top);
    o[2] = f32::from(team == 1);
    o[3 + s.outs.min(2) as usize] = 1.0;
    let b = bases(&s);
    for i in 0..3 {
        o[6 + i] = f32::from(b & (1 << i) != 0);
    }
    o[9] = s.balls as f32 / 3.0;
    o[10] = s.strikes as f32 / 2.0;
    let lead = s.score[team] as i32 - s.score[1 - team] as i32;
    o[11] = (lead as f32 / 5.0).clamp(-2.0, 2.0);
    o[12] = we.for_team(&s, team) as f32;
    o[13] = s.pitch_count as f32 / 100.0;

    let pitcher = fld_team.players[s.pitcher].pitcher.as_ref();
    let p_hand = pitcher.map(|p| p.hand).unwrap_or(Hand::Right);
    let batter = &bat_team.players[s.batter].batter;
    o[14] = quality(batter);
    o[15] = speed(batter);
    o[16] = batter.steal as f32;
    o[17] = batter.bunt as f32;
    // A switch hitter bats from the side opposite to the pitcher's hand.
    let side = if batter.switch {
        if p_hand == Hand::Right {
            Hand::Left
        } else {
            Hand::Right
        }
    } else {
        batter.hand
    };
    o[18] = f32::from(side == Hand::Left);
    o[19] = ((batter.eye - 0.615) / 0.07) as f32;
    o[20] = ((batter.barrel_sd / AVERAGE_BARREL_SD - 1.0) / 0.2) as f32;
    o[21] = ((batter.bat_speed / AVERAGE_BAT_SPEED - 1.0) / 0.03) as f32;
    let lineup = sim.lineup(bat);
    if let Some(slot) = lineup.iter().position(|&p| p == s.batter) {
        o[22] = quality(&bat_team.players[lineup[(slot + 1) % 9]].batter);
    }
    for (i, r) in s.bases.iter().enumerate() {
        if let Some(r) = r {
            let runner = &bat_team.players[*r].batter;
            o[23 + 2 * i] = speed(runner);
            o[24 + 2 * i] = runner.steal as f32;
        }
    }
    if let Some(p) = pitcher {
        let velo = p.pitches.first().map(|d| d.shape.speed / MPH).unwrap_or(94.0);
        o[29] = ((velo - 94.0) / 3.0) as f32;
        o[30] = ((p.time_to_plate - 1.38) / 0.08) as f32;
        o[31] = f32::from(p.hand == Hand::Left);
        o[32] = (s.pitch_count as f64 / p.stamina.max(1.0)) as f32;
        o[33] = (p.deception / 0.1) as f32;
    }
    let catcher = &fld_team.players[sim.defense(fld)[C]].catcher;
    o[34] = ((catcher.pop_time - 1.97) / 0.06) as f32;
    o
}
