//! Season to season: every player develops and ages, some retire, new players come in
//! through the drafts and the amateur signings, KBO and NPB clubs weigh new foreign
//! players against their own, and every organisation places its players at its levels by
//! ability and lets go of those it has no room for.
//!
//! Who comes in and how clubs choose is in the editable file `data/people/careers.json`
//! (`docs/data-files.md`); what a player is worth is `ability.rs`, how he develops
//! `develop.rs`. Clubs here see every player's true ability and ceiling: scouting comes
//! later. Free agents and the players an organisation lets go go to its economy's market
//! (`market.rs`, `docs/contracts.md`); those nobody signs leave the world.

use crate::ability;
use crate::calendar::Date;
use crate::contract::{first_team_seasons, seasons_for_fa, Contract, DraftChange, Economy, FaCompensation, Kind, NpbOwed, Schedule, Terms};
use crate::develop::{develop, develop_idle, AgingTable, Closure, Potential, Style};
use crate::generate::{pitcher, position_player, Squad, Talent};
use crate::league::{Level, World};
use crate::market::{Bidder, Depth, FreeAgent, Why};
use crate::people::{AgeSpread, PeopleMaker};
use crate::player::{Player, Position, Role};
use crate::populate::{level_key, LevelTable, Population, Roster};
use crate::season::Standing;
use bm_engine::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const CAREERS: &str = include_str!("../../../data/people/careers.json");

/// A kind of new player.
#[derive(Clone, Debug, Deserialize)]
pub struct Class {
    /// Age on 1 November of the year he joins.
    pub ages: AgeSpread,
    /// Share of pitchers.
    pub pitchers: f64,
    /// Level of `levels.json` whose talent he starts at, and a shift from it (standard
    /// deviations, on hitting, pitching, fielding and physical alike).
    pub like: String,
    pub shift: f32,
    /// Factor on the mean growth still ahead of a player of his age.
    pub potential: f32,
    /// The level whose skill balance he grows into: `mlb`, `kbo` or `npb`.
    #[serde(default = "mlb")]
    pub peak: String,
    pub birthplaces: String,
    #[serde(default)]
    pub basis: String,
}

fn mlb() -> String {
    "mlb".to_string()
}

/// A league's draft: `rounds` picks per organisation, each pick drawn from the classes in
/// their shares, signed with the chance of its round.
#[derive(Clone, Debug, Deserialize)]
pub struct Draft {
    pub orgs: String,
    pub rounds: u32,
    pub classes: Vec<(String, f64)>,
    /// (last round, chance of signing) in increasing order of rounds.
    pub signed: Vec<(u32, f64)>,
    #[serde(default)]
    pub basis: String,
}

/// Players every organisation of a kind adds each year outside the draft.
#[derive(Clone, Debug, Deserialize)]
pub struct Signing {
    pub orgs: String,
    pub class: String,
    pub per_org: f64,
    #[serde(default)]
    pub basis: String,
}

#[derive(Deserialize)]
struct Valued<T> {
    value: T,
}

#[derive(Deserialize)]
struct Positions {
    weights: Vec<(String, f64)>,
}

#[derive(Deserialize)]
struct Retirement {
    from_age: u32,
    chance: Vec<f64>,
}

/// How long a major-league organisation holds a player on his first minor-league contract
/// (`careers.json` minor_league_contract): parts of `seasons` seasons, or of
/// `seasons_if_younger` for a player who was younger than `younger_than` on the 5 June before
/// he signed, unless he is on the `forty_man` roster. Without it in the file, players never leave this way.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct MinorContract {
    pub seasons: i32,
    pub seasons_if_younger: i32,
    pub younger_than: f64,
    pub forty_man: usize,
    /// A player whose contract has run out may stay with his organisation on a new
    /// minor-league deal, but only in Triple-A or above (the depth veterans sign for); without
    /// it he leaves the world.
    #[serde(default)]
    pub stay_as_depth: bool,
}

/// Korean military service (`careers.json` military_service): every Korean player of a KBO
/// club serves once, by the offseason of the year he turns `deadline_age`; younger players
/// who are not first-team regulars go earlier with `early_chance` a year from
/// `early_from_age`. Sangmu takes the best `sangmu_per_year` who may still apply (no older
/// than `deadline_age` at the end of the year) and plays them in the Futures League; the
/// others serve on active duty and do not play. A share `two_season_share` is away two
/// seasons, the rest one (about 18 months on average).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct MilitaryService {
    pub deadline_age: f64,
    pub early_from_age: f64,
    pub early_chance: f64,
    pub sangmu_per_year: usize,
    pub two_season_share: f64,
    #[serde(default)]
    pub asian_games: Option<AsianGames>,
}

/// The Asian Games every `every` years from `first`: a national team of `team` players
/// (`pitchers` of them pitchers) no older than `max_age` at the end of the year bar
/// `wildcards`, at most `per_club` from one club, wins gold with chance `gold`, and its
/// players who have not served are exempt.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct AsianGames {
    pub first: i32,
    pub every: i32,
    pub team: usize,
    pub pitchers: usize,
    pub wildcards: usize,
    pub max_age: f64,
    pub per_club: usize,
    pub gold: f64,
}

#[derive(Deserialize)]
struct CareerFile {
    classes: BTreeMap<String, Class>,
    drafts: Vec<Draft>,
    signings: Vec<Signing>,
    positions: Positions,
    starter_share: Valued<f64>,
    intake_spread: Valued<f32>,
    #[serde(default)]
    intake_tradeoff: Option<Valued<f32>>,
    retirement: Retirement,
    keep_potential: Valued<f64>,
    board_noise: Valued<f64>,
    foreign_candidates: Valued<usize>,
    balance: Closure,
    #[serde(default)]
    styles: Styles,
    #[serde(default)]
    league_pitcher_shift: LeagueShifts,
    #[serde(default)]
    league_age_discount: LeagueDiscounts,
    prospect_spread: Valued<f64>,
    pitcher_potential: Valued<f32>,
    #[serde(default)]
    pitcher_shift: Option<Valued<f32>>,
    age_discount: Valued<f64>,
    #[serde(default)]
    minor_league_contract: Option<MinorContract>,
    #[serde(default)]
    military_service: Option<MilitaryService>,
}

/// The styles of the three kinds of league (see `Style`).
#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct Styles {
    #[serde(default)]
    mlb: Style,
    #[serde(default)]
    kbo: Style,
    #[serde(default)]
    npb: Style,
}

/// Added, per kind of organisation, to `age_discount`.
#[derive(Default, Deserialize)]
struct LeagueDiscounts {
    #[serde(default)]
    mlb: f64,
    #[serde(default)]
    kbo: f64,
    #[serde(default)]
    npb: f64,
}

/// Standard deviations added, per league, to `pitcher_shift` for the new pitchers it grows.
#[derive(Default, Deserialize)]
struct LeagueShifts {
    #[serde(default)]
    mlb: f32,
    #[serde(default)]
    kbo: f32,
    #[serde(default)]
    npb: f32,
}

/// The contents of `data/people/careers.json`.
#[derive(Clone, Debug)]
pub struct CareerTable {
    pub classes: BTreeMap<String, Class>,
    pub drafts: Vec<Draft>,
    pub signings: Vec<Signing>,
    /// Primary positions of new position players (`None` = designated hitter) and weights.
    pub positions: Vec<(Option<Position>, f64)>,
    pub starter_share: f64,
    /// Spread of a new amateur's attributes around his class's means (`Talent::spread`).
    pub intake_spread: f32,
    /// How much more a new amateur trades bat-to-ball skill for power (`Talent::tradeoff`).
    pub intake_tradeoff: f32,
    pub retire_from: u32,
    pub retire_chance: Vec<f64>,
    pub keep_potential: f64,
    pub board_noise: f64,
    pub foreign_candidates: usize,
    pub balance: Closure,
    /// Styles of the leagues players grow up in, by peak level (`mlb`, `kbo`, `npb`).
    pub styles: BTreeMap<String, Style>,
    /// Added to `pitcher_shift` for the pitchers of each kind of league (`Class::peak`): the
    /// NPB farm's pitchers start further below their first team than its hitters do.
    pub league_pitcher_shift: BTreeMap<String, f32>,
    /// Added to `age_discount` by the organisations of each league (`OrgKind::key`): KBO
    /// clubs keep their veterans far longer than the majors' organisations do.
    pub league_age_discount: BTreeMap<String, f64>,
    /// Spread (standard deviation of the logarithm) of the personal factor on a player's
    /// growth ahead: what makes some players prospects and others organisation depth.
    pub prospect_spread: f64,
    /// Factor on the growth ahead of new pitchers against new position players.
    pub pitcher_potential: f32,
    /// Standard deviations added to new pitchers' pitching on top of their class's talent.
    pub pitcher_shift: f32,
    /// Standard deviations of ability an organisation takes off a player's worth for every
    /// year of age past 30, when it decides whom to keep.
    pub age_discount: f64,
    /// Minor-league free agency, if the rules have it.
    pub minor_contract: Option<MinorContract>,
    /// Korean military service, if the rules have it.
    pub military: Option<MilitaryService>,
}

fn position_code(code: &str) -> Result<Option<Position>, String> {
    if code == "DH" {
        return Ok(None);
    }
    serde_json::from_value(serde_json::Value::String(code.to_string())).map(Some).map_err(|_| format!("unknown position '{code}'"))
}

impl CareerTable {
    /// What a year of age past 30 costs a player in the eyes of an organisation of `kind`.
    pub fn age_discount_of(&self, kind: OrgKind) -> f64 {
        (self.age_discount + self.league_age_discount.get(kind.key()).copied().unwrap_or(0.0)).max(0.0)
    }

    /// The style of the league whose balance a player grows into (`Class::peak`, or a
    /// level's peak level): none for an unknown key.
    pub fn style(&self, peak: &str) -> Style {
        self.styles.get(peak).copied().unwrap_or_default()
    }

    pub fn bundled() -> CareerTable {
        CareerTable::from_json(CAREERS).unwrap_or_else(|e| panic!("data/people/careers.json: {e}"))
    }

    pub fn from_json(text: &str) -> Result<CareerTable, String> {
        let f: CareerFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
        for d in &f.drafts {
            if let Some((c, _)) = d.classes.iter().find(|(c, _)| !f.classes.contains_key(c)) {
                return Err(format!("draft of '{}' names unknown class '{c}'", d.orgs));
            }
        }
        if let Some(s) = f.signings.iter().find(|s| !f.classes.contains_key(&s.class)) {
            return Err(format!("signing of '{}' names unknown class '{}'", s.orgs, s.class));
        }
        let positions = f.positions.weights.iter().map(|(c, w)| position_code(c).map(|p| (p, *w))).collect::<Result<Vec<_>, _>>()?;
        Ok(CareerTable {
            classes: f.classes,
            drafts: f.drafts,
            signings: f.signings,
            positions,
            starter_share: f.starter_share.value,
            intake_spread: f.intake_spread.value,
            intake_tradeoff: f.intake_tradeoff.map_or(0.0, |v| v.value),
            retire_from: f.retirement.from_age,
            retire_chance: f.retirement.chance,
            keep_potential: f.keep_potential.value,
            board_noise: f.board_noise.value,
            foreign_candidates: f.foreign_candidates.value,
            balance: f.balance,
            styles: [("mlb", f.styles.mlb), ("kbo", f.styles.kbo), ("npb", f.styles.npb)].into_iter().map(|(k, s)| (k.to_string(), s)).collect(),
            league_pitcher_shift: [("mlb", f.league_pitcher_shift.mlb), ("kbo", f.league_pitcher_shift.kbo), ("npb", f.league_pitcher_shift.npb)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            league_age_discount: [("mlb", f.league_age_discount.mlb), ("kbo", f.league_age_discount.kbo), ("npb", f.league_age_discount.npb)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            prospect_spread: f.prospect_spread.value,
            pitcher_potential: f.pitcher_potential.value,
            pitcher_shift: f.pitcher_shift.map_or(0.0, |v| v.value),
            age_discount: f.age_discount.value,
            minor_contract: f.minor_league_contract,
            military: f.military_service,
        })
    }

    /// Chance that a player who has just finished a season at `age` stops.
    pub fn retire_chance(&self, age: f64) -> f64 {
        let i = age.floor() as i64 - self.retire_from as i64;
        if i < 0 {
            0.0
        } else {
            self.retire_chance.get(i as usize).copied().unwrap_or(1.0)
        }
    }

    /// Starting talent of a class.
    pub fn talent(&self, class: &Class, levels: &LevelTable) -> Talent {
        let t = levels.get(&class.like).talent();
        Talent {
            hitting: t.hitting + class.shift,
            pitching: t.pitching + class.shift,
            fielding: t.fielding + class.shift,
            physical: t.physical + class.shift,
            ..t
        }
    }
}

// ---------------------------------------------------------------------------------------
// Organisations

/// What kind of organisation: its draft and signings are keyed by this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgKind {
    Major,
    Korean,
    Japanese,
    KoreanIndependent,
    Military,
    JapaneseIndependent,
}

impl OrgKind {
    pub fn key(self) -> &'static str {
        match self {
            OrgKind::Major => "mlb",
            OrgKind::Korean => "kbo",
            OrgKind::Japanese => "npb",
            OrgKind::KoreanIndependent => "kr_independent",
            OrgKind::Military => "kr_military",
            OrgKind::JapaneseIndependent => "jp_independent",
        }
    }
}

/// A first team and the clubs below it, or a club of its own in a second league.
#[derive(Clone, Debug)]
pub struct Org {
    /// The top club's id.
    pub id: String,
    pub kind: OrgKind,
    /// (roster id, level key), top to bottom.
    pub levels: Vec<(String, String)>,
}

fn level_rank(key: &str) -> usize {
    ["mlb", "kbo", "npb", "aaa", "aa", "high_a", "single_a", "kbo2", "npb2", "complex", "academy"].iter().position(|k| *k == key).unwrap_or(99)
}

/// Every organisation of the world, first teams in file order, then the independent clubs.
pub fn organisations(world: &World) -> Vec<Org> {
    let mut orgs = Vec::new();
    for club in world.first_teams() {
        let key = level_key(world, club);
        let kind = match club.country.as_str() {
            "KR" => OrgKind::Korean,
            "JP" => OrgKind::Japanese,
            _ => OrgKind::Major,
        };
        let mut levels = vec![(club.id.clone(), key.to_string())];
        let mut below: Vec<(String, String)> = world.affiliates(&club.id).map(|c| (c.id.clone(), level_key(world, c).to_string())).collect();
        below.sort_by_key(|(_, k)| level_rank(k));
        levels.extend(below);
        if kind == OrgKind::Major {
            for dev in ["complex", "academy"] {
                levels.push((format!("{}/{dev}", club.id), dev.to_string()));
            }
        }
        orgs.push(Org { id: club.id.clone(), kind, levels });
    }
    for club in world.clubs().filter(|c| c.parent.is_none() && world.level_of(c) != Level::Top) {
        let kind = match (club.country.as_str(), club.military) {
            ("KR", true) => OrgKind::Military,
            ("KR", false) => OrgKind::KoreanIndependent,
            _ => OrgKind::JapaneseIndependent,
        };
        orgs.push(Org { id: club.id.clone(), kind, levels: vec![(club.id.clone(), level_key(world, club).to_string())] });
    }
    orgs
}

