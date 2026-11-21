//! Tactics tried several ways from the same moment.
//!
//! Plate appearances are drawn from games; at each one every tactic is played from a copy
//! of the game, and the AI managers finish the game. The differences of the outcomes are
//! the engine's value of the tactic in that very situation (one step of policy improvement
//! on the AI manager by rollouts, Tesauro & Galperin 1997). The same random numbers are
//! given to every tactic of a rollout (common random numbers), which narrows the
//! differences.

use crate::env::{encode, EnvConfig, ManagerEnv};
use crate::league::League;
use crate::obs::observe;
use crate::we::{bases, WinExpectancy};
use bm_engine::{Approach, Rng, Situation, StealOrder};
use serde::Serialize;
use std::sync::Arc;

/// A plan for one plate appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Tactic {
    /// Swing away; runners go on their own.
    HitAway,
    /// Sacrifice bunt until two strikes, then swing away.
    SacrificeBunt,
    /// Bunt for a hit on the first pitch, then swing away.
    BuntForHit,
    /// Send the lead runner on the first pitch.
    Steal,
    /// Hit and run on the first pitch.
    HitAndRun,
    /// Swing away and keep the runners where they are.
    Hold,
}

impl Tactic {
    pub fn name(self) -> &'static str {
        match self {
            Tactic::HitAway => "hit_away",
            Tactic::SacrificeBunt => "sac_bunt",
            Tactic::BuntForHit => "bunt_for_hit",
            Tactic::Steal => "steal",
            Tactic::HitAndRun => "hit_and_run",
            Tactic::Hold => "hold",
        }
    }

    /// The order before a pitch of the plate appearance (`pitch` = pitches already thrown).
    pub fn order(self, s: &Situation, pitch: u32) -> usize {
        let (a, st) = match self {
            Tactic::HitAway => (Approach::Normal, StealOrder::Auto),
            Tactic::SacrificeBunt if s.strikes < 2 => (Approach::SacrificeBunt, StealOrder::Auto),
            Tactic::BuntForHit if pitch == 0 => (Approach::BuntForHit, StealOrder::Auto),
            Tactic::Steal if pitch == 0 => (Approach::Normal, StealOrder::Go),
            Tactic::HitAndRun if pitch == 0 => (Approach::HitAndRun, StealOrder::Auto),
            Tactic::Hold => (Approach::Normal, StealOrder::Hold),
            _ => (Approach::Normal, StealOrder::Auto),
        };
        encode(a, st)
    }
}

/// A family of situations at the start of a plate appearance, and the tactics compared in it
/// (the first is the reference).
pub struct Class {
    pub name: &'static str,
    pub tactics: &'static [Tactic],
    /// Base state (bits), outs, innings and the batting club's lead.
    pub bases: u8,
    pub outs: u8,
    pub innings: (u8, u8),
    pub max_abs_lead: Option<u32>,
}

impl Class {
    fn matches(&self, s: &Situation, lead: i32) -> bool {
        bases(s) == self.bases
            && s.outs == self.outs
            && (self.innings.0..=self.innings.1).contains(&s.inning)
            && self.max_abs_lead.is_none_or(|m| lead.unsigned_abs() <= m)
    }
}

use Tactic::*;

/// The situations of the classic tactical questions.
pub const CLASSES: [Class; 7] = [
    Class {
        name: "first_0out_early_close",
        tactics: &[HitAway, SacrificeBunt, Steal, HitAndRun, Hold],
        bases: 1,
        outs: 0,
        innings: (1, 6),
        max_abs_lead: Some(1),
    },
    Class {
        name: "first_0out_late_close",
        tactics: &[HitAway, SacrificeBunt, Steal, HitAndRun, Hold],
        bases: 1,
        outs: 0,
        innings: (7, 30),
        max_abs_lead: Some(1),
    },
    Class { name: "second_0out_late_close", tactics: &[HitAway, SacrificeBunt, Steal, Hold], bases: 2, outs: 0, innings: (7, 30), max_abs_lead: Some(1) },
    Class { name: "first_second_0out_late_close", tactics: &[HitAway, SacrificeBunt], bases: 3, outs: 0, innings: (7, 30), max_abs_lead: Some(1) },
    Class { name: "first_1out_late_close", tactics: &[HitAway, SacrificeBunt, Steal, HitAndRun], bases: 1, outs: 1, innings: (7, 30), max_abs_lead: Some(1) },
    Class { name: "empty_0out", tactics: &[HitAway, BuntForHit], bases: 0, outs: 0, innings: (1, 30), max_abs_lead: None },
    Class { name: "first_2out", tactics: &[HitAway, Steal, Hold], bases: 1, outs: 2, innings: (1, 30), max_abs_lead: None },
];

