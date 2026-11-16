// Views sent by the engine (see crates/bm-session/src/view.rs).
// Lengths are metres, times seconds; origin at the back tip of home plate,
// x toward first base, y toward the pitcher.

export interface Localized {
  ko: string;
  en: string;
}

export interface ClubInfo {
  id: string;
  city: Localized;
  name: Localized;
  abbr: string;
  colors: [string, string];
}

export type AttrGroup = "batting" | "running" | "fielding" | "catching" | "pitching" | "mental" | "physical";

/** One attribute of a player: its rating on the 1-20 scale and on the 20-80 scouting scale. */
export interface AttrView {
  key: string;
  group: AttrGroup;
  r20: number;
  r80: number;
}

export interface PitchTypeView {
  kind: PitchKind;
  speed_kmh: number;
  usage: number;
  r20: number;
  r80: number;
}

/** How ratings are displayed. */
export type RatingScale = "20" | "80";

export type PitchKind = "FF" | "SI" | "FC" | "SL" | "ST" | "CU" | "CH" | "FS";
export type PositionCode = "P" | "C" | "1B" | "2B" | "3B" | "SS" | "LF" | "CF" | "RF" | "DH";

export interface PlayerView {
  idx: number;
  name: Localized;
  number: number;
  position: PositionCode;
  /** S = switch hitter. */
  bats: "R" | "L" | "S";
  throws: "R" | "L";
  height_cm: number;
  weight_kg: number;
  attrs: AttrView[];
  /** Pitches of a pitcher, most used first; empty for position players. */
  repertoire: PitchTypeView[];
  top_speed_kmh: number | null;
}

export interface TeamView {
  club: ClubInfo;
  players: PlayerView[];
}

export interface ParkView {
  /** [bearing rad, distance m, height m] from the left-field line to the right-field line. */
  fence: [number, number, number][];
  backstop: number;
  foul_width_near: number;
  foul_width_far: number;
}

export interface RulesView {
  id: "kbo" | "mlb" | "npb";
  innings: number;
  max_innings: number | null;
  auto_runner_from: number | null;
  dh: boolean;
  automated_zone: boolean;
  min_batters_faced: number;
  shift_restricted: boolean;
}

export interface MatchView {
  league: Localized;
  teams: [TeamView, TeamView];
  park: ParkView;
  rules: RulesView;
  user_team: number | null;
  seed: number;
}

export interface SituationView {
  inning: number;
  top: boolean;
  outs: number;
  balls: number;
  strikes: number;
  score: [number, number];
  bases: [number | null, number | null, number | null];
  batting: number;
  fielding: number;
  batter: number;
  on_deck: number;
  pitcher: number;
  pitch_count: number;
  in_plate_appearance: boolean;
  over: boolean;
}

export interface BatLineView {
  player: number;
  pa: number;
  ab: number;
  r: number;
  h: number;
  doubles: number;
  triples: number;
  hr: number;
  rbi: number;
  bb: number;
  so: number;
  sb: number;
  sh: number;
}

export interface PitchLineView {
  player: number;
  outs: number;
  h: number;
  r: number;
  er: number;
  bb: number;
  so: number;
  hr: number;
  pitches: number;
}

export interface BoxView {
  batting: BatLineView[];
  pitching: PitchLineView[];
  hits: number;
  errors: number;
  left_on_base: number;
}

export interface Capabilities {
  offense: boolean;
  defense: boolean;
  substitute: boolean;
  pitcher_removable: boolean;
  needs_pitcher: boolean;
  bench: number[];
  bullpen: number[];
}

export interface ResultView {
  score: [number, number];
  innings: number;
  win: [number, number] | null;
  loss: [number, number] | null;
  save: [number, number] | null;
}

export interface MatchState {
  situation: SituationView;
  line_score: [number, number][];
  lineups: [number[], number[]];
  batting_slot: [number, number];
  defense: [number[], number[]];
  boxes: [BoxView, BoxView];
  positions: [number, number][];
  can: Capabilities;
  result: ResultView | null;
}

export interface ZoneView {
  top: number;
  bottom: number;
  half_width: number;
}

export interface BattedView {
  exit_speed_kmh: number;
  launch_angle_deg: number;
  spray_deg: number;
  distance: number;
  hang_time: number;
  apex: number;
  bunt: boolean;
  end: "stopped" | "over_fence" | "out_of_play" | "time_limit";
  /** [t, x, y, z] */
  path: [number, number, number, number][];
}