// ---------------------------------------------------------------------------------------
// Careers

/// How a player came into the world.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Origin {
    /// Already there when the world was made.
    Initial,
    Draft {
        year: i32,
        round: u32,
        pick: u32,
        class: String,
    },
    Signed {
        year: i32,
        class: String,
    },
    Foreign {
        year: i32,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Exit {
    Retired,
    /// His organisation let him go.
    Released,
    /// A KBO or NPB club signed a better foreign player in his place.
    Replaced,
    /// His minor-league contract ran out while he was off the 40-man roster.
    FreeAgent,
}

/// One season on a roster.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stint {
    pub year: i32,
    pub club: String,
    pub level: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Career {
    pub origin: Origin,
    pub seasons: Vec<Stint>,
    /// First season among his major-league organisation's best [`USED_PER_SEASON`] by
    /// ability: a stand-in for having played in the majors.
    pub majors: Option<i32>,
    pub left: Option<(i32, Exit)>,
    /// A Korean player's military service.
    #[serde(default)]
    pub service: Option<Service>,
    /// First-team seasons before the world began (the first world's players; `contract.rs`).
    #[serde(default)]
    pub prior: u32,
    /// The offseason he was made the qualifying offer (once in a career).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualifying_offer: Option<i32>,
    /// Times a KBO or NPB player declared free agency, and his first-team seasons when he
    /// last did (the seasons for the next one count from there).
    #[serde(default)]
    pub free_agencies: u32,
    #[serde(default)]
    pub fa_at: u32,
}

/// Where a Korean player stands with his military service.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Service {
    /// Serving, with Sangmu (who play in the Futures League) or on active duty (who do not
    /// play), while `org` holds his rights; back for the `back` season.
    Serving { sangmu: bool, org: String, back: i32 },
    /// Done by the offseason after the `year` season, or exempt (an Asian Games gold).
    Done { year: i32, exempt: bool },
}

impl Career {
    pub fn new(origin: Origin) -> Career {
        Career { origin, seasons: Vec::new(), majors: None, left: None, service: None, prior: 0, qualifying_offer: None, free_agencies: 0, fa_at: 0 }
    }

    pub fn reached(&self, level: &str) -> bool {
        self.seasons.iter().any(|s| s.level == level)
    }
}

/// Players a major-league club uses in a season: 1,470 players appeared in 2025 MLB
/// regular-season games (MLB Stats API season stats, hitting, pitching and fielding), 49 per
/// club. The 26 on the roster plus the next best of the organisation stand in for them.
pub const USED_PER_SEASON: usize = 49;

