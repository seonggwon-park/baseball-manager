//! The game: a state machine advanced one pitch at a time (innings, plate
//! appearances, pitch-by-pitch resolution), the orders a manager can give
//! (substitutions, bunts, steals, hit-and-run, intentional walks, defensive
//! positioning), an AI manager for the decisions nobody else makes, and the box
//! score.

use crate::batting::{respond_with, Approach, BattedInfo, Miss, Swing, SwingShape};
use crate::fielding::{alignment, move_time, Alignment, InfieldDepth, OutfieldDepth, Shift, C, P};
use crate::params::ModelParams;
use crate::pitching::{
    aim_pitch, choose_pitch, expectation, expectation_adapted, selection_weights, throw_pitch, MatchEnv, PitchOrder, PitcherState, Thrown, MAX_PITCHES,
};
use crate::play::{resolve, PlayInput, PlayKind, PlayResult, Runner};
use crate::profile::{BatterProfile, CatcherProfile, FielderProfile, Hand, PitchKind, PitcherProfile, Player};
use crate::rng::{sigmoid, Rng};
use crate::rules::StrikeZone;
use bm_physics::units::FT;
use bm_physics::{BattedPath, BattedSim, Vec3, BASE_PATH};
use std::sync::Arc;

/// Integration step and sample spacing for batted balls, s.
pub const BATTED_DT: f64 = 0.02;
pub const BATTED_SAMPLE_DT: f64 = 0.06;
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
    /// Position players who do not start.
    pub bench: Vec<usize>,
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
    pub ibb: u16,
    pub hbp: u16,
    pub so: u16,
    pub sf: u16,
    pub sh: u16,
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
    pub ibb: u16,
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
    /// Player indices in the batting and the fielding team.
    pub batter: usize,
    pub pitcher: usize,
    pub pitcher_hand: Hand,
    pub batter_hand: Hand,
    pub thrown: &'a Thrown,
    pub zone: StrikeZone,
    /// Signed distance outside the rulebook zone, m.
    pub zone_distance: f64,
    /// What the batter was trying to do.
    pub approach: Approach,
    pub swung: bool,
    pub result: PitchResult,
    pub bat_speed: Option<f64>,
    /// Geometry of the swing, when there was one.
    pub swing: Option<SwingShape>,
    /// Why the swing missed, for swinging strikes.
    pub miss: Option<Miss>,
    pub batted: Option<&'a BattedInfo>,
    pub path: Option<&'a BattedPath>,
    pub play: Option<&'a PlayResult>,
    /// Where the fielders stood.
    pub positions: &'a [Vec3; 9],
    /// Release speed of the pitcher's four-seamer, m/s, if he throws one.
    pub four_seam_speed: Option<f64>,
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
    /// The walk was intentional.
    pub intentional: bool,
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

/// Things that happen between pitches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    HalfInning {
        inning: u8,
        top: bool,
    },
    PitchingChange {
        team: usize,
        player: usize,
        replaced: usize,
    },
    PinchHitter {
        team: usize,
        player: usize,
        replaced: usize,
    },
    PinchRunner {
        team: usize,
        player: usize,
        replaced: usize,
        base: u8,
    },
    DefensiveChange {
        team: usize,
        player: usize,
        replaced: usize,
        position: usize,
    },
    IntentionalWalk {
        team: usize,
        batter: usize,
    },
    /// `team` is the batting team; `to_base` 2 or 3.
    Steal {
        team: usize,
        runner: usize,
        to_base: u8,
        safe: bool,
    },
    WildPitch,
    GameOver,
}

/// Receives the stream of events of a game.
pub trait Observer {
    fn pitch(&mut self, _e: &PitchEvent) {}
    fn plate_appearance(&mut self, _e: &PaEvent) {}
    fn steal(&mut self, _success: bool) {}
    fn wild_pitch(&mut self) {}
    fn pitching_change(&mut self, _team: usize, _player: usize) {}
    fn event(&mut self, _e: &GameEvent) {}
}

/// Observer that ignores everything.
pub struct NoObserver;
impl Observer for NoObserver {}

/// Relative rates of the AI manager's tactics by situation, from the 2025 Statcast sample
/// (432 games; docs/tactics.md 7, tools/tactics/situations.py). Bunt attempts per plate
/// appearance with runners on first and/or second, none on third and nobody out were 0.039
/// with a runner on first, 0.063 on second and 0.109 on both in innings 1-8, and about a fifth
/// of that with one out; with nobody out they were 0.042, 0.058, 0.070 and 0.126 in innings
/// 1-3, 4-6, 7-8 and 9 on (extra innings start with a runner on second). Bunts for a hit with
/// the bases empty were 0.0084, 0.0080 and 0.0068 per plate appearance with 0, 1 and 2 outs,
/// and 0.0034 with runners on and two outs. Intentional walks per plate appearance with first
/// base open and a runner in scoring position were 0.003, 0.018, 0.032 and 0.040 by the same
/// inning groups, and five times as frequent with runners on second and third (0.048) as with
/// one of them (0.009-0.010).
const SAC_RUNNER_ON_FIRST: f64 = 0.55;
const SAC_RUNNER_ON_SECOND: f64 = 0.9;
const SAC_FIRST_AND_SECOND: f64 = 1.5;
const SAC_ONE_OUT: f64 = 0.2;
const SAC_INNING: [f64; 4] = [0.6, 0.85, 1.0, 1.8];
const BUNT_HIT_TWO_OUTS: f64 = 0.8;
const BUNT_HIT_RUNNERS_TWO_OUTS: f64 = 0.5;
const IBB_INNING: [f64; 4] = [0.1, 0.57, 1.0, 1.25];
const IBB_SECOND_AND_THIRD: f64 = 3.0;
const IBB_FIRST_TAKEN: f64 = 0.15;

/// Innings 1-3, 4-6, 7-8, 9 and later.
fn inning_group(inning: u8) -> usize {
    match inning {
        0..=3 => 0,
        4..=6 => 1,
        7..=8 => 2,
        _ => 3,
    }
}

/// Tendencies of the AI manager. Rates are probabilities per opportunity, before
/// the situation and the players involved are taken into account.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManagerStyle {
    pub sac_bunt: f64,
    pub bunt_for_hit: f64,
    pub hit_and_run: f64,
    pub intentional_walk: f64,
    pub pinch_hit: f64,
    pub pinch_run: f64,
    /// Plays the infield in, at double-play depth, or the outfield deep when the situation calls for it.
    pub situational_defence: bool,
    /// Puts three infielders on the pull side of pull hitters where the rules allow it, as
    /// MLB clubs did until 2022 (thresholds `full_shift_left_below` / `full_shift_right_below`).
    pub full_shift: bool,
}

impl ManagerStyle {
    /// A manager who never calls anything: every batter hits away against a standard defence.
    pub const fn none() -> Self {
        ManagerStyle {
            sac_bunt: 0.0,
            bunt_for_hit: 0.0,
            hit_and_run: 0.0,
            intentional_walk: 0.0,
            pinch_hit: 0.0,
            pinch_run: 0.0,
            situational_defence: false,
            full_shift: false,
        }
    }

    /// Calibrated, with the situational factors above, to the 2025 MLB frequencies of
    /// sacrifice bunts (0.115 per team game) and intentional walks (0.114 per team game).
    pub const fn mlb() -> Self {
        ManagerStyle {
            sac_bunt: 0.39,
            bunt_for_hit: 0.117,
            hit_and_run: 0.03,
            intentional_walk: 0.19,
            pinch_hit: 0.5,
            pinch_run: 0.5,
            situational_defence: true,
            full_shift: false,
        }
    }

    /// [`ManagerStyle::mlb`] with the full shift of the years before the 2023 restriction.
    pub const fn mlb_2022() -> Self {
        let mut s = ManagerStyle::mlb();
        s.full_shift = true;
        s
    }
}

impl Default for ManagerStyle {
    fn default() -> Self {
        ManagerStyle::mlb()
    }
}

/// Which decisions the AI manager makes for a team (the rest are left to whoever
/// gives the orders).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Auto {
    pub pitching: bool,
    pub bench: bool,
    pub offense: bool,
    pub defense: bool,
}

impl Auto {
    pub const ALL: Auto = Auto { pitching: true, bench: true, offense: true, defense: true };
    pub const NONE: Auto = Auto { pitching: false, bench: false, offense: false, defense: false };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StealOrder {
    /// The runner decides for himself.
    #[default]
    Auto,
    /// Stay put.
    Hold,
    /// Go on the next pitch.
    Go,
}

/// Orders to the batting side. They stand until the plate appearance ends, except
/// `StealOrder::Go`, which is for one pitch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OffenseOrders {
    /// `None` leaves it to the bench (the AI manager if it runs the offence, otherwise hit away).
    pub approach: Option<Approach>,
    /// Which side to bunt to; `None` = the usual side for the situation.
    pub bunt_toward_third: Option<bool>,
    pub steal: StealOrder,
}

