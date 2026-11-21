//! The pitching environment: the agent calls the pitches of one club.
//!
//! Before every pitch its pitcher throws, the agent picks a pitch type and an intent
//! (`Intent`: attack the zone, the count's usual aim, or chase). The engine aims and throws
//! it as it does its own pitches. Everything else is left to the AI managers: both clubs'
//! pitching changes, benches, offences and defences. An episode is one game.
//!
//! The reward of a pitch is its run value from the fielding side's point of view: minus the
//! change of the run expectancy (base state, outs and count; `RunExpectancy`) and minus the
//! runs that scored on it. Over a half inning the rewards add up to the runs allowed minus
//! the run expectancy when the agent first took the ball, so the best policy is the one that
//! allows the fewest runs.
//!
//! The batters read the pitcher's tendency (`ModelParams::tendency_strength`, here 1 by
//! default): a pitcher who leans on a pitch more than usual finds them waiting for it. To keep
//! a policy from living off one exact set of constants, the batters' perception constants can
//! be shaken for every game (`PitchEnvConfig::perturb`).

use crate::league::{Fixture, League};
use crate::re::RunExpectancy;
use crate::we::bases;
use bm_engine::pitching::{expectation, expectation_adapted, selection_weights, MAX_PITCHES};
use bm_engine::profile::{AVERAGE_BARREL_SD, AVERAGE_BAT_SPEED};
use bm_engine::{
    Auto, GameSim, Hand, Intent, ManagerStyle, ModelParams, NoObserver, Observer, PitchEvent, PitchGroup, PitchKind, PitchOrder, PitchResult, Rng, Situation,
    Step,
};
use std::sync::Arc;

pub const KINDS: [PitchKind; 8] = PitchKind::ALL;
pub const N_PITCH_ACTIONS: usize = KINDS.len() * 3;

/// An action is `kind * 3 + intent`, indices into [`KINDS`] and [`Intent::ALL`].
pub fn decode(action: usize) -> (PitchKind, Intent) {
    (KINDS[action / 3], Intent::ALL[action % 3])
}

pub fn encode(kind: PitchKind, intent: Intent) -> usize {
    kind.index() * 3 + Intent::ALL.iter().position(|&i| i == intent).expect("intent")
}

pub fn kind_name(kind: PitchKind) -> &'static str {
    match kind {
        PitchKind::FourSeam => "four_seam",
        PitchKind::Sinker => "sinker",
        PitchKind::Cutter => "cutter",
        PitchKind::Slider => "slider",
        PitchKind::Sweeper => "sweeper",
        PitchKind::Curve => "curve",
        PitchKind::Changeup => "changeup",
        PitchKind::Splitter => "splitter",
    }
}

pub fn action_name(action: usize) -> String {
    let (k, i) = decode(action);
    let i = match i {
        Intent::Attack => "attack",
        Intent::Normal => "normal",
        Intent::Chase => "chase",
    };
    format!("{}+{i}", kind_name(k))
}

#[derive(Clone, Debug)]
pub struct PitchEnvConfig {
    /// Side whose pitches the agent calls (0 = away, 1 = home); `None` draws it for every game.
    pub side: Option<usize>,
    pub opponent: ManagerStyle,
    pub own: ManagerStyle,
    /// Seed of the league's clubs.
    pub league_seed: u64,
    /// How much of the pitcher's deviation from his usual mix the batters read (0 = none).
    pub adapt: f64,
    /// Spread of the per-game shake of the batters' perception constants, in units of the
    /// default spreads (`perturb`); 0 = the engine's constants.
    pub perturb: f64,
}

impl Default for PitchEnvConfig {
    fn default() -> Self {
        PitchEnvConfig { side: None, opponent: ManagerStyle::mlb(), own: ManagerStyle::mlb(), league_seed: 2025, adapt: 1.0, perturb: 0.0 }
    }
}