/// Record the season every rostered player starts, and who is among the players each
/// major-league organisation would use.
pub fn record_season(world: &World, pop: &mut Population, year: i32) {
    for r in &pop.rosters {
        for p in r.players() {
            let c = pop.careers.entry(p.id).or_insert_with(|| Career::new(Origin::Initial));
            if c.seasons.last().is_none_or(|s| s.year != year) {
                c.seasons.push(Stint { year, club: r.club.clone(), level: r.level.clone() });
            }
        }
    }
    let index: BTreeMap<&str, &Roster> = pop.rosters.iter().map(|r| (r.club.as_str(), r)).collect();
    let mut forty: Vec<u32> = Vec::new();
    for org in organisations(world).iter().filter(|o| o.kind == OrgKind::Major) {
        let mut top: Vec<u32> = Vec::new();
        let mut rest: Vec<(f64, u32)> = Vec::new();
        for (i, (club, _)) in org.levels.iter().enumerate() {
            let Some(r) = index.get(club.as_str()) else { continue };
            for p in r.players() {
                if i == 0 {
                    top.push(p.id);
                } else {
                    // Position players and pitchers on one scale: standard deviations.
                    rest.push((ability::value(p) / ability::per_sd(p), p.id));
                }
            }
        }
        rest.sort_by(|a, b| b.0.total_cmp(&a.0));
        forty.extend(top);
        forty.extend(rest.iter().take(USED_PER_SEASON.saturating_sub(26)).map(|x| x.1));
    }
    for id in forty {
        if let Some(c) = pop.careers.get_mut(&id) {
            c.majors.get_or_insert(year);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Building a club's squad

/// Best assignment of rows to distinct columns, maximising the total value (Hungarian
/// method; needs no more rows than columns). Returns the column of each row.
fn assign(value: &[Vec<f64>]) -> Vec<usize> {
    let n = value.len();
    if n == 0 {
        return Vec::new();
    }
    let m = value[0].len();
    assert!(n <= m, "more slots than players");
    let cost = |i: usize, j: usize| -value[i][j];
    let inf = f64::INFINITY;
    let (mut u, mut v) = (vec![0.0; n + 1], vec![0.0; m + 1]);
    let (mut p, mut way) = (vec![0usize; m + 1], vec![0usize; m + 1]);
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0;
        let mut minv = vec![inf; m + 1];
        let mut used = vec![false; m + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let (mut delta, mut j1) = (inf, 0);
            for j in 1..=m {
                if !used[j] {
                    let cur = cost(i0 - 1, j - 1) - u[i0] - v[j];
                    if cur < minv[j] {
                        minv[j] = cur;
                        way[j] = j0;
                    }
                    if minv[j] < delta {
                        delta = minv[j];
                        j1 = j;
                    }
                }
            }
            for j in 0..=m {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut out = vec![0; n];
    for j in 1..=m {
        if p[j] != 0 {
            out[p[j] - 1] = j - 1;
        }
    }
    out
}

/// Lineup slots in the engine's order: catcher ... right field, then the designated hitter.
const LINEUP: [Option<Position>; 9] = [
    Some(Position::Catcher),
    Some(Position::First),
    Some(Position::Second),
    Some(Position::Third),
    Some(Position::Short),
    Some(Position::Left),
    Some(Position::Center),
    Some(Position::Right),
    None,
];

/// Pick the regulars (and a second catcher) among `hitters`: the indices of the players
/// for each slot of `slots`.
fn regulars(hitters: &[Player], slots: &[Option<Position>]) -> Vec<usize> {
    if hitters.len() < slots.len() {
        // Not enough players for every slot: the best ones, in order.
        return (0..hitters.len()).collect();
    }
    let value: Vec<Vec<f64>> = slots.iter().map(|&s| hitters.iter().map(|p| ability::at_position(p, s)).collect()).collect();
    assign(&value)
}

fn sort_by_value(v: &mut [Player], f: impl Fn(&Player) -> f64) {
    v.sort_by(|a, b| f(b).total_cmp(&f(a)));
}

/// The pitchers of a staff in order: the five starters first (those with the stamina,
/// best first), then everyone else by quality.
fn staff_order(mut pitchers: Vec<Player>) -> Vec<Player> {
    sort_by_value(&mut pitchers, |p| ability::pitching_rate(p) + if ability::can_start(p) { 100.0 } else { 0.0 });
    let starters: Vec<Player> = pitchers.iter().take(5).cloned().collect();
    let mut rest: Vec<Player> = pitchers.into_iter().skip(5).collect();
    sort_by_value(&mut rest, ability::pitching_rate);
    starters.into_iter().chain(rest).collect()
}

/// A club's roster from the players placed at its level: the best lineup the positions
/// allow, a second catcher and the best three others on the bench, five starters and
/// eight relievers; everyone else is a reserve.
pub fn build_roster(club: &str, level: &str, players: Vec<Player>) -> Roster {
    let (pitchers, mut hitters): (Vec<Player>, Vec<Player>) = players.into_iter().partition(Player::is_pitcher);
    let mut staff = staff_order(pitchers);
    // A club short of position players borrows its worst pitchers.
    while hitters.len() < 10 && staff.len() > 13 {
        hitters.push(staff.pop().unwrap());
    }
    let mut slots: Vec<Option<Position>> = LINEUP.to_vec();
    slots.push(Some(Position::Catcher));
    let picked = regulars(&hitters, &slots[..slots.len().min(hitters.len())]);
    let taken: BTreeSet<usize> = picked.iter().copied().collect();
    let mut lineup = Vec::with_capacity(9);
    let mut bench = Vec::with_capacity(4);
    for (k, &i) in picked.iter().enumerate() {
        let entry = (hitters[i].clone(), slots[k]);
        if k < 9 {
            lineup.push(entry);
        } else {
            bench.push(entry);
        }
    }
    let mut rest: Vec<Player> = hitters.into_iter().enumerate().filter(|(i, _)| !taken.contains(i)).map(|(_, p)| p).collect();
    sort_by_value(&mut rest, ability::value);
    let mut reserves = Vec::new();
    for p in rest {
        let pos = ability::best_position(&p).0;
        if bench.len() < 4 {
            bench.push((p, pos));
        } else {
            let pos = pos.or(Some(p.position));
            reserves.push((p, pos));
        }
    }
    let mut starters = Vec::with_capacity(5);
    let mut relievers = Vec::with_capacity(8);
    for (i, mut p) in staff.into_iter().enumerate() {
        if let Some(pit) = p.pitching.as_mut() {
            pit.role = if i < 5 { Role::Starter } else { Role::Reliever };
        }
        if i < 5 {
            starters.push(p);
        } else if i < 13 {
            relievers.push(p);
        } else {
            reserves.push((p, Some(Position::Pitcher)));
        }
    }
    Roster { club: club.to_string(), level: level.to_string(), squad: Squad { hitters: lineup, bench, starters, relievers }, reserves }
}

// ---------------------------------------------------------------------------------------
// Placing an organisation's players

/// Pitchers and position players a roster of `size` holds: 13 of each on the game-day
/// squad, and the reserves split evenly, the odd one a pitcher.
pub fn roster_split(size: usize) -> (usize, usize) {
    let extra = size.saturating_sub(26);
    (13 + extra.div_ceil(2), 13 + extra / 2)
}

/// Take a level's players out of `pool`: `pinned` players first, then the five best
/// starters and the best other pitchers, the best lineup the positions allow with a
/// second catcher, and the best other position players.
fn take_level(pool: &mut Vec<Player>, size: usize, pinned: &BTreeSet<u32>) -> Vec<Player> {
    let (n_p, n_h) = roster_split(size);
    let (mut out, rest): (Vec<Player>, Vec<Player>) = std::mem::take(pool).into_iter().partition(|p| pinned.contains(&p.id));
    let pin_p = out.iter().filter(|p| p.is_pitcher()).count();
    let pin_h = out.len() - pin_p;
    let (pitchers, hitters): (Vec<Player>, Vec<Player>) = rest.into_iter().partition(Player::is_pitcher);
    let mut staff = staff_order(pitchers).into_iter();
    out.extend(staff.by_ref().take(n_p.saturating_sub(pin_p)));
    pool.extend(staff);
    let need = n_h.saturating_sub(pin_h);
    let slots: Vec<Option<Position>> = LINEUP.iter().copied().chain([Some(Position::Catcher)]).take(need).collect();
    let picked: BTreeSet<usize> = regulars(&hitters, &slots).into_iter().collect();
    let (mut chosen, mut others): (Vec<Player>, Vec<Player>) = (Vec::new(), Vec::new());
    for (i, p) in hitters.into_iter().enumerate() {
        if picked.contains(&i) {
            chosen.push(p);
        } else {
            others.push(p);
        }
    }
    sort_by_value(&mut others, ability::value);
    let more = need.saturating_sub(chosen.len());
    out.extend(chosen);
    let mut others = others.into_iter();
    out.extend(others.by_ref().take(more));
    pool.extend(others);
    out
}

/// Take the youngest pitchers and position players out of `pool` for a roster of `size`.
fn take_youngest(pool: &mut Vec<Player>, size: usize, age: &dyn Fn(u32) -> f64) -> Vec<Player> {
    let (n_p, n_h) = roster_split(size);
    pool.sort_by(|a, b| age(a.id).total_cmp(&age(b.id)));
    let (mut k_p, mut k_h) = (0, 0);
    let mut out = Vec::new();
    let mut rest = Vec::new();
    for p in pool.drain(..) {
        if p.is_pitcher() && k_p < n_p {
            k_p += 1;
            out.push(p);
        } else if !p.is_pitcher() && k_h < n_h {
            k_h += 1;
            out.push(p);
        } else {
            rest.push(p);
        }
    }
    *pool = rest;
    out
}

/// What a player is worth at his ceiling (pitch grades grow with stuff).
pub fn ceiling_value(p: &Player, pot: &Potential) -> f64 {
    let mut top = p.clone();
    let stuff_gap = (pot.attrs[crate::attr::Attr::Stuff] - p.attrs[crate::attr::Attr::Stuff]).max(0.0);
    top.attrs = pot.attrs;
    if let Some(pit) = top.pitching.as_mut() {
        pit.pitches.iter_mut().for_each(|k| k.grade += stuff_gap);
    }
    ability::value(&top)
}

/// How an organisation ranks a player when it decides whom to keep: his ability now plus
/// a share of the growth still ahead of him.
pub fn keep_value(p: &Player, pot: Option<&Potential>, share: f64) -> f64 {
    let now = ability::value(p);
    match pot {
        Some(pot) => now + share * (ceiling_value(p, pot) - now).max(0.0),
        None => now,
    }
}

/// What a year of age past 30 costs a player in his organisation's eyes, in standard
/// deviations of his ability: older players cost more and have less ahead of them.
fn age_cost(age: f64, per_year: f64) -> f64 {
    per_year * (age - 30.0).max(0.0)
}

/// Split `players` into the ones an organisation keeps and the ones it lets go, best first
/// by `value`. Every level takes players up to an age (`deepest` gives the lowest level a
/// player may still play at, `None` if none), so a player is kept only if everyone kept so
/// far can still be placed: for every level k, the players who can go no lower than k fit
/// on the levels down to k. `caps` holds the pitchers and position players of each level;
/// at least `catchers` catchers and `starters` pitchers with a starter's stamina are kept;
/// `pinned` players (with their level) always are, and `protected` ones go before the rest.
fn keep(
    players: Vec<Player>,
    value: &dyn Fn(&Player) -> f64,
    caps: &[(usize, usize)],
    (catchers, starters): (usize, usize),
    deepest: &dyn Fn(&Player) -> Option<usize>,
    pinned: &BTreeMap<u32, usize>,
    protected: &BTreeSet<u32>,
) -> (Vec<Player>, Vec<Player>) {
    let n = caps.len();
    let cum = |pitchers: bool| -> Vec<usize> {
        caps.iter()
            .scan(0, |acc, &(p, h)| {
                *acc += if pitchers { p } else { h };
                Some(*acc)
            })
            .collect()
    };
    let (cum_p, cum_h) = (cum(true), cum(false));
    let (mut cnt_p, mut cnt_h) = (vec![0usize; n], vec![0usize; n]);
    let mut scored: Vec<(f64, Player)> = players.into_iter().map(|p| (if pinned.contains_key(&p.id) { f64::INFINITY } else { value(&p) }, p)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut accept = |p: &Player| -> bool {
        let Some(d) = pinned.get(&p.id).copied().or_else(|| deepest(p)) else { return false };
        let (cnt, cum) = if p.is_pitcher() { (&mut cnt_p, &cum_p) } else { (&mut cnt_h, &cum_h) };
        if (d..n).any(|k| cnt[k] + 1 > cum[k]) {
            return false;
        }
        for c in cnt.iter_mut().skip(d) {
            *c += 1;
        }
        true
    };
    let (mut kept, mut gone) = (Vec::new(), Vec::new());
    // Pinned and protected players, then catchers and starters up to their minimums, then
    // everyone else.
    let (mut k_c, mut k_s) = (0, 0);
    let mut rest = Vec::new();
    for (v, p) in scored {
        let catcher = !p.is_pitcher() && p.position == Position::Catcher;
        let starter = p.is_pitcher() && ability::can_start(&p);
        let first = pinned.contains_key(&p.id) || protected.contains(&p.id) || (catcher && k_c < catchers) || (starter && k_s < starters);
        if first && accept(&p) {
            if !pinned.contains_key(&p.id) {
                k_c += usize::from(catcher);
                k_s += usize::from(starter);
            }
            kept.push(p);
        } else {
            rest.push((v, p));
        }
    }
    for (_, p) in rest {
        if accept(&p) {
            kept.push(p);
        } else {
            gone.push(p);
        }
    }
    (kept, gone)
}

/// Players of one kind who must be placed at level `i` so that everyone left still fits
/// below: for every deeper level k, those who can go no lower than k must fit on levels
/// i+1..=k, and the best of any surplus play here.
fn forced(pool: &[Player], i: usize, pitchers: bool, caps: &[(usize, usize)], deepest: &dyn Fn(&Player) -> usize) -> Vec<u32> {
    let cap = |j: usize| if pitchers { caps[j].0 } else { caps[j].1 };
    let mut cands: Vec<&Player> = pool.iter().filter(|p| p.is_pitcher() == pitchers).collect();
    cands.sort_by(|a, b| ability::value(b).total_cmp(&ability::value(a)));
    let mut chosen: BTreeSet<u32> = BTreeSet::new();
    let mut below = 0;
    for k in i..caps.len() {
        if k > i {
            below += cap(k);
        }
        let bound: Vec<&&Player> = cands.iter().filter(|p| deepest(p) <= k).collect();
        let here = bound.iter().filter(|p| chosen.contains(&p.id)).count();
        let need = bound.len().saturating_sub(below).saturating_sub(here);
        chosen.extend(bound.iter().filter(|p| !chosen.contains(&p.id)).take(need).map(|p| p.id).collect::<Vec<_>>());
    }
    chosen.into_iter().collect()
}

// ---------------------------------------------------------------------------------------
// New players

/// Makes new players and the people behind them.
struct Intake<'a> {
    table: &'a CareerTable,
    levels: &'a LevelTable,
    aging: &'a AgingTable,
    people: &'a PeopleMaker,
    date: Date,
    next_id: u32,
}

/// A new player with his person and ceiling.
struct Newcomer {
    player: Player,
    person: crate::people::Person,
    potential: Potential,
}

impl Intake<'_> {
    fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn age(&self, p: &crate::people::Person) -> f64 {
        self.date.days_since(p.born) as f64 / 365.2425
    }

    fn position(&self, rng: &mut Rng) -> Option<Position> {
        let total: f64 = self.table.positions.iter().map(|(_, w)| w).sum();
        let mut x = rng.f64() * total;
        for &(p, w) in &self.table.positions {
            if x < w {
                return p;
            }
            x -= w;
        }
        Some(Position::Short)
    }

    /// A player of `class`.
    fn amateur(&mut self, class: &Class, rng: &mut Rng) -> Newcomer {
        let is_pitcher = rng.chance(class.pitchers);
        self.amateur_as(class, is_pitcher, rng)
    }

    /// A pitcher or a position player of `class`.
    fn amateur_as(&mut self, class: &Class, is_pitcher: bool, rng: &mut Rng) -> Newcomer {
        let t = Talent { spread: self.table.intake_spread, tradeoff: self.table.intake_tradeoff, ..self.table.talent(class, self.levels) };
        let id = self.id();
        let mut player = if is_pitcher {
            let role = if rng.chance(self.table.starter_share) { Role::Starter } else { Role::Reliever };
            let league = self.table.league_pitcher_shift.get(&class.peak).copied().unwrap_or(0.0);
            let t = Talent { pitching: t.pitching + self.table.pitcher_shift + league, ..t };
            pitcher(rng, id, role, &t)
        } else {
            let pos = self.position(rng);
            position_player(rng, id, pos, &t)
        };
        self.table.style(&class.peak).shape(&mut player);
        let person = self.people.make(id, &class.birthplaces, &class.ages, self.date, rng);
        let factor = class.potential * if is_pitcher { self.table.pitcher_potential } else { 1.0 };
        let peak = self.levels.get(&class.peak).talent();
        let potential = self.ceiling(&player, &person, &t, &peak, factor, rng);
        Newcomer { player, person, potential }
    }

    /// A foreign player for a KBO or NPB club: talent of the level's foreign players, age
    /// of the level's players.
    fn foreign(&mut self, slot: &ForeignSlot, rng: &mut Rng) -> Newcomer {
        let id = self.id();
        let player = match slot.pitcher {
            Some(role) => pitcher(rng, id, role, &slot.talent),
            None => position_player(rng, id, slot.position, &slot.talent),
        };
        let person = self.people.make(id, &slot.birthplaces, &slot.ages, self.date, rng);
        let potential = self.ceiling(&player, &person, &slot.talent, &slot.talent, 1.0, rng);
        Newcomer { player, person, potential }
    }

    fn ceiling(&self, p: &Player, person: &crate::people::Person, t: &Talent, peak: &Talent, factor: f32, rng: &mut Rng) -> Potential {
        let scale = (self.table.prospect_spread * rng.gauss(0.0, 1.0)).exp() as f32 * factor;
        let mut pot = Potential::draw(p, self.age(person), scale, self.aging, rng);
        pot.balance(p, t, peak, &self.table.balance);
        pot
    }
}

/// One foreign player a KBO or NPB club fields.
#[derive(Clone, Debug)]
struct ForeignSlot {
    /// Index of the organisation's level he plays at.
    level: usize,
    /// `Some(role)` for a pitcher.
    pitcher: Option<Role>,
    position: Option<Position>,
    birthplaces: String,
    talent: Talent,
    ages: AgeSpread,
}

fn foreign_slots(org: &Org, levels: &LevelTable) -> Vec<ForeignSlot> {
    let mut slots = Vec::new();
    for (i, (_, key)) in org.levels.iter().enumerate() {
        let profile = levels.get(key);
        let Some(f) = &profile.foreign else { continue };
        let t = crate::populate::talent(&f.talent);
        let slot = |pitcher: Option<Role>, position: Option<Position>, group: &str| ForeignSlot {
            level: i,
            pitcher,
            position,
            birthplaces: group.to_string(),
            talent: t,
            ages: if pitcher.is_some() { profile.pitchers } else { profile.batters },
        };
        for _ in 0..f.starters {
            slots.push(slot(Some(Role::Starter), Some(Position::Pitcher), &f.birthplaces));
        }
        for _ in 0..f.relievers {
            slots.push(slot(Some(Role::Reliever), Some(Position::Pitcher), &f.birthplaces));
        }
        for &h in &f.hitters {
            slots.push(slot(None, LINEUP.get(h).copied().flatten(), &f.birthplaces));
        }
        if let Some(asia) = &f.asia_quota {
            slots.push(slot(Some(Role::Reliever), Some(Position::Pitcher), asia));
        }
        for k in 0..f.reserves {
            if k % 2 == 0 {
                slots.push(slot(Some(Role::Reliever), Some(Position::Pitcher), &f.birthplaces));
            } else {
                slots.push(slot(None, Some([Position::Catcher, Position::Short, Position::Center][k / 2 % 3]), &f.birthplaces));
            }
        }
    }
    slots
}

// ---------------------------------------------------------------------------------------
// The offseason

/// Everything the offseason reads.
pub struct Offseason<'a> {
    pub world: &'a World,
    pub levels: &'a LevelTable,
    pub aging: &'a AgingTable,
    pub careers: &'a CareerTable,
    pub people: &'a PeopleMaker,
    /// Contract rules: every player gets a contract for the next season (`contract.rs`).
    pub contracts: &'a crate::contract::ContractRules,
}

/// Whether a player who came into a major-league organisation by `origin` has played out his
/// minor-league contract after the `year` season (2023-2027 Minor League Basic Agreement Art.
/// III(A)(1): parts of seven seasons, or six for a player who was 19 or older on the 5 June
/// before he signed; docs/research/world/contracts_mlb.md 5.8). Draftees sign in July and play
/// part of their draft year; international amateurs sign on 15 January and start that
/// season. The first world's players have no signing date and are left alone.
fn contract_ran_out(origin: &Origin, born: Date, year: i32, terms: &MinorContract) -> bool {
    let (first_season, signed) = match origin {
        Origin::Draft { year: y, .. } => (*y, Date::new(*y, 7, 15)),
        Origin::Signed { year: y, .. } => (*y + 1, Date::new(*y + 1, 1, 15)),
        Origin::Initial | Origin::Foreign { .. } => return false,
    };
    let june = Date::new(signed.year(), 6, 5);
    let june = if signed.days_since(june) >= 0 { june } else { Date::new(signed.year() - 1, 6, 5) };
    let age = june.days_since(born) as f64 / 365.2425;
    let seasons = if age >= terms.younger_than { terms.seasons } else { terms.seasons_if_younger };
    year - first_season + 1 >= seasons
}

/// What one offseason did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub retired: usize,
    pub released: usize,
    pub drafted: usize,
    pub signed: usize,
    pub foreign_signed: usize,
    pub foreign_replaced: usize,
    pub minor_free_agents: usize,
    /// Korean players who went into military service, of them with Sangmu, who came back,
    /// and who were exempt after an Asian Games gold.
    pub enlisted: usize,
    pub sangmu: usize,
    pub discharged: usize,
    pub exempt: usize,
    /// Each market ([MLB, KBO, NPB]): players on it, signed to a first-team contract (a
    /// major-league one; a KBO or NPB registered one) and to a depth one (a minor-league one; a
    /// second team's place, an NPB development contract or an independent club), and those
    /// nobody signed.
    pub free_agents: [usize; 3],
    pub signed_major: [usize; 3],
    pub signed_minor: [usize; 3],
    pub unsigned: [usize; 3],
    /// Before the market: opt-outs used, club options exercised and declined, players not
    /// tendered a contract, qualifying offers made and accepted, compensation picks awarded.
    pub opt_outs_due: usize,
    pub opted_out: usize,
    pub options_exercised: usize,
    pub options_declined: usize,
    pub non_tendered: usize,
    pub qualifying_offers: usize,
    pub qualifying_accepted: usize,
    pub compensation_picks: usize,
    /// Free agency in the KBO and NPB ([KBO, NPB]): players with the seasons for it, those
    /// who declared it, re-signed with their club and moved to another; the KBO's compensation
    /// players and NPB's compensation picks.
    pub fa_eligible: [usize; 2],
    pub fa_declared: [usize; 2],
    pub fa_stayed: [usize; 2],
    pub fa_moved: [usize; 2],
    pub compensation_players: usize,
    pub npb_compensation_picks: usize,
}

/// What a club that signs another club's KBO or NPB free agent owes the old club beyond the
/// cash the market charged it.
enum Owed {
    /// KBO grades A and B: a player from outside the signing club's best `protected`, with
    /// the smaller cash, or the larger cash alone (the old club's choice).
    Player { protected: usize },
    /// NPB ranks A and B: an extra pick in this round of the next draft.
    Pick(u32),
}

/// Order in which organisations of one kind pick in the draft: worst record first, or,
/// without standings, weakest first team first.
fn draft_order(orgs: &[&Org], rosters: &BTreeMap<String, Vec<Player>>, standings: Option<&BTreeMap<String, Standing>>) -> Vec<String> {
    let strength = |o: &Org| -> f64 {
        if let Some(s) = standings.and_then(|s| s.get(&o.id)) {
            if s.won + s.lost > 0 {
                return s.pct();
            }
        }
        let ps = rosters.get(&o.levels[0].0).map(Vec::as_slice).unwrap_or(&[]);
        ps.iter().map(ability::value).sum::<f64>() / ps.len().max(1) as f64 / 1000.0
    };
    let mut v: Vec<(f64, String)> = orgs.iter().map(|o| (strength(o), o.id.clone())).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    v.into_iter().map(|x| x.1).collect()
}

fn count(per: f64, rng: &mut Rng) -> usize {
    per.floor() as usize + usize::from(rng.chance(per.fract()))
}

/// Korean military service in the offseason after the `year` season (decision 2026-11-17):
/// players whose service ends go back to the organisation that holds their rights, an Asian
/// Games gold exempts the national team, and the players due go in, the best who may still
/// apply with Sangmu (the base's club, `pinned` to it) and the others on active duty, off
/// every roster (`Population::away`). The first world's players of the base serve for a KBO
/// club drawn at random; a player older than the deadline with no record is taken to have
/// served before he turned professional.
#[allow(clippy::too_many_arguments)]
fn military(
    ctx: &Offseason,
    ms: &MilitaryService,
    pop: &mut Population,
    orgs: &[Org],
    pools: &mut [Vec<Player>],
    pinned: &mut [BTreeMap<u32, usize>],
    regulars: &BTreeSet<u32>,
    year: i32,
    rng: &mut Rng,
    report: &mut Report,
) {
    let kbo: Vec<usize> = orgs.iter().enumerate().filter(|(_, o)| o.kind == OrgKind::Korean).map(|(k, _)| k).collect();
    let Some(base) = orgs.iter().position(|o| o.kind == OrgKind::Military) else { return };
    if kbo.is_empty() {
        return;
    }
    let index: BTreeMap<String, usize> = orgs.iter().enumerate().map(|(k, o)| (o.id.clone(), k)).collect();
    let end_of_year = Date::new(year, 12, 31);
    let ages: BTreeMap<u32, f64> = pop.people.iter().map(|(id, p)| (*id, end_of_year.days_since(p.born) as f64 / 365.2425)).collect();
    let age = |id: u32| ages.get(&id).copied().unwrap_or(25.0);
    let status = |pop: &Population, id: u32| pop.careers.get(&id).and_then(|c| c.service.clone());
    let worth = |p: &Player| ability::value(p) / ability::per_sd(p);

    // The first world's players of the base.
    for p in &pools[base] {
        if !matches!(status(pop, p.id), Some(Service::Serving { .. })) {
            let org = orgs[kbo[rng.below(kbo.len())]].id.clone();
            let back = year + 1 + i32::from(rng.chance(0.5));
            pop.careers.entry(p.id).or_insert_with(|| Career::new(Origin::Initial)).service = Some(Service::Serving { sangmu: true, org, back });
        }
    }
    // Back from service.
    let home = |pop: &Population, id: u32| match status(pop, id) {
        Some(Service::Serving { org, back, .. }) if back <= year + 1 => Some(org),
        _ => None,
    };
    let mut back: Vec<(String, Player)> = Vec::new();
    let (going, staying): (Vec<Player>, Vec<Player>) = std::mem::take(&mut pools[base]).into_iter().partition(|p| home(pop, p.id).is_some());
    pools[base] = staying;
    for p in going {
        back.push((home(pop, p.id).unwrap_or_default(), p));
    }
    let due: Vec<u32> = pop.away.keys().copied().filter(|&id| home(pop, id).is_some()).collect();
    for id in due {
        if let Some(a) = pop.away.remove(&id) {
            back.push((a.org, a.player));
        }
    }
    for (org, p) in back {
        if let Some(c) = pop.careers.get_mut(&p.id) {
            c.service = Some(Service::Done { year, exempt: false });
        }
        pools[index.get(&org).copied().unwrap_or(kbo[0])].push(p);
        report.discharged += 1;
    }

    // The Asian Games: the best Korean players of the KBO clubs and Sangmu, the pitchers and
    // the position players apart, up to the age bar the wildcards, a few from each club.
    if let Some(ag) = ms.asian_games {
        if year >= ag.first && (year - ag.first) % ag.every.max(1) == 0 {
            let mut cands: Vec<(f64, u32, usize, bool)> = kbo
                .iter()
                .chain([&base])
                .flat_map(|&k| pools[k].iter().filter(|p| !pop.foreign.contains_key(&p.id)).map(move |p| (worth(p), p.id, k, p.is_pitcher())))
                .collect();
            cands.sort_by(|a, b| b.0.total_cmp(&a.0));
            let mut per_club: BTreeMap<usize, usize> = BTreeMap::new();
            let (mut pitchers, mut hitters, mut wild) = (0, 0, 0);
            let mut team: Vec<(u32, usize)> = Vec::new();
            for (_, id, k, pitcher) in cands {
                if team.len() >= ag.team || per_club.get(&k).copied().unwrap_or(0) >= ag.per_club {
                    continue;
                }
                let side = if pitcher { pitchers < ag.pitchers } else { hitters < ag.team - ag.pitchers };
                let young = age(id) <= ag.max_age;
                if !side || (!young && wild >= ag.wildcards) {
                    continue;
                }
                if pitcher {
                    pitchers += 1;
                } else {
                    hitters += 1;
                }
                wild += usize::from(!young);
                *per_club.entry(k).or_default() += 1;
                team.push((id, k));
            }
            if rng.chance(ag.gold) {
                for (id, k) in team {
                    match status(pop, id) {
                        Some(Service::Done { .. }) => continue,
                        Some(Service::Serving { org, .. }) if k == base => {
                            // Discharged: back to his club.
                            if let Some(i) = pools[base].iter().position(|p| p.id == id) {
                                let p = pools[base].remove(i);
                                pools[index.get(&org).copied().unwrap_or(kbo[0])].push(p);
                            }
                        }
                        _ => {}
                    }
                    pop.careers.entry(id).or_insert_with(|| Career::new(Origin::Initial)).service = Some(Service::Done { year, exempt: true });
                    report.exempt += 1;
                }
            }
        }
    }

    // Who goes in.
    let mut enlist: Vec<(usize, Player)> = Vec::new();
    for &k in &kbo {
        let mut stay = Vec::with_capacity(pools[k].len());
        for p in std::mem::take(&mut pools[k]) {
            if pop.foreign.contains_key(&p.id) || status(pop, p.id).is_some() {
                stay.push(p);
                continue;
            }
            let a = age(p.id);
            if a >= ms.deadline_age + 1.0 {
                pop.careers.entry(p.id).or_insert_with(|| Career::new(Origin::Initial)).service = Some(Service::Done { year, exempt: false });
                stay.push(p);
            } else if a >= ms.deadline_age || (a >= ms.early_from_age && !regulars.contains(&p.id) && rng.chance(ms.early_chance)) {
                enlist.push((k, p));
            } else {
                stay.push(p);
            }
        }
        pools[k] = stay;
    }
    // Sangmu takes the best of those who may still apply, as far as its roster has room for
    // pitchers and for position players.
    let (cap_p, cap_h) = roster_split(ctx.levels.get(&orgs[base].levels[0].1).roster.max(26));
    let have_p = pools[base].iter().filter(|p| p.is_pitcher()).count();
    let (mut room_p, mut room_h) = (cap_p.saturating_sub(have_p), cap_h.saturating_sub(pools[base].len() - have_p));
    let mut ranked: Vec<(f64, usize)> = enlist.iter().enumerate().filter(|(_, (_, p))| age(p.id) <= ms.deadline_age).map(|(i, (_, p))| (worth(p), i)).collect();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut to_sangmu: BTreeSet<usize> = BTreeSet::new();
    // A staff first: the best pitchers until it has the thirteen a team needs, then the best
    // of everyone (without it, a year short of pitchers among the applicants left Sangmu
    // without a bullpen).
    let short = 13usize.saturating_sub(have_p);
    for &(_, i) in ranked.iter().filter(|(_, i)| enlist[*i].1.is_pitcher()).take(short.min(room_p).min(ms.sangmu_per_year)) {
        room_p -= 1;
        to_sangmu.insert(i);
    }
    for (_, i) in ranked {
        if to_sangmu.len() >= ms.sangmu_per_year {
            break;
        }
        if to_sangmu.contains(&i) {
            continue;
        }
        let room = if enlist[i].1.is_pitcher() { &mut room_p } else { &mut room_h };
        if *room > 0 {
            *room -= 1;
            to_sangmu.insert(i);
        }
    }
    for (i, (k, p)) in enlist.into_iter().enumerate() {
        let back = year + if rng.chance(ms.two_season_share) { 3 } else { 2 };
        let sangmu = to_sangmu.contains(&i);
        let org = orgs[k].id.clone();
        pop.careers.entry(p.id).or_insert_with(|| Career::new(Origin::Initial)).service = Some(Service::Serving { sangmu, org: org.clone(), back });
        report.enlisted += 1;
        if sangmu {
            report.sangmu += 1;
            pools[base].push(p);
        } else {
            pop.away.insert(p.id, crate::populate::Away { org, player: p });
        }
    }
    // Sangmu keeps everyone who serves there.
    for p in &pools[base] {
        pinned[base].insert(p.id, 0);
    }
}

/// An organisation's levels: the roster size of each, its pitchers and position players, its
/// age limits (the oldest age of its spread in `levels.json`, falling from the top level
/// down), and where the Dominican academy is.
struct Shape {
    sizes: Vec<usize>,
    caps: Vec<(usize, usize)>,
    limits: Vec<(f64, f64)>,
    academy_at: Option<usize>,
}

impl Shape {
    fn of(org: &Org, levels: &LevelTable) -> Shape {
        let sizes: Vec<usize> = org.levels.iter().map(|(_, key)| levels.get(key).roster.max(26)).collect();
        let caps = sizes.iter().map(|&s| roster_split(s)).collect();
        let limits = org.levels.iter().map(|(_, key)| (levels.get(key).batters.max, levels.get(key).pitchers.max)).collect();
        // The Dominican academy is for international amateurs only (MLB's Dominican Summer
        // League rosters are 99.5 % born outside the United States; birthplaces.json).
        let academy_at = org.levels.iter().position(|(_, key)| key == "academy");
        Shape { sizes, caps, limits, academy_at }
    }

    /// The lowest level a player of `age` may still play at, `None` if none: the age limits,
    /// Triple-A at most for a minor-league free agent kept as depth, and the academy for
    /// international amateurs only.
    fn deepest(&self, is_pitcher: bool, age: f64, depth_only: bool, international: bool) -> Option<usize> {
        let d = self.limits.iter().take_while(|(b, pt)| age <= if is_pitcher { *pt } else { *b }).count().checked_sub(1)?;
        // Past his minor-league contract: Triple-A depth or nothing.
        let d = if depth_only { d.min(1) } else { d };
        match self.academy_at {
            Some(a) if d >= a && !international => a.checked_sub(1),
            _ => Some(d),
        }
    }
}

/// The offseason after the `year` season, on 1 November: develop and age everyone,
/// retire, weigh new foreign players in Korea and Japan, hold the drafts and signings,
/// and let every organisation keep the players it has room for and place them at its
/// levels. The population moves to 1 February of the next year. `standings` (of the
/// first teams, by club id) set the draft order.
pub fn offseason(ctx: &Offseason, pop: &mut Population, year: i32, standings: Option<&BTreeMap<String, Standing>>, seed: u64) -> Report {
    let date = Date::new(year, 11, 1);
    let mut rng = Rng::new(seed ^ 0x0FF5_EA50 ^ (year as u64).wrapping_mul(0x9E37_79B9));
    let mut report = Report::default();
    let orgs = organisations(ctx.world);
    let born: BTreeMap<u32, Date> = pop.people.iter().map(|(id, p)| (*id, p.born)).collect();
    let age_of = |id: u32, on: Date| born.get(&id).map_or(25.0, |b| on.days_since(*b) as f64 / 365.2425);

    // Everyone off the rosters, by club. A season as a regular at a position closes half
    // the gap to full familiarity with it: a shortstop moved to third base or a centre
    // fielder to a corner learns the position (without this, 41 % of the majors' regulars
    // played a position they knew only in part, and balls in play fell for hits).
    let mut by_club: BTreeMap<String, Vec<Player>> = BTreeMap::new();
    let order: Vec<String> = pop.rosters.iter().map(|r| r.club.clone()).collect();
    // The KBO first teams' regulars of the season just played (lineup and staff): they do
    // not go into military service before they must.
    let regulars: BTreeSet<u32> = pop
        .rosters
        .iter()
        .filter(|r| r.level == "kbo")
        .flat_map(|r| r.squad.hitters.iter().map(|(p, _)| p.id).chain(r.squad.starters.iter().chain(&r.squad.relievers).map(|p| p.id)))
        .collect();
    for r in std::mem::take(&mut pop.rosters) {
        let Roster { club, mut squad, reserves, .. } = r;
        for (p, pos) in squad.hitters.iter_mut() {
            if let Some(pos) = pos {
                let f = &mut p.positions[pos.index()];
                *f += (20 - (*f).min(20)).div_ceil(2);
            }
        }
        let v = by_club.entry(club).or_default();
        v.extend(squad.hitters.into_iter().chain(squad.bench).chain(reserves).map(|(p, _)| p));
        v.extend(squad.starters.into_iter().chain(squad.relievers));
    }

    // A year of development, then retirements.
    let leave = |pop: &mut Population, id: u32, exit: Exit| {
        if let Some(c) = pop.careers.get_mut(&id) {
            c.left = Some((year, exit));
        }
        pop.potentials.remove(&id);
        pop.foreign.remove(&id);
    };
    // With the market, players of the major-league organisations retire on their own less:
    // most careers end when nobody signs them (decision 2026-11-18).
    let factor_of: BTreeMap<&str, f64> = orgs
        .iter()
        .filter(|o| o.kind != OrgKind::Military)
        .filter_map(|o| ctx.contracts.market.get(&Economy::of(o.kind)).map(|m| (o, m.retirement)))
        .flat_map(|(o, f)| o.levels.iter().map(move |(c, _)| (c.as_str(), f)))
        .collect();
    for (club, players) in by_club.iter_mut() {
        let own = factor_of.get(club.as_str()).copied().unwrap_or(1.0);
        let mut stay = Vec::with_capacity(players.len());
        for mut p in players.drain(..) {
            let age = age_of(p.id, date);
            let pot = pop.potentials.entry(p.id).or_insert_with(|| Potential { attrs: p.attrs });
            develop(&mut p, pot, age, ctx.aging, &mut rng);
            if rng.chance(own * ctx.careers.retire_chance(age)) {
                leave(pop, p.id, Exit::Retired);
                report.retired += 1;
            } else {
                stay.push(p);
            }
        }
        *players = stay;
    }
    // Players on active military duty age but do not grow.
    for (id, a) in pop.away.iter_mut() {
        let age = age_of(*id, date);
        let pot = pop.potentials.entry(*id).or_insert_with(|| Potential { attrs: a.player.attrs });
        develop_idle(&mut a.player, pot, age, ctx.aging, &mut rng);
    }

    // Minor-league free agency (decision 2026-11-16): a major-league organisation's minor-
    // leaguer off its 40-man roster leaves once his contract has run its parts of seasons.
    // The 40-man roster: the major-league club's players and the organisation's next best by
    // the value it keeps players by. No market signs them yet, so they leave the world.
    let mut depth_only: BTreeSet<u32> = BTreeSet::new();
    for org in orgs.iter().filter(|o| o.kind == OrgKind::Major) {
        let Some(terms) = ctx.careers.minor_contract else { break };
        let Some((first, minors)) = org.levels.split_first() else { continue };
        let on_major_club = by_club.get(&first.0).map_or(0, Vec::len);
        let worth = |p: &Player| {
            let v = keep_value(p, pop.potentials.get(&p.id), ctx.careers.keep_potential);
            (v - age_cost(age_of(p.id, date), ctx.careers.age_discount_of(org.kind)) * ability::per_sd(p)) / ability::per_sd(p)
        };
        let mut ranked: Vec<(f64, u32)> = minors.iter().flat_map(|(club, _)| by_club.get(club).into_iter().flatten()).map(|p| (worth(p), p.id)).collect();
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
        let forty: BTreeSet<u32> = ranked.iter().take(terms.forty_man.saturating_sub(on_major_club)).map(|x| x.1).collect();
        let free: BTreeSet<u32> = ranked
            .iter()
            .map(|x| x.1)
            .filter(|id| !forty.contains(id))
            .filter(|id| pop.careers.get(id).zip(born.get(id)).is_some_and(|(c, b)| contract_ran_out(&c.origin, *b, year, &terms)))
            .collect();
        if terms.stay_as_depth {
            depth_only.extend(free);
            continue;
        }
        for (club, _) in minors {
            if let Some(players) = by_club.get_mut(club) {
                players.retain(|p| !free.contains(&p.id));
            }
        }
        for id in free {
            leave(pop, id, Exit::FreeAgent);
            report.minor_free_agents += 1;
        }
    }

    let mut intake = Intake {
        table: ctx.careers,
        levels: ctx.levels,
        aging: ctx.aging,
        people: ctx.people,
        date,
        next_id: pop.people.keys().next_back().map_or(1, |k| k + 1),
    };
    let add = |pop: &mut Population, n: Newcomer, origin: Origin| -> Player {
        pop.people.insert(n.person.id, n.person);
        pop.potentials.insert(n.player.id, n.potential);
        pop.careers.insert(n.player.id, Career::new(origin));
        n.player
    };

    // Each organisation's players, pooled.
    let mut pools: Vec<Vec<Player>> = orgs.iter().map(|o| o.levels.iter().flat_map(|(club, _)| by_club.remove(club).unwrap_or_default()).collect()).collect();

    // Foreign players: the better of the club's own and a new one, slot by slot.
    let mut pinned: Vec<BTreeMap<u32, usize>> = vec![BTreeMap::new(); orgs.len()];
    for (k, org) in orgs.iter().enumerate() {
        let slots = foreign_slots(org, ctx.levels);
        if slots.is_empty() {
            continue;
        }
        let (mut own, domestic): (Vec<Player>, Vec<Player>) = std::mem::take(&mut pools[k]).into_iter().partition(|p| pop.foreign.contains_key(&p.id));
        pools[k] = domestic;
        sort_by_value(&mut own, ability::value);
        for slot in &slots {
            let fits = |p: &Player| p.is_pitcher() == slot.pitcher.is_some() && pop.foreign.get(&p.id) == Some(&slot.birthplaces);
            let incumbent = own.iter().position(fits).map(|i| own.remove(i));
            let mut best = incumbent;
            for _ in 0..ctx.careers.foreign_candidates.max(1) {
                let n = intake.foreign(slot, &mut rng);
                if best.as_ref().is_none_or(|b| ability::value(&n.player) > ability::value(b)) {
                    if let Some(old) = best.take() {
                        if pop.foreign.contains_key(&old.id) && pop.careers.contains_key(&old.id) {
                            leave(pop, old.id, Exit::Replaced);
                            report.foreign_replaced += 1;
                        }
                    }
                    let group = slot.birthplaces.clone();
                    let p = add(pop, n, Origin::Foreign { year });
                    pop.foreign.insert(p.id, group);
                    report.foreign_signed += 1;
                    best = Some(p);
                }
            }
            if let Some(p) = best {
                pinned[k].insert(p.id, slot.level);
                pools[k].push(p);
            }
        }
        for p in own {
            leave(pop, p.id, Exit::Replaced);
            report.foreign_replaced += 1;
        }
    }

    if let Some(ms) = ctx.careers.military {
        military(ctx, &ms, pop, &orgs, &mut pools, &mut pinned, &regulars, year, &mut rng, &mut report);
    }

    // Drafts, in reverse order of the standings.
    for draft in &ctx.careers.drafts {
        let members: Vec<&Org> = orgs.iter().filter(|o| o.kind.key() == draft.orgs).collect();
        if members.is_empty() {
            continue;
        }
        let first_teams: BTreeMap<String, Vec<Player>> =
            members.iter().map(|o| (o.levels[0].0.clone(), pools[orgs.iter().position(|x| x.id == o.id).unwrap()].clone())).collect();
        let picking = draft_order(&members, &first_teams, standings);
        let n_orgs = members.len();
        // The picks in order: each round in the draft order, less the picks organisations gave
        // up for signing qualified free agents, with the compensation picks (the majors' for
        // qualified free agents, NPB's for its free agents) after their round (several after
        // one round go in the draft order).
        let changes: Vec<DraftChange> = pop.draft_changes.iter().filter(|c| c.year == year && members.iter().any(|o| o.id == c.org)).cloned().collect();
        let mut slots: Vec<(u32, String)> = Vec::with_capacity(draft.rounds as usize * n_orgs);
        for round in 1..=draft.rounds {
            for org in &picking {
                let lost = changes.iter().filter(|c| !c.extra && c.round == round && &c.org == org).count();
                if lost == 0 {
                    slots.push((round, org.clone()));
                }
            }
            for org in &picking {
                for _ in changes.iter().filter(|c| c.extra && c.round == round && &c.org == org) {
                    slots.push((round, org.clone()));
                }
            }
        }
        let total = slots.len();
        let share_sum: f64 = draft.classes.iter().map(|c| c.1).sum();
        let mut class_pool: Vec<(f64, Newcomer, String)> = (0..total)
            .map(|_| {
                let mut x = rng.f64() * share_sum;
                let mut name = &draft.classes[0].0;
                for (c, w) in &draft.classes {
                    if x < *w {
                        name = c;
                        break;
                    }
                    x -= w;
                }
                let n = intake.amateur(&ctx.careers.classes[name], &mut rng);
                // One board for pitchers and position players: worth in standard deviations.
                let board = keep_value(&n.player, Some(&n.potential), ctx.careers.keep_potential) / ability::per_sd(&n.player)
                    + rng.gauss(0.0, ctx.careers.board_noise);
                (board, n, name.clone())
            })
            .collect();
        class_pool.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (pick, (_, newcomer, class)) in class_pool.into_iter().enumerate() {
            let (round, org) = &slots[pick];
            let round = *round;
            let chance = draft.signed.iter().find(|(last, _)| round <= *last).map_or(1.0, |x| x.1);
            if !rng.chance(chance) {
                continue;
            }
            let k = orgs.iter().position(|o| &o.id == org).unwrap();
            let p = add(pop, newcomer, Origin::Draft { year, round, pick: pick as u32 + 1, class });
            pools[k].push(p);
            report.drafted += 1;
        }
    }

    pop.draft_changes.retain(|c| c.year > year);

    // Signings outside the draft.
    for s in &ctx.careers.signings {
        let class = &ctx.careers.classes[&s.class];
        for (k, org) in orgs.iter().enumerate() {
            if org.kind.key() != s.orgs {
                continue;
            }
            for _ in 0..count(s.per_org, &mut rng) {
                let n = intake.amateur(class, &mut rng);
                let p = add(pop, n, Origin::Signed { year, class: s.class.clone() });
                pools[k].push(p);
                report.signed += 1;
            }
        }
    }

    // Keep what the rosters hold and place it level by level (newcomers have birthdays now).
    let next_season = Date::new(year + 1, 2, 1);
    let born: BTreeMap<u32, Date> = pop.people.iter().map(|(id, p)| (*id, p.born)).collect();
    let age_of = |id: u32, on: Date| born.get(&id).map_or(25.0, |b| on.days_since(*b) as f64 / 365.2425);
    let intl: BTreeSet<u32> =
        pop.careers.iter().filter(|(_, c)| matches!(&c.origin, Origin::Signed { class, .. } if class == "international")).map(|(id, _)| *id).collect();
    let international = |id: u32| intl.contains(&id);
    let shapes: Vec<Shape> = orgs.iter().map(|o| Shape::of(o, ctx.levels)).collect();
    let rules = ctx.contracts;
    let markets = &rules.market;
    // Each economy's market (`market.rs`): its first teams bid, and in the KBO and NPB the
    // independent clubs sign depth. The military club takes no part.
    let economy_of = |o: &Org| Some(Economy::of(o.kind)).filter(|e| o.kind != OrgKind::Military && markets.contains_key(e));
    let mut members: BTreeMap<Economy, Vec<usize>> = BTreeMap::new();
    for (k, o) in orgs.iter().enumerate() {
        if let Some(e) = economy_of(o) {
            members.entry(e).or_default().push(k);
        }
    }
    // Each organisation's bidder in its economy's market.
    let bidder_of: BTreeMap<usize, usize> = members.values().flat_map(|ks| ks.iter().enumerate().map(|(b, &k)| (k, b))).collect();
    let independent = |k: usize| matches!(orgs[k].kind, OrgKind::KoreanIndependent | OrgKind::JapaneseIndependent);
    let mut agents: BTreeMap<Economy, Vec<FreeAgent>> = BTreeMap::new();
    let mut guaranteed: BTreeSet<u32> = BTreeSet::new();
    // Free agency in the majors (decision 2026-11-18: the market): a major-league player whose
    // contract ran out with six years of service goes to market; players under a guaranteed
    // contract are kept first (`protected`).
    if let Some(m) = markets.get(&Economy::Mlb) {
        let r = rules.of(Economy::Mlb);
        let (svc_arb, svc_fa) = r.service.map_or((3, 6), |s| (s.arbitration, s.free_agency));
        let price = |war: f64| rules.free_agent_pay(war, year + 1);
        let qualifying = r.qualifying_offer.as_ref().map(|s| s.at(year + 1, r.raise.rate));
        for (k, org) in orgs.iter().enumerate() {
            if org.kind != OrgKind::Major {
                continue;
            }
            // Where each of the organisation's players would play next season.
            let mut depth = Depth::new(Economy::Mlb);
            for p in &pools[k] {
                depth.add(rules, p);
            }
            let place = |p: &Player| depth.place(rules, p, m.bench_share);
            let mut go: Vec<(Player, Why, bool)> = Vec::new();
            let mut stay: Vec<Player> = Vec::new();
            for p in std::mem::take(&mut pools[k]) {
                let war = rules.war(&p, Economy::Mlb);
                let service = pop.careers.get(&p.id).map_or(0, |c| first_team_seasons(c, "mlb", year + 1));
                let pinned_here = pinned[k].contains_key(&p.id);
                let (club_lean, player_lean) = rules.options.as_ref().map_or((0.0, 0.0), |o| (o.club_lean, o.player_lean));
                if let Some(c) = pop.contracts.get_mut(&p.id).filter(|c| c.kind == Kind::Major && !pinned_here) {
                    // An opt-out after this season: he leaves when the price he expects is above
                    // his pay.
                    if c.opt_out == Some(year) && c.covers(year + 1) {
                        c.opt_out = None;
                        report.opt_outs_due += 1;
                        if price(war) * (1.0 + player_lean) > c.pay(year + 1).unwrap_or(0.0) {
                            c.salary.truncate((year - c.from + 1).max(0) as usize);
                            c.option = None;
                            report.opted_out += 1;
                        }
                    }
                    // A club option on next season: exercised when he is worth its salary less
                    // the buyout to the organisation.
                    if c.last() == year {
                        if let Some(o) = c.option.take() {
                            if price(war * place(&p)) * (1.0 + club_lean) >= o.salary - o.buyout {
                                c.salary.push(o.salary);
                                report.options_exercised += 1;
                            } else {
                                report.options_declined += 1;
                            }
                        }
                    }
                }
                let Some(c) = pop.contracts.get(&p.id).filter(|c| c.kind == Kind::Major && !c.covers(year + 1) && !pinned_here) else {
                    if pop.contracts.get(&p.id).is_some_and(|c| c.kind == Kind::Major && c.covers(year + 1)) {
                        guaranteed.insert(p.id);
                    }
                    stay.push(p);
                    continue;
                };
                let previous = c.pay(year).unwrap_or(0.0);
                if service >= svc_fa {
                    // Free agency. The organisation makes the qualifying offer (once in his
                    // career) when his price is near it; he takes it when his price is below.
                    let first = pop.careers.get(&p.id).is_none_or(|c| c.qualifying_offer.is_none());
                    let ask = price(war);
                    match qualifying.filter(|q| first && ask >= m.qualifying_offer * q) {
                        Some(q) => {
                            report.qualifying_offers += 1;
                            if let Some(c) = pop.careers.get_mut(&p.id) {
                                c.qualifying_offer = Some(year);
                            }
                            if ask < q {
                                report.qualifying_accepted += 1;
                                pop.contracts.insert(p.id, Contract::new(&org.id, Kind::Major, Terms::FreeAgent, year + 1, &[q]));
                                guaranteed.insert(p.id);
                                stay.push(p);
                            } else {
                                go.push((p, Why::Service, true));
                            }
                        }
                        None => go.push((p, Why::Service, false)),
                    }
                } else if service >= svc_arb {
                    // Arbitration: not tendered when his salary would be more than he is worth
                    // to the organisation (the salary may not fall by more than the limit).
                    let pay = rules.arbitration_pay(&p, war * place(&p), year + 1, service - svc_arb + 1).max(rules.lowest_renewal(Economy::Mlb, previous));
                    if pay > price(war * place(&p)) {
                        report.non_tendered += 1;
                        go.push((p, Why::LetGo, false));
                    } else {
                        stay.push(p);
                    }
                } else {
                    stay.push(p);
                }
            }
            pools[k] = stay;
            let from = bidder_of.get(&k).copied();
            let list = agents.entry(Economy::Mlb).or_default();
            for (p, why, qualified) in go {
                let age = age_of(p.id, next_season);
                list.push(FreeAgent { qualified, ..FreeAgent::new(p, age, from, why) });
            }
        }
    }

    // Free agency in the KBO and NPB (decision 2026-11-18: each league's rules, and its
    // preference for the old club fitted to its declarations). A player whose contract runs
    // out with the seasons for free agency (`contract::seasons_for_fa`; again `again` seasons
    // after he used it; NPB's domestic right) declares it when the pay he expects on the market
    // is enough above what his club would pay him; one who does not stays reserved and keeps
    // the right. A multi-year contract defers it to its end (the KBO's deferred eligibility),
    // and such players are kept first. Each declared player carries what another club signing
    // him owes his old one: by his grade in the KBO (salary ranks in the club and the league),
    // by his rank in NPB (salary rank in the club; the 2027 rules).
    let mut owed: BTreeMap<u32, Owed> = BTreeMap::new();
    let mut outside_limit: BTreeMap<Economy, usize> = BTreeMap::new();
    for e in [Economy::Kbo, Economy::Npb] {
        let (Some(m), Some(ks)) = (markets.get(&e), members.get(&e)) else { continue };
        let r = rules.of(e);
        let ix = usize::from(e == Economy::Npb);
        let depths: Vec<Depth> = ks
            .iter()
            .map(|&k| {
                let mut d = Depth::new(e);
                for p in &pools[k] {
                    d.add(rules, p);
                }
                d
            })
            .collect();
        // Last season's salaries of the domestic registered players, for grades and ranks: the
        // KBO ranks those not on a running free-agent contract, in the club and in the
        // league; NPB all of the club's Japanese players.
        let ranked = |id: u32| -> Option<f64> {
            if pop.foreign.contains_key(&id) {
                return None;
            }
            let c = pop.contracts.get(&id).filter(|c| c.kind == Kind::Registered && c.terms != Terms::Foreign)?;
            if e == Economy::Kbo && c.terms == Terms::FreeAgent && c.covers(year + 1) {
                return None;
            }
            c.pay(year)
        };
        let club_pays: BTreeMap<usize, Vec<f64>> = ks.iter().map(|&k| (k, pools[k].iter().filter_map(|p| ranked(p.id)).collect())).collect();
        let league_pays: Vec<f64> = club_pays.values().flatten().copied().collect();
        let rank = |v: &[f64], pay: f64| v.iter().filter(|x| **x > pay).count() + 1;
        let terms = r.fa_compensation.as_ref();
        let again = r.seasons_for_fa.unwrap_or_default().again.max(1);
        let mut declared: Vec<FreeAgent> = Vec::new();
        for &k in ks {
            if independent(k) {
                continue;
            }
            let mut stay = Vec::with_capacity(pools[k].len());
            for p in std::mem::take(&mut pools[k]) {
                let id = p.id;
                let contract = pop.contracts.get(&id);
                let running = contract.is_some_and(|c| c.covers(year + 1));
                if running && contract.is_some_and(|c| c.kind == Kind::Registered) {
                    guaranteed.insert(id);
                }
                let registered = contract.is_some_and(|c| c.kind == Kind::Registered && c.terms != Terms::Foreign);
                let open = registered && !running && !pop.foreign.contains_key(&id) && !pinned[k].contains_key(&id);
                let Some(career) = pop.careers.get(&id).filter(|_| open) else {
                    stay.push(p);
                    continue;
                };
                let seasons = first_team_seasons(career, e.top_level(), year + 1);
                let used = career.free_agencies;
                let needed = if used == 0 { seasons_for_fa(r, career) } else { again };
                if seasons.saturating_sub(career.fa_at) < needed {
                    stay.push(p);
                    continue;
                }
                report.fa_eligible[ix] += 1;
                // The pay he expects on the market: his price, raised by the premium of the
                // clubs that would want him as a regular; a bench player's share of it; the
                // minimum if nobody wants him. Against it, what his club would pay him for
                // another season (not below NPB's reduction limit).
                let war = rules.war(&p, e);
                let min = r.minimum(year + 1);
                let ask = crate::market::price(rules, m, e, war, year + 1);
                let places: Vec<f64> = depths.iter().map(|d| d.place(rules, &p, m.bench_share)).collect();
                let regulars = places.iter().filter(|x| **x >= 1.0).count().min(m.max_bidders);
                let expected = if regulars > 0 {
                    ask * (1.0 + m.premium * (regulars - 1) as f64)
                } else if places.iter().any(|x| *x > 0.0) {
                    (m.bench_share * ask).max(min)
                } else {
                    min
                };
                let last = contract.and_then(|c| c.pay(year)).unwrap_or(min);
                let club = rules.club_pay(e, war, year + 1).max(rules.lowest_renewal(e, last));
                if (expected / club).ln() + rng.gauss(0.0, m.taste) <= m.declare {
                    stay.push(p);
                    continue;
                }
                report.fa_declared[ix] += 1;
                let (cash, counts, what) = match terms {
                    Some(FaCompensation::Kbo(t)) => {
                        let born_year = born.get(&id).map_or(year - 30, |b| b.year());
                        let grade = t.grade(rank(&club_pays[&k], last), rank(&league_pays, last), used, year - born_year >= t.veteran_age);
                        let terms = t.terms(grade);
                        (Some(terms.cash_only * last), true, (terms.with_player > 0.0).then_some(Owed::Player { protected: terms.protected }))
                    }
                    Some(FaCompensation::Npb(t)) => {
                        // C ranks and veterans owe nothing and do not count toward the signing
                        // club's limit.
                        let veteran = age_of(id, Date::new(year + 1, 4, 1)) >= f64::from(t.veteran_age);
                        match t.owed(rank(&club_pays[&k], last), veteran) {
                            NpbOwed::Pick(round) => (Some(0.0), true, Some(Owed::Pick(round))),
                            NpbOwed::Cash(share) => (Some(share * last), true, None),
                            NpbOwed::Nothing => (None, false, None),
                        }
                    }
                    None => (None, true, None),
                };
                if let Some(c) = pop.careers.get_mut(&id) {
                    c.free_agencies += 1;
                    c.fa_at = seasons;
                }
                if let Some(o) = what {
                    owed.insert(id, o);
                }
                let age = age_of(id, next_season);
                declared.push(FreeAgent { compensation: cash, counts, ..FreeAgent::new(p, age, bidder_of.get(&k).copied(), Why::Service) });
            }
            pools[k] = stay;
        }
        // The signings of other clubs' free agents a club may make follow the number who
        // declared.
        outside_limit.insert(e, terms.map_or(usize::MAX, |t| t.outside_limit(declared.len())));
        agents.entry(e).or_default().extend(declared);
    }

    // First every organisation decides whom it keeps.
    let mut kept_pools: Vec<Vec<Player>> = Vec::with_capacity(orgs.len());
    let mut let_go: Vec<Player> = Vec::new();
    {
        let deepest_of = |k: usize, p: &Player| shapes[k].deepest(p.is_pitcher(), age_of(p.id, next_season), depth_only.contains(&p.id), international(p.id));
        for (k, org) in orgs.iter().enumerate() {
            let caps = &shapes[k].caps;
            let deepest = |p: &Player| deepest_of(k, p);
            let pots = &pop.potentials;
            let value = |p: &Player| {
                keep_value(p, pots.get(&p.id), ctx.careers.keep_potential)
                    - age_cost(age_of(p.id, next_season), ctx.careers.age_discount_of(org.kind)) * ability::per_sd(p)
            };
            // The players who would make the first team on their ability now (less the same
            // discount for age) stay whatever the prospects behind them promise: no club
            // releases its regulars for draftees (ranked with the growth ahead of them, the
            // KBO's high-school picks pushed the first teams three years below their age). A
            // major-league organisation keeps its Triple-A depth the same way (the 40-man
            // roster and the veterans it calls up); without it Triple-A was two years too young.
            let protected: BTreeSet<u32> = {
                let now = |p: &Player| ability::value(p) - age_cost(age_of(p.id, next_season), ctx.careers.age_discount_of(org.kind)) * ability::per_sd(p);
                let top = if org.kind == OrgKind::Major { 2 } else { 1 };
                let (n_p, n_h) = caps[..top.min(caps.len())].iter().fold((0, 0), |a, c| (a.0 + c.0, a.1 + c.1));
                let (mut ps, mut hs): (Vec<&Player>, Vec<&Player>) = pools[k].iter().partition(|p| p.is_pitcher());
                ps.sort_by(|a, b| now(b).total_cmp(&now(a)));
                hs.sort_by(|a, b| now(b).total_cmp(&now(a)));
                ps.iter()
                    .take(n_p)
                    .chain(hs.iter().take(n_h))
                    .map(|p| p.id)
                    .chain(pools[k].iter().filter(|p| guaranteed.contains(&p.id)).map(|p| p.id))
                    .collect()
            };
            // Two catchers and six starters (a rotation and a spare) for every level.
            let (kept, gone) =
                keep(std::mem::take(&mut pools[k]), &value, caps, (2 * org.levels.len(), 6 * org.levels.len()), &deepest, &pinned[k], &protected);
            kept_pools.push(kept);
            // The players a first team's organisation lets go go to its economy's market; the
            // independent and military clubs' leave.
            let market = economy_of(org).filter(|_| !independent(k));
            for p in gone {
                match market {
                    Some(e) => {
                        let why = if depth_only.contains(&p.id) { Why::MinorContract } else { Why::LetGo };
                        let age = age_of(p.id, next_season);
                        agents.entry(e).or_default().push(FreeAgent::new(p, age, bidder_of.get(&k).copied(), why));
                    }
                    None => let_go.push(p),
                }
            }
        }
    }
    for p in let_go {
        if depth_only.contains(&p.id) {
            leave(pop, p.id, Exit::FreeAgent);
            report.minor_free_agents += 1;
        } else {
            leave(pop, p.id, Exit::Released);
            report.released += 1;
        }
    }

    // The markets: every club bids for its economy's free agents with what its budget leaves
    // after the salaries it already owes for the next season (major-league contracts; the
    // KBO's and NPB's registered domestic players, the reserved ones at the pay their club
    // would set).
    let mut signed_by: BTreeSet<u32> = BTreeSet::new();
    let budgets = crate::contract::Budgets::bundled();
    for (&e, ks) in &members {
        let m = &markets[&e];
        let r = rules.of(e);
        let ix = Economy::ALL.iter().position(|x| *x == e).unwrap_or(0);
        let list = agents.remove(&e).unwrap_or_default();
        report.free_agents[ix] += list.len();
        let svc = r.service.map_or((3, 6), |s| (s.arbitration, s.free_agency));
        let mut bidders: Vec<Bidder> = Vec::with_capacity(ks.len());
        for &k in ks {
            let mut depth = Depth::new(e);
            let mut committed = 0.0;
            for p in &kept_pools[k] {
                depth.add(rules, p);
                if pop.foreign.contains_key(&p.id) {
                    continue;
                }
                let Some(c) = pop.contracts.get(&p.id).filter(|c| matches!(c.kind, Kind::Major | Kind::Registered) && c.terms != Terms::Foreign) else {
                    continue;
                };
                committed += match c.pay(year + 1) {
                    Some(pay) => pay,
                    None if e == Economy::Mlb => {
                        let service = pop.careers.get(&p.id).map_or(0, |c| first_team_seasons(c, "mlb", year + 1));
                        if service < svc.0 {
                            r.minimum(year + 1)
                        } else {
                            rules.arbitration_pay(p, rules.war(p, Economy::Mlb), year + 1, service - svc.0 + 1)
                        }
                    }
                    None => rules.club_pay(e, rules.war(p, e), year + 1).max(rules.lowest_renewal(e, c.pay(year).unwrap_or(0.0))),
                };
            }
            let depth_club = independent(k);
            let budget = if depth_club { 0.0 } else { budgets.of(&orgs[k].id, e, year + 1, rules).unwrap_or(0.0) };
            let outside_left = outside_limit.get(&e).copied().filter(|_| !depth_club);
            bidders.push(Bidder { depth, committed, budget, depth_signed: 0, outside_left, depth_only: depth_club });
        }
        let org_of: &[usize] = ks;
        // Status for the compensation of the majors' qualified free agents: the organisations
        // that paid the luxury tax on the season just played, and those taken to receive
        // revenue sharing (the smallest budgets).
        let (tax_payer, payee, guarantee_line) = if e == Economy::Mlb {
            let payroll_of =
                |k: usize| -> f64 { pop.contracts.values().filter(|c| c.org == orgs[k].id && c.kind == Kind::Major).filter_map(|c| c.pay(year)).sum() };
            let threshold = r.tax.as_ref().map_or(f64::INFINITY, |t| Schedule::lookup_at(&t.thresholds, year, r.raise.rate));
            let tax_payer: BTreeSet<usize> = org_of.iter().copied().filter(|&k| payroll_of(k) > threshold).collect();
            let mut by_budget: Vec<(f64, usize)> = org_of.iter().map(|&k| (budgets.of(&orgs[k].id, e, year + 1, rules).unwrap_or(0.0), k)).collect();
            by_budget.sort_by(|a, b| a.0.total_cmp(&b.0));
            let payee: BTreeSet<usize> = by_budget.iter().take(m.revenue_sharing_payees).map(|x| x.1).collect();
            (tax_payer, payee, m.compensation_guarantee * (1.0 + r.raise.rate).powi((year + 1 - 2026).max(0)))
        } else {
            (BTreeSet::new(), BTreeSet::new(), f64::INFINITY)
        };
        let outcome = crate::market::run(rules, m, e, list, &mut bidders, year + 1, &mut rng);
        for s in outcome.signed {
            let k = org_of[s.org];
            let id = s.player.id;
            let old = s.from.map(|b| org_of[b]);
            if s.major {
                report.signed_major[ix] += 1;
            } else {
                report.signed_minor[ix] += 1;
            }
            if e != Economy::Mlb && s.why == Why::Service {
                if old == Some(k) {
                    report.fa_stayed[ix - 1] += 1;
                } else {
                    report.fa_moved[ix - 1] += 1;
                }
            }
            let contract = match e {
                Economy::Mlb if s.major => {
                    let mut c = Contract::new(&orgs[k].id, Kind::Major, Terms::FreeAgent, year + 1, &vec![s.pay; s.years]);
                    c.option = s.option.clone();
                    c.opt_out = s.opt_out_after.map(|a| year + a as i32);
                    // A qualified free agent who signs with another organisation: a pick for the
                    // one he left, picks given up by the one he joined (BA Art. XX(B)(4)).
                    if let Some(old) = old.filter(|&o| s.qualified && o != k) {
                        let after = if tax_payer.contains(&old) {
                            4
                        } else if payee.contains(&old) && s.pay * s.years as f64 >= guarantee_line {
                            1
                        } else {
                            2
                        };
                        pop.draft_changes.push(DraftChange { year: year + 1, org: orgs[old].id.clone(), round: after, extra: true });
                        report.compensation_picks += 1;
                        let lost: &[u32] = if payee.contains(&k) {
                            &[3]
                        } else if tax_payer.contains(&k) {
                            &[2, 5]
                        } else {
                            &[2]
                        };
                        for &round in lost {
                            pop.draft_changes.push(DraftChange { year: year + 1, org: orgs[k].id.clone(), round, extra: false });
                        }
                    }
                    c
                }
                Economy::Mlb => {
                    depth_only.insert(id);
                    Contract::one_season(&orgs[k].id, Kind::Minor, Terms::FreeAgent, year + 1, s.pay)
                }
                _ if s.major => Contract::new(&orgs[k].id, Kind::Registered, Terms::FreeAgent, year + 1, &vec![s.pay; s.years]),
                _ => {
                    // A second team's place: an NPB club signs a development contract; an
                    // independent club's players are not paid by the model.
                    let kind = if independent(k) {
                        Kind::Other
                    } else if e == Economy::Npb {
                        Kind::Development
                    } else {
                        Kind::Registered
                    };
                    Contract::one_season(&orgs[k].id, kind, Terms::FreeAgent, year + 1, s.pay)
                }
            };
            pop.contracts.insert(id, contract);
            signed_by.insert(id);
            kept_pools[k].push(s.player);
            // Compensation to the old club of a KBO or NPB free agent.
            let Some(old) = old.filter(|&o| o != k && s.why == Why::Service && e != Economy::Mlb) else { continue };
            match owed.get(&id) {
                Some(Owed::Player { protected }) => {
                    // The signing club protects its best `protected` players (by the value it
                    // keeps players by; foreign players, this offseason's signings and
                    // compensation players are off the list); the old club takes the best of
                    // the rest if he is above replacement, and the smaller cash with him,
                    // otherwise the larger cash alone.
                    let pots = &pop.potentials;
                    let value = |p: &Player| {
                        keep_value(p, pots.get(&p.id), ctx.careers.keep_potential)
                            - age_cost(age_of(p.id, next_season), ctx.careers.age_discount_of(orgs[k].kind)) * ability::per_sd(p)
                    };
                    let mut list: Vec<(f64, usize)> = kept_pools[k]
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| !pop.foreign.contains_key(&p.id) && !signed_by.contains(&p.id) && !pinned[k].contains_key(&p.id))
                        .map(|(i, p)| (value(p), i))
                        .collect();
                    list.sort_by(|a, b| b.0.total_cmp(&a.0));
                    let taken = list.get(*protected).map(|x| x.1).filter(|&i| {
                        let p = &kept_pools[k][i];
                        rules.war_at(p, e, keep_value(p, pots.get(&p.id), ctx.careers.keep_potential)) > 0.0
                    });
                    if let Some(i) = taken {
                        let p = kept_pools[k].swap_remove(i);
                        if let Some(c) = pop.contracts.get_mut(&p.id) {
                            c.org = orgs[old].id.clone();
                        }
                        signed_by.insert(p.id);
                        report.compensation_players += 1;
                        kept_pools[old].push(p);
                    }
                }
                Some(Owed::Pick(round)) => {
                    pop.draft_changes.push(DraftChange { year: year + 1, org: orgs[old].id.clone(), round: *round, extra: true });
                    report.npb_compensation_picks += 1;
                }
                _ => {}
            }
        }
        for fa in outcome.unsigned {
            report.unsigned[ix] += 1;
            let exit = if fa.why == Why::LetGo { Exit::Released } else { Exit::FreeAgent };
            leave(pop, fa.player.id, exit);
        }
    }
    let deepest_of = |k: usize, p: &Player| shapes[k].deepest(p.is_pitcher(), age_of(p.id, next_season), depth_only.contains(&p.id), international(p.id));

    // Then every organisation fills its rosters and places its players level by level.
    let mut built: BTreeMap<String, Roster> = BTreeMap::new();
    for (k, org) in orgs.iter().enumerate() {
        let Shape { sizes, caps, academy_at, .. } = &shapes[k];
        let (caps, academy_at) = (caps.as_slice(), *academy_at);
        let deepest = |p: &Player| deepest_of(k, p);
        let mut pool = std::mem::take(&mut kept_pools[k]);
        // An organisation that signed players on the market keeps its players again within
        // its levels' places (the new signings and the guaranteed contracts first): the
        // veterans it signed may only play at the top levels.
        if pool.iter().any(|p| signed_by.contains(&p.id)) {
            let pots = &pop.potentials;
            let value = |p: &Player| {
                keep_value(p, pots.get(&p.id), ctx.careers.keep_potential)
                    - age_cost(age_of(p.id, next_season), ctx.careers.age_discount_of(org.kind)) * ability::per_sd(p)
            };
            let first: BTreeSet<u32> = pool.iter().filter(|p| signed_by.contains(&p.id) || guaranteed.contains(&p.id)).map(|p| p.id).collect();
            let (kept, gone) = keep(pool, &value, caps, (2 * org.levels.len(), 6 * org.levels.len()), &deepest, &pinned[k], &first);
            for p in gone {
                leave(pop, p.id, Exit::Released);
                report.released += 1;
            }
            pool = kept;
        }
        // An organisation whose players cannot fill its rosters signs amateurs nobody drafted
        // (undrafted college players in the majors' organisations, development players in
        // Korea and Japan) for the pitchers' and position players' places left open. Sangmu
        // holds only the players who serve there.
        let fill = match org.kind {
            OrgKind::Major => Some("us_college"),
            OrgKind::Korean => Some("kbo_college"),
            OrgKind::Japanese => Some("npb_development"),
            OrgKind::KoreanIndependent => Some("kr_independent"),
            OrgKind::JapaneseIndependent => Some("jp_independent"),
            OrgKind::Military => None,
        };
        if let Some(class) = fill.and_then(|c| ctx.careers.classes.get(c)) {
            let (cap_p, cap_h) = caps.iter().fold((0, 0), |a, c| (a.0 + c.0, a.1 + c.1));
            let have_p = pool.iter().filter(|p| p.is_pitcher()).count();
            let open = [(true, cap_p.saturating_sub(have_p)), (false, cap_h.saturating_sub(pool.len() - have_p))];
            for (is_pitcher, n) in open {
                for _ in 0..n {
                    let newcomer = intake.amateur_as(class, is_pitcher, &mut rng);
                    let p = add(pop, newcomer, Origin::Signed { year, class: fill.unwrap_or_default().to_string() });
                    pool.push(p);
                    report.signed += 1;
                }
            }
        }
        let lowest = |p: &Player| pinned[k].get(&p.id).copied().or_else(|| deepest(p)).unwrap_or(0);
        let mut academy: Option<Vec<Player>> = None;
        let mut placed: Vec<Vec<Player>> = Vec::with_capacity(org.levels.len());
        for (i, (_, key)) in org.levels.iter().enumerate() {
            // The academy takes the youngest international amateurs, before the complex
            // takes its players from everyone left.
            let youngest = |pool: &mut Vec<Player>, size: usize| -> Vec<Player> {
                let (mut eligible, others): (Vec<Player>, Vec<Player>) = std::mem::take(pool).into_iter().partition(|p| international(p.id));
                let out = take_youngest(&mut eligible, size, &|id| age_of(id, next_season));
                pool.extend(eligible);
                pool.extend(others);
                out
            };
            let players = if key == "academy" {
                academy.take().unwrap_or_else(|| youngest(&mut pool, sizes[i]))
            } else {
                if key == "complex" {
                    if let Some(j) = academy_at {
                        academy = Some(youngest(&mut pool, sizes[j]));
                    }
                }
                // Foreign players at their level, and whoever can go no lower.
                let mut here: BTreeSet<u32> = pinned[k].iter().filter(|(_, l)| **l == i).map(|(id, _)| *id).collect();
                for pitchers in [true, false] {
                    here.extend(forced(&pool, i, pitchers, caps, &lowest));
                }
                take_level(&mut pool, sizes[i], &here)
            };
            placed.push(players);
        }
        // A rotation needs five pitchers with a starter's stamina. A level the players who
        // could go no lower left short of them trades its weakest relievers, one for one, for
        // the best starters of the levels below, as long as the reliever may play at that
        // level (age limit; the academy and a club's foreign players stay out of it): the
        // organisation sends a reliever down rather than open the season with bullpen games.
        let academy_level = academy_at.unwrap_or(usize::MAX);
        let rate = ability::pitching_rate;
        for i in 0..placed.len() {
            while placed[i].iter().filter(|p| p.is_pitcher() && ability::can_start(p)).count() < 5 {
                let best = (i + 1..placed.len())
                    .filter(|&j| j != academy_level)
                    .flat_map(|j| placed[j].iter().enumerate().map(move |(q, p)| (j, q, p)))
                    .filter(|(_, _, p)| p.is_pitcher() && ability::can_start(p) && !pinned[k].contains_key(&p.id))
                    .max_by(|a, b| rate(a.2).total_cmp(&rate(b.2)))
                    .map(|(j, q, _)| (j, q));
                let Some((j, q)) = best else { break };
                let relievers = |placed: &[Vec<Player>], down: bool| {
                    placed[i]
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.is_pitcher() && !ability::can_start(p) && !pinned[k].contains_key(&p.id) && (!down || lowest(p) >= j))
                        .min_by(|a, b| rate(a.1).total_cmp(&rate(b.1)))
                        .map(|(w, _)| w)
                };
                if let Some(w) = relievers(&placed, true) {
                    // j > i: swap across the two levels.
                    let (upper, lower) = placed.split_at_mut(j);
                    std::mem::swap(&mut upper[i][w], &mut lower[0][q]);
                    continue;
                }
                // No reliever may go down (veterans kept as Triple-A depth, or players at
                // their age limit): the organisation lets the weakest go, brings the starter
                // up and fills his place from the players left over, if one may play there.
                let Some(w) = relievers(&placed, false) else { break };
                let gone = placed[i].swap_remove(w);
                let starter = placed[j].swap_remove(q);
                placed[i].push(starter);
                if let Some(r) =
                    pool.iter().enumerate().filter(|(_, p)| p.is_pitcher() && lowest(p) >= j).max_by(|a, b| rate(a.1).total_cmp(&rate(b.1))).map(|(r, _)| r)
                {
                    placed[j].push(pool.swap_remove(r));
                }
                if depth_only.contains(&gone.id) {
                    report.minor_free_agents += 1;
                } else {
                    report.released += 1;
                }
                let exit = if depth_only.contains(&gone.id) { Exit::FreeAgent } else { Exit::Released };
                leave(pop, gone.id, exit);
            }
        }
        for ((club, key), players) in org.levels.iter().zip(placed) {
            built.insert(club.clone(), build_roster(club, key, players));
        }
        for p in pool {
            leave(pop, p.id, Exit::Released);
            report.released += 1;
        }
    }
    pop.rosters = order.into_iter().filter_map(|c| built.remove(&c)).collect();
    pop.rosters.extend(built.into_values());
    pop.date = next_season;
    record_season(ctx.world, pop, year + 1);
    crate::contract::settle(ctx.world, pop, ctx.contracts, year + 1, seed);
    report
}

