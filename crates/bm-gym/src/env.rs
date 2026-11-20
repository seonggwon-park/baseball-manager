//! The environment: the agent manages one club's offence pitch by pitch.
//!
//! Before every pitch to one of its hitters the agent picks a batting approach and a
//! steal order (`Action`). Everything else is left to the engine's AI managers: the
//! opponent's whole game, and the agent's own pitching changes, pinch-hitters and
//! -runners, defensive alignment and intentional walks. An episode is one game.

use crate::league::{Fixture, League};
use crate::obs::{observe, N_OBS};
use crate::we::{bases, WinExpectancy};
use bm_engine::{Approach, Auto, GameSim, ManagerStyle, NoObserver, OffenseOrders, Rng, Situation, StealOrder, Step};
use std::sync::Arc;

/// Batting approaches the agent can order.
pub const APPROACHES: [Approach; 5] = [Approach::Normal, Approach::SacrificeBunt, Approach::BuntForHit, Approach::HitAndRun, Approach::Take];
/// Steal orders: leave it to the runner, hold him, send him on this pitch.
pub const STEALS: [StealOrder; 3] = [StealOrder::Auto, StealOrder::Hold, StealOrder::Go];
pub const N_ACTIONS: usize = APPROACHES.len() * STEALS.len();

/// Share of the agent's bunts laid down toward third base, as the AI managers do.
const BUNT_TOWARD_THIRD: f64 = 0.7;

/// An action is `approach * 3 + steal`, indices into [`APPROACHES`] and [`STEALS`].
pub fn decode(action: usize) -> (Approach, StealOrder) {
    (APPROACHES[action / STEALS.len()], STEALS[action % STEALS.len()])
}

pub fn encode(approach: Approach, steal: StealOrder) -> usize {
    let a = APPROACHES.iter().position(|&x| x == approach).expect("approach");
    let s = STEALS.iter().position(|&x| x == steal).expect("steal");
    a * STEALS.len() + s
}

pub fn action_name(action: usize) -> String {
    let (a, s) = decode(action);
    let a = match a {
        Approach::Normal => "hit",
        Approach::SacrificeBunt => "sac_bunt",
        Approach::BuntForHit => "bunt_for_hit",
        Approach::HitAndRun => "hit_and_run",
        Approach::Take => "take",
    };
    let s = match s {
        StealOrder::Auto => "auto",
        StealOrder::Hold => "hold",
        StealOrder::Go => "steal",
    };
    format!("{a}+{s}")
}

/// A runner who can be sent: first base with second open, or second with third open.
fn can_steal(s: &Situation) -> bool {
    (s.bases[0].is_some() && s.bases[1].is_none()) || (s.bases[1].is_some() && s.bases[2].is_none())
}

/// Which actions make sense in the situation: steal orders need a runner who can go, a
/// sacrifice needs a runner on first or second and fewer than two outs, the hit and run
/// needs a runner who can go and sends him itself (so only with the runner's own call).
pub fn mask(s: &Situation) -> [bool; N_ACTIONS] {
    let runner = can_steal(s);
    let sac = (s.bases[0].is_some() || s.bases[1].is_some()) && s.outs < 2;
    let mut m = [false; N_ACTIONS];
    for (i, ok) in m.iter_mut().enumerate() {
        let (a, st) = decode(i);
        let steal_ok = st == StealOrder::Auto || runner;
        let approach_ok = match a {
            Approach::SacrificeBunt => sac,
            Approach::HitAndRun => runner && st == StealOrder::Auto,
            _ => true,
        };
        *ok = steal_ok && approach_ok;
    }
    m
}