/// Shake the batters' perception constants for one game: recognition of each off-speed type
/// (±0.03 on the eye scale), letting a recognised type go (±0.05), the pull of the location
/// prior (±15 %), how much of a timing misread is corrected (±5 %), and the scales of
/// misjudgement and mistiming (±10 %). `scale` multiplies all spreads.
pub fn perturb(params: &ModelParams, scale: f64, rng: &mut Rng) -> ModelParams {
    let mut p = params.clone();
    if scale <= 0.0 {
        return p;
    }
    let mul = |x: f64, sd: f64, rng: &mut Rng| x * (rng.gauss(0.0, sd * scale)).exp();
    for (k, kind) in KINDS.iter().enumerate() {
        if kind.group() != PitchGroup::Fastball {
            p.kind_recognition[k] += rng.gauss(0.0, 0.03 * scale);
        }
        if *kind != PitchKind::FourSeam {
            for x in p.take_kind[k].iter_mut() {
                *x = (*x + rng.gauss(0.0, 0.05 * scale)).clamp(-1.0, 1.0);
            }
        }
    }
    for row in p.prior_pull.iter_mut() {
        for x in row.iter_mut() {
            *x = mul(*x, 0.15, rng);
        }
    }
    p.timing_adjust = mul(p.timing_adjust, 0.05, rng).min(0.95);
    p.recognition_scale = mul(p.recognition_scale, 0.10, rng);
    p.recognition_time_scale = mul(p.recognition_time_scale, 0.10, rng);
    p.fooled_time = mul(p.fooled_time, 0.10, rng);
    p.fooled_speed_loss = mul(p.fooled_speed_loss, 0.10, rng).min(0.95);
    p
}

/// Names of the observation's entries, in order.
pub fn obs_names() -> Vec<String> {
    let mut v = Vec::with_capacity(N_PITCH_OBS);
    for b in 0..4 {
        for s in 0..3 {
            v.push(format!("count_{b}_{s}"));
        }
    }
    v.extend(["outs_0", "outs_1", "outs_2", "on_first", "on_second", "on_third", "inning", "lead", "run_expectancy"].map(String::from));
    v.extend(["same_side", "batter_quality", "batter_eye", "batter_barrel", "batter_bat_speed", "pitcher_left", "pitcher_fatigue"].map(String::from));
    for k in KINDS {
        let n = kind_name(k);
        v.extend([format!("{n}_available"), format!("{n}_usual"), format!("{n}_read"), format!("{n}_command"), format!("{n}_last")]);
    }
    v.extend(["first_pitch", "batters_lean", "batters_zone_lean"].map(String::from));
    v
}

pub const N_PITCH_OBS: usize = 12 + 9 + 7 + 8 * 5 + 3;

/// What the step left, for logging and analysis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchInfo {
    pub inning: u8,
    pub top: bool,
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    pub bases: u8,
    /// Runs of the agent's club and the opponent's.
    pub score: [u32; 2],
    pub side: usize,
    /// Pitches the agent has called in this game.
    pub decisions: u32,
    /// Runs the opponent scored on the last step's pitch.
    pub runs: u32,
    /// The pitch type and slot actually thrown on the last step (`None` before the first).
    pub thrown: Option<(PitchKind, usize)>,
    /// Where that pitch crossed the plate and the batter's zone: (x, z, zone bottom, zone top), m.
    pub location: Option<[f64; 4]>,
    /// What became of it.
    pub result: Option<PitchResult>,
}

/// Records where the pitch of a step crossed the plate and what became of it.
struct Where(Option<[f64; 4]>, Option<PitchResult>);

impl Observer for Where {
    fn pitch(&mut self, e: &PitchEvent) {
        let p = e.thrown.flight.plate_pos;
        self.0 = Some([p.x, p.z, e.zone.bottom, e.zone.top]);
        self.1 = Some(e.result);
    }
}