#[derive(Clone, Debug)]
pub struct Config {
    /// Plate appearances to evaluate in each class.
    pub states: usize,
    /// Rollouts of each tactic from each of them.
    pub rollouts: usize,
    pub seed: u64,
    pub threads: usize,
    /// Chance that a plate appearance of a class still short of states is taken (spreads
    /// them over more games).
    pub sample: f64,
}

/// One plate appearance and what each tactic of its class returned.
#[derive(Clone, Debug, Serialize)]
pub struct StateResult {
    pub class: usize,
    pub inning: u8,
    pub lead: i32,
    pub home: bool,
    pub win_expectancy: f64,
    /// Observation features (tenths of `quality`, the speed scale of the observation).
    pub batter_quality: f32,
    pub batter_speed: f32,
    pub batter_bunt: f32,
    pub lead_runner_speed: f32,
    pub lead_runner_steal: f32,
    pub pitcher_time_to_plate: f32,
    pub catcher_pop_time: f32,
    /// Share of rollouts won and mean runs to the end of the half-inning, by tactic.
    pub win: Vec<f64>,
    pub runs: Vec<f64>,
}

fn evaluate_state(env: &ManagerEnv, class: usize, rollouts: usize, rng: &mut Rng) -> StateResult {
    let sim = env.sim().expect("game");
    let s = sim.situation();
    let side = env.side();
    let lead = s.score[side] as i32 - s.score[1 - side] as i32;
    let o = observe(sim, side, env.win_expectancy_table());
    let lead_base = if s.bases[1].is_some() && s.bases[2].is_none() { 1 } else { 0 };
    let tactics = CLASSES[class].tactics;
    let mut win = vec![0.0; tactics.len()];
    let mut runs = vec![0.0; tactics.len()];
    for _ in 0..rollouts {
        let seed = rng.next_u64();
        for (ti, t) in tactics.iter().enumerate() {
            let mut e = env.clone();
            e.seed(seed);
            let (won, r) = e.rollout(|s, p| t.order(s, p));
            win[ti] += f64::from(u8::from(won));
            runs[ti] += r as f64;
        }
    }
    let n = rollouts as f64;
    StateResult {
        class,
        inning: s.inning,
        lead,
        home: side == 1,
        win_expectancy: env.win_expectancy_table().for_team(&s, side),
        batter_quality: o[14],
        batter_speed: o[15],
        batter_bunt: o[17],
        lead_runner_speed: o[23 + 2 * lead_base],
        lead_runner_steal: o[24 + 2 * lead_base],
        pitcher_time_to_plate: o[30],
        catcher_pop_time: o[34],
        win: win.into_iter().map(|w| w / n).collect(),
        runs: runs.into_iter().map(|r| r / n).collect(),
    }
}