/// The nearest sensible action: the steal order falls back to the runner's own call, the
/// approach to hitting away.
pub fn coerce(s: &Situation, action: usize) -> usize {
    let m = mask(s);
    if m[action] {
        return action;
    }
    let (a, _) = decode(action);
    let auto = encode(a, StealOrder::Auto);
    if m[auto] {
        auto
    } else {
        encode(Approach::Normal, StealOrder::Auto)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    /// The change of the agent's win expectancy over the step; the final step brings it to
    /// 1 for a win or 0 for a loss, so an episode's rewards add up to the result minus the
    /// win expectancy at the first decision.
    WinExpectancy,
    /// 1 at the end of a won game, 0 otherwise.
    Win,
}

#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub reward: Reward,
    /// Side the agent manages (0 = away, 1 = home); `None` draws it for every game.
    pub side: Option<usize>,
    /// The opponent's AI manager.
    pub opponent: ManagerStyle,
    /// The AI manager that makes the agent's other decisions (pitching, bench, defence).
    pub own: ManagerStyle,
    /// Seed of the league's clubs (the CLI's `--seed` gives the same clubs).
    pub league_seed: u64,
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfig { reward: Reward::WinExpectancy, side: None, opponent: ManagerStyle::mlb(), own: ManagerStyle::mlb(), league_seed: 2025 }
    }
}

/// What the step left, for logging and analysis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Info {
    pub inning: u8,
    pub top: bool,
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    /// Base state as bits (first = 1, second = 2, third = 4).
    pub bases: u8,
    /// Runs of the agent's club and the opponent's.
    pub score: [u32; 2],
    /// The agent's win expectancy from the bundled table.
    pub win_expectancy: f64,
    /// The side the agent manages.
    pub side: usize,
    /// Decisions made in this game so far.
    pub decisions: u32,
    /// `Some(won)` once the game is over.
    pub won: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub obs: [f32; N_OBS],
    pub reward: f64,
    pub terminated: bool,
    pub truncated: bool,
    pub info: Info,
}

/// One game at a time. Cloning the environment copies the game in progress, so an action
/// can be tried several ways from the same moment.
#[derive(Clone)]
pub struct ManagerEnv {
    cfg: EnvConfig,
    league: Arc<League>,
    we: Arc<WinExpectancy>,
    rng: Rng,
    sim: Option<GameSim>,
    fixture: Option<Fixture>,
    side: usize,
    last_we: f64,
    decisions: u32,
}

impl ManagerEnv {
    pub fn new(cfg: EnvConfig, seed: u64) -> Self {
        let league = Arc::new(League::mlb(cfg.league_seed));
        ManagerEnv::with(cfg, league, Arc::new(WinExpectancy::bundled()), seed)
    }

    /// An environment sharing a league and a table with others (vectorised training).
    pub fn with(cfg: EnvConfig, league: Arc<League>, we: Arc<WinExpectancy>, seed: u64) -> Self {
        ManagerEnv { cfg, league, we, rng: Rng::new(seed), sim: None, fixture: None, side: 1, last_we: 0.5, decisions: 0 }
    }

    pub fn config(&self) -> &EnvConfig {
        &self.cfg
    }

    pub fn league(&self) -> &League {
        &self.league
    }

    pub fn win_expectancy_table(&self) -> &WinExpectancy {
        &self.we
    }

    /// The game in progress.
    pub fn sim(&self) -> Option<&GameSim> {
        self.sim.as_ref()
    }

    pub fn fixture(&self) -> Option<&Fixture> {
        self.fixture.as_ref()
    }

    /// The side the agent manages in the current game.
    pub fn side(&self) -> usize {
        self.side
    }

    /// Reseed the environment's random numbers (games, and the engine within them).
    pub fn seed(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
    }

    /// Start a new game and play it up to the agent's first decision.
    pub fn reset(&mut self, seed: Option<u64>) -> ([f32; N_OBS], Info) {
        if let Some(seed) = seed {
            self.seed(seed);
        }
        let f = self.league.fixture(&mut self.rng);
        let mut sim = self.league.start(&f);
        self.side = self.cfg.side.unwrap_or_else(|| self.rng.below(2));
        let opp = 1 - self.side;
        sim.set_style(opp, self.cfg.opponent);
        sim.set_style(self.side, self.cfg.own);
        sim.set_auto(opp, Auto::ALL);
        sim.set_auto(self.side, Auto { pitching: true, bench: true, offense: false, defense: true });
        self.sim = Some(sim);
        self.fixture = Some(f);
        self.decisions = 0;
        self.advance();
        self.last_we = self.current_we();
        (self.observation(), self.info())
    }