#[derive(Clone, Debug)]
pub struct PitchStep {
    pub obs: [f32; N_PITCH_OBS],
    pub reward: f64,
    pub terminated: bool,
    pub truncated: bool,
    pub info: PitchInfo,
}

/// One game at a time; cloning copies the game in progress.
#[derive(Clone)]
pub struct PitchingEnv {
    cfg: PitchEnvConfig,
    league: Arc<League>,
    re: Arc<RunExpectancy>,
    rng: Rng,
    sim: Option<GameSim>,
    fixture: Option<Fixture>,
    side: usize,
    decisions: u32,
    last: Option<PitchKind>,
    last_runs: u32,
    thrown: Option<(PitchKind, usize)>,
    location: Option<[f64; 4]>,
    result: Option<PitchResult>,
}

impl PitchingEnv {
    pub fn new(cfg: PitchEnvConfig, seed: u64) -> Self {
        let league = Arc::new(League::mlb(cfg.league_seed));
        PitchingEnv::with(cfg, league, Arc::new(RunExpectancy::bundled()), seed)
    }

    pub fn with(cfg: PitchEnvConfig, league: Arc<League>, re: Arc<RunExpectancy>, seed: u64) -> Self {
        PitchingEnv {
            cfg,
            league,
            re,
            rng: Rng::new(seed),
            sim: None,
            fixture: None,
            side: 1,
            decisions: 0,
            last: None,
            last_runs: 0,
            thrown: None,
            location: None,
            result: None,
        }
    }

    pub fn config(&self) -> &PitchEnvConfig {
        &self.cfg
    }

    pub fn sim(&self) -> Option<&GameSim> {
        self.sim.as_ref()
    }

    pub fn side(&self) -> usize {
        self.side
    }

    pub fn run_expectancy(&self) -> &RunExpectancy {
        &self.re
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
    }

    /// Start a new game and play it up to the agent's first pitch.
    pub fn reset(&mut self, seed: Option<u64>) -> ([f32; N_PITCH_OBS], PitchInfo) {
        if let Some(seed) = seed {
            self.seed(seed);
        }
        let f = self.league.fixture(&mut self.rng);
        self.side = self.cfg.side.unwrap_or_else(|| self.rng.below(2));
        let base = &self.league.envs[f.env].params;
        let mut params = perturb(base, self.cfg.perturb, &mut self.rng);
        params.tendency_strength = self.cfg.adapt;
        let mut sim = self.league.start_with(&f, params);
        let opp = 1 - self.side;
        sim.set_style(opp, self.cfg.opponent);
        sim.set_style(self.side, self.cfg.own);
        sim.set_auto(opp, Auto::ALL);
        sim.set_auto(self.side, Auto::ALL);
        self.sim = Some(sim);
        self.fixture = Some(f);
        self.decisions = 0;
        self.last = None;
        self.last_runs = 0;
        self.thrown = None;
        self.location = None;
        self.result = None;
        self.advance();
        (self.observation(), self.info())
    }

    /// The slot of `kind` in the current pitcher's arsenal, or his most-used pitch in this
    /// count if he has no such pitch.
    fn slot(&self, kind: PitchKind) -> usize {
        let sim = self.sim.as_ref().expect("game");
        let (prof, _) = sim.pitcher_now(self.side);
        if let Some(i) = prof.pitches.iter().position(|p| p.kind == kind) {
            return i;
        }
        let s = sim.situation();
        let (w, n) = usual(sim, &s, self.side);
        (0..n).max_by(|&a, &b| w[a].total_cmp(&w[b])).unwrap_or(0)
    }

