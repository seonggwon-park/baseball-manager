//! The game: a state machine advanced one pitch at a time (innings, plate
//! appearances, pitch-by-pitch resolution), substitutions by a simple AI manager,
//! and the box score.

use crate::batting::{respond, BattedInfo, Miss, Swing};
use crate::fielding::{move_time, standard_alignment, C, P};
use crate::params::ModelParams;
use crate::pitching::{choose_pitch, expectation, selection_weights, throw_pitch, MatchEnv, PitcherState, Thrown, MAX_PITCHES};
use crate::play::{resolve, PlayInput, PlayKind, PlayResult, Runner};
use crate::profile::{CatcherProfile, FielderProfile, Hand, PitcherProfile, Player};
use crate::rng::{sigmoid, Rng};
use crate::rules::StrikeZone;
use bm_physics::{BattedPath, BattedSim, Vec3, BASE_PATH};
use std::sync::Arc;

/// Integration step and sample spacing for batted balls, s.
const BATTED_DT: f64 = 0.02;
const BATTED_SAMPLE_DT: f64 = 0.06;
/// Spray angle (rad) beyond which a ball launched below `OBVIOUS_FOUL_LAUNCH` (rad)
/// cannot come back fair or be caught.
const OBVIOUS_FOUL_SPRAY: f64 = 1.0;
const OBVIOUS_FOUL_LAUNCH: f64 = 0.30;