/// The talent offsets (in the order of `levels.json`: hitting, pitching, fielding,
/// physical, power on top of hitting, command on top of pitching, velocity on top of
/// pitching, plate discipline on top of hitting) that a group of players shows on average.
pub fn talent_of<'a>(players: impl Iterator<Item = &'a Player>) -> [f64; 8] {
    use crate::attr::Attr::*;
    let mean = |p: &Player, a: &[crate::attr::Attr]| a.iter().map(|&x| p.attrs.z(x)).sum::<f64>() / a.len() as f64;
    let (mut h, mut pw, mut f, mut n_h) = (0.0, 0.0, 0.0, 0.0);
    let (mut pt, mut cm, mut vel, mut n_p) = (0.0, 0.0, 0.0, 0.0);
    let (mut ph, mut n, mut dis) = (0.0, 0.0, 0.0);
    for p in players {
        ph += mean(p, &[Strength, Durability, Agility]);
        n += 1.0;
        if p.is_pitcher() {
            let relief = if ability::can_start(p) { 0.0 } else { 0.27 };
            let base = mean(p, &[Stuff, Deception]);
            pt += base;
            cm += mean(p, &[Control, Command]) - base;
            vel += p.attrs.z(Velocity) - relief - base;
            n_p += 1.0;
        } else {
            let hit = mean(p, &[Contact, Timing, BatControl, Recognition]);
            h += hit;
            pw += p.attrs.z(Power) - hit;
            dis += p.attrs.z(Discipline) - hit;
            f += mean(p, &[Hands, Range, Reads, ArmAccuracy, Footwork, Baserunning]);
            n_h += 1.0;
        }
    }
    let d = |x: f64, k: f64| if k > 0.0 { x / k } else { 0.0 };
    [d(h, n_h), d(pt, n_p), d(f, n_h), d(ph, n), d(pw, n_h), d(cm, n_p), d(vel, n_p), d(dis, n_h)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::populate::populate;

    #[test]
    fn the_career_file_is_consistent() {
        let t = CareerTable::bundled();
        let levels = LevelTable::bundled();
        for (name, c) in &t.classes {
            assert!(levels.levels.contains_key(&c.like), "class {name}: no level '{}'", c.like);
            assert!((0.0..=1.0).contains(&c.pitchers), "{name}");
        }
        for d in &t.drafts {
            let total: f64 = d.classes.iter().map(|c| c.1).sum();
            assert!((total - 1.0).abs() < 0.01, "draft of {}: shares add to {total}", d.orgs);
        }
        assert_eq!(t.retire_chance(30.0), 0.0);
        assert!(t.retire_chance(38.0) > t.retire_chance(34.0));
        assert_eq!(t.retire_chance(60.0), 1.0);
    }

    #[test]
    fn korean_players_serve_their_time_and_come_back() {
        let (world, levels, aging, careers, people) = ctx_parts();
        let ctx = Offseason {
            world: &world,
            levels: &levels,
            aging: &aging,
            careers: &careers,
            people: &people,
            contracts: &crate::contract::ContractRules::bundled(),
        };
        let ms = careers.military.expect("careers.json: military_service");
        let base = world.clubs().find(|c| c.military).map(|c| c.id.clone()).unwrap();
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 7);
        let mut back = 0;
        for year in 2027..2035 {
            let r = offseason(&ctx, &mut pop, year, None, 7);
            assert!(r.enlisted > 20 && r.sangmu <= ms.sangmu_per_year && r.sangmu <= r.enlisted, "{year}: {r:?}");
            back += r.discharged;
        }
        assert!(back > 100, "{back} back from service");
        // Those away serve on active duty for the KBO club that holds their rights, back
        // within two seasons; Sangmu's players all serve there.
        for (id, a) in &pop.away {
            match &pop.careers[id].service {
                Some(Service::Serving { sangmu: false, org, back }) => assert!(org == &a.org && (2036..=2037).contains(back), "{id}: {org} {back}"),
                other => panic!("{id} away with {other:?}"),
            }
            assert!(world.club(&a.org).is_some_and(|c| c.country == "KR" && c.parent.is_none()));
        }
        for p in pop.roster(&base).unwrap().players() {
            assert!(matches!(pop.careers[&p.id].service, Some(Service::Serving { sangmu: true, .. })), "{} at Sangmu", p.id);
        }
        // Nobody of the deadline's age is left at a KBO club without having served.
        let end = Date::new(2034, 12, 31);
        let kbo: BTreeSet<String> =
            organisations(&world).into_iter().filter(|o| o.kind == OrgKind::Korean).flat_map(|o| o.levels.into_iter().map(|l| l.0)).collect();
        for r in pop.rosters.iter().filter(|r| kbo.contains(&r.club)) {
            for p in r.players().filter(|p| !pop.foreign.contains_key(&p.id)) {
                let age = end.days_since(pop.people[&p.id].born) as f64 / 365.2425;
                if age >= ms.deadline_age {
                    assert!(matches!(pop.careers[&p.id].service, Some(Service::Done { .. })), "{} ({age:.1}) at {}", p.id, r.club);
                }
            }
        }
    }

    #[test]
    fn an_asian_games_gold_exempts_the_team() {
        let (world, levels, aging, mut careers, people) = ctx_parts();
        let ms = careers.military.as_mut().unwrap();
        let ag = ms.asian_games.as_mut().unwrap();
        (ag.first, ag.gold) = (2027, 1.0);
        let team = ag.team;
        let ctx = Offseason {
            world: &world,
            levels: &levels,
            aging: &aging,
            careers: &careers,
            people: &people,
            contracts: &crate::contract::ContractRules::bundled(),
        };
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 7);
        let r = offseason(&ctx, &mut pop, 2027, None, 7);
        assert!(r.exempt > 0 && r.exempt <= team, "{r:?}");
        let exempt = pop.careers.values().filter(|c| matches!(c.service, Some(Service::Done { exempt: true, .. }))).count();
        assert_eq!(exempt, r.exempt);
    }

    #[test]
    fn minor_league_contracts_run_six_or_seven_seasons() {
        let terms = CareerTable::bundled().minor_contract.expect("careers.json: minor_league_contract");
        assert_eq!((terms.seasons, terms.seasons_if_younger, terms.forty_man, terms.stay_as_depth), (6, 7, 40, true));
        let contract_ran_out = |o: &Origin, b: Date, y: i32| contract_ran_out(o, b, y, &terms);
        let draft = |year| Origin::Draft { year, round: 3, pick: 75, class: "us_college".into() };
        // A college pick (21.9 on draft day) plays part of 2030 and five more seasons.
        let college = Date::new(2008, 9, 1);
        assert!(!contract_ran_out(&draft(2030), college, 2034));
        assert!(contract_ran_out(&draft(2030), college, 2035));
        // A high-school pick (18.6) gets seven.
        let school = Date::new(2011, 12, 1);
        assert!(!contract_ran_out(&draft(2030), school, 2035));
        assert!(contract_ran_out(&draft(2030), school, 2036));
        // An international amateur signed that offseason (16.9 on 15 January 2031) starts in
        // 2031 and gets seven.
        let intl = Origin::Signed { year: 2030, class: "international".into() };
        let teen = Date::new(2014, 3, 1);
        assert!(!contract_ran_out(&intl, teen, 2036));
        assert!(contract_ran_out(&intl, teen, 2037));
        // Players of the first world and foreign players are left alone.
        assert!(!contract_ran_out(&Origin::Initial, college, 2060));
        assert!(!contract_ran_out(&Origin::Foreign { year: 2030 }, college, 2060));
    }

    #[test]
    fn every_roster_belongs_to_one_organisation() {
        let world = World::bundled();
        let orgs = organisations(&world);
        let count = |k: OrgKind| orgs.iter().filter(|o| o.kind == k).count();
        assert_eq!((count(OrgKind::Major), count(OrgKind::Korean), count(OrgKind::Japanese)), (30, 10, 12));
        assert!(orgs.iter().filter(|o| o.kind == OrgKind::Major).all(|o| o.levels.len() == 7));
        assert_eq!(
            orgs.iter().find(|o| o.kind == OrgKind::Major).unwrap().levels.iter().map(|l| l.1.as_str()).collect::<Vec<_>>(),
            ["mlb", "aaa", "aa", "high_a", "single_a", "complex", "academy"]
        );
        let pop = populate(&world, &LevelTable::bundled(), Date::new(2027, 2, 1), 3);
        let mut seen = BTreeSet::new();
        for o in &orgs {
            for (club, _) in &o.levels {
                assert!(seen.insert(club.clone()), "{club} in two organisations");
            }
        }
        for r in &pop.rosters {
            assert!(seen.contains(&r.club), "{} belongs to no organisation", r.club);
        }
    }

    #[test]
    fn the_assignment_is_optimal() {
        // Greedy would give the first slot its best player (10) and leave the second 1;
        // the best total is 9 + 8.
        assert_eq!(assign(&[vec![10.0, 9.0], vec![8.0, 1.0]]), vec![1, 0]);
        // Three slots, four players: 4 + 3 + 5 = 12 beats every other choice.
        assert_eq!(assign(&[vec![5.0, 4.0, 0.0, 0.0], vec![5.0, 0.0, 3.0, 0.0], vec![5.0, 0.0, 0.0, 2.0]]), vec![1, 2, 0]);
    }

    #[test]
    fn a_roster_has_a_full_squad() {
        let mut rng = Rng::new(21);
        let squad = Squad::random(&mut rng, 100, &Talent::MAJOR);
        let mut players: Vec<Player> = squad.hitters.into_iter().chain(squad.bench).map(|(p, _)| p).collect();
        players.extend(squad.starters.into_iter().chain(squad.relievers));
        players.push(pitcher(&mut rng, 200, Role::Reliever, &Talent::MAJOR));
        players.push(position_player(&mut rng, 201, Some(Position::Left), &Talent::MAJOR));
        let r = build_roster("x", "mlb", players);
        assert_eq!((r.squad.hitters.len(), r.squad.bench.len(), r.squad.starters.len(), r.squad.relievers.len(), r.reserves.len()), (9, 4, 5, 8, 2));
        // The catcher's slot goes to a catcher, the rotation to pitchers who can start.
        let (c, pos) = &r.squad.hitters[0];
        assert_eq!((c.position, *pos), (Position::Catcher, Some(Position::Catcher)));
        assert!(r.squad.starters.iter().all(ability::can_start));
        assert!(r.squad.bench.iter().any(|(p, _)| p.position == Position::Catcher), "a second catcher on the bench");
        // Every regular plays where he is worth the most the positions allow.
        let total: f64 = r.squad.hitters.iter().map(|(p, s)| ability::at_position(p, *s)).sum();
        let mut swapped = r.squad.hitters.clone();
        swapped.swap(4, 6);
        let other: f64 = swapped.iter().map(|(p, _)| p).zip(LINEUP).map(|(p, s)| ability::at_position(p, s)).sum();
        assert!(total >= other);
    }

    fn ctx_parts() -> (World, LevelTable, AgingTable, CareerTable, PeopleMaker) {
        (World::bundled(), LevelTable::bundled(), AgingTable::bundled(), CareerTable::bundled(), PeopleMaker::new())
    }

    #[test]
    fn kbo_and_npb_free_agents_declare_move_and_bring_compensation() {
        let (world, levels, aging, careers, people) = ctx_parts();
        let rules = crate::contract::ContractRules::bundled();
        let ctx = Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people, contracts: &rules };
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 7);
        let japanese: BTreeSet<String> = organisations(&world).into_iter().filter(|o| o.kind == OrgKind::Japanese).map(|o| o.id).collect();
        let mut total = Report::default();
        for year in 2027..2033 {
            let r = offseason(&ctx, &mut pop, year, None, 7);
            for i in 0..2 {
                assert!(r.fa_declared[i] <= r.fa_eligible[i] && r.fa_stayed[i] + r.fa_moved[i] <= r.fa_declared[i], "{r:?}");
                total.fa_declared[i] += r.fa_declared[i];
                total.fa_moved[i] += r.fa_moved[i];
            }
            total.compensation_players += r.compensation_players;
            total.npb_compensation_picks += r.npb_compensation_picks;
            // NPB's compensation picks wait for the next draft; the ones of this one are used.
            let picks = pop.draft_changes.iter().filter(|c| japanese.contains(&c.org)).collect::<Vec<_>>();
            assert_eq!(picks.len(), r.npb_compensation_picks, "{picks:?}");
            assert!(picks.iter().all(|c| c.year == year + 1 && c.extra && (2..=3).contains(&c.round)));
            // A player who declared counts the seasons for the next free agency from then.
            for c in pop.careers.values().filter(|c| c.free_agencies > 0 && c.left.is_none()) {
                assert!(c.fa_at <= crate::contract::first_team_seasons(c, "kbo", year + 1).max(crate::contract::first_team_seasons(c, "npb", year + 1)));
            }
        }
        assert!(total.fa_declared.iter().all(|n| *n > 0) && total.fa_moved.iter().all(|n| *n > 0), "{total:?}");
        assert!(total.compensation_players > 0 && total.npb_compensation_picks > 0, "{total:?}");
    }

    #[test]
    fn offseasons_keep_the_world_whole() {
        let (world, levels, aging, careers, people) = ctx_parts();
        let ctx = Offseason {
            world: &world,
            levels: &levels,
            aging: &aging,
            careers: &careers,
            people: &people,
            contracts: &crate::contract::ContractRules::bundled(),
        };
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 5);
        let start = pop.players().count();
        let mut again = pop.clone();
        for year in 2027..2030 {
            let report = offseason(&ctx, &mut pop, year, None, 5);
            assert!(report.drafted > 600 && report.signed > 1000, "{report:?}");
            // Players leave by retiring, by being let go by a KBO or NPB club, or (in the
            // majors' organisations) by finding no contract on the market.
            assert!(report.retired + report.released + report.unsigned.iter().sum::<usize>() > 1000, "{report:?}");
        }
        assert_eq!(pop.date, Date::new(2030, 2, 1));
        // Every roster as big as its level says, everyone once, with a person, a ceiling
        // and a career.
        let ids: Vec<u32> = pop.players().map(|p| p.id).collect();
        let unique: BTreeSet<u32> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "a player on two rosters");
        // The armed forces' club holds the players serving with it: at most its roster.
        let base = world.clubs().find(|c| c.military).map(|c| c.id.clone()).unwrap();
        let short = levels.get("kbo2").roster - pop.roster(&base).unwrap().players().count();
        for r in &pop.rosters {
            let size = levels.get(&r.level).roster.max(26);
            if r.club == base {
                assert!((26..=size).contains(&r.players().count()), "{}: {}", r.club, r.players().count());
            } else {
                assert_eq!(r.players().count(), size, "{}", r.club);
            }
            assert_eq!((r.squad.hitters.len(), r.squad.starters.len(), r.squad.relievers.len()), (9, 5, 8), "{}", r.club);
        }
        assert_eq!(ids.len() + short, start);
        for id in &ids {
            assert!(pop.people.contains_key(id) && pop.potentials.contains_key(id), "{id}");
            assert_eq!(pop.careers[id].seasons.last().map(|s| s.year), Some(2030));
        }
        // KBO first teams still field three foreign players and an Asia-quota player.
        let kbo = world.clubs().find(|c| c.league == "kr-top").unwrap();
        let foreign = pop.roster(&kbo.id).unwrap().players().filter(|p| pop.foreign.contains_key(&p.id)).count();
        assert_eq!(foreign, 4);
        // The academies stay young.
        let academy_age: f64 = pop
            .rosters
            .iter()
            .filter(|r| r.level == "academy")
            .flat_map(|r| r.players())
            .map(|p| pop.date.days_since(pop.people[&p.id].born) as f64 / 365.2425)
            .sum::<f64>()
            / (30.0 * 35.0);
        assert!(academy_age < 19.5, "academy players are {academy_age:.1} on average");
        // The same seed gives the same world.
        for year in 2027..2030 {
            offseason(&ctx, &mut again, year, None, 5);
        }
        assert_eq!(serde_json::to_string(&again.rosters).unwrap(), serde_json::to_string(&pop.rosters).unwrap());
    }

    /// Mean of `attr` over the domestic pitchers (or position players) of one level.
    fn level_mean(pop: &Population, level: &str, pitchers: bool, attr: crate::attr::Attr) -> f64 {
        let v: Vec<f64> = pop
            .rosters
            .iter()
            .filter(|r| r.level == level)
            .flat_map(|r| r.players())
            .filter(|p| p.is_pitcher() == pitchers && !pop.foreign.contains_key(&p.id))
            .map(|p| p.attrs.z(attr))
            .collect();
        v.iter().sum::<f64>() / v.len().max(1) as f64
    }

    #[test]
    fn twenty_offseasons_keep_the_staffs_and_the_scale() {
        let (world, levels, aging, careers, people) = ctx_parts();
        let ctx = Offseason {
            world: &world,
            levels: &levels,
            aging: &aging,
            careers: &careers,
            people: &people,
            contracts: &crate::contract::ContractRules::bundled(),
        };
        use crate::attr::Attr::*;
        // Character counts for nobody's value, so neither growth nor selection may move it;
        // composure counts for a pitcher's (selection lifts the majors' by about 0.4) and
        // would rise past 1 if prospects grew into it as they grow into their skills. A
        // world's mean moves by about 0.08 between seeds (400 players a side), so the drift
        // is judged on three.
        let traits = [(Determination, 0.15), (Leadership, 0.15), (Teamwork, 0.15), (Composure, 0.6), (Concentration, 0.6)];
        let seeds = [11, 12, 13];
        let mut drift = vec![(0.0, 0.0); traits.len()];
        for seed in seeds {
            let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), seed);
            let before: Vec<(f64, f64)> = traits.iter().map(|&(a, _)| (level_mean(&pop, "mlb", true, a), level_mean(&pop, "mlb", false, a))).collect();
            for year in 2027..2047 {
                offseason(&ctx, &mut pop, year, None, seed);
            }
            // Every first team and Triple-A club still has a rotation of five pitchers with a
            // starter's stamina (valuing starters over their innings once emptied the KBO's).
            for r in pop.rosters.iter().filter(|r| ["mlb", "aaa", "kbo", "npb"].contains(&r.level.as_str())) {
                assert!(r.squad.starters.iter().all(ability::can_start), "seed {seed}, {}: rotation without a starter's stamina", r.club);
            }
            for (k, &(a, _)) in traits.iter().enumerate() {
                drift[k].0 += (level_mean(&pop, "mlb", true, a) - before[k].0) / seeds.len() as f64;
                drift[k].1 += (level_mean(&pop, "mlb", false, a) - before[k].1) / seeds.len() as f64;
            }
        }
        // What the levels do not differ in stays where it was in the majors: no growth
        // lifts every new generation (composure of major-league pitchers had reached +1.2).
        for (k, &(a, bound)) in traits.iter().enumerate() {
            let (p, h) = drift[k];
            assert!(p.abs() < bound && h.abs() < bound, "{a:?}: pitchers {p:+.2}, hitters {h:+.2} over 20 offseasons");
        }
    }
}