    /// Call the next pitch and play on to the agent's next pitch or the end of the game.
    pub fn step(&mut self, action: usize) -> PitchStep {
        assert!(action < N_PITCH_ACTIONS, "action {action} out of range");
        let side = self.side;
        let (kind, intent) = decode(action);
        let s0 = self.sim.as_ref().expect("reset before step").situation();
        assert!(!s0.over, "the game is over: reset");
        assert_eq!(s0.fielding, side, "not the agent's pitch");
        let half = (s0.inning, s0.top);
        let batting = s0.batting;
        let re0 = self.re.value(bases(&s0), s0.outs, s0.balls, s0.strikes);
        let runs0 = s0.score[batting];
        // The managers' moves are already made (`advance` settles the game up to the pitch),
        // so the pitcher who throws is the one the call was made for. An intentional walk
        // may still come first; the call then stands for the next batter's first pitch.
        let mut seen = Where(None, None);
        loop {
            let idx = self.slot(kind);
            let sim = self.sim.as_mut().expect("game");
            sim.set_pitch_order(PitchOrder { idx, intent });
            match sim.step(&mut self.rng, &mut seen) {
                Step::Pitch => {
                    let (prof, _) = sim.pitcher_now(side);
                    let thrown = prof.pitches.get(idx).map(|p| p.kind).unwrap_or(kind);
                    self.thrown = Some((thrown, idx));
                    self.last = Some(thrown);
                    break;
                }
                Step::GameOver => break,
                Step::Change | Step::IntentionalWalk => {
                    sim.clear_pitch_order();
                    let s = sim.situation();
                    if s.over || s.fielding != side || (s.inning, s.top) != half {
                        break;
                    }
                    while sim.settle(&mut self.rng, &mut NoObserver) == Step::Change {}
                }
            }
        }
        self.decisions += 1;
        self.location = seen.0;
        self.result = seen.1;
        let sim = self.sim.as_ref().expect("game");
        let s1 = sim.situation();
        let runs = s1.score[batting] - runs0;
        let over_half = s1.over || (s1.inning, s1.top) != half;
        let re1 = if over_half { 0.0 } else { self.re.value(bases(&s1), s1.outs, s1.balls, s1.strikes) };
        if over_half || (s1.balls == 0 && s1.strikes == 0) {
            self.last = None;
        }
        self.last_runs = runs;
        let reward = -(re1 - re0 + runs as f64);
        self.advance();
        let terminated = self.sim.as_ref().is_some_and(|g| g.is_over());
        PitchStep { obs: self.observation(), reward, terminated, truncated: false, info: self.info() }
    }

    /// Play on while it is not the agent's pitch (its club bats), then let the managers make
    /// their moves before the pitch, so that the observation shows who throws it to whom.
    fn advance(&mut self) {
        let side = self.side;
        let sim = self.sim.as_mut().expect("game");
        loop {
            while !sim.is_over() && sim.situation().fielding != side {
                if sim.step(&mut self.rng, &mut NoObserver) == Step::GameOver {
                    break;
                }
                self.last = None;
            }
            if sim.is_over() {
                return;
            }
            while sim.settle(&mut self.rng, &mut NoObserver) == Step::Change {}
            if sim.situation().fielding == side || sim.is_over() {
                return;
            }
        }
    }

    /// Actions with a pitch type the current pitcher throws (all of them once the game is over).
    pub fn action_mask(&self) -> [bool; N_PITCH_ACTIONS] {
        let sim = self.sim.as_ref().expect("reset first");
        if sim.is_over() {
            return [true; N_PITCH_ACTIONS];
        }
        let (prof, _) = sim.pitcher_now(self.side);
        let mut m = [false; N_PITCH_ACTIONS];
        for p in &prof.pitches {
            for i in 0..3 {
                m[p.kind.index() * 3 + i] = true;
            }
        }
        m
    }

    pub fn observation(&self) -> [f32; N_PITCH_OBS] {
        let sim = self.sim.as_ref().expect("reset first");
        observe(sim, self.side, &self.re, self.last)
    }