/// Orders to the fielding side. They stand until the plate appearance ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DefenseOrders {
    /// `None` leaves the positioning to the bench.
    pub alignment: Option<Alignment>,
    /// Put the batter on first base.
    pub intentional_walk: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Substitution {
    /// New pitcher, for the team in the field.
    Pitcher { player: usize },
    /// Bats for the hitter who is due up.
    PinchHitter { player: usize },
    /// Runs for the runner on `base` (1..=3).
    PinchRunner { base: usize, player: usize },
    /// Takes over a defensive position (1 = C ... 8 = RF) and the batting-order spot of the player who leaves.
    Fielder { position: usize, player: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderError {
    GameOver,
    /// Substitutions are made between plate appearances.
    PlateAppearanceInProgress,
    /// The team is not on the right side of the ball for this move.
    WrongSide,
    /// The player has already been in the game or is not on the roster.
    PlayerUnavailable,
    /// The player cannot pitch.
    NotAPitcher,
    /// The current pitcher has not yet faced the required number of batters.
    PitcherMustFaceMoreBatters,
    /// Nobody to replace there.
    NothingToReplace,
}

/// A snapshot of the game for whoever has to make the next decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Situation {
    pub inning: u8,
    pub top: bool,
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    pub score: [u32; 2],
    /// Player indices (in the batting team) of the runners on first, second, third.
    pub bases: [Option<usize>; 3],
    /// Team at bat and team in the field (0 = away, 1 = home).
    pub batting: usize,
    pub fielding: usize,
    /// The batter (or the hitter due up) and the pitcher.
    pub batter: usize,
    pub pitcher: usize,
    pub pitch_count: u32,
    /// A plate appearance is under way (substitutions have to wait).
    pub in_plate_appearance: bool,
    pub over: bool,
}

#[derive(Clone)]
struct Side {
    team: Arc<Team>,
    lineup: [usize; 9],
    order: usize,
    defense: [usize; 9],
    fielders: [FielderProfile; 9],
    catcher: CatcherProfile,
    pitcher: usize,
    pitcher_state: PitcherState,
    /// Players who have been in the game.
    used: Vec<bool>,
    is_starter: bool,
    cur_bf: u32,
    cur_runs: u32,
    cur_inning_runs: u32,
    cur_inning_bf: u32,
    /// A half inning has ended since the current pitcher came in.
    finished_inning: bool,
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
        let mut used = vec![false; team.players.len()];
        for &i in team.lineup.iter().chain(team.defense.iter()) {
            used[i] = true;
        }
        Side {
            lineup: team.lineup,
            order: 0,
            defense: team.defense,
            fielders,
            catcher: team.players[team.defense[C]].catcher,
            pitcher,
            pitcher_state: PitcherState::new(profile_of(&team, pitcher), env),
            used,
            is_starter: true,
            cur_bf: 0,
            cur_runs: 0,
            cur_inning_runs: 0,
            cur_inning_bf: 0,
            finished_inning: false,
            entry_lead: 0,
            box_score,
            team,
        }
    }

    /// The AI's next reliever: the closer in a save situation, otherwise the first
    /// unused arm in bullpen order, the closer last.
    fn next_reliever(&self, save_spot: bool) -> Option<usize> {
        let pen = &self.team.bullpen;
        let (&closer, rest) = pen.split_last()?;
        if save_spot && !self.used[closer] {
            return Some(closer);
        }
        rest.iter().copied().find(|&i| !self.used[i]).or((!self.used[closer]).then_some(closer))
    }
}

fn profile_of(team: &Team, idx: usize) -> &PitcherProfile {
    team.players[idx].pitcher.as_ref().expect("player used as a pitcher has a pitching profile")
}

/// A runner leaving with the pitch.
#[derive(Clone, Copy)]
struct Stealing {
    runner: Runner,
    /// Base he leaves: 0 = first, 1 = second.
    from: usize,
    /// A runner on first follows him (double steal).
    with_trail: bool,
    /// Extra time before he gets going (hit-and-run), s.
    late: f64,
}

/// The plate appearance in progress.
#[derive(Clone)]
struct Pa {
    batter_idx: usize,
    pitcher_idx: usize,
    zone: StrikeZone,
    me: Runner,
    box_delay: f64,
    bases_before: u8,
    outs_before: u8,
    /// The first baseman holds a runner on.
    hold_first: bool,
    positions: [Vec3; 9],
    balls: usize,
    strikes: usize,
    pitches: u8,
    /// `None` while the plate appearance is still going on.
    outcome: Option<PaOutcome>,
    intentional: bool,
    rbi: u8,
    ground_ball: bool,
    score_before: u32,
    /// What the AI manager decided to do with this hitter (`None` = not decided yet).
    ai_plan: Option<Approach>,
    /// Side the AI manager wants a bunt on.
    ai_bunt_third: bool,
    ai_walk_checked: bool,
}

/// What a call to [`GameSim::step`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// A pitch was thrown (and everything that followed from it was resolved).
    Pitch,
    /// The batter was walked intentionally; no pitch was thrown.
    IntentionalWalk,
    /// The AI manager made a substitution; the other side may want to answer before the next pitch.
    Change,
    /// The game is over; nothing happened.
    GameOver,
}

/// A game in progress, advanced one pitch at a time. Cloning it copies the whole
/// state, so that a decision can be tried several ways from the same moment.
#[derive(Clone)]
pub struct GameSim {
    env: Arc<MatchEnv>,
    sides: [Side; 2],
    auto: [Auto; 2],
    style: [ManagerStyle; 2],
    offense_orders: OffenseOrders,
    defense_orders: DefenseOrders,
    /// Order for the next pitch only (`set_pitch_order`).
    pitch_order: Option<PitchOrder>,
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
    /// The AI managers have had their say before the next plate appearance.
    defence_checked: bool,
    offence_checked: bool,
    result: Option<GameResult>,
}

/// Play one game from start to finish with the default AI managers. `away` bats first.
pub fn play_game<O: Observer>(env: &MatchEnv, away: &Team, home: &Team, rng: &mut Rng, obs: &mut O) -> GameResult {
    play_game_styled(env, away, home, [ManagerStyle::default(); 2], rng, obs)
}

