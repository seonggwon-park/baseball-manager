//! A match being played through the interface: the engine's `GameSim` plus
//! names, the user's role, and conversion of everything to serialisable views.

use crate::names::{korean_league, ClubInfo, Localized, NameGenerator};
use crate::view::*;
use bm_engine::fielding::{B1, B2, B3, C, POSITION_CODES, SS};
use bm_engine::profile::NO_POSITION;
use bm_engine::{
    Alignment, Approach, Auto, BatLine, Club, DefenseOrders, GameEvent, GameSim, Hand, InfieldDepth, Level, MatchEnv, Miss, ModelParams, Observer,
    OffenseOrders, OrderError, OutfieldDepth, PaEvent, PaOutcome, PitchEvent, PitchLine, PitchResult, PlayKind, Player, Rng, Rules, Shift, StealOrder, Step,
    Substitution, Team, ZoneCall,
};
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::{Atmosphere, Park, PathEnd, Vec3, Wind};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const KMH: f64 = 1000.0 / 3600.0;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleSet {
    #[default]
    Kbo,
    Mlb,
    Npb,
}

/// Which team the user manages.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Control {
    #[default]
    Home,
    Away,
    Watch,
}

/// Decisions of the user's team that are left to the bench coach.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Delegate {
    pub pitching: bool,
    pub bench: bool,
    pub offense: bool,
    pub defense: bool,
}

impl Default for Delegate {
    fn default() -> Self {
        Delegate { pitching: false, bench: false, offense: false, defense: true }
    }
}