/// Draw plate appearances from games in which the agent's club swings away (its runners
/// going on their own), and evaluate every class's tactics at them.
pub fn evaluate(league: Arc<League>, we: Arc<WinExpectancy>, cfg: &Config) -> Vec<StateResult> {
    let threads = cfg.threads.max(1);
    let mut master = Rng::new(cfg.seed);
    let seeds: Vec<u64> = (0..threads).map(|_| master.next_u64()).collect();
    let mut out = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .enumerate()
            .map(|(t, seed)| {
                let league = league.clone();
                let we = we.clone();
                // This thread's share of each class.
                let quota = (cfg.states + threads - 1 - t) / threads;
                scope.spawn(move || {
                    let mut rng = Rng::new(seed);
                    let mut env = ManagerEnv::with(EnvConfig::default(), league, we, rng.next_u64());
                    let mut have = vec![0usize; CLASSES.len()];
                    let mut res = Vec::new();
                    let hit = encode(Approach::Normal, StealOrder::Auto);
                    while have.iter().any(|&h| h < quota) {
                        env.reset(Some(rng.next_u64()));
                        loop {
                            let sim = env.sim().expect("game");
                            if sim.is_over() {
                                break;
                            }
                            let s = sim.situation();
                            if !s.in_plate_appearance && s.balls == 0 && s.strikes == 0 {
                                let side = env.side();
                                let lead = s.score[side] as i32 - s.score[1 - side] as i32;
                                let class = CLASSES.iter().enumerate().position(|(c, k)| have[c] < quota && k.matches(&s, lead));
                                if let Some(c) = class {
                                    if rng.chance(cfg.sample) {
                                        res.push(evaluate_state(&env, c, cfg.rollouts, &mut rng));
                                        have[c] += 1;
                                    }
                                }
                            }
                            if env.step(hit).terminated {
                                break;
                            }
                        }
                    }
                    res
                })
            })
            .collect();
        for h in handles {
            out.extend(h.join().expect("thread"));
        }
    });
    out.sort_by_key(|r| r.class);
    out
}

/// Mean and standard error of a sample.
pub fn mean_se(x: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    if n == 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let m = x.iter().sum::<f64>() / n;
    let v = if n > 1.0 { x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (n - 1.0) } else { 0.0 };
    (m, (v / n).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tactics_order_what_they_say() {
        let mut env = ManagerEnv::new(EnvConfig { side: Some(0), ..EnvConfig::default() }, 2);
        env.reset(Some(4));
        let s = env.sim().expect("game").situation();
        assert_eq!(HitAway.order(&s, 0), 0);
        assert_eq!(SacrificeBunt.order(&s, 0), encode(Approach::SacrificeBunt, StealOrder::Auto));
        assert_eq!(Steal.order(&s, 1), 0, "steal only on the first pitch");
        let mut two = s;
        two.strikes = 2;
        assert_eq!(SacrificeBunt.order(&two, 3), 0, "no bunting with two strikes");
    }

    #[test]
    fn rollouts_finish_and_are_repeatable() {
        let league = Arc::new(League::mlb(2025));
        let we = Arc::new(WinExpectancy::bundled());
        let cfg = Config { states: 1, rollouts: 6, seed: 3, threads: 1, sample: 1.0 };
        let a = evaluate(league.clone(), we.clone(), &cfg);
        let b = evaluate(league, we, &cfg);
        assert_eq!(a.len(), CLASSES.len());
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.win, y.win);
            assert_eq!(x.runs, y.runs);
            assert_eq!(x.win.len(), CLASSES[x.class].tactics.len());
            assert!(x.win.iter().all(|w| (0.0..=1.0).contains(w)));
            assert!(x.runs.iter().all(|r| (0.0..10.0).contains(r)));
            assert!(CLASSES[x.class].matches(&situation_of(x), x.lead));
        }
    }

    /// The parts of a situation a class looks at, rebuilt from a result.
    fn situation_of(r: &StateResult) -> Situation {
        let c = &CLASSES[r.class];
        let b = |i: u8| (c.bases & (1 << i) != 0).then_some(0usize);
        Situation {
            inning: r.inning,
            top: true,
            outs: c.outs,
            balls: 0,
            strikes: 0,
            score: [0, 0],
            bases: [b(0), b(1), b(2)],
            batting: 0,
            fielding: 1,
            batter: 0,
            pitcher: 0,
            pitch_count: 0,
            in_plate_appearance: false,
            over: false,
        }
    }
}