/// Play one game from start to finish with the given AI managers.
pub fn play_game_styled<O: Observer>(env: &MatchEnv, away: &Team, home: &Team, style: [ManagerStyle; 2], rng: &mut Rng, obs: &mut O) -> GameResult {
    let mut g = GameSim::new(Arc::new(env.clone()), Arc::new(away.clone()), Arc::new(home.clone()));
    g.style = style;
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
            auto: [Auto::ALL; 2],
            style: [ManagerStyle::default(); 2],
            offense_orders: OffenseOrders::default(),
            defense_orders: DefenseOrders::default(),
            pitch_order: None,
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
            defence_checked: false,
            offence_checked: false,
            result: None,
        }
    }

    // ---- read access ----

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

    pub fn env(&self) -> &MatchEnv {
        &self.env
    }

    pub fn team(&self, team: usize) -> &Team {
        &self.sides[team].team
    }

    /// Current batting order of a team.
    pub fn lineup(&self, team: usize) -> &[usize; 9] {
        &self.sides[team].lineup
    }

    /// Slot in the batting order of the next hitter of a team.
    pub fn batting_slot(&self, team: usize) -> usize {
        self.sides[team].order
    }

    /// Current fielders of a team by position.
    pub fn defense(&self, team: usize) -> &[usize; 9] {
        &self.sides[team].defense
    }

    /// True for every player of a team who has been in the game.
    pub fn used(&self, team: usize) -> &[bool] {
        &self.sides[team].used
    }

    pub fn box_score(&self, team: usize) -> &TeamBox {
        &self.sides[team].box_score
    }

    pub fn line_score(&self) -> &[[u8; 2]] {
        &self.line_score
    }

    /// The pitcher of record of a team (the last one who actually pitched).
    pub fn pitcher(&self, team: usize) -> usize {
        self.sides[team].pitcher
    }

    /// Pitches thrown by the current pitcher of a team.
    pub fn pitch_count(&self, team: usize) -> u32 {
        self.sides[team].pitcher_state.pitch_count
    }

    pub fn situation(&self) -> Situation {
        let (bat, fld) = self.sides_now();
        let s = &self.sides[bat];
        let (balls, strikes, batter) = match &self.pa {
            Some(pa) => (pa.balls as u8, pa.strikes as u8, pa.batter_idx),
            None => (0, 0, s.lineup[s.order]),
        };
        // Between half innings the bases and the out count still show the half that just ended.
        let fresh = !self.half_open;
        Situation {
            inning: self.inning,
            top: self.top,
            outs: if fresh { 0 } else { self.outs },
            balls,
            strikes,
            score: self.score,
            bases: if fresh { [None; 3] } else { [self.bases[0].map(|r| r.player), self.bases[1].map(|r| r.player), self.bases[2].map(|r| r.player)] },
            batting: bat,
            fielding: fld,
            batter,
            pitcher: self.sides[fld].pitcher,
            pitch_count: self.sides[fld].pitcher_state.pitch_count,
            in_plate_appearance: self.pa.is_some(),
            over: self.result.is_some(),
        }
    }

    /// Where the fielders will stand for the next pitch.
    pub fn positions(&self) -> [Vec3; 9] {
        let (bat, fld) = self.sides_now();
        let s = &self.sides[bat];
        let batter = &s.team.players[self.pa.as_ref().map_or(s.lineup[s.order], |pa| pa.batter_idx)].batter;
        let hold = self.pa.as_ref().map_or(self.half_open && self.bases[0].is_some() && self.bases[1].is_none(), |pa| pa.hold_first);
        let bunt = self.offense_orders.approach == Some(Approach::SacrificeBunt);
        let set = self.defense_orders.alignment.unwrap_or_else(|| if self.half_open { self.ai_alignment(bat, fld, batter) } else { Alignment::default() });
        alignment(&set, batter.hand.pull_sign(), self.env.params.lean(), hold, bunt, self.env.rules.shift_restricted)
    }

    // ---- orders ----

    /// Choose which decisions the AI manager makes for a team.
    pub fn set_auto(&mut self, team: usize, auto: Auto) {
        self.auto[team] = auto;
    }

    pub fn set_style(&mut self, team: usize, style: ManagerStyle) {
        self.style[team] = style;
    }

    /// Orders for the team at bat (see [`OffenseOrders`]).
    pub fn set_offense_orders(&mut self, orders: OffenseOrders) {
        self.offense_orders = orders;
    }

    /// Orders for the team in the field (see [`DefenseOrders`]).
    /// Which pitch the fielding side throws next and how it aims it; good for that pitch
    /// only. An index outside the arsenal is ignored (the pitcher picks as usual).
    pub fn set_pitch_order(&mut self, order: PitchOrder) {
        self.pitch_order = Some(order);
    }

    /// Withdraw an order for the next pitch that has not been used.
    pub fn clear_pitch_order(&mut self) {
        self.pitch_order = None;
    }

    /// The pitcher of `team` (0 = away, 1 = home) and his state in this game.
    pub fn pitcher_now(&self, team: usize) -> (&PitcherProfile, &PitcherState) {
        let s = &self.sides[team];
        (profile_of(&s.team, s.pitcher), &s.pitcher_state)
    }

    pub fn set_defense_orders(&mut self, orders: DefenseOrders) {
        self.defense_orders = orders;
    }

    /// The fielding team must bring in a pitcher before play can go on (a pinch
    /// hitter batted for the pitcher). `step` picks one itself if nobody does.
    pub fn needs_pitcher(&self, team: usize) -> bool {
        let s = &self.sides[team];
        s.team.players[s.defense[P]].pitcher.is_none()
    }

    /// Can the current pitcher of a team be taken out now?
    pub fn pitcher_removable(&self, team: usize) -> bool {
        let s = &self.sides[team];
        s.cur_bf >= self.env.rules.min_batters_faced as u32 || s.finished_inning || self.needs_pitcher(team)
    }

    /// Make a substitution. Only between plate appearances.
    pub fn substitute<O: Observer>(&mut self, team: usize, sub: Substitution, obs: &mut O) -> Result<(), OrderError> {
        if self.result.is_some() {
            return Err(OrderError::GameOver);
        }
        if self.pa.is_some() {
            return Err(OrderError::PlateAppearanceInProgress);
        }
        let (bat, fld) = self.sides_now();
        let available = |s: &Side, player: usize| player < s.used.len() && !s.used[player];
        match sub {
            Substitution::Pitcher { player } => {
                if team != fld {
                    return Err(OrderError::WrongSide);
                }
                if !available(&self.sides[team], player) {
                    return Err(OrderError::PlayerUnavailable);
                }
                if self.sides[team].team.players[player].pitcher.is_none() {
                    return Err(OrderError::NotAPitcher);
                }
                if !self.pitcher_removable(team) {
                    return Err(OrderError::PitcherMustFaceMoreBatters);
                }
                self.change_pitcher(team, player, obs);
            }
            Substitution::PinchHitter { player } => {
                if team != bat {
                    return Err(OrderError::WrongSide);
                }
                if !available(&self.sides[team], player) {
                    return Err(OrderError::PlayerUnavailable);
                }
                let old = self.sides[team].lineup[self.sides[team].order];
                self.replace_player(team, old, player);
                obs.event(&GameEvent::PinchHitter { team, player, replaced: old });
            }
            Substitution::PinchRunner { base, player } => {
                if team != bat {
                    return Err(OrderError::WrongSide);
                }
                if !available(&self.sides[team], player) {
                    return Err(OrderError::PlayerUnavailable);
                }
                if !(1..=3).contains(&base) || !self.half_open {
                    return Err(OrderError::NothingToReplace);
                }
                let Some(old) = self.bases[base - 1] else { return Err(OrderError::NothingToReplace) };
                self.replace_player(team, old.player, player);
                let b = &self.sides[team].team.players[player].batter;
                self.bases[base - 1] = Some(Runner { player, sprint: b.sprint, accel: b.accel, aggr: b.baserunning, judge: b.judgement, ..old });
                obs.event(&GameEvent::PinchRunner { team, player, replaced: old.player, base: base as u8 });
            }
            Substitution::Fielder { position, player } => {
                if team != fld {
                    return Err(OrderError::WrongSide);
                }
                if !(1..=8).contains(&position) {
                    return Err(OrderError::NothingToReplace);
                }
                if !available(&self.sides[team], player) {
                    return Err(OrderError::PlayerUnavailable);
                }
                let old = self.sides[team].defense[position];
                self.replace_player(team, old, player);
                obs.event(&GameEvent::DefensiveChange { team, player, replaced: old, position });
            }
        }
        Ok(())
    }

    /// `new` takes over the batting-order spot and the defensive position of `old`.
    fn replace_player(&mut self, team: usize, old: usize, new: usize) {
        let s = &mut self.sides[team];
        s.used[new] = true;
        for slot in s.lineup.iter_mut() {
            if *slot == old {
                *slot = new;
            }
        }
        if let Some(pos) = s.defense.iter().position(|&d| d == old) {
            s.defense[pos] = new;
            s.fielders[pos] = s.team.players[new].fielder;
            if pos == C {
                s.catcher = s.team.players[new].catcher;
            }
        }
    }

    fn change_pitcher<O: Observer>(&mut self, team: usize, new_idx: usize, obs: &mut O) {
        let lead = self.score[team] as i32 - self.score[1 - team] as i32;
        let env = Arc::clone(&self.env);
        let s = &mut self.sides[team];
        let old = s.defense[P];
        s.used[new_idx] = true;
        s.pitcher = new_idx;
        s.defense[P] = new_idx;
        s.fielders[P] = s.team.players[new_idx].fielder;
        s.pitcher_state = PitcherState::new(profile_of(&s.team, new_idx), &env);
        s.is_starter = false;
        s.cur_bf = 0;
        s.cur_runs = 0;
        s.cur_inning_runs = 0;
        s.finished_inning = false;
        s.entry_lead = lead;
        s.box_score.pitching[new_idx].appeared = true;
        // Without a DH the new pitcher takes over the old pitcher's spot in the order.
        for slot in s.lineup.iter_mut() {
            if *slot == old {
                *slot = new_idx;
            }
        }
        obs.pitching_change(team, new_idx);
        obs.event(&GameEvent::PitchingChange { team, player: new_idx, replaced: old });
    }

    #[inline]
    fn sides_now(&self) -> (usize, usize) {
        if self.top {
            (0, 1)
        } else {
            (1, 0)
        }
    }

    // ---- the game ----

    /// Advance the game: throw the next pitch and resolve everything that follows
    /// from it, or let an AI manager make a move first.
    pub fn step<O: Observer>(&mut self, rng: &mut Rng, obs: &mut O) -> Step {
        match self.settle(rng, obs) {
            Step::Pitch => {}
            other => return other,
        }
        let (bat, fld) = self.sides_now();
        let thrown = self.pitch(bat, fld, rng, obs);
        // The home side wins the moment it goes ahead in the last inning, even in the
        // middle of a plate appearance (a wild pitch, say).
        let walk_off = !self.top && self.inning >= self.env.rules.innings && self.score[1] > self.score[0];
        if self.pa.as_ref().is_some_and(|pa| pa.outcome.is_some()) || self.outs >= 3 || walk_off {
            self.end_pa(bat, fld, obs);
            if self.outs >= 3 || walk_off {
                self.close_half(bat, fld, obs);
            }
        }
        if thrown {
            Step::Pitch
        } else {
            Step::IntentionalWalk
        }
    }

    /// Bring the game up to the next pitch without throwing it: open the half inning, let
    /// the AI managers make their moves before the plate appearance (each comes back as
    /// `Step::Change`; call again) and start it. `Step::Pitch` means the next `step` throws
    /// (or the batter is walked intentionally), so the pitcher, batter and runners seen now
    /// are those of that pitch. `step` does the same first, drawing the same random numbers.
    pub fn settle<O: Observer>(&mut self, rng: &mut Rng, obs: &mut O) -> Step {
        if self.result.is_some() {
            return Step::GameOver;
        }
        if !self.half_open {
            self.open_half(obs);
        }
        let (bat, fld) = self.sides_now();
        if self.pa.is_none() {
            // The managers move before the plate appearance, the defence first; after
            // each move the other side gets the chance to answer.
            if !self.defence_checked {
                self.defence_checked = true;
                if self.ai_pitching(fld, bat, obs) {
                    return Step::Change;
                }
            }
            if !self.offence_checked {
                self.offence_checked = true;
                if self.ai_bench(bat, fld, rng, obs) {
                    return Step::Change;
                }
            }
            self.begin_pa(bat, fld);
        }
        Step::Pitch
    }

    fn open_half<O: Observer>(&mut self, obs: &mut O) {
        if self.top {
            self.line_score.push([0, 0]);
        }
        let (bat, fld) = self.sides_now();
        self.outs = 0;
        self.bases = [None; 3];
        self.phantom_outs = 0;
        self.sides[fld].cur_inning_runs = 0;
        self.sides[fld].cur_inning_bf = 0;
        if self.env.rules.auto_runner_from.is_some_and(|n| self.inning >= n) {
            let s = &self.sides[bat];
            let idx = s.lineup[(s.order + 8) % 9];
            let b = &s.team.players[idx].batter;
            self.bases[1] = Some(Runner {
                player: idx,
                sprint: b.sprint,
                accel: b.accel,
                aggr: b.baserunning,
                judge: b.judgement,
                pitcher: self.sides[fld].pitcher,
                earned: false,
            });
        }
        self.half_open = true;
        obs.event(&GameEvent::HalfInning { inning: self.inning, top: self.top });
    }

    /// The half inning is over: move on to the next one, or end the game.
    fn close_half<O: Observer>(&mut self, bat: usize, fld: usize, obs: &mut O) {
        self.sides[bat].box_score.left_on_base += self.bases.iter().flatten().count() as u32;
        self.sides[fld].finished_inning = true;
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
            obs.event(&GameEvent::GameOver);
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

    // ---- AI manager ----

    /// Pitching change by the AI manager (or forced, when the pitcher's spot holds a
    /// pinch hitter). Returns true if a new pitcher came in.
    fn ai_pitching<O: Observer>(&mut self, fld: usize, bat: usize, obs: &mut O) -> bool {
        let lead = self.score[fld] as i32 - self.score[bat] as i32;
        let save_spot = self.inning >= self.env.rules.innings && (1..=3).contains(&lead);
        if self.needs_pitcher(fld) {
            let Some(new_idx) = self.sides[fld].next_reliever(save_spot) else { return false };
            self.change_pitcher(fld, new_idx, obs);
            return true;
        }
        if !self.auto[fld].pitching {
            return false;
        }
        let runners = self.bases.iter().flatten().count();
        let inning = self.inning;
        let s = &self.sides[fld];
        let stamina = profile_of(&s.team, s.pitcher).stamina;
        let pc = s.pitcher_state.pitch_count as f64;
        let fresh_inning = self.outs == 0 && s.cur_inning_bf == 0;
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
            return false;
        }
        let Some(new_idx) = s.next_reliever(save_spot) else { return false };
        self.change_pitcher(fld, new_idx, obs);
        true
    }

    /// Pinch hitters and pinch runners by the AI manager. Returns true if it made a move.
    fn ai_bench<O: Observer>(&mut self, bat: usize, fld: usize, rng: &mut Rng, obs: &mut O) -> bool {
        let style = self.style[bat];
        if !self.auto[bat].bench || (style.pinch_hit <= 0.0 && style.pinch_run <= 0.0) {
            return false;
        }
        let s = &self.sides[bat];
        let team = Arc::clone(&s.team);
        let free: Vec<usize> = team.bench.iter().copied().filter(|&i| !s.used[i]).collect();
        if free.is_empty() {
            return false;
        }
        let diff = self.score[bat] as i32 - self.score[fld] as i32;
        let runners = self.bases.iter().flatten().count();
        // Pinch hitter: late, the game within reach, and a clearly better bat on the bench.
        if style.pinch_hit > 0.0 && self.inning >= 7 && (-3..=1).contains(&diff) && (runners > 0 || diff < 0) {
            let throws = profile_of(&self.sides[fld].team, self.sides[fld].pitcher).hand;
            let value = |i: usize| {
                let b = &team.players[i].batter;
                b.quality() + if b.hand != throws { 0.02 } else { -0.02 }
            };
            let due = s.lineup[s.order];
            let best = free.iter().copied().max_by(|&a, &b| value(a).partial_cmp(&value(b)).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(best) = best {
                if value(best) > value(due) + 0.08
                    && rng.chance(style.pinch_hit)
                    && self.substitute(bat, Substitution::PinchHitter { player: best }, obs).is_ok()
                {
                    return true;
                }
            }
        }
        // Pinch runner: the tying or go-ahead run is on base late and he is slow.
        if style.pinch_run > 0.0 && self.inning >= 8 && (-1..=0).contains(&diff) {
            let fastest = free
                .iter()
                .copied()
                .max_by(|&a, &b| team.players[a].batter.sprint.partial_cmp(&team.players[b].batter.sprint).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(fastest) = fastest {
                let speed = team.players[fastest].batter.sprint;
                for base in [2usize, 1] {
                    let Some(r) = self.bases[base - 1] else { continue };
                    if r.sprint < 26.2 * FT && speed > r.sprint + 2.0 * FT && rng.chance(style.pinch_run) {
                        return self.substitute(bat, Substitution::PinchRunner { base, player: fastest }, obs).is_ok();
                    }
                }
            }
        }
        false
    }

    /// What the AI manager has the batter do on this pitch.
    fn ai_approach(&self, pa: &mut Pa, bat: usize, fld: usize, rng: &mut Rng) -> Approach {
        let style = &self.style[bat];
        let batter = &self.sides[bat].team.players[pa.batter_idx].batter;
        let diff = self.score[bat] as i32 - self.score[fld] as i32;
        let (r1, r2, r3) = (self.bases[0].is_some(), self.bases[1].is_some(), self.bases[2].is_some());
        let plan = match pa.ai_plan {
            Some(plan) => plan,
            None => {
                let mut plan = Approach::Normal;
                if style.sac_bunt > 0.0 && self.outs < 2 && (r1 || r2) && !r3 && diff.abs() <= 2 {
                    // Sacrifice: a runner to move up, a close game and a hitter who is not
                    // expected to do damage. How often by base state, outs and inning follows
                    // the bunt attempts of the 2025 Statcast sample (docs/tactics.md).
                    let weak = sigmoid((0.97 - batter.quality()) / 0.05);
                    let close = if diff.abs() <= 1 { 1.0 } else { 0.5 };
                    let base = match (r1, r2) {
                        (true, false) => SAC_RUNNER_ON_FIRST,
                        (false, true) => SAC_RUNNER_ON_SECOND,
                        _ => SAC_FIRST_AND_SECOND,
                    };
                    let outs = if self.outs == 0 { 1.0 } else { SAC_ONE_OUT };
                    let inning = SAC_INNING[inning_group(self.inning)];
                    if rng.chance((style.sac_bunt * weak * close * base * outs * inning * (0.5 + batter.bunt)).min(0.95)) {
                        plan = Approach::SacrificeBunt;
                    }
                } else if style.bunt_for_hit > 0.0 && !r3 && ((!r1 && !r2) || self.outs == 2) {
                    // Bunting for a hit is for quick runners who can handle the bat: mostly with
                    // the bases empty, now and then with two outs and runners on.
                    let speed = ((batter.sprint / FT - 28.0) / 1.5).clamp(0.0, 1.5);
                    let situation = if r1 || r2 {
                        BUNT_HIT_RUNNERS_TWO_OUTS
                    } else if self.outs == 2 {
                        BUNT_HIT_TWO_OUTS
                    } else {
                        1.0
                    };
                    let prob = style.bunt_for_hit * speed * situation * (0.5 + batter.bunt);
                    if prob > 0.0 && rng.chance(prob.min(0.5)) {
                        plan = Approach::BuntForHit;
                    }
                }
                if plan.is_bunt() {
                    // Most bunts go to the third-base side.
                    pa.ai_bunt_third = rng.chance(0.7);
                }
                pa.ai_plan = Some(plan);
                plan
            }
        };
        if plan.is_bunt() {
            // A foul bunt with two strikes is a strikeout, and a bunt for a hit only works as a surprise.
            if pa.strikes >= 2 || (plan == Approach::BuntForHit && pa.pitches >= 1) {
                return Approach::Normal;
            }
            return plan;
        }
        // Hit and run: a runner on first only, fewer than two outs, a count in which the
        // pitcher has to throw a strike, and a hitter who makes contact.
        if style.hit_and_run > 0.0 && r1 && !r2 && !r3 && self.outs < 2 && matches!((pa.balls, pa.strikes), (1, 0) | (2, 0) | (2, 1) | (1, 1) | (3, 1)) {
            let contact = sigmoid((0.019 - batter.barrel_sd) / 0.002);
            if rng.chance((style.hit_and_run * contact).min(0.5)) {
                return Approach::HitAndRun;
            }
        }
        Approach::Normal
    }

    /// Defensive set-up chosen by the AI manager for the situation and the batter.
    fn ai_alignment(&self, bat: usize, fld: usize, batter: &BatterProfile) -> Alignment {
        let mut a = Alignment::default();
        if !self.style[fld].situational_defence {
            return a;
        }
        // Infields shade toward the pull side of hitters who pull the ball, far more often
        // against left-handed batters (2025 Statcast: 46 % of their plate appearances
        // against 8 % for right-handed batters).
        let p = &self.env.params;
        let pull_hitter = match batter.hand {
            Hand::Left => batter.timing_bias < p.shade_left_below,
            Hand::Right => batter.timing_bias < p.shade_right_below,
        };
        if pull_hitter {
            a.shift = Shift::Shade;
        }
        // Before the 2023 restriction, three infielders on the pull side.
        if self.style[fld].full_shift && !self.env.rules.shift_restricted {
            let full = match batter.hand {
                Hand::Left => batter.timing_bias < p.full_shift_left_below,
                Hand::Right => batter.timing_bias < p.full_shift_right_below,
            };
            if full {
                a.shift = Shift::Full;
            }
        }
        let lead = self.score[fld] as i32 - self.score[bat] as i32;
        let innings = self.env.rules.innings;
        let (r1, r3) = (self.bases[0].is_some(), self.bases[2].is_some());
        let last_chance = !self.top && self.inning >= innings && lead == 0;
        if r3 && self.outs < 2 && (last_chance || (self.inning + 1 >= innings && (0..=1).contains(&lead))) {
            // The run on third decides the game: cut it off at the plate.
            a.infield = InfieldDepth::In;
            if last_chance {
                a.outfield = OutfieldDepth::Shallow;
            }
        } else if r1 && self.outs < 2 {
            a.infield = InfieldDepth::DoublePlay;
        }
        if self.inning >= innings && (1..=2).contains(&lead) && a.outfield == OutfieldDepth::Normal {
            a.outfield = OutfieldDepth::Deep;
        }
        a
    }

    /// Does the AI manager walk this hitter on purpose?
    fn ai_walk(&self, pa: &Pa, bat: usize, fld: usize, rng: &mut Rng) -> bool {
        let style = &self.style[fld];
        let [r1, r2, r3] = [self.bases[0].is_some(), self.bases[1].is_some(), self.bases[2].is_some()];
        if style.intentional_walk <= 0.0 || (!r2 && !r3) {
            return false;
        }
        // With first base taken only to face a weaker hitter with two outs, never with the
        // bases loaded.
        if r1 && (self.outs < 2 || (r2 && r3)) {
            return false;
        }
        let lead = self.score[fld] as i32 - self.score[bat] as i32;
        let innings = self.env.rules.innings;
        if !(-1..=2).contains(&lead) {
            return false;
        }
        let s = &self.sides[bat];
        let quality = |i: usize| s.team.players[i].batter.quality();
        let gap = quality(pa.batter_idx) - quality(s.lineup[(s.order + 1) % 9]);
        // Walk a dangerous hitter to get to a weaker one; more readily with two outs
        // (any out ends the inning) or to set up a force when the winning run is on third.
        let winning_run = !self.top && self.inning >= innings && lead == 0 && self.bases[2].is_some() && self.outs < 2;
        let situation = if winning_run {
            1.6
        } else if self.outs == 2 {
            1.0
        } else if self.outs == 1 {
            0.7
        } else {
            0.3
        };
        // How often by inning and base state follows the 2025 Statcast sample (docs/tactics.md):
        // rarely early, mostly with runners on second and third to set up a force.
        let inning = IBB_INNING[inning_group(self.inning)];
        let state = if r1 {
            IBB_FIRST_TAKEN
        } else if r2 && r3 {
            IBB_SECOND_AND_THIRD
        } else {
            1.0
        };
        rng.chance((style.intentional_walk * situation * inning * state * sigmoid((gap - 0.04) / 0.04)).min(0.95))
    }

    /// How much is at stake right now, 0..=1: late innings of a close game, more so
    /// with runners in scoring position.
    fn pressure(&self) -> f64 {
        let late = ((self.inning as f64 - 5.0) / 4.0).clamp(0.0, 1.0);
        let gap = (self.score[0] as f64 - self.score[1] as f64).abs();
        let close = (1.0 - gap / 4.0).max(0.0);
        let scoring = self.bases[1].is_some() as u8 + self.bases[2].is_some() as u8;
        (late * close * (0.6 + 0.2 * scoring as f64)).min(1.0)
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
        let me = Runner {
            player: batter_idx,
            sprint: batter.sprint,
            accel: batter.accel,
            aggr: batter.baserunning,
            judge: batter.judgement,
            pitcher: pitcher_idx,
            earned: true,
        };
        let box_delay = match batter.side_against(profile_of(&self.sides[fld].team, pitcher_idx).hand) {
            Hand::Right => params.box_delay_right,
            Hand::Left => params.box_delay_left,
        };
        let hold_first = self.bases[0].is_some() && self.bases[1].is_none();
        self.pa = Some(Pa {
            batter_idx,
            pitcher_idx,
            zone,
            me,
            box_delay,
            bases_before: self.bases_bits(),
            outs_before: self.outs,
            hold_first,
            positions: [Vec3::ZERO; 9],
            balls: 0,
            strikes: 0,
            pitches: 0,
            outcome: None,
            intentional: false,
            rbi: 0,
            ground_ball: false,
            score_before: self.score[bat],
            ai_plan: None,
            ai_bunt_third: true,
            ai_walk_checked: false,
        });
    }

    /// One pitch of the current plate appearance. Returns false if no pitch was
    /// thrown (intentional walk).
    fn pitch<O: Observer>(&mut self, bat: usize, fld: usize, rng: &mut Rng, obs: &mut O) -> bool {
        let mut pa = self.pa.take().expect("a plate appearance is in progress");
        let thrown = self.pitch_inner(&mut pa, bat, fld, rng, obs);
        self.pa = Some(pa);
        thrown
    }

    fn pitch_inner<O: Observer>(&mut self, pa: &mut Pa, bat: usize, fld: usize, rng: &mut Rng, obs: &mut O) -> bool {
        let env = Arc::clone(&self.env);
        let env = &*env;
        let params = &env.params;
        let bat_team = Arc::clone(&self.sides[bat].team);
        let fld_team = Arc::clone(&self.sides[fld].team);
        let batter_idx = pa.batter_idx;
        let pitcher_idx = pa.pitcher_idx;
        let pprof = profile_of(&fld_team, pitcher_idx);
        // The batter as he is against this pitcher in this situation.
        let pressure = self.pressure();
        let plain = &bat_team.players[batter_idx].batter;
        let adjusted;
        let batter = if plain.switch || pprof.deception != 0.0 || plain.eye_same != 0.0 || (pressure > 0.0 && plain.pressure != 0.0) {
            adjusted = plain.facing(pprof, pressure);
            &adjusted
        } else {
            plain
        };
        let command_scale = 1.0 + pressure * pprof.pressure;
        let zone = pa.zone;
        let catcher = self.sides[fld].catcher;
        let (balls, strikes) = (pa.balls, pa.strikes);

        // ---- orders for this pitch ----
        let defense = self.defense_orders;
        let mut walk = defense.intentional_walk;
        if !walk && self.auto[fld].defense && !pa.ai_walk_checked {
            pa.ai_walk_checked = true;
            walk = self.ai_walk(pa, bat, fld, rng);
        }
        if walk {
            pa.outcome = Some(PaOutcome::Walk);
            pa.intentional = true;
            obs.event(&GameEvent::IntentionalWalk { team: fld, batter: batter_idx });
            return false;
        }
        let offense = self.offense_orders;
        let approach = match offense.approach {
            Some(a) => a,
            None if self.auto[bat].offense => self.ai_approach(pa, bat, fld, rng),
            None => Approach::Normal,
        };
        let toward_third = offense.bunt_toward_third.unwrap_or(pa.ai_bunt_third);
        let set = match defense.alignment {
            Some(a) => a,
            None if self.auto[fld].defense => self.ai_alignment(bat, fld, batter),
            None => Alignment::default(),
        };
        // With a runner on first and second base open the first baseman holds him on,
        // which opens the hole on the right side.
        pa.hold_first = self.bases[0].is_some() && self.bases[1].is_none();
        pa.positions =
            alignment(&set, batter.hand.pull_sign(), env.params.lean(), pa.hold_first, approach == Approach::SacrificeBunt, env.rules.shift_restricted);

        let mut w = [0.0; MAX_PITCHES];
        let n = selection_weights(pprof, batter.hand, balls, strikes, params, &mut w);
        let exp = if params.tendency_strength > 0.0 {
            expectation_adapted(pprof, &self.sides[fld].pitcher_state, &w[..n], params)
        } else {
            expectation(pprof, &self.sides[fld].pitcher_state, &w[..n], params.expect_mix)
        };
        let choice = match self.pitch_order.take() {
            Some(order) if order.idx < n => aim_pitch(pprof, batter.hand, &zone, order.idx, order.intent.value(balls, strikes, params), params, rng),
            _ => choose_pitch(pprof, batter.hand, &zone, balls, strikes, &w[..n], params, rng),
        };
        let thrown = throw_pitch(pprof, &mut self.sides[fld].pitcher_state, &choice, command_scale, env, rng);
        self.sides[fld].pitcher_state.tendency.record(choice.idx, &w[..n], params);
        pa.pitches = pa.pitches.saturating_add(1);

        // ---- runners leaving with the pitch ----
        let hit_and_run = approach == Approach::HitAndRun;
        let stealing = match offense.steal {
            StealOrder::Hold if !hit_and_run => None,
            StealOrder::Go | StealOrder::Hold => self.send_runner(if hit_and_run { params.hit_and_run_jump } else { 0.0 }),
            StealOrder::Auto if hit_and_run => self.send_runner(params.hit_and_run_jump),
            StealOrder::Auto => self.steal_decision(bat, pprof, &catcher, rng),
        };
        if offense.steal == StealOrder::Go {
            self.offense_orders.steal = StealOrder::Auto;
        }

        let f = thrown.flight;
        let zone_distance = zone.signed_distance(f.plate_pos.x, f.plate_pos.z, env.ball.radius);
        let mut ev = PitchEvent {
            inning: self.inning,
            top: self.top,
            balls: balls as u8,
            strikes: strikes as u8,
            outs: self.outs,
            batter: batter_idx,
            pitcher: pitcher_idx,
            pitcher_hand: pprof.hand,
            batter_hand: batter.hand,
            thrown: &thrown,
            zone,
            zone_distance,
            approach,
            swung: true,
            result: PitchResult::Ball,
            bat_speed: None,
            swing: None,
            miss: None,
            batted: None,
            path: None,
            play: None,
            positions: &pa.positions,
            four_seam_speed: pprof.pitches.iter().find(|d| d.kind == PitchKind::FourSeam).map(|d| d.shape.speed),
        };
        let line = &mut self.sides[fld].box_score.pitching[pitcher_idx];
        line.pitches += 1;

        // A foul is a strike unless the batter already has two; a foul bunt with two strikes is strike three.
        let foul = |strikes: &mut usize| {
            if *strikes < 2 || approach.is_bunt() {
                *strikes += 1;
            }
        };
        let sw = respond_with(env, batter, &zone, balls, strikes, &thrown, &exp, approach, toward_third, rng);
        let mut live_ball = false; // pitch not put in play or fouled: steals and wild pitches can happen
        match sw {
            Swing::Take => {
                ev.swung = false;
                if hit_by_pitch(params, batter.hand, f.plate_pos, rng) {
                    ev.result = PitchResult::HitByPitch;
                    obs.pitch(&ev);
                    pa.outcome = Some(PaOutcome::HitByPitch);
                    return true;
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
            Swing::Whiff { shape, miss } => {
                pa.strikes += 1;
                line.strikes += 1;
                ev.result = PitchResult::SwingingStrike;
                ev.bat_speed = Some(shape.bat_speed);
                ev.swing = Some(shape);
                ev.miss = Some(miss);
                live_ball = true;
                obs.pitch(&ev);
            }
            Swing::FoulTip { caught, shape } => {
                line.strikes += 1;
                ev.bat_speed = Some(shape.bat_speed);
                ev.swing = Some(shape);
                if caught {
                    pa.strikes += 1;
                    ev.result = PitchResult::FoulTip;
                } else {
                    foul(&mut pa.strikes);
                    ev.result = PitchResult::Foul;
                }
                obs.pitch(&ev);
            }
            Swing::FoulBack { shape } => {
                line.strikes += 1;
                foul(&mut pa.strikes);
                ev.result = PitchResult::Foul;
                ev.bat_speed = Some(shape.bat_speed);
                ev.swing = Some(shape);
                obs.pitch(&ev);
            }
            Swing::Batted { state, info } => {
                line.strikes += 1;
                ev.swing = Some(info.shape);
                // A ball hit low and well outside the lines is foul without any play on it.
                if info.spray.abs() > OBVIOUS_FOUL_SPRAY && info.launch_angle < OBVIOUS_FOUL_LAUNCH {
                    foul(&mut pa.strikes);
                    ev.result = PitchResult::Foul;
                    ev.bat_speed = Some(info.bat_speed);
                    obs.pitch(&ev);
                } else {
                    let mut flight = env.batted_flight;
                    flight.aero.cd0 = (flight.aero.cd0 + rng.gauss(0.0, params.drag_jitter)).max(0.15);
                    let mut sim = BattedSim::new(&flight, &env.ball, &env.park);
                    sim.dt = BATTED_DT;
                    sim.sample_dt = BATTED_SAMPLE_DT;
                    sim.run(state, &mut self.path);
                    // A ball chopped straight down often comes off the plate, the box or the batter foul.
                    let chopped = !info.bunt && self.path.landing.is_some_and(|l| l.pos.horiz() < params.chop_foul_dist);
                    if chopped && rng.chance(params.chop_foul_prob) {
                        foul(&mut pa.strikes);
                        ev.result = PitchResult::Foul;
                        ev.bat_speed = Some(info.bat_speed);
                        obs.pitch(&ev);
                        return true;
                    }
                    // Runners sent on a hit-and-run are in full stride at contact.
                    let mut running = [0.0; 3];
                    if hit_and_run {
                        if let Some(st) = stealing {
                            let skill = bat_team.players[st.runner.player].batter.steal;
                            let head = (pprof.time_to_plate - steal_jump(skill) - st.late).max(0.0);
                            running[st.from] = head;
                            if st.with_trail {
                                running[0] = head;
                            }
                        }
                    }
                    let box_delay = match approach {
                        Approach::BuntForHit => (pa.box_delay - params.bunt_hit_jump).max(0.0),
                        Approach::SacrificeBunt => pa.box_delay + 0.10,
                        _ => pa.box_delay,
                    };
                    let inp = PlayInput {
                        env,
                        path: &self.path,
                        info: &info,
                        fielders: &self.sides[fld].fielders,
                        positions: &pa.positions,
                        bases: self.bases,
                        batter: pa.me,
                        box_delay,
                        outs: self.outs,
                        running,
                        infield_in: set.infield == InfieldDepth::In,
                    };
                    let res = resolve(&inp, rng);
                    ev.bat_speed = Some(info.bat_speed);
                    ev.batted = Some(&info);
                    ev.path = Some(&self.path);
                    ev.play = Some(&res);
                    if res.kind == PlayKind::Foul {
                        foul(&mut pa.strikes);
                        ev.result = PitchResult::Foul;
                        obs.pitch(&ev);
                    } else {
                        ev.result = PitchResult::InPlay;
                        obs.pitch(&ev);
                        pa.outcome = Some(PaOutcome::Play(res.kind));
                        pa.ground_ball = self.path.landing.is_some_and(|l| l.t < 0.9 && l.pos.horiz() < 45.0);
                        pa.rbi = res.rbi;
                        self.apply_play(bat, fld, batter_idx, pitcher_idx, &res);
                        return true;
                    }
                }
            }
        }

        // Strike three with two outs ends the inning before anything happens on the bases.
        let inning_over = pa.strikes >= 3 && self.outs >= 2;
        if live_ball && !inning_over && self.bases.iter().any(|b| b.is_some()) {
            if let Some(st) = stealing {
                self.resolve_steal(bat, fld, st, pprof, &catcher, &thrown, rng, obs);
            } else if is_wild(f.plate_pos) && rng.chance((params.wild_pitch_prob * catcher.blocking).min(0.9)) {
                self.sides[fld].box_score.pitching[pitcher_idx].wp += 1;
                self.advance_all_one(bat, fld);
                obs.wild_pitch();
                obs.event(&GameEvent::WildPitch);
            }
        }
        if pa.strikes >= 3 {
            pa.outcome = Some(PaOutcome::Strikeout);
        } else if pa.balls >= 4 {
            pa.outcome = Some(PaOutcome::Walk);
        }
        true
    }

    /// Book-keeping at the end of a plate appearance.
    fn end_pa<O: Observer>(&mut self, bat: usize, fld: usize, obs: &mut O) {
        let pa = self.pa.take().expect("a plate appearance is in progress");
        self.offense_orders = OffenseOrders::default();
        self.defense_orders = DefenseOrders::default();
        self.defence_checked = false;
        self.offence_checked = false;
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
                    if pa.intentional {
                        bl.ibb += 1;
                        pl.ibb += 1;
                    }
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
            intentional: pa.intentional,
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
            match res.kind {
                PlayKind::SacFly => bl.sf += 1,
                PlayKind::SacBunt => bl.sh += 1,
                _ => bl.ab += 1,
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

    /// The lead runner who can steal, sent by the bench.
    fn send_runner(&self, late: f64) -> Option<Stealing> {
        match self.bases {
            [trail, Some(r), None] => Some(Stealing { runner: r, from: 1, with_trail: trail.is_some(), late }),
            [Some(r), None, _] => Some(Stealing { runner: r, from: 0, with_trail: false, late }),
            _ => None,
        }
    }

    /// Does a runner decide to go on this pitch?
    fn steal_decision(&self, bat: usize, pitcher: &PitcherProfile, catcher: &CatcherProfile, rng: &mut Rng) -> Option<Stealing> {
        let p = &self.env.params;
        let team = &self.sides[bat].team;
        if let (Some(runner), None) = (self.bases[0], self.bases[1]) {
            let skill = team.players[runner.player].batter.steal;
            let margin = steal_defence_time(p, pitcher, catcher, 0) - steal_run_time(p, &runner, skill, 0);
            let prob = p.steal_rate * skill * sigmoid((margin - p.steal_margin) / 0.06);
            return rng.chance(prob).then_some(Stealing { runner, from: 0, with_trail: false, late: 0.0 });
        }
        if p.steal_third_rate > 0.0 {
            if let [None, Some(runner), None] = self.bases {
                let skill = team.players[runner.player].batter.steal;
                let margin = steal_defence_time(p, pitcher, catcher, 1) - steal_run_time(p, &runner, skill, 1);
                let prob = p.steal_rate * p.steal_third_rate * skill * sigmoid((margin - p.steal_margin - p.steal_third_margin) / 0.06);
                return rng.chance(prob).then_some(Stealing { runner, from: 1, with_trail: false, late: 0.0 });
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_steal<O: Observer>(
        &mut self,
        bat: usize,
        fld: usize,
        st: Stealing,
        pitcher: &PitcherProfile,
        catcher: &CatcherProfile,
        thrown: &Thrown,
        rng: &mut Rng,
        obs: &mut O,
    ) {
        let env = Arc::clone(&self.env);
        let p = &env.params;
        let runner = st.runner;
        let skill = self.sides[bat].team.players[runner.player].batter.steal;
        // Slow pitches and pitches in the dirt give the runner more time.
        let v = thrown.flight.plate_vel.norm();
        let pitch_penalty = (p.ref_plate_speed - v).max(0.0) * 0.012 + if thrown.flight.plate_pos.z < 0.35 { 0.10 } else { 0.0 };
        let defence = steal_defence_time(p, pitcher, catcher, st.from) + pitch_penalty + rng.gauss(0.0, 0.07);
        let run = steal_run_time(p, &runner, skill, st.from) + st.late + rng.gauss(0.0, 0.06);
        self.bases[st.from] = None;
        let safe = run < defence;
        if safe {
            self.bases[st.from + 1] = Some(runner);
            self.sides[bat].box_score.batting[runner.player].sb += 1;
        } else {
            self.outs += 1;
            self.phantom_outs += 1;
            self.sides[bat].box_score.batting[runner.player].cs += 1;
            let pidx = self.sides[fld].pitcher;
            self.sides[fld].box_score.pitching[pidx].outs += 1;
        }
        // On a double steal the throw goes to third and the trailing runner takes second.
        if st.with_trail && self.outs < 3 {
            if let Some(trail) = self.bases[0].take() {
                self.bases[1] = Some(trail);
                self.sides[bat].box_score.batting[trail.player].sb += 1;
            }
        }
        obs.steal(safe);
        obs.event(&GameEvent::Steal { team: bat, runner: runner.player, to_base: st.from as u8 + 2, safe });
    }
}

/// Time from the pitcher's first move until the tag is applied at the base
/// (`from`: 0 = steal of second, 1 = steal of third).
fn steal_defence_time(p: &ModelParams, pitcher: &PitcherProfile, catcher: &CatcherProfile, from: usize) -> f64 {
    pitcher.time_to_plate + catcher.pop_time * if from == 1 { p.pop_third_factor } else { 1.0 } + p.tag_time
}

/// Reaction of a base stealer to the pitcher's first move, s.
fn steal_jump(skill: f64) -> f64 {
    0.24 - 0.08 * skill
}

fn steal_run_time(p: &ModelParams, runner: &Runner, skill: f64, from: usize) -> f64 {
    // Reaction to the pitcher's first move, then a sprint from the lead to a slide
    // that reaches the bag about 1.8 m early.
    // Nobody holds a runner on second, so he can stretch his lead as the pitcher comes set.
    let lead = p.lead[from] + 0.3 + 0.6 * skill + if from == 1 { 1.2 } else { 0.0 };
    steal_jump(skill) + move_time(BASE_PATH - lead - 1.8, runner.accel * 1.1, runner.sprint * 0.98)
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
    fn ai_managers_bunt_with_one_out_and_walk_hitters_before_the_seventh() {
        // The 2025 Statcast sample has a fifth of the sacrifice attempts with one out and
        // intentional walks from the fourth inning on (docs/tactics.md 7).
        struct Seen {
            inning: u8,
            one_out_bunts: u32,
            early_walks: u32,
        }
        impl Observer for Seen {
            fn pitch(&mut self, e: &PitchEvent) {
                if e.approach == Approach::SacrificeBunt && e.outs == 1 {
                    self.one_out_bunts += 1;
                }
            }
            fn event(&mut self, e: &GameEvent) {
                match *e {
                    GameEvent::HalfInning { inning, .. } => self.inning = inning,
                    GameEvent::IntentionalWalk { .. } if self.inning < 7 => self.early_walks += 1,
                    _ => {}
                }
            }
        }
        let env = MatchEnv::neutral_mlb();
        let (a, h) = clubs(9);
        let mut rng = Rng::new(11);
        let mut seen = Seen { inning: 0, one_out_bunts: 0, early_walks: 0 };
        for g in 0..300 {
            play_game_styled(&env, &a.team_for_game(g % 5, true), &h.team_for_game((g + 2) % 5, true), [ManagerStyle::mlb(); 2], &mut rng, &mut seen);
        }
        assert!(seen.one_out_bunts > 0 && seen.early_walks > 0, "one-out bunt pitches {}, walks before the seventh {}", seen.one_out_bunts, seen.early_walks);
    }

    #[test]
    fn full_shift_only_for_managers_who_use_it_where_the_rules_allow_it() {
        // Pitches to left-handed batters, and those with the shortstop right of second base.
        struct Shifts(u32, u32);
        impl Observer for Shifts {
            fn pitch(&mut self, e: &PitchEvent) {
                if e.batter_hand == Hand::Left {
                    self.1 += 1;
                    if e.positions[crate::fielding::SS].x > 0.0 {
                        self.0 += 1;
                    }
                }
            }
        }
        let (a, h) = clubs(5);
        let mut free = MatchEnv::neutral_mlb();
        free.rules.shift_restricted = false;
        let run = |env: &MatchEnv, style: ManagerStyle| {
            let mut o = Shifts(0, 0);
            let mut rng = Rng::new(3);
            for _ in 0..6 {
                play_game_styled(env, &a.team_for_game(0, true), &h.team_for_game(0, true), [style; 2], &mut rng, &mut o);
            }
            o
        };
        let shifted = run(&free, ManagerStyle::mlb_2022());
        assert!(shifted.1 > 100 && shifted.0 > 0, "{} of {}", shifted.0, shifted.1);
        assert_eq!(run(&free, ManagerStyle::mlb()).0, 0);
        assert_eq!(run(&MatchEnv::neutral_mlb(), ManagerStyle::mlb_2022()).0, 0);
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
                // Plate appearances are at bats plus walks, hit batters and sacrifices.
                for b in &bat.batting {
                    assert_eq!(b.pa, b.ab + b.bb + b.hbp + b.sf + b.sh);
                    assert!(b.ibb <= b.bb);
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

    fn sim(seed: u64) -> GameSim {
        let (a, h) = clubs(seed);
        GameSim::new(Arc::new(MatchEnv::neutral_mlb()), Arc::new(a.team_for_game(0, true)), Arc::new(h.team_for_game(0, true)))
    }

    /// Put a runner from the batting order on a base (test set-up).
    fn place_runner(g: &mut GameSim, base: usize, slot: usize) -> usize {
        let idx = g.sides[0].lineup[slot];
        let b = &g.sides[0].team.players[idx].batter;
        g.bases[base - 1] =
            Some(Runner { player: idx, sprint: b.sprint, accel: b.accel, aggr: b.baserunning, judge: b.judgement, pitcher: g.sides[1].pitcher, earned: true });
        idx
    }

    #[test]
    fn stepping_a_game_matches_playing_it_through() {
        let env = MatchEnv::neutral_mlb();
        let (a, h) = clubs(5);
        let (away, home) = (a.team_for_game(0, true), h.team_for_game(1, true));
        let whole = play_game(&env, &away, &home, &mut Rng::new(42), &mut NoObserver);
        let mut g = GameSim::new(Arc::new(env), Arc::new(away), Arc::new(home));
        let mut rng = Rng::new(42);
        let mut steps = 0;
        while !g.is_over() {
            let before = g.situation();
            assert!(!before.over);
            g.step(&mut rng, &mut NoObserver);
            steps += 1;
            assert!(steps < 2000);
        }
        assert_eq!(g.step(&mut rng, &mut NoObserver), Step::GameOver);
        let res = g.result().unwrap();
        assert_eq!(res.score, whole.score);
        assert_eq!(res.line_score, whole.line_score);
        assert_eq!(res.boxes[0].batting, whole.boxes[0].batting);
        assert_eq!(g.situation().score, whole.score);
    }

    #[test]
    fn pitch_orders_set_the_next_pitch_only() {
        use crate::pitching::{Intent, PitchOrder};
        // Where an intent puts the pitch is tested with the aim itself (`pitching`).
        struct Thrown(Vec<(bool, usize, f64)>);
        impl Observer for Thrown {
            fn pitch(&mut self, e: &PitchEvent) {
                self.0.push((e.top, e.thrown.idx, e.zone_distance));
            }
        }
        let mut g = sim(6);
        let mut rng = Rng::new(4);
        // The home starter stays in, so the ordered index is always in his arsenal.
        g.set_auto(1, Auto::NONE);
        let last = g.pitcher_now(1).0.pitches.len() - 1;
        let mut seen = Thrown(Vec::new());
        let mut ordered = 0;
        while ordered < 100 && !g.situation().over {
            let s = g.situation();
            let mine = s.fielding == 1;
            if mine {
                g.set_pitch_order(PitchOrder { idx: last, intent: if ordered % 2 == 0 { Intent::Chase } else { Intent::Attack } });
            }
            let before = seen.0.len();
            if g.step(&mut rng, &mut seen) == Step::Pitch && mine && seen.0.len() > before {
                let (top, idx, _) = seen.0[before];
                assert!(top && idx == last, "ordered pitch {last}, thrown {idx}");
                // The order was used up by this pitch.
                assert!(g.pitch_order.is_none());
                ordered += 1;
            }
        }
        assert_eq!(ordered, 100);
        // Without an order the away pitcher keeps mixing his pitches.
        let away: std::collections::HashSet<usize> = seen.0.iter().filter(|p| !p.0).map(|p| p.1).collect();
        assert!(away.len() > 1);
    }

    #[test]
    fn intentional_walk_on_order() {
        let mut g = sim(3);
        let mut rng = Rng::new(1);
        g.set_auto(1, Auto::NONE);
        g.set_defense_orders(DefenseOrders { intentional_walk: true, ..Default::default() });
        assert_eq!(g.step(&mut rng, &mut NoObserver), Step::IntentionalWalk);
        let s = g.situation();
        let leadoff = g.lineup(0)[0];
        assert_eq!(s.bases, [Some(leadoff), None, None]);
        assert_eq!(g.pitch_count(1), 0);
        let line = g.box_score(0).batting[leadoff];
        assert_eq!((line.pa, line.ab, line.bb, line.ibb), (1, 0, 1, 1));
        // The order does not carry over to the next hitter.
        assert_eq!(g.step(&mut rng, &mut NoObserver), Step::Pitch);
    }

    #[test]
    fn substitutions_follow_the_rules() {
        let mut g = sim(4);
        let mut rng = Rng::new(2);
        let obs = &mut NoObserver;
        let bench = g.team(0).bench.clone();
        let pen = g.team(1).bullpen.clone();
        assert!(bench.len() >= 3 && pen.len() >= 3);
        // The starter has not faced anybody yet (three-batter minimum under MLB rules).
        assert_eq!(g.substitute(1, Substitution::Pitcher { player: pen[0] }, obs), Err(OrderError::PitcherMustFaceMoreBatters));
        // The fielding side cannot send up a pinch hitter, nor the batting side a pitcher.
        assert_eq!(g.substitute(1, Substitution::PinchHitter { player: g.team(1).bench[0] }, obs), Err(OrderError::WrongSide));
        assert_eq!(g.substitute(0, Substitution::Pitcher { player: g.team(0).bullpen[0] }, obs), Err(OrderError::WrongSide));
        // A position player cannot be brought in to pitch.
        assert_eq!(g.substitute(1, Substitution::Pitcher { player: g.team(1).bench[0] }, obs), Err(OrderError::NotAPitcher));
        // Pinch hitter for the lead-off man: he takes the spot in the order and the position.
        let leadoff = g.lineup(0)[0];
        let pos = g.defense(0).iter().position(|&d| d == leadoff);
        assert_eq!(g.substitute(0, Substitution::PinchHitter { player: bench[0] }, obs), Ok(()));
        assert_eq!(g.lineup(0)[0], bench[0]);
        if let Some(pos) = pos {
            assert_eq!(g.defense(0)[pos], bench[0]);
        }
        assert_eq!(g.situation().batter, bench[0]);
        // Nobody comes back, and nobody enters twice.
        assert_eq!(g.substitute(0, Substitution::PinchHitter { player: leadoff }, obs), Err(OrderError::PlayerUnavailable));
        assert_eq!(g.substitute(0, Substitution::PinchHitter { player: bench[0] }, obs), Err(OrderError::PlayerUnavailable));
        // No substitutions in the middle of a plate appearance.
        g.set_auto(0, Auto::NONE);
        g.set_auto(1, Auto::NONE);
        g.set_offense_orders(OffenseOrders { approach: Some(Approach::Take), ..Default::default() });
        g.step(&mut rng, obs);
        assert!(g.situation().in_plate_appearance);
        assert_eq!(g.substitute(0, Substitution::PinchHitter { player: bench[1] }, obs), Err(OrderError::PlateAppearanceInProgress));
        // Play until the starter has faced three batters; then he can be replaced.
        while g.sides[1].cur_bf < 3 || g.situation().in_plate_appearance {
            g.step(&mut rng, obs);
        }
        if g.situation().fielding == 1 {
            let old = g.situation().pitcher;
            assert_eq!(g.substitute(1, Substitution::Pitcher { player: pen[1] }, obs), Ok(()));
            assert_eq!(g.situation().pitcher, pen[1]);
            assert_eq!(g.defense(1)[P], pen[1]);
            assert!(g.used(1)[old] && g.used(1)[pen[1]]);
            assert_eq!(g.pitch_count(1), 0);
            // The new man has to face three batters too.
            assert_eq!(g.substitute(1, Substitution::Pitcher { player: pen[2] }, obs), Err(OrderError::PitcherMustFaceMoreBatters));
        }
    }

    #[test]
    fn pinch_runner_takes_the_base() {
        let mut g = sim(6);
        let obs = &mut NoObserver;
        g.open_half(obs);
        let slow = place_runner(&mut g, 1, 4);
        let bench = g.team(0).bench.clone();
        assert_eq!(g.substitute(0, Substitution::PinchRunner { base: 2, player: bench[0] }, obs), Err(OrderError::NothingToReplace));
        assert_eq!(g.substitute(0, Substitution::PinchRunner { base: 1, player: bench[2] }, obs), Ok(()));
        assert_eq!(g.situation().bases[0], Some(bench[2]));
        assert!(!g.lineup(0).contains(&slow) && g.lineup(0).contains(&bench[2]));
    }

    #[test]
    fn sacrifice_bunt_moves_the_runner_up() {
        let (mut moved, mut sacrifices, mut trials) = (0, 0, 0);
        for seed in 0..300u64 {
            let mut g = sim(7);
            let obs = &mut NoObserver;
            let mut rng = Rng::new(1000 + seed);
            g.set_auto(0, Auto::NONE);
            g.open_half(obs);
            let runner = place_runner(&mut g, 1, 8);
            let batter = g.situation().batter;
            loop {
                g.set_offense_orders(OffenseOrders { approach: Some(Approach::SacrificeBunt), ..Default::default() });
                g.step(&mut rng, obs);
                if !g.situation().in_plate_appearance {
                    break;
                }
            }
            trials += 1;
            let s = g.situation();
            if s.bases[1] == Some(runner) || s.bases[2] == Some(runner) {
                moved += 1;
            }
            let line = g.box_score(0).batting[batter];
            sacrifices += line.sh as u32;
            // A sacrifice is not an at bat.
            assert_eq!(line.pa, line.ab + line.bb + line.hbp + line.sh);
        }
        let (moved, sacrifices) = (moved as f64 / trials as f64, sacrifices as f64 / trials as f64);
        assert!(moved > 0.55, "runner advanced in {moved} of the bunt attempts");
        assert!(sacrifices > 0.35 && sacrifices < moved + 1e-9, "sacrifices {sacrifices}");
    }

    #[test]
    fn steal_and_hold_orders_are_obeyed() {
        let (mut attempts, mut held) = (0, 0);
        for seed in 0..200u64 {
            for go in [true, false] {
                let mut g = sim(8);
                let obs = &mut NoObserver;
                let mut rng = Rng::new(50 + seed);
                g.set_auto(0, Auto::NONE);
                g.open_half(obs);
                let runner = place_runner(&mut g, 1, 0);
                g.sides[0].order = 3;
                let order = if go { StealOrder::Go } else { StealOrder::Hold };
                g.set_offense_orders(OffenseOrders { approach: Some(Approach::Take), steal: order, ..Default::default() });
                g.step(&mut rng, obs);
                let line = g.box_score(0).batting[runner];
                if go {
                    attempts += (line.sb + line.cs) as u32;
                } else {
                    assert_eq!(line.sb + line.cs, 0);
                    held += 1;
                }
            }
        }
        // The batter takes, so every pitch that does not hit him is a live ball.
        assert!(attempts > 190, "steal attempts on order: {attempts} of 200");
        assert_eq!(held, 200);
    }

    #[test]
    fn full_games_with_every_tactic_keep_the_books_straight() {
        // A manager who calls for everything all the time stresses the bookkeeping.
        let env = MatchEnv::neutral_mlb();
        let wild = ManagerStyle {
            sac_bunt: 3.0,
            bunt_for_hit: 1.0,
            hit_and_run: 2.0,
            intentional_walk: 2.0,
            pinch_hit: 1.0,
            pinch_run: 1.0,
            situational_defence: true,
            full_shift: true,
        };
        let (a, h) = clubs(12);
        let mut rng = Rng::new(77);
        let (mut sh, mut ibb, mut subs) = (0u32, 0u32, 0u32);
        struct Subs(u32);
        impl Observer for Subs {
            fn event(&mut self, e: &GameEvent) {
                if matches!(e, GameEvent::PinchHitter { .. } | GameEvent::PinchRunner { .. }) {
                    self.0 += 1;
                }
            }
        }
        for game in 0..60 {
            let mut o = Subs(0);
            let res = play_game_styled(&env, &a.team_for_game(game, true), &h.team_for_game(game, true), [wild; 2], &mut rng, &mut o);
            subs += o.0;
            for t in 0..2 {
                let outs: u32 = res.boxes[1 - t].pitching.iter().map(|p| p.outs as u32).sum();
                assert!(outs >= 24 && outs <= 3 * res.innings as u32);
                for b in &res.boxes[t].batting {
                    assert_eq!(b.pa, b.ab + b.bb + b.hbp + b.sf + b.sh);
                    sh += b.sh as u32;
                    ibb += b.ibb as u32;
                }
                let runs: u32 = res.boxes[t].batting.iter().map(|b| b.r as u32).sum();
                assert_eq!(runs, res.score[t]);
                // Nobody bats in two spots and everyone in the final order has appeared.
                let bf: u32 = res.boxes[1 - t].pitching.iter().map(|p| p.bf as u32).sum();
                let pa: u32 = res.boxes[t].batting.iter().map(|b| b.pa as u32).sum();
                assert_eq!(bf, pa);
            }
        }
        assert!(sh > 20 && ibb > 20 && subs > 20, "sh {sh} ibb {ibb} substitutions {subs}");
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