/// A team as it takes the field for one game.
#[derive(Clone, Debug)]
pub struct Team {
    pub players: Vec<Player>,
    /// Batting order (indices into `players`).
    pub lineup: [usize; 9],
    /// Fielders by position: P, C, 1B, 2B, 3B, SS, LF, CF, RF. `defense[0]` is the starting pitcher.
    pub defense: [usize; 9],
    /// Relievers in the order they are used; the last one is the closer.
    pub bullpen: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BatLine {
    pub pa: u16,
    pub ab: u16,
    pub h: u16,
    pub doubles: u16,
    pub triples: u16,
    pub hr: u16,
    pub bb: u16,
    pub hbp: u16,
    pub so: u16,
    pub sf: u16,
    pub gidp: u16,
    pub r: u16,
    pub rbi: u16,
    pub sb: u16,
    pub cs: u16,
    pub roe: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PitchLine {
    pub outs: u16,
    pub bf: u16,
    pub h: u16,
    pub r: u16,
    pub er: u16,
    pub bb: u16,
    pub so: u16,
    pub hr: u16,
    pub hbp: u16,
    pub pitches: u16,
    pub strikes: u16,
    pub wp: u16,
    pub started: bool,
    pub appeared: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TeamBox {
    pub batting: Vec<BatLine>,
    pub pitching: Vec<PitchLine>,
    pub hits: u32,
    pub errors: u32,
    pub left_on_base: u32,
}

#[derive(Clone, Debug)]
pub struct GameResult {
    /// Runs: `[away, home]`.
    pub score: [u32; 2],
    pub innings: u8,
    pub line_score: Vec<[u8; 2]>,
    pub boxes: [TeamBox; 2],
    /// `(team, player index)` of the winning, losing and saving pitcher.
    pub win: Option<(usize, usize)>,
    pub loss: Option<(usize, usize)>,
    pub save: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchResult {
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    FoulTip,
    InPlay,
    HitByPitch,
}

/// Everything about one pitch, for play-by-play, the match view and validation.
pub struct PitchEvent<'a> {
    pub inning: u8,
    pub top: bool,
    pub balls: u8,
    pub strikes: u8,
    pub outs: u8,
    pub pitcher_hand: Hand,
    pub batter_hand: Hand,
    pub thrown: &'a Thrown,
    pub zone: StrikeZone,
    /// Signed distance outside the rulebook zone, m.
    pub zone_distance: f64,
    pub swung: bool,
    pub result: PitchResult,
    pub bat_speed: Option<f64>,
    /// Why the swing missed, for swinging strikes.
    pub miss: Option<Miss>,
    pub batted: Option<&'a BattedInfo>,
    pub path: Option<&'a BattedPath>,
    pub play: Option<&'a PlayResult>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaOutcome {
    Strikeout,
    Walk,
    HitByPitch,
    Play(PlayKind),
    /// The inning ended on the bases before the plate appearance was completed.
    Interrupted,
}

#[derive(Clone, Copy, Debug)]
pub struct PaEvent {
    pub inning: u8,
    pub top: bool,
    pub batter: usize,
    pub pitcher: usize,
    pub outcome: PaOutcome,
    pub rbi: u8,
    pub runs: u8,
    pub outs_made: u8,
    pub pitches: u8,
    /// Base state before the plate appearance (bit 0 = first, 1 = second, 2 = third).
    pub bases_before: u8,
    /// Base state after the plate appearance.
    pub bases_after: u8,
    pub outs_before: u8,
    /// The ball in play was a ground ball (first touched the ground within 45 m in under 0.9 s).
    pub ground_ball: bool,
}

/// Receives the stream of events of a game.
pub trait Observer {
    fn pitch(&mut self, _e: &PitchEvent) {}
    fn plate_appearance(&mut self, _e: &PaEvent) {}
    fn steal(&mut self, _success: bool) {}
    fn wild_pitch(&mut self) {}
    fn pitching_change(&mut self, _team: usize, _player: usize) {}
}

/// Observer that ignores everything.
pub struct NoObserver;
impl Observer for NoObserver {}

struct Side {
    team: Arc<Team>,
    lineup: [usize; 9],
    order: usize,
    defense: [usize; 9],
    fielders: [FielderProfile; 9],
    catcher: CatcherProfile,
    pitcher: usize,
    pitcher_state: PitcherState,
    bullpen_used: usize,
    closer_used: bool,
    is_starter: bool,
    cur_bf: u32,
    cur_runs: u32,
    cur_inning_runs: u32,
    cur_inning_bf: u32,
    entry_lead: i32,
    box_score: TeamBox,
}

impl Side {
    fn new(team: Arc<Team>, env: &MatchEnv) -> Self {
        let pitcher = team.defense[P];
        let mut fielders = [FielderProfile::league_average(); 9];
        for (i, f) in fielders.iter_mut().enumerate() {
            *f = team.players[team.defense[i]].fielder;
        }
        let mut box_score =
            TeamBox { batting: vec![BatLine::default(); team.players.len()], pitching: vec![PitchLine::default(); team.players.len()], ..Default::default() };
        box_score.pitching[pitcher].started = true;
        box_score.pitching[pitcher].appeared = true;
        Side {
            lineup: team.lineup,
            order: 0,
            defense: team.defense,
            fielders,
            catcher: team.players[team.defense[C]].catcher,
            pitcher,
            pitcher_state: PitcherState::new(profile_of(&team, pitcher), env),
            bullpen_used: 0,
            closer_used: false,
            is_starter: true,
            cur_bf: 0,
            cur_runs: 0,
            cur_inning_runs: 0,
            cur_inning_bf: 0,
            entry_lead: 0,
            box_score,
            team,
        }
    }
}

fn profile_of(team: &Team, idx: usize) -> &PitcherProfile {
    team.players[idx].pitcher.as_ref().expect("player used as a pitcher has a pitching profile")
}

/// The plate appearance in progress.
struct Pa {
    batter_idx: usize,
    pitcher_idx: usize,
    zone: StrikeZone,
    me: Runner,
    box_delay: f64,
    bases_before: u8,
    outs_before: u8,
    positions: [Vec3; 9],
    balls: usize,
    strikes: usize,
    pitches: u8,
    /// `None` while the plate appearance is still going on.
    outcome: Option<PaOutcome>,
    rbi: u8,
    ground_ball: bool,
    score_before: u32,
}

/// What a call to [`GameSim::step`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// A pitch was thrown (and everything that followed from it was resolved).
    Pitch,
    /// The game is over; nothing happened.
    GameOver,
}

/// A game in progress, advanced one pitch at a time.
pub struct GameSim {
    env: Arc<MatchEnv>,
    sides: [Side; 2],
    positions: [Vec3; 9],
    inning: u8,
    top: bool,
    outs: u8,
    bases: [Option<Runner>; 3],
    score: [u32; 2],
    line_score: Vec<[u8; 2]>,
    path: BattedPath,
    /// Outs plus errors that should have been outs in this half inning (for earned runs).
    phantom_outs: u8,
    win_candidate: Option<(usize, usize)>,
    loss_candidate: Option<(usize, usize)>,
    pa: Option<Pa>,
    /// The current half inning has started.
    half_open: bool,
    result: Option<GameResult>,
}

/// Play one game from start to finish. `away` bats first.
pub fn play_game<O: Observer>(env: &MatchEnv, away: &Team, home: &Team, rng: &mut Rng, obs: &mut O) -> GameResult {
    let mut g = GameSim::new(Arc::new(env.clone()), Arc::new(away.clone()), Arc::new(home.clone()));
    while !g.is_over() {
        g.step(rng, obs);
    }
    g.into_result()
}

impl GameSim {
    pub fn new(env: Arc<MatchEnv>, away: Arc<Team>, home: Arc<Team>) -> Self {
        let sides = [Side::new(away, &env), Side::new(home, &env)];
        GameSim {
            env,
            sides,
            positions: standard_alignment(),
            inning: 1,
            top: true,
            outs: 0,
            bases: [None; 3],
            score: [0, 0],
            line_score: Vec::with_capacity(12),
            path: BattedPath::default(),
            phantom_outs: 0,
            win_candidate: None,
            loss_candidate: None,
            pa: None,
            half_open: false,
            result: None,
        }
    }

    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// The final result, once the game is over.
    pub fn result(&self) -> Option<&GameResult> {
        self.result.as_ref()
    }

    /// Consume a finished game.
    pub fn into_result(self) -> GameResult {
        self.result.expect("the game is over")
    }

    /// Throw the next pitch and resolve everything that follows from it.
    pub fn step<O: Observer>(&mut self, rng: &mut Rng, obs: &mut O) -> Step {
        if self.result.is_some() {
            return Step::GameOver;
        }
        if !self.half_open {
            self.open_half();
        }
        let (bat, fld) = if self.top { (0, 1) } else { (1, 0) };
        if self.pa.is_none() {
            self.consider_pitching_change(fld, bat, obs);
            self.begin_pa(bat, fld);
        }
        self.pitch(bat, fld, rng, obs);
        if self.pa.as_ref().is_some_and(|pa| pa.outcome.is_some()) || self.outs >= 3 {
            self.end_pa(bat, fld, obs);
            let walk_off = !self.top && self.inning >= self.env.rules.innings && self.score[1] > self.score[0];
            if self.outs >= 3 || walk_off {
                self.close_half(bat);
            }
        }
        Step::Pitch
    }

    fn open_half(&mut self) {
        if self.top {
            self.line_score.push([0, 0]);
        }
        let (bat, fld) = if self.top { (0, 1) } else { (1, 0) };
        self.outs = 0;
        self.bases = [None; 3];
        self.phantom_outs = 0;
        self.sides[fld].cur_inning_runs = 0;
        self.sides[fld].cur_inning_bf = 0;
        if self.env.rules.auto_runner_from.is_some_and(|n| self.inning >= n) {
            let s = &self.sides[bat];
            let idx = s.lineup[(s.order + 8) % 9];
            let b = &s.team.players[idx].batter;
            self.bases[1] =
                Some(Runner { player: idx, sprint: b.sprint, accel: b.accel, aggr: b.baserunning, pitcher: self.sides[fld].pitcher, earned: false });
        }
        self.half_open = true;
    }

    /// The half inning is over: move on to the next one, or end the game.
    fn close_half(&mut self, bat: usize) {
        self.sides[bat].box_score.left_on_base += self.bases.iter().flatten().count() as u32;
        self.half_open = false;
        let (innings, max_innings) = (self.env.rules.innings, self.env.rules.max_innings);
        if self.top {
            let skip_bottom = self.inning >= innings && self.score[1] > self.score[0];
            if !skip_bottom {
                self.top = false;
                return;
            }
        }
        let decided = self.score[0] != self.score[1];
        let finished = self.inning >= innings && (decided || max_innings.is_some_and(|m| self.inning >= m));
        if finished || self.inning >= 60 {
            self.result = Some(self.finish());
            return;
        }
        self.inning += 1;
        self.top = true;
    }

    fn finish(&self) -> GameResult {
        let mut win = None;
        let mut loss = None;
        let mut save = None;
        if self.score[0] != self.score[1] {
            let w = if self.score[0] > self.score[1] { 0 } else { 1 };
            win = self.win_candidate.filter(|c| c.0 == w);
            loss = self.loss_candidate.filter(|c| c.0 != w);
            // A starter needs five innings for the win; otherwise credit the reliever with the most outs.
            if let Some((t, pidx)) = win {
                let lines = &self.sides[t].box_score.pitching;
                if lines[pidx].started && lines[pidx].outs < 15 {
                    let alt = lines.iter().enumerate().filter(|(i, l)| l.appeared && !l.started && *i != pidx).max_by_key(|(_, l)| l.outs).map(|(i, _)| i);
                    if let Some(i) = alt {
                        win = Some((t, i));
                    }
                }
            }
            let side = &self.sides[w];
            let last = side.pitcher;
            let line = side.box_score.pitching[last];
            if win.is_some_and(|(_, p)| p != last) && !line.started && ((side.entry_lead <= 3 && line.outs >= 3) || line.outs >= 9) && side.entry_lead > 0 {
                save = Some((w, last));
            }
        }
        let boxes = [self.sides[0].box_score.clone(), self.sides[1].box_score.clone()];
        GameResult { score: self.score, innings: self.inning, line_score: self.line_score.clone(), boxes, win, loss, save }
    }

    fn consider_pitching_change<O: Observer>(&mut self, fld: usize, bat: usize, obs: &mut O) {
        let lead = self.score[fld] as i32 - self.score[bat] as i32;
        let runners = self.bases.iter().flatten().count();
        let inning = self.inning;
        let outs = self.outs;
        let s = &self.sides[fld];
        let stamina = profile_of(&s.team, s.pitcher).stamina;
        let pc = s.pitcher_state.pitch_count as f64;
        let fresh_inning = outs == 0 && s.cur_inning_bf == 0;
        let want = if s.is_starter {
            pc >= stamina + 12.0
                || (pc >= stamina - 5.0 && (fresh_inning || runners >= 2))
                || s.cur_runs >= 7
                || (s.cur_runs >= 5 && pc >= 60.0)
                || (inning >= 6 && pc >= stamina - 18.0 && s.cur_inning_runs >= 2)
        } else {
            s.cur_bf >= 3 && (pc >= stamina + 5.0 || (fresh_inning && pc >= 12.0) || s.cur_inning_runs >= 3)
        };
        if !want {
            return;
        }
        let pen = &s.team.bullpen;
        if pen.is_empty() {
            return;
        }
        let closer = pen.len() - 1;
        let save_spot = inning >= self.env.rules.innings && (1..=3).contains(&lead);
        let next = if save_spot && !s.closer_used {
            Some(closer)
        } else if s.bullpen_used < closer {
            Some(s.bullpen_used)
        } else if !s.closer_used {
            Some(closer)
        } else {
            None
        };
        let Some(slot) = next else { return };
        let new_idx = pen[slot];
        let s = &mut self.sides[fld];
        if slot == closer {
            s.closer_used = true;
        } else {
            s.bullpen_used = slot + 1;
        }
        let old = s.pitcher;
        s.pitcher = new_idx;
        s.defense[P] = new_idx;
        s.fielders[P] = s.team.players[new_idx].fielder;
        s.pitcher_state = PitcherState::new(profile_of(&s.team, new_idx), &self.env);
        s.is_starter = false;
        s.cur_bf = 0;
        s.cur_runs = 0;
        s.cur_inning_runs = 0;
        s.entry_lead = lead;
        s.box_score.pitching[new_idx].appeared = true;
        // Without a DH the new pitcher takes over the old pitcher's spot in the order.
        for slot in s.lineup.iter_mut() {
            if *slot == old {
                *slot = new_idx;
            }
        }
        obs.pitching_change(fld, new_idx);
    }

    fn bases_bits(&self) -> u8 {
        self.bases.iter().enumerate().fold(0, |acc, (i, b)| acc | ((b.is_some() as u8) << i))
    }

    fn begin_pa(&mut self, bat: usize, fld: usize) {
        let params = &self.env.params;
        let batter_idx = self.sides[bat].lineup[self.sides[bat].order];
        let batter = &self.sides[bat].team.players[batter_idx].batter;
        let pitcher_idx = self.sides[fld].pitcher;
        let zone = StrikeZone::for_batter(batter.height, &self.env.rules);
        let me = Runner { player: batter_idx, sprint: batter.sprint, accel: batter.accel, aggr: batter.baserunning, pitcher: pitcher_idx, earned: true };
        let box_delay = match batter.hand {
            Hand::Right => params.box_delay_right,
            Hand::Left => params.box_delay_left,
        };
        // With a runner on first and second base open the first baseman holds him on,
        // which opens the hole on the right side.
        let mut positions = self.positions;
        if self.bases[0].is_some() && self.bases[1].is_none() {
            positions[crate::fielding::B1] = Vec3::new(20.4, 22.4, 0.0);
        }
        self.pa = Some(Pa {
            batter_idx,
            pitcher_idx,
            zone,
            me,
            box_delay,
            bases_before: self.bases_bits(),
            outs_before: self.outs,
            positions,
            balls: 0,
            strikes: 0,
            pitches: 0,
            outcome: None,
            rbi: 0,
            ground_ball: false,
            score_before: self.score[bat],
        });
    }

    /// One pitch of the current plate appearance.
    fn pitch<O: Observer>(&mut self, bat: usize, fld: usize, rng: &mut Rng, obs: &mut O) {
        let mut pa = self.pa.take().expect("a plate appearance is in progress");
        self.pitch_inner(&mut pa, bat, fld, rng, obs);
        self.pa = Some(pa);
    }

    fn pitch_inner<O: Observer>(&mut self, pa: &mut Pa, bat: usize, fld: usize, rng: &mut Rng, obs: &mut O) {
        let env = Arc::clone(&self.env);
        let env = &*env;
        let params = &env.params;
        let bat_team = Arc::clone(&self.sides[bat].team);
        let fld_team = Arc::clone(&self.sides[fld].team);
        let batter_idx = pa.batter_idx;
        let batter = &bat_team.players[batter_idx].batter;
        let pitcher_idx = pa.pitcher_idx;
        let pprof = profile_of(&fld_team, pitcher_idx);
        let zone = pa.zone;
        let catcher = self.sides[fld].catcher;
        let (balls, strikes) = (pa.balls, pa.strikes);
        let mut w = [0.0; MAX_PITCHES];

        let n = selection_weights(pprof, batter.hand, balls, strikes, params, &mut w);
        let exp = expectation(pprof, &self.sides[fld].pitcher_state, &w[..n], params.expect_mix);
        let choice = choose_pitch(pprof, batter.hand, &zone, balls, strikes, &w[..n], params, rng);
        let thrown = throw_pitch(pprof, &mut self.sides[fld].pitcher_state, &choice, env, rng);
        pa.pitches = pa.pitches.saturating_add(1);
        let stealing = self.steal_decision(bat, pprof, &catcher, rng);
        let f = thrown.flight;
        let zone_distance = zone.signed_distance(f.plate_pos.x, f.plate_pos.z, env.ball.radius);
        let mut ev = PitchEvent {
            inning: self.inning,
            top: self.top,
            balls: balls as u8,
            strikes: strikes as u8,
            outs: self.outs,
            pitcher_hand: pprof.hand,
            batter_hand: batter.hand,
            thrown: &thrown,
            zone,
            zone_distance,
            swung: true,
            result: PitchResult::Ball,
            bat_speed: None,
            miss: None,
            batted: None,
            path: None,
            play: None,
        };
        let line = &mut self.sides[fld].box_score.pitching[pitcher_idx];
        line.pitches += 1;

        let sw = respond(env, batter, &zone, balls, strikes, &thrown, &exp, rng);
        let mut live_ball = false; // pitch not put in play or fouled: steals and wild pitches can happen
        match sw {
            Swing::Take => {
                ev.swung = false;
                if hit_by_pitch(params, batter.hand, f.plate_pos, rng) {
                    ev.result = PitchResult::HitByPitch;
                    obs.pitch(&ev);
                    pa.outcome = Some(PaOutcome::HitByPitch);
                    return;
                }
                if zone.called_strike(&env.rules, params, f.plate_pos, f.plate_vel, env.ball.radius, catcher.framing, rng.f64()) {
                    pa.strikes += 1;
                    line.strikes += 1;
                    ev.result = PitchResult::CalledStrike;
                } else {
                    pa.balls += 1;
                    ev.result = PitchResult::Ball;
                }
                live_ball = true;
                obs.pitch(&ev);
            }
            Swing::Whiff { bat_speed, miss } => {
                pa.strikes += 1;
                line.strikes += 1;
                ev.result = PitchResult::SwingingStrike;
                ev.bat_speed = Some(bat_speed);
                ev.miss = Some(miss);
                live_ball = true;
                obs.pitch(&ev);
            }
            Swing::FoulTip { caught, bat_speed } => {
                line.strikes += 1;
                ev.bat_speed = Some(bat_speed);
                if caught {
                    pa.strikes += 1;
                    ev.result = PitchResult::FoulTip;
                } else {
                    if pa.strikes < 2 {
                        pa.strikes += 1;
                    }
                    ev.result = PitchResult::Foul;
                }
                obs.pitch(&ev);
            }
            Swing::FoulBack { bat_speed } => {
                line.strikes += 1;
                if pa.strikes < 2 {
                    pa.strikes += 1;
                }
                ev.result = PitchResult::Foul;
                ev.bat_speed = Some(bat_speed);
                obs.pitch(&ev);
            }
            Swing::Batted { state, info } => {
                line.strikes += 1;
                // A ball hit low and well outside the lines is foul without any play on it.
                if info.spray.abs() > OBVIOUS_FOUL_SPRAY && info.launch_angle < OBVIOUS_FOUL_LAUNCH {
                    if pa.strikes < 2 {
                        pa.strikes += 1;
                    }
                    ev.result = PitchResult::Foul;
                    ev.bat_speed = Some(info.bat_speed);
                    obs.pitch(&ev);
                    return;
                }
                let mut flight = env.batted_flight;
                flight.aero.cd0 = (flight.aero.cd0 + rng.gauss(0.0, params.drag_jitter)).max(0.15);
                let mut sim = BattedSim::new(&flight, &env.ball, &env.park);
                sim.dt = BATTED_DT;
                sim.sample_dt = BATTED_SAMPLE_DT;
                sim.run(state, &mut self.path);
                let inp = PlayInput {
                    env,
                    path: &self.path,
                    info: &info,
                    fielders: &self.sides[fld].fielders,
                    positions: &pa.positions,
                    bases: self.bases,
                    batter: pa.me,
                    box_delay: pa.box_delay,
                    outs: self.outs,
                };
                let res = resolve(&inp, rng);
                ev.bat_speed = Some(info.bat_speed);
                ev.batted = Some(&info);
                ev.path = Some(&self.path);
                ev.play = Some(&res);
                if res.kind == PlayKind::Foul {
                    if pa.strikes < 2 {
                        pa.strikes += 1;
                    }
                    ev.result = PitchResult::Foul;
                    obs.pitch(&ev);
                } else {
                    ev.result = PitchResult::InPlay;
                    obs.pitch(&ev);
                    pa.outcome = Some(PaOutcome::Play(res.kind));
                    pa.ground_ball = self.path.landing.is_some_and(|l| l.t < 0.9 && l.pos.horiz() < 45.0);
                    pa.rbi = res.rbi;
                    self.apply_play(bat, fld, batter_idx, pitcher_idx, &res);
                }
                return;
            }
        }

        // Strike three with two outs ends the inning before anything happens on the bases.
        let inning_over = pa.strikes >= 3 && self.outs >= 2;
        if live_ball && !inning_over && self.bases.iter().any(|b| b.is_some()) {
            if let Some(runner) = stealing {
                self.resolve_steal(bat, fld, runner, pprof, &catcher, &thrown, rng, obs);
            } else if is_wild(f.plate_pos) && rng.chance((params.wild_pitch_prob * catcher.blocking).min(0.9)) {
                self.sides[fld].box_score.pitching[pitcher_idx].wp += 1;
                self.advance_all_one(bat, fld);
                obs.wild_pitch();
            }
        }
        if pa.strikes >= 3 {
            pa.outcome = Some(PaOutcome::Strikeout);
        } else if pa.balls >= 4 {
            pa.outcome = Some(PaOutcome::Walk);
        }
    }

    /// Book-keeping at the end of a plate appearance.
    fn end_pa<O: Observer>(&mut self, bat: usize, fld: usize, obs: &mut O) {
        let pa = self.pa.take().expect("a plate appearance is in progress");
        let outcome = pa.outcome.unwrap_or(PaOutcome::Interrupted);
        let (batter_idx, pitcher_idx) = (pa.batter_idx, pa.pitcher_idx);
        let mut rbi = pa.rbi;
        let bl = &mut self.sides[bat].box_score.batting[batter_idx];
        let pl = &mut self.sides[fld].box_score.pitching[pitcher_idx];
        match outcome {
            PaOutcome::Strikeout => {
                bl.pa += 1;
                bl.ab += 1;
                bl.so += 1;
                pl.bf += 1;
                pl.so += 1;
                pl.outs += 1;
                self.outs += 1;
                self.phantom_outs += 1;
            }
            PaOutcome::Walk | PaOutcome::HitByPitch => {
                bl.pa += 1;
                pl.bf += 1;
                if outcome == PaOutcome::Walk {
                    bl.bb += 1;
                    pl.bb += 1;
                } else {
                    bl.hbp += 1;
                    pl.hbp += 1;
                }
                rbi = self.force_walk(bat, fld, pa.me);
                self.sides[bat].box_score.batting[batter_idx].rbi += rbi as u16;
            }
            PaOutcome::Play(_) | PaOutcome::Interrupted => {}
        }
        if outcome != PaOutcome::Interrupted {
            let s = &mut self.sides[bat];
            s.order = (s.order + 1) % 9;
            let d = &mut self.sides[fld];
            d.cur_bf += 1;
            d.cur_inning_bf += 1;
        }
        let runs = (self.score[bat] - pa.score_before).min(255) as u8;
        let bases_after = self.bases_bits();
        obs.plate_appearance(&PaEvent {
            inning: self.inning,
            top: self.top,
            batter: batter_idx,
            pitcher: pitcher_idx,
            outcome,
            rbi,
            runs,
            outs_made: self.outs - pa.outs_before,
            pitches: pa.pitches,
            bases_before: pa.bases_before,
            bases_after,
            outs_before: pa.outs_before,
            ground_ball: pa.ground_ball,
        });
    }

    /// Record a run for `runner`.
    fn run_scores(&mut self, bat: usize, fld: usize, runner: Runner) {
        let was = (self.score[bat] as i32 - self.score[fld] as i32).signum();
        self.score[bat] += 1;
        let now = (self.score[bat] as i32 - self.score[fld] as i32).signum();
        let half = if self.top { 0 } else { 1 };
        if let Some(l) = self.line_score.last_mut() {
            l[half] = l[half].saturating_add(1);
        }
        self.sides[bat].box_score.batting[runner.player].r += 1;
        let earned = runner.earned && self.phantom_outs < 3;
        let d = &mut self.sides[fld];
        let pl = &mut d.box_score.pitching[runner.pitcher];
        pl.r += 1;
        if earned {
            pl.er += 1;
        }
        if runner.pitcher == d.pitcher {
            d.cur_runs += 1;
            d.cur_inning_runs += 1;
        }
        if now > 0 && was <= 0 {
            // Go-ahead run: the batting side's current pitcher is in line for the win.
            self.win_candidate = Some((bat, self.sides[bat].pitcher));
            self.loss_candidate = Some((fld, runner.pitcher));
        } else if now == 0 {
            self.win_candidate = None;
            self.loss_candidate = None;
        }
    }

    fn apply_play(&mut self, bat: usize, fld: usize, batter_idx: usize, pitcher_idx: usize, res: &PlayResult) {
        self.bases = res.bases;
        self.outs += res.outs;
        {
            let bl = &mut self.sides[bat].box_score.batting[batter_idx];
            bl.pa += 1;
            if res.kind != PlayKind::SacFly {
                bl.ab += 1;
            } else {
                bl.sf += 1;
            }
            bl.rbi += res.rbi as u16;
            match res.kind {
                PlayKind::Single => bl.h += 1,
                PlayKind::Double | PlayKind::GroundRuleDouble => {
                    bl.h += 1;
                    bl.doubles += 1;
                }
                PlayKind::Triple => {
                    bl.h += 1;
                    bl.triples += 1;
                }
                PlayKind::HomeRun => {
                    bl.h += 1;
                    bl.hr += 1;
                }
                PlayKind::Error => bl.roe += 1,
                _ => {}
            }
            if res.gidp {
                bl.gidp += 1;
            }
        }
        {
            let d = &mut self.sides[fld].box_score;
            let pl = &mut d.pitching[pitcher_idx];
            pl.bf += 1;
            pl.outs += res.outs as u16;
            if res.kind.is_hit() {
                pl.h += 1;
            }
            if res.kind == PlayKind::HomeRun {
                pl.hr += 1;
            }
            if res.kind == PlayKind::Error {
                d.errors += 1;
            }
        }
        if res.kind.is_hit() {
            self.sides[bat].box_score.hits += 1;
        }
        // Earned-run accounting: an error that should have been an out extends the inning.
        self.phantom_outs += res.outs + if res.kind == PlayKind::Error { 1 } else { 0 };
        for r in res.scored.iter().take(res.n_scored as usize).flatten() {
            self.run_scores(bat, fld, *r);
        }
    }

    /// Batter awarded first base; forced runners move up. Returns the RBI count.
    fn force_walk(&mut self, bat: usize, fld: usize, batter: Runner) -> u8 {
        let mut rbi = 0;
        if self.bases[0].is_some() {
            if self.bases[1].is_some() {
                if let Some(r) = self.bases[2].take() {
                    self.run_scores(bat, fld, r);
                    rbi = 1;
                }
                self.bases[2] = self.bases[1].take();
            }
            self.bases[1] = self.bases[0].take();
        }
        self.bases[0] = Some(batter);
        rbi
    }

    /// Every runner moves up one base (wild pitch, passed ball).
    fn advance_all_one(&mut self, bat: usize, fld: usize) {
        if let Some(r) = self.bases[2].take() {
            self.run_scores(bat, fld, r);
        }
        self.bases[2] = self.bases[1].take();
        self.bases[1] = self.bases[0].take();
    }

    /// Does the runner on first go on this pitch?
    fn steal_decision(&self, bat: usize, pitcher: &PitcherProfile, catcher: &CatcherProfile, rng: &mut Rng) -> Option<Runner> {
        let p = &self.env.params;
        let runner = self.bases[0]?;
        if self.bases[1].is_some() {
            return None;
        }
        let skill = self.sides[bat].team.players[runner.player].batter.steal;
        let margin = steal_defence_time(p, pitcher, catcher) - steal_run_time(p, &runner, skill);
        let prob = p.steal_rate * skill * sigmoid((margin - p.steal_margin) / 0.06);
        rng.chance(prob).then_some(runner)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_steal<O: Observer>(
        &mut self,
        bat: usize,
        fld: usize,
        runner: Runner,
        pitcher: &PitcherProfile,
        catcher: &CatcherProfile,
        thrown: &Thrown,
        rng: &mut Rng,
        obs: &mut O,
    ) {
        let env = Arc::clone(&self.env);
        let p = &env.params;
        let skill = self.sides[bat].team.players[runner.player].batter.steal;
        // Slow pitches and pitches in the dirt give the runner more time.
        let v = thrown.flight.plate_vel.norm();
        let pitch_penalty = (p.ref_plate_speed - v).max(0.0) * 0.012 + if thrown.flight.plate_pos.z < 0.35 { 0.10 } else { 0.0 };
        let defence = steal_defence_time(p, pitcher, catcher) + pitch_penalty + rng.gauss(0.0, 0.07);
        let run = steal_run_time(p, &runner, skill) + rng.gauss(0.0, 0.06);
        self.bases[0] = None;
        let safe = run < defence;
        if safe {
            self.bases[1] = Some(runner);
            self.sides[bat].box_score.batting[runner.player].sb += 1;
        } else {
            self.outs += 1;
            self.phantom_outs += 1;
            self.sides[bat].box_score.batting[runner.player].cs += 1;
            let pidx = self.sides[fld].pitcher;
            self.sides[fld].box_score.pitching[pidx].outs += 1;
        }
        obs.steal(safe);
    }
}

fn steal_defence_time(p: &ModelParams, pitcher: &PitcherProfile, catcher: &CatcherProfile) -> f64 {
    pitcher.time_to_plate + catcher.pop_time + p.tag_time
}

fn steal_run_time(p: &ModelParams, runner: &Runner, skill: f64) -> f64 {
    // Reaction to the pitcher's first move, then a sprint from the lead to a slide
    // that reaches the bag about 1.8 m early.
    let jump = 0.24 - 0.08 * skill;
    let lead = p.lead[0] + 0.3 + 0.6 * skill;
    jump + move_time(BASE_PATH - lead - 1.8, runner.accel * 1.1, runner.sprint * 0.98)
}

/// Pitch that the catcher may fail to stop (in the dirt, far outside, or over his head).
fn is_wild(plate: Vec3) -> bool {
    plate.z < 0.18 || plate.x.abs() > 0.72 || plate.z > 2.25
}

/// Does a pitch that was not swung at hit the batter?
fn hit_by_pitch(p: &ModelParams, hand: Hand, plate: Vec3, rng: &mut Rng) -> bool {
    if plate.z < 0.10 || plate.z > 1.95 {
        return false;
    }
    let inside = -hand.outside_sign() * plate.x;
    rng.chance(sigmoid((inside - p.batter_body_x) / p.hbp_width) * p.hbp_no_evade)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::population::{Club, Level};
    use crate::rules::Rules;
    use bm_physics::{Atmosphere, Park, Wind};

    fn clubs(seed: u64) -> (Club, Club) {
        let mut rng = Rng::new(seed);
        let level = Level::mlb();
        (Club::random(&mut rng, 1000, &level), Club::random(&mut rng, 2000, &level))
    }

    #[test]
    fn same_seed_gives_the_same_game() {
        let env = MatchEnv::neutral_mlb();
        let (a, h) = clubs(5);
        let (away, home) = (a.team_for_game(0, true), h.team_for_game(1, true));
        let r1 = play_game(&env, &away, &home, &mut Rng::new(99), &mut NoObserver);
        let r2 = play_game(&env, &away, &home, &mut Rng::new(99), &mut NoObserver);
        assert_eq!(r1.score, r2.score);
        assert_eq!(r1.line_score, r2.line_score);
        assert_eq!(r1.boxes[0].batting, r2.boxes[0].batting);
        assert_eq!(r1.boxes[1].pitching, r2.boxes[1].pitching);
        let r3 = play_game(&env, &away, &home, &mut Rng::new(100), &mut NoObserver);
        assert!(r3.line_score != r1.line_score || r3.boxes[0].batting != r1.boxes[0].batting);
    }

    /// Counts everything a second time from the event stream.
    #[derive(Default)]
    struct Audit {
        pa: [u32; 2],
        strikeouts: [u32; 2],
        hits: [u32; 2],
        pitches: [u32; 2],
        outs: [u32; 2],
        runs: [u32; 2],
    }

    impl Observer for Audit {
        fn pitch(&mut self, e: &PitchEvent) {
            self.pitches[if e.top { 1 } else { 0 }] += 1;
            assert!(e.balls < 4 && e.strikes < 3 && e.outs < 3);
            assert!(e.thrown.flight.plate_pos.is_finite() && e.thrown.flight.time > 0.3 && e.thrown.flight.time < 0.7);
        }
        fn plate_appearance(&mut self, e: &PaEvent) {
            let t = if e.top { 0 } else { 1 };
            self.outs[t] += e.outs_made as u32;
            self.runs[t] += e.runs as u32;
            if e.outcome == PaOutcome::Interrupted {
                return;
            }
            self.pa[t] += 1;
            match e.outcome {
                PaOutcome::Strikeout => self.strikeouts[t] += 1,
                PaOutcome::Play(k) if k.is_hit() => self.hits[t] += 1,
                _ => {}
            }
        }
    }

    #[test]
    fn box_scores_are_consistent() {
        let env = MatchEnv::neutral_mlb();
        let (a, h) = clubs(11);
        let mut rng = Rng::new(3);
        for g in 0..60 {
            let (away, home) = (a.team_for_game(g, true), h.team_for_game(g + 2, true));
            let mut audit = Audit::default();
            let res = play_game(&env, &away, &home, &mut rng, &mut audit);
            assert!(res.innings >= 9);
            assert_ne!(res.score[0], res.score[1], "MLB rules play to a decision");
            for t in 0..2 {
                let bat = &res.boxes[t];
                let pit = &res.boxes[1 - t];
                let sum = |f: fn(&BatLine) -> u16| bat.batting.iter().map(|b| f(b) as u32).sum::<u32>();
                let psum = |f: fn(&PitchLine) -> u16| pit.pitching.iter().map(|p| f(p) as u32).sum::<u32>();
                // Runs, hits and strikeouts agree between batting lines, pitching lines and the event stream.
                assert_eq!(sum(|b| b.r), res.score[t]);
                assert_eq!(psum(|p| p.r), res.score[t]);
                assert_eq!(res.line_score.iter().map(|i| i[t] as u32).sum::<u32>(), res.score[t]);
                assert_eq!(audit.runs[t], res.score[t]);
                assert_eq!(sum(|b| b.h), bat.hits);
                assert_eq!(psum(|p| p.h), bat.hits);
                assert_eq!(audit.hits[t], bat.hits);
                assert_eq!(sum(|b| b.so), psum(|p| p.so));
                assert_eq!(audit.strikeouts[t], sum(|b| b.so));
                assert_eq!(sum(|b| b.pa), audit.pa[t]);
                assert_eq!(psum(|p| p.bf), audit.pa[t]);
                assert_eq!(psum(|p| p.pitches), audit.pitches[1 - t]);
                // Plate appearances are at bats plus walks, hit batters and sacrifice flies.
                for b in &bat.batting {
                    assert_eq!(b.pa, b.ab + b.bb + b.hbp + b.sf);
                    assert!(b.h <= b.ab && b.doubles + b.triples + b.hr <= b.h);
                }
                assert!(psum(|p| p.er) <= psum(|p| p.r));
                // Outs recorded by the pitchers match the outs made by the offence.
                assert_eq!(psum(|p| p.outs), audit.outs[t]);
                assert!(sum(|b| b.rbi) <= res.score[t]);
            }
            // Full innings: the visitors always make 3 outs per inning; the home side may
            // skip or cut short its last turn.
            assert_eq!(audit.outs[0], 3 * res.innings as u32);
            assert!(audit.outs[1] <= 3 * res.innings as u32 && audit.outs[1] >= 3 * (res.innings as u32 - 1));
            let (w, _) = res.win.expect("a decided game has a winning pitcher");
            assert_eq!(w, if res.score[0] > res.score[1] { 0 } else { 1 });
            let (l, _) = res.loss.expect("and a losing pitcher");
            assert_ne!(l, w);
        }
    }

    #[test]
    fn ties_are_possible_under_kbo_rules_and_abs_calls_are_exact() {
        let env = MatchEnv::new(
            &Atmosphere::new(22.0, 760.0, 60.0, 50.0),
            Wind::CALM,
            Park::symmetric(99.0, 118.0, 122.0, 3.0),
            Rules::kbo_2025(),
            ModelParams::default(),
        );
        let (a, h) = clubs(21);
        let mut rng = Rng::new(8);
        let mut ties = 0;
        let mut long = 0;
        // About one game in ten goes to extra innings and about a third of those stay
        // tied through the 11th, so 600 games leave no realistic chance of seeing none.
        for g in 0..600 {
            let res = play_game(&env, &a.team_for_game(g, true), &h.team_for_game(g, true), &mut rng, &mut NoObserver);
            assert!(res.innings <= 11, "KBO games end after the 11th inning");
            if res.score[0] == res.score[1] {
                ties += 1;
                assert_eq!(res.innings, 11);
                assert!(res.win.is_none() && res.loss.is_none());
            }
            if res.innings > 9 {
                long += 1;
            }
        }
        assert!(long > 20, "some games should go to extra innings: {long}");
        assert!(ties >= 1 && ties < long, "ties {ties} of {long} extra-inning games");
    }

    /// A coarse guard against regressions in the calibration: a few hundred games must
    /// land in generous bands around the 2025 MLB rates.
    #[test]
    fn league_rates_stay_in_band() {
        let env = MatchEnv::neutral_mlb();
        let mut rng = Rng::new(2025);
        let level = Level::mlb();
        let clubs: Vec<Club> = (0..8).map(|i| Club::random(&mut rng, 1000 * (i + 1), &level)).collect();
        let (mut pa, mut so, mut bb, mut h, mut hr, mut ab, mut runs, mut games) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
        for g in 0..400 {
            let (i, j) = (g % 8, (g * 3 + 1) % 8);
            if i == j {
                continue;
            }
            let res = play_game(&env, &clubs[i].team_for_game(g, true), &clubs[j].team_for_game(g + 1, true), &mut rng, &mut NoObserver);
            games += 1;
            runs += res.score[0] + res.score[1];
            for b in res.boxes.iter().flat_map(|t| t.batting.iter()) {
                pa += b.pa as u32;
                so += b.so as u32;
                bb += b.bb as u32;
                h += b.h as u32;
                hr += b.hr as u32;
                ab += b.ab as u32;
            }
        }
        let f = |x: u32, y: u32| x as f64 / y as f64;
        let (k, w, avg, hr_rate, rpg) = (f(so, pa), f(bb, pa), f(h, ab), f(hr, pa), f(runs, 2 * games));
        assert!((0.195..0.255).contains(&k), "K% {k}");
        assert!((0.065..0.105).contains(&w), "BB% {w}");
        assert!((0.225..0.270).contains(&avg), "AVG {avg}");
        assert!((0.022..0.040).contains(&hr_rate), "HR% {hr_rate}");
        assert!((3.7..5.2).contains(&rpg), "R/G {rpg}");
    }
}