    pub fn info(&self) -> PitchInfo {
        let sim = self.sim.as_ref().expect("reset first");
        let s = sim.situation();
        PitchInfo {
            inning: s.inning,
            top: s.top,
            outs: s.outs,
            balls: s.balls,
            strikes: s.strikes,
            bases: bases(&s),
            score: [s.score[self.side], s.score[1 - self.side]],
            side: self.side,
            decisions: self.decisions,
            runs: self.last_runs,
            thrown: self.thrown,
            location: self.location,
            result: self.result,
        }
    }
}

/// The current pitcher's usual selection weights in this count against this batter.
fn usual(sim: &GameSim, s: &Situation, side: usize) -> ([f64; MAX_PITCHES], usize) {
    let (prof, _) = sim.pitcher_now(side);
    let batter = sim.team(s.batting).players[s.batter].batter.facing(prof, 0.0);
    let mut w = [0.0; MAX_PITCHES];
    let n = selection_weights(prof, batter.hand, s.balls as usize, s.strikes as usize, &sim.env().params, &mut w);
    (w, n)
}

/// The observation for the side that is pitching (`side`). `last` is the type of the
/// previous pitch of this plate appearance.
pub fn observe(sim: &GameSim, side: usize, re: &RunExpectancy, last: Option<PitchKind>) -> [f32; N_PITCH_OBS] {
    let mut o = [0.0f32; N_PITCH_OBS];
    let s = sim.situation();
    if s.over || s.fielding != side {
        return o;
    }
    let params = &sim.env().params;
    o[s.balls.min(3) as usize * 3 + s.strikes.min(2) as usize] = 1.0;
    o[12 + s.outs.min(2) as usize] = 1.0;
    let b = bases(&s);
    for i in 0..3 {
        o[15 + i] = f32::from(b & (1 << i) != 0);
    }
    o[18] = ((s.inning as f32 - 1.0) / 8.0).min(1.5);
    let lead = s.score[side] as i32 - s.score[1 - side] as i32;
    o[19] = (lead as f32 / 5.0).clamp(-2.0, 2.0);
    o[20] = re.value(b, s.outs, s.balls, s.strikes) as f32;
    let (prof, st) = sim.pitcher_now(side);
    let batter = sim.team(s.batting).players[s.batter].batter.facing(prof, 0.0);
    o[21] = f32::from(batter.hand == prof.hand);
    o[22] = ((batter.quality() - 1.0) / 0.1) as f32;
    o[23] = ((batter.eye - 0.615) / 0.07) as f32;
    o[24] = ((batter.barrel_sd / AVERAGE_BARREL_SD - 1.0) / 0.2) as f32;
    o[25] = ((batter.bat_speed / AVERAGE_BAT_SPEED - 1.0) / 0.03) as f32;
    o[26] = f32::from(prof.hand == Hand::Left);
    o[27] = (st.pitch_count as f64 / prof.stamina.max(1.0)) as f32;
    let mut w = [0.0; MAX_PITCHES];
    let n = selection_weights(prof, batter.hand, s.balls as usize, s.strikes as usize, params, &mut w);
    let total: f64 = w[..n].iter().sum();
    let read = st.tendency.beyond_chance(params);
    for (i, p) in prof.pitches.iter().enumerate().take(n) {
        let k = 28 + p.kind.index() * 5;
        o[k] = 1.0;
        o[k + 1] = if total > 0.0 { (w[i] / total) as f32 } else { 0.0 };
        o[k + 2] = (read[i] * params.tendency_strength / 0.1) as f32;
        // Location error of the pitch, against a typical 7 in.
        o[k + 3] = (p.command / (7.0 * 0.0254) - 1.0) as f32;
    }
    if let Some(k) = last {
        o[28 + k.index() * 5 + 4] = 1.0;
    }
    o[68] = f32::from(s.balls == 0 && s.strikes == 0);
    // How far the batters' timing has moved off the fastball (10 ms units).
    if params.tendency_strength > 0.0 && n > 0 {
        let plain = expectation(prof, st, &w[..n], params.expect_mix);
        let adapted = expectation_adapted(prof, st, &w[..n], params);
        o[69] = ((adapted.time - plain.time) / 0.01) as f32;
        // How much readier (+) or slower (−) to swing the batters are from the pitcher's
        // location in this game, in tenths.
        o[70] = (adapted.swing_lean / 0.1) as f32;
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_round_trip_and_names_line_up() {
        for a in 0..N_PITCH_ACTIONS {
            let (k, i) = decode(a);
            assert_eq!(encode(k, i), a);
        }
        assert_eq!(action_name(encode(PitchKind::Slider, Intent::Chase)), "slider+chase");
        assert_eq!(obs_names().len(), N_PITCH_OBS);
    }

    #[test]
    fn calls_are_thrown_and_rewards_add_up_to_the_runs() {
        let mut env = PitchingEnv::new(PitchEnvConfig { side: Some(1), ..Default::default() }, 3);
        let (obs, info) = env.reset(None);
        assert_eq!(info.side, 1);
        assert_eq!(obs.iter().take(12).sum::<f32>(), 1.0);
        let re00 = env.run_expectancy().value(0, 0, 0, 0);
        let (mut total, mut steps, mut halves) = (0.0, 0, 0);
        let mut rng = Rng::new(1);
        let mut half = None;
        loop {
            let s = env.sim().unwrap().situation();
            if half != Some((s.inning, s.top)) {
                half = Some((s.inning, s.top));
                halves += 1;
                // A half's rewards add up to its expectancy at the first call (bases empty and
                // nobody out, or an extra inning's runner on second) minus the runs allowed in it.
                let start = env.run_expectancy().value(bases(&s), s.outs, s.balls, s.strikes);
                assert!(s.inning > 9 || start == re00);
                total += start;
            }
            let mask = env.action_mask();
            let options: Vec<usize> = (0..N_PITCH_ACTIONS).filter(|&a| mask[a]).collect();
            let a = options[rng.below(options.len())];
            let r = env.step(a);
            if let Some((kind, _)) = r.info.thrown {
                assert_eq!(kind, decode(a).0, "called {}, thrown {kind:?}", action_name(a));
            }
            total -= r.reward;
            steps += 1;
            if r.terminated {
                let allowed = r.info.score[1];
                assert!((total - allowed as f64).abs() < 1e-6, "rewards {total} against {allowed} runs allowed over {halves} halves");
                break;
            }
        }
        assert!(steps > 80, "pitches called {steps}");
    }

    #[test]
    fn the_same_seed_gives_the_same_game() {
        let play = |seed| {
            let mut env = PitchingEnv::new(PitchEnvConfig { perturb: 1.0, ..Default::default() }, seed);
            env.reset(None);
            let mut rewards = Vec::new();
            loop {
                let m = env.action_mask();
                let a = (0..N_PITCH_ACTIONS).find(|&a| m[a]).unwrap();
                let r = env.step(a);
                rewards.push(r.reward);
                if r.terminated {
                    return (rewards, r.info.score);
                }
            }
        };
        assert_eq!(play(7), play(7));
        assert_ne!(play(7).0, play(8).0);
    }

    #[test]
    fn shaking_moves_the_perception_constants_only() {
        let base = ModelParams::default();
        let mut rng = Rng::new(2);
        let p = perturb(&base, 1.0, &mut rng);
        assert_ne!(p.prior_pull, base.prior_pull);
        assert_ne!(p.kind_recognition, base.kind_recognition);
        assert_eq!(p.kind_recognition[PitchKind::FourSeam.index()], base.kind_recognition[PitchKind::FourSeam.index()]);
        assert_eq!(p.zone_intent, base.zone_intent);
        assert_eq!(p.count_kind, base.count_kind);
        let same = perturb(&base, 0.0, &mut rng);
        assert_eq!(same.prior_pull, base.prior_pull);
    }
}