impl From<Delegate> for Auto {
    fn from(d: Delegate) -> Auto {
        Auto { pitching: d.pitching, bench: d.bench, offense: d.offense, defense: d.defense }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct NewMatch {
    /// Random seed; the same seed and the same orders give the same game.
    pub seed: Option<u64>,
    pub rules: RuleSet,
    pub control: Control,
    pub delegate: Delegate,
    /// Club ids from `data/leagues/kr.json`; two different clubs are drawn when missing.
    pub away: Option<String>,
    pub home: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApproachDto {
    #[default]
    Normal,
    SacBunt,
    BuntForHit,
    HitAndRun,
    Take,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StealDto {
    #[default]
    Auto,
    Hold,
    Go,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct OffenseOrdersDto {
    /// Missing = leave it to the bench.
    pub approach: Option<ApproachDto>,
    pub bunt_toward_third: Option<bool>,
    pub steal: StealDto,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InfieldDto {
    Normal,
    DoublePlay,
    In,
    CornersIn,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutfieldDto {
    Normal,
    Shallow,
    Deep,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShiftDto {
    None,
    Shade,
    Full,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DefenseOrdersDto {
    /// When all three are missing the bench positions the fielders.
    pub infield: Option<InfieldDto>,
    pub outfield: Option<OutfieldDto>,
    pub shift: Option<ShiftDto>,
    pub intentional_walk: bool,
}

/// How far to play before handing control back.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// One pitch (or one move by the other manager).
    #[default]
    Pitch,
    /// To the end of the plate appearance.
    PlateAppearance,
    /// To the end of the half inning, the bench managing the user's team.
    HalfInning,
    /// To the end of the game, the bench managing the user's team.
    Game,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AdvanceRequest {
    pub mode: Mode,
    pub offense: Option<OffenseOrdersDto>,
    pub defense: Option<DefenseOrdersDto>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubRequest {
    Pitcher { player: usize },
    PinchHitter { player: usize },
    PinchRunner { base: usize, player: usize },
    Fielder { position: usize, player: usize },
}

/// An error the interface can show.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ApiError {
    /// Stable identifier for translation.
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        ApiError { code, message: message.into() }
    }
}

impl From<OrderError> for ApiError {
    fn from(e: OrderError) -> Self {
        let (code, message) = match e {
            OrderError::GameOver => ("game_over", "the game is over"),
            OrderError::PlateAppearanceInProgress => ("plate_appearance_in_progress", "substitutions are made between plate appearances"),
            OrderError::WrongSide => ("wrong_side", "the team is not on the right side of the ball for this move"),
            OrderError::PlayerUnavailable => ("player_unavailable", "the player has already been in the game"),
            OrderError::NotAPitcher => ("not_a_pitcher", "the player cannot pitch"),
            OrderError::PitcherMustFaceMoreBatters => ("pitcher_must_face_more_batters", "the pitcher has not yet faced the required number of batters"),
            OrderError::NothingToReplace => ("nothing_to_replace", "there is nobody to replace"),
        };
        ApiError::new(code, message)
    }
}

pub struct Session {
    sim: GameSim,
    rng: Rng,
    view: MatchView,
    user_team: Option<usize>,
    delegate: Delegate,
    /// Players who have occupied each spot of the batting order, in order of appearance.
    slots: [Vec<Vec<usize>>; 2],
    /// Pitchers in order of appearance.
    pitchers: [Vec<usize>; 2],
    /// Events produced outside `advance` (substitutions by the user).
    pending: Vec<EventView>,
}

fn hand_code(h: Hand) -> &'static str {
    match h {
        Hand::Right => "R",
        Hand::Left => "L",
    }
}

/// Map a value onto the 1-20 scale: the population mean lands on 10-11 and three
/// standard deviations above it on 20.
fn rating(z: f64) -> u8 {
    (10.5 + 3.2 * z).round().clamp(1.0, 20.0) as u8
}

fn player_view(idx: usize, p: &Player, name: Localized, number: u8) -> PlayerView {
    let b = &p.batter;
    let f = &p.fielder;
    let batting = BattingRatings {
        contact: rating(-0.6 * (b.barrel_sd - 0.0191) / 0.0023 - 0.4 * (b.timing_sd - 0.0175) / 0.00175),
        power: rating((b.bat_speed / MPH - 71.9) / 2.6),
        eye: rating((b.eye - 0.62) / 0.07),
        speed: rating((b.sprint / FT - 27.0) / 1.5),
        bunt: rating((b.bunt - 0.5) / 0.18),
        fielding: rating(-(f.hands - 1.0) / 0.2 * 0.5 + (f.accel - 4.9) / 0.6 * 0.5),
        arm: rating((f.arm / MPH - 86.0) / 3.5),
    };
    let pitching = p.pitcher.as_ref().map(|pp| {
        let top = pp.pitches.iter().map(|d| d.shape.speed).fold(0.0, f64::max);
        let total: f64 = pp.pitches.iter().map(|d| d.usage_same + d.usage_opp).sum();
        let mut repertoire: Vec<PitchTypeView> = pp
            .pitches
            .iter()
            .map(|d| PitchTypeView { kind: d.kind.code(), speed_kmh: r1(d.shape.speed / KMH), usage: r3((d.usage_same + d.usage_opp) / total) })
            .collect();
        repertoire.sort_by(|a, b| b.usage.partial_cmp(&a.usage).unwrap_or(std::cmp::Ordering::Equal));
        PitchingRatings {
            velocity: rating((top / MPH - 94.8) / 2.3),
            control: rating(-(pp.pitches[0].command / INCH - 6.6) / 0.7),
            // Starters around 92 pitches, relievers around 28: one scale for both.
            stamina: ((pp.stamina / 6.0).round().clamp(1.0, 20.0)) as u8,
            top_speed_kmh: r1(top / KMH),
            repertoire,
        }
    });
    PlayerView {
        idx,
        name,
        number,
        position: if p.position == NO_POSITION { "DH" } else { POSITION_CODES[p.position] },
        bats: hand_code(b.hand),
        throws: p.pitcher.as_ref().map(|pp| hand_code(pp.hand)),
        batting,
        pitching,
    }
}

fn team_view(team: &Team, club: ClubInfo, names: &NameGenerator, rng: &mut Rng) -> TeamView {
    let mut numbers: Vec<u8> = (1..=99).collect();
    let players = team
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let number = numbers.swap_remove(rng.below(numbers.len()));
            player_view(i, p, names.next(rng), number)
        })
        .collect();
    TeamView { club, players }
}

fn play_kind(k: PlayKind) -> &'static str {
    match k {
        PlayKind::Foul => "foul",
        PlayKind::Single => "single",
        PlayKind::Double => "double",
        PlayKind::Triple => "triple",
        PlayKind::HomeRun => "home_run",
        PlayKind::GroundRuleDouble => "ground_rule_double",
        PlayKind::GroundOut => "ground_out",
        PlayKind::FlyOut => "fly_out",
        PlayKind::LineOut => "line_out",
        PlayKind::PopOut => "pop_out",
        PlayKind::FoulOut => "foul_out",
        PlayKind::DoublePlay => "double_play",
        PlayKind::FieldersChoice => "fielders_choice",
        PlayKind::Error => "error",
        PlayKind::SacFly => "sac_fly",
        PlayKind::SacBunt => "sac_bunt",
    }
}

fn xy(v: Vec3) -> [f64; 2] {
    [r3(v.x), r3(v.y)]
}

fn xyz(v: Vec3) -> [f64; 3] {
    [r3(v.x), r3(v.y), r3(v.z)]
}

/// Fielder who takes a throw at a base.
fn cover(base: u8, thrower: usize, from: Vec3) -> usize {
    match base {
        1 if thrower == B1 => B2,
        1 => B1,
        2 => {
            // The middle infielder on the far side of the ball covers second.
            let first_choice = if from.x > 0.0 { SS } else { B2 };
            if thrower == first_choice {
                if first_choice == SS {
                    B2
                } else {
                    SS
                }
            } else {
                first_choice
            }
        }
        3 if thrower == B3 => SS,
        3 => B3,
        _ => C,
    }
}

/// Turns the engine's event stream into views.
struct Collector {
    events: Vec<EventView>,
    score: [u32; 2],
}

impl Observer for Collector {
    fn pitch(&mut self, e: &PitchEvent) {
        let f = &e.thrown.flight;
        let batted = e.batted.zip(e.path).map(|(info, path)| {
            let landing = path.landing.map_or(path.last().pos, |l| l.pos);
            BattedView {
                exit_speed_kmh: r1(info.exit_speed / KMH),
                launch_angle_deg: r1(info.launch_angle.to_degrees()),
                spray_deg: r1(info.spray.to_degrees()),
                distance: r1(landing.horiz()),
                hang_time: r3(path.landing.map_or(path.last().t, |l| l.t)),
                apex: r1(path.apex),
                bunt: info.bunt,
                end: match path.end {
                    PathEnd::Stopped => "stopped",
                    PathEnd::OverFence => "over_fence",
                    PathEnd::OutOfPlay => "out_of_play",
                    PathEnd::TimeLimit => "time_limit",
                },
                path: path.points.iter().map(|p| [r3(p.t), r3(p.pos.x), r3(p.pos.y), r3(p.pos.z)]).collect(),
            }
        });
        let play = e.play.map(|p| PlayView {
            kind: play_kind(p.kind),
            outs: p.outs,
            fielder: p.fielder,
            field_pos: xy(p.field_pos),
            fielded_t: r3(p.trace.fielded_t),
            caught: p.trace.caught,
            rbi: p.rbi,
            scored: p.scored.iter().take(p.n_scored as usize).flatten().map(|r| r.player).collect(),
            bases_after: [p.bases[0].map(|r| r.player), p.bases[1].map(|r| r.player), p.bases[2].map(|r| r.player)],
            gidp: p.gidp,
            throws: p
                .trace
                .throws
                .iter()
                .flatten()
                .map(|t| ThrowView {
                    by: t.by,
                    to: cover(t.to_base, t.by, t.from),
                    from: xy(t.from),
                    base: t.to_base,
                    t_release: r3(t.t_release),
                    t_arrive: r3(t.t_arrive),
                    carried: t.carried,
                })
                .collect(),
            runs: p
                .trace
                .runs
                .iter()
                .flatten()
                .map(|r| RunView { player: r.player, from: r.from, to: r.to, t_start: r3(r.t_start), t_arrive: r3(r.t_arrive), out: r.out })
                .collect(),
        });
        let view = PitchView {
            inning: e.inning,
            top: e.top,
            balls: e.balls,
            strikes: e.strikes,
            outs: e.outs,
            batter: e.batter,
            pitcher: e.pitcher,
            batter_hand: hand_code(e.batter_hand),
            pitcher_hand: hand_code(e.pitcher_hand),
            kind: e.thrown.kind.code(),
            speed_kmh: r1(e.thrown.spec.speed / KMH),
            spin_rpm: (e.thrown.spec.spin_rate / RPM).round(),
            release: xyz(e.thrown.spec.release_pos),
            plate: xyz(f.plate_pos),
            plate_vel: xyz(f.plate_vel),
            flight_time: r3(f.time),
            zone: ZoneView { top: r3(e.zone.top), bottom: r3(e.zone.bottom), half_width: r3(e.zone.half_width) },
            zone_distance: r3(e.zone_distance),
            approach: match e.approach {
                Approach::Normal => "normal",
                Approach::SacrificeBunt => "sac_bunt",
                Approach::BuntForHit => "bunt_for_hit",
                Approach::HitAndRun => "hit_and_run",
                Approach::Take => "take",
            },
            swung: e.swung,
            result: match e.result {
                PitchResult::Ball => "ball",
                PitchResult::CalledStrike => "called_strike",
                PitchResult::SwingingStrike => "swinging_strike",
                PitchResult::Foul => "foul",
                PitchResult::FoulTip => "foul_tip",
                PitchResult::InPlay => "in_play",
                PitchResult::HitByPitch => "hit_by_pitch",
            },
            miss: e.miss.map(|m| match m {
                Miss::Early => "early",
                Miss::Late => "late",
                Miss::OffTheEnd => "off_the_end",
                Miss::Jammed => "jammed",
                Miss::Over => "over",
                Miss::Under => "under",
            }),
            bat_speed_kmh: e.bat_speed.map(|v| r1(v / KMH)),
            batted,
            play,
            positions: e.positions.iter().map(|p| xy(*p)).collect(),
        };
        self.events.push(EventView::Pitch(Box::new(view)));
    }

    fn plate_appearance(&mut self, e: &PaEvent) {
        self.score[if e.top { 0 } else { 1 }] += e.runs as u32;
        let outcome = match e.outcome {
            PaOutcome::Strikeout => "strikeout",
            PaOutcome::Walk if e.intentional => "intentional_walk",
            PaOutcome::Walk => "walk",
            PaOutcome::HitByPitch => "hit_by_pitch",
            PaOutcome::Play(k) => play_kind(k),
            PaOutcome::Interrupted => "interrupted",
        };
        self.events.push(EventView::PlateAppearance(PlateAppearanceView {
            inning: e.inning,
            top: e.top,
            batter: e.batter,
            pitcher: e.pitcher,
            outcome,
            rbi: e.rbi,
            runs: e.runs,
            outs_made: e.outs_made,
            pitches: e.pitches,
            outs_after: e.outs_before + e.outs_made,
            score_after: self.score,
        }));
    }

    fn event(&mut self, e: &GameEvent) {
        let sub = |kind, team, player, replaced, base, position| EventView::Substitution { kind, team, player, replaced, base, position };
        self.events.push(match *e {
            GameEvent::HalfInning { inning, top } => EventView::HalfInning { inning, top },
            GameEvent::PitchingChange { team, player, replaced } => sub("pitcher", team, player, replaced, None, None),
            GameEvent::PinchHitter { team, player, replaced } => sub("pinch_hitter", team, player, replaced, None, None),
            GameEvent::PinchRunner { team, player, replaced, base } => sub("pinch_runner", team, player, replaced, Some(base), None),
            GameEvent::DefensiveChange { team, player, replaced, position } => sub("fielder", team, player, replaced, None, Some(position)),
            GameEvent::IntentionalWalk { team, batter } => EventView::IntentionalWalk { team, batter },
            GameEvent::Steal { team, runner, to_base, safe } => EventView::Steal { team, runner, to_base, safe },
            GameEvent::WildPitch => EventView::WildPitch,
            GameEvent::GameOver => EventView::GameOver,
        });
    }
}

impl Session {
    pub fn new(cfg: &NewMatch) -> Self {
        let seed = cfg.seed.unwrap_or_else(|| {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
            t ^ (t >> 29)
        });
        let mut rng = Rng::new(seed);
        // Generic parks until real stadium dimensions are in: a typical Korean, Japanese
        // and North American outfield.
        let (rules, park, rules_id) = match cfg.rules {
            RuleSet::Kbo => (Rules::kbo_2025(), Park::symmetric(99.0, 116.0, 121.0, 2.8), "kbo"),
            RuleSet::Mlb => (Rules::mlb_2025(), Park::mlb_average(), "mlb"),
            RuleSet::Npb => (Rules::npb_2025(true), Park::symmetric(100.0, 116.0, 122.0, 3.5), "npb"),
        };
        let env = MatchEnv::new(&Atmosphere::new(22.0, 760.0, 55.0, 50.0), Wind::CALM, park, rules, ModelParams::default());
        let level = Level::mlb();
        let dh = env.rules.dh;
        let away = Club::random(&mut rng, 1000, &level).team_for_game(0, dh);
        let home = Club::random(&mut rng, 2000, &level).team_for_game(0, dh);

        let (league, clubs) = korean_league();
        let find = |id: &Option<String>| id.as_ref().and_then(|id| clubs.iter().position(|c| &c.id == id));
        let away_club = find(&cfg.away).unwrap_or_else(|| rng.below(clubs.len()));
        let mut home_club = find(&cfg.home).unwrap_or_else(|| rng.below(clubs.len()));
        if home_club == away_club {
            home_club = (away_club + 1 + rng.below(clubs.len() - 1)) % clubs.len();
        }
        let names = NameGenerator::korean();
        let teams = [team_view(&away, clubs[away_club].clone(), &names, &mut rng), team_view(&home, clubs[home_club].clone(), &names, &mut rng)];

        let user_team = match cfg.control {
            Control::Away => Some(0),
            Control::Home => Some(1),
            Control::Watch => None,
        };
        let r = &env.rules;
        let view = MatchView {
            league,
            teams,
            park: ParkView {
                fence: env.park.fence.iter().map(|p| [r3(p.angle), r3(p.distance), r3(p.height)]).collect(),
                backstop: env.park.backstop,
                foul_width_near: env.park.foul_width_near,
                foul_width_far: env.park.foul_width_far,
            },
            rules: RulesView {
                id: rules_id,
                innings: r.innings,
                max_innings: r.max_innings,
                auto_runner_from: r.auto_runner_from,
                dh: r.dh,
                automated_zone: r.zone_call == ZoneCall::Automated,
                min_batters_faced: r.min_batters_faced,
                shift_restricted: r.shift_restricted,
            },
            user_team,
            seed,
        };
        let slots = [away.lineup.iter().map(|&i| vec![i]).collect(), home.lineup.iter().map(|&i| vec![i]).collect()];
        let pitchers = [vec![away.defense[0]], vec![home.defense[0]]];
        let mut sim = GameSim::new(Arc::new(env), Arc::new(away), Arc::new(home));
        if let Some(t) = user_team {
            sim.set_auto(t, cfg.delegate.into());
        }
        Session { sim, rng, view, user_team, delegate: cfg.delegate, slots, pitchers, pending: Vec::new() }
    }

    pub fn view(&self) -> &MatchView {
        &self.view
    }

    /// Record who has entered the batting order and who has pitched.
    fn sync(&mut self) {
        for t in 0..2 {
            for (slot, &p) in self.sim.lineup(t).iter().enumerate() {
                if self.slots[t][slot].last() != Some(&p) {
                    self.slots[t][slot].push(p);
                }
            }
            let p = self.sim.pitcher(t);
            if self.pitchers[t].last() != Some(&p) {
                self.pitchers[t].push(p);
            }
        }
    }

    fn box_view(&self, t: usize) -> BoxView {
        let b = self.sim.box_score(t);
        let bat = |player: usize, l: &BatLine| BatLineView {
            player,
            pa: l.pa,
            ab: l.ab,
            r: l.r,
            h: l.h,
            doubles: l.doubles,
            triples: l.triples,
            hr: l.hr,
            rbi: l.rbi,
            bb: l.bb,
            so: l.so,
            sb: l.sb,
            sh: l.sh,
        };
        let pit =
            |player: usize, l: &PitchLine| PitchLineView { player, outs: l.outs, h: l.h, r: l.r, er: l.er, bb: l.bb, so: l.so, hr: l.hr, pitches: l.pitches };
        BoxView {
            batting: self.slots[t].iter().flatten().map(|&p| bat(p, &b.batting[p])).collect(),
            pitching: self.pitchers[t].iter().map(|&p| pit(p, &b.pitching[p])).collect(),
            hits: b.hits,
            errors: b.errors,
            left_on_base: b.left_on_base,
        }
    }

    pub fn state(&self) -> MatchState {
        let s = self.sim.situation();
        let lineup = self.sim.lineup(s.batting);
        let slot = self.sim.batting_slot(s.batting);
        let situation = SituationView {
            inning: s.inning,
            top: s.top,
            outs: s.outs,
            balls: s.balls,
            strikes: s.strikes,
            score: s.score,
            bases: s.bases,
            batting: s.batting,
            fielding: s.fielding,
            batter: s.batter,
            on_deck: lineup[(slot + 1) % 9],
            pitcher: s.pitcher,
            pitch_count: s.pitch_count,
            in_plate_appearance: s.in_plate_appearance,
            over: s.over,
        };
        let can = match self.user_team {
            Some(t) if !s.over => {
                let used = self.sim.used(t);
                let team = self.sim.team(t);
                let free = |list: &[usize]| list.iter().copied().filter(|&i| !used[i]).collect::<Vec<_>>();
                Capabilities {
                    offense: s.batting == t,
                    defense: s.fielding == t,
                    substitute: !s.in_plate_appearance,
                    pitcher_removable: s.fielding == t && !s.in_plate_appearance && self.sim.pitcher_removable(t),
                    needs_pitcher: self.sim.needs_pitcher(t),
                    bench: free(&team.bench),
                    bullpen: free(&team.bullpen),
                }
            }
            _ => Capabilities {
                offense: false,
                defense: false,
                substitute: false,
                pitcher_removable: false,
                needs_pitcher: false,
                bench: Vec::new(),
                bullpen: Vec::new(),
            },
        };
        let pair = |x: Option<(usize, usize)>| x.map(|(t, p)| [t, p]);
        MatchState {
            situation,
            line_score: self.sim.line_score().to_vec(),
            lineups: [*self.sim.lineup(0), *self.sim.lineup(1)],
            batting_slot: [self.sim.batting_slot(0), self.sim.batting_slot(1)],
            defense: [*self.sim.defense(0), *self.sim.defense(1)],
            boxes: [self.box_view(0), self.box_view(1)],
            positions: self.sim.positions().iter().map(|p| xy(*p)).collect(),
            can,
            result: self.sim.result().map(|r| ResultView { score: r.score, innings: r.innings, win: pair(r.win), loss: pair(r.loss), save: pair(r.save) }),
        }
    }

    fn apply_orders(&mut self, req: &AdvanceRequest, first: bool) {
        let s = self.sim.situation();
        if self.user_team == Some(s.batting) {
            if let Some(o) = req.offense {
                self.sim.set_offense_orders(OffenseOrders {
                    approach: o.approach.map(|a| match a {
                        ApproachDto::Normal => Approach::Normal,
                        ApproachDto::SacBunt => Approach::SacrificeBunt,
                        ApproachDto::BuntForHit => Approach::BuntForHit,
                        ApproachDto::HitAndRun => Approach::HitAndRun,
                        ApproachDto::Take => Approach::Take,
                    }),
                    bunt_toward_third: o.bunt_toward_third,
                    steal: match o.steal {
                        StealDto::Auto => StealOrder::Auto,
                        StealDto::Hold => StealOrder::Hold,
                        // "Go" is for the next pitch only.
                        StealDto::Go if first => StealOrder::Go,
                        StealDto::Go => StealOrder::Auto,
                    },
                });
            }
        }
        if self.user_team == Some(s.fielding) {
            if let Some(d) = req.defense {
                let alignment = (d.infield.is_some() || d.outfield.is_some() || d.shift.is_some()).then_some(Alignment {
                    infield: match d.infield {
                        Some(InfieldDto::DoublePlay) => InfieldDepth::DoublePlay,
                        Some(InfieldDto::In) => InfieldDepth::In,
                        Some(InfieldDto::CornersIn) => InfieldDepth::CornersIn,
                        Some(InfieldDto::Normal) | None => InfieldDepth::Normal,
                    },
                    outfield: match d.outfield {
                        Some(OutfieldDto::Shallow) => OutfieldDepth::Shallow,
                        Some(OutfieldDto::Deep) => OutfieldDepth::Deep,
                        Some(OutfieldDto::Normal) | None => OutfieldDepth::Normal,
                    },
                    shift: match d.shift {
                        Some(ShiftDto::Shade) => Shift::Shade,
                        Some(ShiftDto::Full) => Shift::Full,
                        Some(ShiftDto::None) | None => Shift::None,
                    },
                });
                // An intentional walk is ordered once, for the batter at the plate.
                self.sim.set_defense_orders(DefenseOrders { alignment, intentional_walk: d.intentional_walk && first });
            }
        }
    }

    /// Play on, with the user's standing orders, until the requested stopping point.
    pub fn advance(&mut self, req: &AdvanceRequest) -> AdvanceView {
        let mut col = Collector { events: std::mem::take(&mut self.pending), score: self.sim.situation().score };
        let quick = matches!(req.mode, Mode::HalfInning | Mode::Game);
        if let Some(t) = self.user_team {
            self.sim.set_auto(t, if quick { Auto::ALL } else { self.delegate.into() });
        }
        let start = self.sim.situation();
        let mut first = true;
        for _ in 0..20_000 {
            if self.sim.is_over() {
                break;
            }
            self.apply_orders(req, first);
            first = false;
            let step = self.sim.step(&mut self.rng, &mut col);
            self.sync();
            let s = self.sim.situation();
            let stop = match req.mode {
                Mode::Pitch => true,
                Mode::PlateAppearance => !s.in_plate_appearance || step == Step::Change,
                Mode::HalfInning => (s.inning, s.top) != (start.inning, start.top),
                Mode::Game => false,
            };
            if stop {
                break;
            }
        }
        if let Some(t) = self.user_team {
            self.sim.set_auto(t, self.delegate.into());
        }
        AdvanceView { events: col.events, state: self.state() }
    }

    /// Substitution ordered by the user for his team.
    pub fn substitute(&mut self, req: &SubRequest) -> Result<MatchState, ApiError> {
        let Some(team) = self.user_team else { return Err(ApiError::new("not_managing", "no team is managed in this match")) };
        let sub = match *req {
            SubRequest::Pitcher { player } => Substitution::Pitcher { player },
            SubRequest::PinchHitter { player } => Substitution::PinchHitter { player },
            SubRequest::PinchRunner { base, player } => Substitution::PinchRunner { base, player },
            SubRequest::Fielder { position, player } => Substitution::Fielder { position, player },
        };
        let mut col = Collector { events: Vec::new(), score: self.sim.situation().score };
        self.sim.substitute(team, sub, &mut col)?;
        self.pending.append(&mut col.events);
        self.sync();
        Ok(self.state())
    }

    /// Change what is left to the bench coach.
    pub fn set_delegate(&mut self, delegate: Delegate) -> MatchState {
        self.delegate = delegate;
        if let Some(t) = self.user_team {
            self.sim.set_auto(t, delegate.into());
        }
        self.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(events: &[EventView], pred: impl Fn(&EventView) -> bool) -> usize {
        events.iter().filter(|e| pred(e)).count()
    }

    #[test]
    fn a_watched_game_runs_to_the_end_and_adds_up() {
        let mut s = Session::new(&NewMatch { seed: Some(11), control: Control::Watch, ..Default::default() });
        assert_eq!(s.view().teams[0].players.len(), s.view().teams[1].players.len());
        assert_ne!(s.view().teams[0].club.id, s.view().teams[1].club.id);
        let out = s.advance(&AdvanceRequest { mode: Mode::Game, ..Default::default() });
        let st = &out.state;
        assert!(st.situation.over);
        let result = st.result.as_ref().expect("a finished game has a result");
        assert_eq!(result.score, st.situation.score);
        assert_eq!(count(&out.events, |e| matches!(e, EventView::GameOver)), 1);
        // Runs in the plate-appearance events add up to the final score.
        let mut runs = [0u32; 2];
        let mut last_score = [0u32; 2];
        for e in &out.events {
            if let EventView::PlateAppearance(pa) = e {
                runs[if pa.top { 0 } else { 1 }] += pa.runs as u32;
                last_score = pa.score_after;
            }
        }
        assert_eq!(runs, result.score);
        assert_eq!(last_score, result.score);
        // The box score lists every spot of the order and its runs match.
        for t in 0..2 {
            assert!(st.boxes[t].batting.len() >= 9);
            assert_eq!(st.boxes[t].batting.iter().map(|l| l.r as u32).sum::<u32>(), result.score[t]);
            assert!(st.boxes[t].pitching.iter().map(|l| l.outs as u32).sum::<u32>() >= 24);
        }
        // Balls in play carry a path and a play with a timeline.
        let in_play: Vec<&PitchView> = out
            .events
            .iter()
            .filter_map(|e| match e {
                EventView::Pitch(p) if p.result == "in_play" => Some(&**p),
                _ => None,
            })
            .collect();
        assert!(in_play.len() > 30);
        for p in &in_play {
            let batted = p.batted.as_ref().unwrap();
            assert!(batted.path.len() >= 2 && batted.path.windows(2).all(|w| w[1][0] >= w[0][0]));
            let play = p.play.as_ref().unwrap();
            assert!(play.runs.iter().all(|r| r.t_arrive >= r.t_start && r.to >= r.from));
            assert!(play.throws.iter().all(|t| t.t_arrive >= t.t_release && t.to != t.by || t.carried));
            assert_eq!(p.positions.len(), 9);
        }
    }

    #[test]
    fn the_same_seed_and_orders_give_the_same_game() {
        let play = || {
            let mut s = Session::new(&NewMatch { seed: Some(5), control: Control::Home, ..Default::default() });
            let mut log = Vec::new();
            for _ in 0..40 {
                let out = s.advance(&AdvanceRequest { mode: Mode::PlateAppearance, ..Default::default() });
                log.push(serde_json::to_string(&out.state.situation).unwrap());
            }
            (serde_json::to_string(s.view()).unwrap(), log)
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn the_user_can_bunt_walk_and_substitute() {
        let mut s = Session::new(&NewMatch { seed: Some(21), control: Control::Home, rules: RuleSet::Kbo, ..Default::default() });
        // Top of the first: the user is in the field and walks the lead-off man on purpose.
        let st = s.state();
        assert!(st.can.defense && !st.can.offense && st.can.substitute);
        let walk = AdvanceRequest { mode: Mode::Pitch, defense: Some(DefenseOrdersDto { intentional_walk: true, ..Default::default() }), ..Default::default() };
        let out = s.advance(&walk);
        assert_eq!(count(&out.events, |e| matches!(e, EventView::IntentionalWalk { .. })), 1);
        assert!(out.state.situation.bases[0].is_some());
        // The starter has faced one batter: under KBO rules he can be replaced.
        assert!(out.state.can.pitcher_removable);
        let reliever = out.state.can.bullpen[0];
        let st = s.substitute(&SubRequest::Pitcher { player: reliever }).expect("pitching change");
        assert_eq!(st.situation.pitcher, reliever);
        assert!(!st.can.bullpen.contains(&reliever));
        assert_eq!(st.boxes[1].pitching.len(), 2);
        // The substitution shows up in the next batch of events.
        let out = s.advance(&AdvanceRequest { mode: Mode::HalfInning, ..Default::default() });
        assert!(matches!(out.events[0], EventView::Substitution { kind: "pitcher", team: 1, .. }));
        // Bottom of the first: the user bats and has every hitter bunt.
        let st = &out.state;
        assert!(st.can.offense && !st.situation.top);
        let bunt = AdvanceRequest {
            mode: Mode::PlateAppearance,
            offense: Some(OffenseOrdersDto { approach: Some(ApproachDto::SacBunt), ..Default::default() }),
            ..Default::default()
        };
        let mut bunts = 0;
        for _ in 0..3 {
            let out = s.advance(&bunt);
            bunts += count(&out.events, |e| matches!(e, EventView::Pitch(p) if p.approach == "sac_bunt"));
            if out.state.situation.top {
                break;
            }
        }
        assert!(bunts >= 3);
        // A pinch hitter from the bench for the next man.
        let st = s.state();
        if st.can.offense && st.can.substitute {
            let sub = st.can.bench[0];
            let st = s.substitute(&SubRequest::PinchHitter { player: sub }).expect("pinch hitter");
            assert_eq!(st.situation.batter, sub);
            assert!(st.boxes[1].batting.iter().any(|l| l.player == sub));
            let err = s.substitute(&SubRequest::PinchHitter { player: sub }).unwrap_err();
            assert_eq!(err.code, "player_unavailable");
        }
    }

    #[test]
    fn a_watcher_cannot_give_orders() {
        let mut s = Session::new(&NewMatch { seed: Some(2), control: Control::Watch, ..Default::default() });
        assert_eq!(s.substitute(&SubRequest::PinchHitter { player: 20 }).unwrap_err().code, "not_managing");
        let st = s.state();
        assert!(!st.can.offense && !st.can.defense && !st.can.substitute);
    }
}
