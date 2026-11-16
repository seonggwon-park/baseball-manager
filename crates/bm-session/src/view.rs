//! Serialisable views of a match: what the interface receives.
//!
//! Lengths are metres, times seconds, in the engine's coordinates (origin at the
//! back tip of home plate, x toward first base, y toward the pitcher, z up).
//! Players are referred to by their index in their team's `players` list.

use crate::names::{ClubInfo, Localized};
use serde::Serialize;

/// Round to millimetres / milliseconds to keep the payload small.
pub fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// One attribute of a player as shown on the screen.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AttrView {
    /// Attribute key (`contact`, `velocity` ...); the interface translates it.
    pub key: &'static str,
    /// batting, running, fielding, catching, pitching, mental or physical.
    pub group: &'static str,
    /// Rating on the 1-20 scale and on the 20-80 scouting scale.
    pub r20: u8,
    pub r80: u8,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PitchTypeView {
    /// FF, SI, FC, SL, ST, CU, CH, FS.
    pub kind: &'static str,
    pub speed_kmh: f64,
    /// Share of his pitches, 0..1 (average of both batter hands).
    pub usage: f64,
    /// Grade of the pitch on the two scales.
    pub r20: u8,
    pub r80: u8,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PlayerView {
    pub idx: usize,
    pub name: Localized,
    pub number: u8,
    /// P, C, 1B, 2B, 3B, SS, LF, CF, RF or DH.
    pub position: &'static str,
    /// R, L or S (switch hitter).
    pub bats: &'static str,
    /// R or L.
    pub throws: &'static str,
    pub height_cm: u16,
    pub weight_kg: u16,
    /// Every attribute, in display order.
    pub attrs: Vec<AttrView>,
    /// Pitches of a pitcher, most used first (empty for position players).
    pub repertoire: Vec<PitchTypeView>,
    /// Average speed of his fastest pitch, km/h.
    pub top_speed_kmh: Option<f64>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TeamView {
    pub club: ClubInfo,
    pub players: Vec<PlayerView>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ParkView {
    /// Name of the home club's ballpark.
    pub name: Localized,
    /// Elevation above sea level, m.
    pub elevation_m: f64,
    pub turf: bool,
    /// Outfield fence: `[bearing rad, distance m, height m]` from the left-field line to the right-field line.
    pub fence: Vec<[f64; 3]>,
    pub backstop: f64,
    pub foul_width_near: f64,
    pub foul_width_far: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RulesView {
    /// kbo, mlb or npb.
    pub id: &'static str,
    pub innings: u8,
    pub max_innings: Option<u8>,
    pub auto_runner_from: Option<u8>,
    pub dh: bool,
    pub automated_zone: bool,
    pub min_batters_faced: u8,
    pub shift_restricted: bool,
}

/// Everything about a match that does not change while it is played.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MatchView {
    pub league: Localized,
    pub teams: [TeamView; 2],
    pub park: ParkView,
    pub rules: RulesView,
    /// Team the user manages (0 = away, 1 = home), or `None` when only watching.
    pub user_team: Option<usize>,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct SituationView {
    pub inning: u8,
    pub top: bool,
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    pub score: [u32; 2],
    pub bases: [Option<usize>; 3],
    pub batting: usize,
    pub fielding: usize,
    pub batter: usize,
    pub on_deck: usize,
    pub pitcher: usize,
    pub pitch_count: u32,
    pub in_plate_appearance: bool,
    pub over: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct BatLineView {
    pub player: usize,
    pub pa: u16,
    pub ab: u16,
    pub r: u16,
    pub h: u16,
    pub doubles: u16,
    pub triples: u16,
    pub hr: u16,
    pub rbi: u16,
    pub bb: u16,
    pub so: u16,
    pub sb: u16,
    pub sh: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PitchLineView {
    pub player: usize,
    /// Outs recorded (innings pitched = outs / 3).
    pub outs: u16,
    pub h: u16,
    pub r: u16,
    pub er: u16,
    pub bb: u16,
    pub so: u16,
    pub hr: u16,
    pub pitches: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct BoxView {
    /// Everyone who has batted or is in the order, in batting order, substitutes after the man they replaced.
    pub batting: Vec<BatLineView>,
    /// Pitchers in order of appearance.
    pub pitching: Vec<PitchLineView>,
    pub hits: u32,
    pub errors: u32,
    pub left_on_base: u32,
}

/// What the user may do right now.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Capabilities {
    /// The user's team is batting / fielding.
    pub offense: bool,
    pub defense: bool,
    /// Substitutions are possible (between plate appearances).
    pub substitute: bool,
    /// The user's pitcher may be replaced now.
    pub pitcher_removable: bool,
    /// The user's team has to bring in a pitcher before the next pitch.
    pub needs_pitcher: bool,
    /// Bench players and relievers of the user's team who have not been used.
    pub bench: Vec<usize>,
    pub bullpen: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ResultView {
    pub score: [u32; 2],
    pub innings: u8,
    /// `[team, player]` of the winning, losing and saving pitcher.
    pub win: Option<[usize; 2]>,
    pub loss: Option<[usize; 2]>,
    pub save: Option<[usize; 2]>,
}

/// The state of the match between two pitches.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MatchState {
    pub situation: SituationView,
    pub line_score: Vec<[u8; 2]>,
    pub lineups: [[usize; 9]; 2],
    pub batting_slot: [usize; 2],
    pub defense: [[usize; 9]; 2],
    pub boxes: [BoxView; 2],
    /// Where the nine fielders stand for the next pitch: `[x, y]`.
    pub positions: Vec<[f64; 2]>,
    pub can: Capabilities,
    pub result: Option<ResultView>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ZoneView {
    pub top: f64,
    pub bottom: f64,
    pub half_width: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct BattedView {
    pub exit_speed_kmh: f64,
    pub launch_angle_deg: f64,
    pub spray_deg: f64,
    /// Horizontal distance of the first landing point (or of the last point for a ball that left the park), m.
    pub distance: f64,
    pub hang_time: f64,
    pub apex: f64,
    pub bunt: bool,
    /// stopped, over_fence, out_of_play or time_limit.
    pub end: &'static str,
    /// Samples of the ball's path: `[t, x, y, z]`.
    pub path: Vec<[f64; 4]>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ThrowView {
    /// Position index of the thrower and of the fielder covering the base.
    pub by: usize,
    pub to: usize,
    pub from: [f64; 2],
    /// Base thrown to: 1..=3, 4 = home.
    pub base: u8,
    pub t_release: f64,
    pub t_arrive: f64,
    pub carried: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RunView {
    pub player: usize,
    pub from: u8,
    pub to: u8,
    pub t_start: f64,
    pub t_arrive: f64,
    pub out: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PlayView {
    /// single, double, triple, home_run, ground_rule_double, ground_out, fly_out, line_out, pop_out, foul_out,
    /// double_play, fielders_choice, error, sac_fly, sac_bunt or foul.
    pub kind: &'static str,
    pub outs: u8,
    /// Position index of the fielder who made the first play, and where.
    pub fielder: usize,
    pub field_pos: [f64; 2],
    pub fielded_t: f64,
    pub caught: bool,
    pub rbi: u8,
    pub scored: Vec<usize>,
    pub bases_after: [Option<usize>; 3],
    pub gidp: bool,
    pub throws: Vec<ThrowView>,
    pub runs: Vec<RunView>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PitchView {
    pub inning: u8,
    pub top: bool,
    /// Count and outs before the pitch.
    pub balls: u8,
    pub strikes: u8,
    pub outs: u8,
    pub batter: usize,
    pub pitcher: usize,
    pub batter_hand: &'static str,
    pub pitcher_hand: &'static str,
    /// FF, SI, FC, SL, ST, CU, CH, FS.
    pub kind: &'static str,
    pub speed_kmh: f64,
    pub spin_rpm: f64,
    pub release: [f64; 3],
    /// Position and velocity crossing the front of the plate, and the flight time.
    pub plate: [f64; 3],
    pub plate_vel: [f64; 3],
    pub flight_time: f64,
    pub zone: ZoneView,
    /// Signed distance outside the strike zone, m (negative = inside).
    pub zone_distance: f64,
    /// normal, sac_bunt, bunt_for_hit, hit_and_run or take.
    pub approach: &'static str,
    pub swung: bool,
    /// ball, called_strike, swinging_strike, foul, foul_tip, in_play or hit_by_pitch.
    pub result: &'static str,
    /// early, late, off_the_end, jammed, over or under.
    pub miss: Option<&'static str>,
    pub bat_speed_kmh: Option<f64>,
    pub batted: Option<BattedView>,
    pub play: Option<PlayView>,
    /// Fielder positions at the pitch: `[x, y]`.
    pub positions: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PlateAppearanceView {
    pub inning: u8,
    pub top: bool,
    pub batter: usize,
    pub pitcher: usize,
    /// strikeout, walk, intentional_walk, hit_by_pitch, interrupted or a play kind (see [`PlayView::kind`]).
    pub outcome: &'static str,
    pub rbi: u8,
    pub runs: u8,
    pub outs_made: u8,
    pub pitches: u8,
    pub outs_after: u8,
    pub score_after: [u32; 2],
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventView {
    HalfInning {
        inning: u8,
        top: bool,
    },
    Pitch(Box<PitchView>),
    PlateAppearance(PlateAppearanceView),
    Steal {
        team: usize,
        runner: usize,
        to_base: u8,
        safe: bool,
    },
    WildPitch,
    IntentionalWalk {
        team: usize,
        batter: usize,
    },
    Substitution {
        /// pitcher, pinch_hitter, pinch_runner or fielder.
        kind: &'static str,
        team: usize,
        player: usize,
        replaced: usize,
        base: Option<u8>,
        position: Option<usize>,
    },
    GameOver,
}

/// Result of advancing the match.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AdvanceView {
    pub events: Vec<EventView>,
    pub state: MatchState,
}