    /// Order the next pitch and play on to the agent's next decision or the end of the game.
    pub fn step(&mut self, action: usize) -> StepResult {
        assert!(action < N_ACTIONS, "action {action} out of range");
        let sim = self.sim.as_mut().expect("reset before step");
        assert!(!sim.is_over(), "the game is over: reset");
        let s = sim.situation();
        let (approach, steal) = decode(coerce(&s, action));
        let bunt_toward_third = approach.is_bunt().then(|| self.rng.chance(BUNT_TOWARD_THIRD));
        sim.set_offense_orders(OffenseOrders { approach: Some(approach), bunt_toward_third, steal });
        sim.step(&mut self.rng, &mut NoObserver);
        self.decisions += 1;
        self.advance();
        let terminated = self.sim.as_ref().is_some_and(|g| g.is_over());
        let now = self.current_we();
        let reward = match self.cfg.reward {
            Reward::WinExpectancy => now - self.last_we,
            Reward::Win => {
                if terminated {
                    now
                } else {
                    0.0
                }
            }
        };
        self.last_we = now;
        StepResult { obs: self.observation(), reward, terminated, truncated: false, info: self.info() }
    }

    /// Play on while it is not the agent's turn: the opponent bats, or the game is over.
    fn advance(&mut self) {
        let side = self.side;
        let sim = self.sim.as_mut().expect("game");
        while !sim.is_over() && sim.situation().batting != side {
            if sim.step(&mut self.rng, &mut NoObserver) == Step::GameOver {
                break;
            }
        }
    }

    /// The agent's win expectancy now: 1 or 0 once the game is over.
    fn current_we(&self) -> f64 {
        let sim = self.sim.as_ref().expect("game");
        if let Some(r) = sim.result() {
            return if r.score[self.side] > r.score[1 - self.side] { 1.0 } else { 0.0 };
        }
        self.we.for_team(&sim.situation(), self.side)
    }

    pub fn observation(&self) -> [f32; N_OBS] {
        let sim = self.sim.as_ref().expect("reset first");
        observe(sim, self.side, &self.we)
    }

    /// Actions that make sense before the next pitch (all of them once the game is over).
    pub fn action_mask(&self) -> [bool; N_ACTIONS] {
        let sim = self.sim.as_ref().expect("reset first");
        if sim.is_over() {
            return [true; N_ACTIONS];
        }
        mask(&sim.situation())
    }

    pub fn info(&self) -> Info {
        let sim = self.sim.as_ref().expect("reset first");
        let s = sim.situation();
        let won = sim.result().map(|r| r.score[self.side] > r.score[1 - self.side]);
        Info {
            inning: s.inning,
            top: s.top,
            outs: s.outs,
            balls: s.balls,
            strikes: s.strikes,
            bases: bases(&s),
            score: [s.score[self.side], s.score[1 - self.side]],
            win_expectancy: self.current_we(),
            side: self.side,
            decisions: self.decisions,
            won,
        }
    }