export interface ThrowView {
  by: number;
  to: number;
  from: [number, number];
  base: number;
  t_release: number;
  t_arrive: number;
  carried: boolean;
}

export interface RunView {
  player: number;
  from: number;
  to: number;
  t_start: number;
  t_arrive: number;
  out: boolean;
}

export type PlayKind =
  | "foul"
  | "single"
  | "double"
  | "triple"
  | "home_run"
  | "ground_rule_double"
  | "ground_out"
  | "fly_out"
  | "line_out"
  | "pop_out"
  | "foul_out"
  | "double_play"
  | "fielders_choice"
  | "error"
  | "sac_fly"
  | "sac_bunt";

export interface PlayView {
  kind: PlayKind;
  outs: number;
  fielder: number;
  field_pos: [number, number];
  fielded_t: number;
  caught: boolean;
  rbi: number;
  scored: number[];
  bases_after: [number | null, number | null, number | null];
  gidp: boolean;
  throws: ThrowView[];
  runs: RunView[];
}

export type Approach = "normal" | "sac_bunt" | "bunt_for_hit" | "hit_and_run" | "take";
export type PitchResult = "ball" | "called_strike" | "swinging_strike" | "foul" | "foul_tip" | "in_play" | "hit_by_pitch";

export interface PitchView {
  inning: number;
  top: boolean;
  balls: number;
  strikes: number;
  outs: number;
  batter: number;
  pitcher: number;
  batter_hand: "R" | "L";
  pitcher_hand: "R" | "L";
  kind: PitchKind;
  speed_kmh: number;
  spin_rpm: number;
  release: [number, number, number];
  plate: [number, number, number];
  plate_vel: [number, number, number];
  flight_time: number;
  zone: ZoneView;
  zone_distance: number;
  approach: Approach;
  swung: boolean;
  result: PitchResult;
  miss: string | null;
  bat_speed_kmh: number | null;
  batted: BattedView | null;
  play: PlayView | null;
  positions: [number, number][];
}

export type PaOutcome = "strikeout" | "walk" | "intentional_walk" | "hit_by_pitch" | "interrupted" | PlayKind;

export interface PlateAppearanceView {
  inning: number;
  top: boolean;
  batter: number;
  pitcher: number;
  outcome: PaOutcome;
  rbi: number;
  runs: number;
  outs_made: number;
  pitches: number;
  outs_after: number;
  score_after: [number, number];
}

export type SubKind = "pitcher" | "pinch_hitter" | "pinch_runner" | "fielder";

export type EventView =
  | { type: "half_inning"; inning: number; top: boolean }
  | ({ type: "pitch" } & PitchView)
  | ({ type: "plate_appearance" } & PlateAppearanceView)
  | { type: "steal"; team: number; runner: number; to_base: number; safe: boolean }
  | { type: "wild_pitch" }
  | { type: "intentional_walk"; team: number; batter: number }
  | { type: "substitution"; kind: SubKind; team: number; player: number; replaced: number; base: number | null; position: number | null }
  | { type: "game_over" };

export interface AdvanceView {
  events: EventView[];
  state: MatchState;
}

// ---- requests ----

export type RuleSet = "kbo" | "mlb" | "npb";
export type Control = "home" | "away" | "watch";

export interface Delegate {
  pitching: boolean;
  bench: boolean;
  offense: boolean;
  defense: boolean;
}

export interface NewMatch {
  seed?: number;
  rules: RuleSet;
  control: Control;
  delegate: Delegate;
}

export type Mode = "pitch" | "plate_appearance" | "half_inning" | "game";
export type StealOrder = "auto" | "hold" | "go";
export type InfieldDepth = "normal" | "double_play" | "in" | "corners_in";
export type OutfieldDepth = "normal" | "shallow" | "deep";
export type Shift = "none" | "shade" | "full";

export interface OffenseOrders {
  approach?: Approach;
  bunt_toward_third?: boolean;
  steal: StealOrder;
}

export interface DefenseOrders {
  infield?: InfieldDepth;
  outfield?: OutfieldDepth;
  shift?: Shift;
  intentional_walk: boolean;
}

export interface AdvanceRequest {
  mode: Mode;
  offense?: OffenseOrders;
  defense?: DefenseOrders;
}

export type SubRequest =
  | { kind: "pitcher"; player: number }
  | { kind: "pinch_hitter"; player: number }
  | { kind: "pinch_runner"; base: number; player: number }
  | { kind: "fielder"; position: number; player: number };

export const POSITION_CODES: PositionCode[] = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];