    /// Play the rest of the game with the agent's offence handed back to its AI manager:
    /// returns whether the agent's club won. For comparing a decision with what follows it.
    pub fn finish_with_ai(&mut self) -> bool {
        let side = self.side;
        let sim = self.sim.as_mut().expect("game");
        sim.set_auto(side, Auto::ALL);
        while !sim.is_over() {
            sim.step(&mut self.rng, &mut NoObserver);
        }
        let r = sim.result().expect("over");
        r.score[side] > r.score[1 - side]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit_away() -> usize {
        encode(Approach::Normal, StealOrder::Auto)
    }

    #[test]
    fn actions_round_trip() {
        for a in 0..N_ACTIONS {
            let (ap, st) = decode(a);
            assert_eq!(encode(ap, st), a);
            assert!(!action_name(a).is_empty());
        }
    }

    #[test]
    fn episode_runs_to_the_end_and_rewards_telescope() {
        let mut env = ManagerEnv::new(EnvConfig::default(), 7);
        for game in 0..6u64 {
            let (_, info0) = env.reset(Some(100 + game));
            let start = info0.win_expectancy;
            let mut total = 0.0;
            let mut steps = 0;
            loop {
                let r = env.step(hit_away());
                total += r.reward;
                steps += 1;
                for x in r.obs {
                    assert!(x.is_finite());
                }
                if r.terminated {
                    let won = r.info.won.expect("over");
                    assert!((total - (f64::from(u8::from(won)) - start)).abs() < 1e-9, "{total} {won} {start}");
                    break;
                }
                assert!(steps < 2000, "game does not end");
            }
            // About 4 pitches a plate appearance, 4.3 plate appearances an inning.
            assert!((60..400).contains(&steps), "{steps} decisions");
        }
    }

    #[test]
    fn same_seed_same_game() {
        let run = |seed| {
            let mut env = ManagerEnv::new(EnvConfig { side: Some(0), ..EnvConfig::default() }, 1);
            env.reset(Some(seed));
            let mut trace = Vec::new();
            loop {
                let m = env.action_mask();
                // A fixed rule that uses several actions.
                let a = if m[encode(Approach::SacrificeBunt, StealOrder::Auto)] {
                    encode(Approach::SacrificeBunt, StealOrder::Auto)
                } else if m[encode(Approach::Normal, StealOrder::Go)] {
                    encode(Approach::Normal, StealOrder::Go)
                } else {
                    0
                };
                let r = env.step(a);
                trace.push((r.info.score, r.info.outs, r.info.bases));
                if r.terminated {
                    return trace;
                }
            }
        };
        assert_eq!(run(5), run(5));
        assert_ne!(run(5), run(6));
    }

    #[test]
    fn decisions_come_only_on_the_agents_turn() {
        for side in [0, 1] {
            let mut env = ManagerEnv::new(EnvConfig { side: Some(side), ..EnvConfig::default() }, 3);
            env.reset(Some(11));
            loop {
                let sim = env.sim().expect("game");
                assert_eq!(sim.situation().batting, side);
                if env.step(0).terminated {
                    break;
                }
            }
        }
    }

    #[test]
    fn masks_follow_the_situation() {
        let mut env = ManagerEnv::new(EnvConfig::default(), 9);
        env.reset(Some(1));
        let mut saw_runner = false;
        let mut saw_empty = false;
        loop {
            let s = env.sim().expect("game").situation();
            let m = env.action_mask();
            assert!(m[0], "hitting away is always allowed");
            let empty = s.bases.iter().all(|b| b.is_none());
            if empty {
                saw_empty = true;
                assert!(!m[encode(Approach::SacrificeBunt, StealOrder::Auto)]);
                assert!(!m[encode(Approach::Normal, StealOrder::Go)]);
                assert!(!m[encode(Approach::HitAndRun, StealOrder::Auto)]);
            }
            if s.bases[0].is_some() && s.bases[1].is_none() {
                saw_runner = true;
                assert!(m[encode(Approach::Normal, StealOrder::Go)]);
                assert!(m[encode(Approach::HitAndRun, StealOrder::Auto)]);
                assert!(!m[encode(Approach::HitAndRun, StealOrder::Go)]);
            }
            // An action that makes no sense is played as the nearest one that does.
            assert!(mask(&s)[coerce(&s, encode(Approach::SacrificeBunt, StealOrder::Go))]);
            if env.step(0).terminated {
                break;
            }
        }
        assert!(saw_runner && saw_empty);
    }

    #[test]
    fn a_cloned_game_can_be_played_two_ways() {
        let mut env = ManagerEnv::new(EnvConfig { side: Some(1), ..EnvConfig::default() }, 4);
        env.reset(Some(21));
        // Play to a runner on first with fewer than two outs.
        loop {
            let s = env.sim().expect("game").situation();
            if s.bases[0].is_some() && s.bases[1].is_none() && s.outs < 2 && !s.in_plate_appearance {
                break;
            }
            if env.step(0).terminated {
                env.reset(None);
            }
        }
        let before = env.info();
        let mut a = env.clone();
        let mut b = env.clone();
        a.seed(77);
        b.seed(77);
        let ra = a.step(encode(Approach::SacrificeBunt, StealOrder::Auto));
        let rb = b.step(encode(Approach::Normal, StealOrder::Auto));
        // The original is untouched.
        assert_eq!(env.info(), before);
        // The same random numbers, two different orders: the plays can differ, the score so far cannot.
        assert!(ra.info.score[0] >= before.score[0] && rb.info.score[0] >= before.score[0]);
        // Either branch can be finished by the AI manager.
        let won = a.finish_with_ai();
        assert_eq!(a.info().won, Some(won));
    }
}
