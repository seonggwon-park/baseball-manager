//! Season to season: every player develops and ages, some retire, new players come in
//! through the drafts and the amateur signings, KBO and NPB clubs weigh new foreign
//! players against their own, and every organisation places its players at its levels by
//! ability and lets go of those it has no room for.
//!
//! Who comes in and how clubs choose is in the editable file `data/people/careers.json`
//! (`docs/data-files.md`); what a player is worth is `ability.rs`, how he develops
//! `develop.rs`. Clubs here see every player's true ability and ceiling: scouting comes
//! later. Players who leave an organisation leave the world: there is no market between
//! clubs yet.

use crate::ability;
use crate::calendar::Date;
use crate::develop::{develop, AgingTable, Closure, Potential};
use crate::generate::{pitcher, position_player, Squad, Talent};
use crate::league::{Level, World};
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

#[derive(Deserialize)]
struct CareerFile {
    classes: BTreeMap<String, Class>,
    drafts: Vec<Draft>,
    signings: Vec<Signing>,
    positions: Positions,
    starter_share: Valued<f64>,
    retirement: Retirement,
    keep_potential: Valued<f64>,
    board_noise: Valued<f64>,
    foreign_candidates: Valued<usize>,
    balance: Closure,
    prospect_spread: Valued<f64>,
    pitcher_potential: Valued<f32>,
    #[serde(default)]
    pitcher_shift: Option<Valued<f32>>,
    age_discount: Valued<f64>,
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
    pub retire_from: u32,
    pub retire_chance: Vec<f64>,
    pub keep_potential: f64,
    pub board_noise: f64,
    pub foreign_candidates: usize,
    pub balance: Closure,
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
}

fn position_code(code: &str) -> Result<Option<Position>, String> {
    if code == "DH" {
        return Ok(None);
    }
    serde_json::from_value(serde_json::Value::String(code.to_string())).map(Some).map_err(|_| format!("unknown position '{code}'"))
}

impl CareerTable {
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
            retire_from: f.retirement.from_age,
            retire_chance: f.retirement.chance,
            keep_potential: f.keep_potential.value,
            board_noise: f.board_noise.value,
            foreign_candidates: f.foreign_candidates.value,
            balance: f.balance,
            prospect_spread: f.prospect_spread.value,
            pitcher_potential: f.pitcher_potential.value,
            pitcher_shift: f.pitcher_shift.map_or(0.0, |v| v.value),
            age_discount: f.age_discount.value,
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
}

impl Career {
    pub fn new(origin: Origin) -> Career {
        Career { origin, seasons: Vec::new(), majors: None, left: None }
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
/// `pinned` players (with their level) always are.
fn keep(
    players: Vec<Player>,
    value: &dyn Fn(&Player) -> f64,
    caps: &[(usize, usize)],
    (catchers, starters): (usize, usize),
    deepest: &dyn Fn(&Player) -> Option<usize>,
    pinned: &BTreeMap<u32, usize>,
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
    // Pinned players, then catchers and starters up to their minimums, then everyone else.
    let (mut k_c, mut k_s) = (0, 0);
    let mut rest = Vec::new();
    for (v, p) in scored {
        let catcher = !p.is_pitcher() && p.position == Position::Catcher;
        let starter = p.is_pitcher() && ability::can_start(&p);
        let first = pinned.contains_key(&p.id) || (catcher && k_c < catchers) || (starter && k_s < starters);
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
        let t = self.table.talent(class, self.levels);
        let id = self.id();
        let is_pitcher = rng.chance(class.pitchers);
        let player = if is_pitcher {
            let role = if rng.chance(self.table.starter_share) { Role::Starter } else { Role::Reliever };
            let t = Talent { pitching: t.pitching + self.table.pitcher_shift, ..t };
            pitcher(rng, id, role, &t)
        } else {
            let pos = self.position(rng);
            position_player(rng, id, pos, &t)
        };
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

    // Everyone off the rosters, by club.
    let mut by_club: BTreeMap<String, Vec<Player>> = BTreeMap::new();
    let order: Vec<String> = pop.rosters.iter().map(|r| r.club.clone()).collect();
    for r in std::mem::take(&mut pop.rosters) {
        let Roster { club, squad, reserves, .. } = r;
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
    for players in by_club.values_mut() {
        let mut stay = Vec::with_capacity(players.len());
        for mut p in players.drain(..) {
            let age = age_of(p.id, date);
            let pot = pop.potentials.entry(p.id).or_insert_with(|| Potential { attrs: p.attrs });
            develop(&mut p, pot, age, ctx.aging, &mut rng);
            if rng.chance(ctx.careers.retire_chance(age)) {
                leave(pop, p.id, Exit::Retired);
                report.retired += 1;
            } else {
                stay.push(p);
            }
        }
        *players = stay;
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
        let total = draft.rounds as usize * n_orgs;
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
            let round = (pick / n_orgs) as u32 + 1;
            let chance = draft.signed.iter().find(|(last, _)| round <= *last).map_or(1.0, |x| x.1);
            if !rng.chance(chance) {
                continue;
            }
            let org = &picking[pick % n_orgs];
            let k = orgs.iter().position(|o| &o.id == org).unwrap();
            let p = add(pop, newcomer, Origin::Draft { year, round, pick: pick as u32 + 1, class });
            pools[k].push(p);
            report.drafted += 1;
        }
    }

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
    let mut built: BTreeMap<String, Roster> = BTreeMap::new();
    for (k, org) in orgs.iter().enumerate() {
        let sizes: Vec<usize> = org.levels.iter().map(|(_, key)| ctx.levels.get(key).roster.max(26)).collect();
        let caps: Vec<(usize, usize)> = sizes.iter().map(|&s| roster_split(s)).collect();
        // Each level takes players up to the oldest age of its age spread (levels.json);
        // the limits fall from the top level down.
        let limits: Vec<(f64, f64)> = org.levels.iter().map(|(_, key)| (ctx.levels.get(key).batters.max, ctx.levels.get(key).pitchers.max)).collect();
        // The Dominican academy is for international amateurs only (MLB's Dominican Summer
        // League rosters are 99.5 % born outside the United States; birthplaces.json).
        let academy_at = org.levels.iter().position(|(_, key)| key == "academy");
        let international = |id: u32| intl.contains(&id);
        let deepest = |p: &Player| -> Option<usize> {
            let age = age_of(p.id, next_season);
            let d = limits.iter().take_while(|(b, pt)| age <= if p.is_pitcher() { *pt } else { *b }).count().checked_sub(1)?;
            match academy_at {
                Some(a) if d >= a && !international(p.id) => a.checked_sub(1),
                _ => Some(d),
            }
        };
        let pots = &pop.potentials;
        let value = |p: &Player| {
            keep_value(p, pots.get(&p.id), ctx.careers.keep_potential) - age_cost(age_of(p.id, next_season), ctx.careers.age_discount) * ability::per_sd(p)
        };
        // Two catchers and six starters (a rotation and a spare) for every level.
        let (kept, gone) = keep(std::mem::take(&mut pools[k]), &value, &caps, (2 * org.levels.len(), 6 * org.levels.len()), &deepest, &pinned[k]);
        for p in gone {
            leave(pop, p.id, Exit::Released);
            report.released += 1;
        }
        let lowest = |p: &Player| pinned[k].get(&p.id).copied().or_else(|| deepest(p)).unwrap_or(0);
        let mut pool = kept;
        let mut academy: Option<Vec<Player>> = None;
        for (i, (club, key)) in org.levels.iter().enumerate() {
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
                    here.extend(forced(&pool, i, pitchers, &caps, &lowest));
                }
                take_level(&mut pool, sizes[i], &here)
            };
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
    report
}

/// The talent offsets (in the order of `levels.json`: hitting, pitching, fielding,
/// physical, power on top of hitting, command on top of pitching, velocity on top of
/// pitching) that a group of players shows on average.
pub fn talent_of<'a>(players: impl Iterator<Item = &'a Player>) -> [f64; 7] {
    use crate::attr::Attr::*;
    let mean = |p: &Player, a: &[crate::attr::Attr]| a.iter().map(|&x| p.attrs.z(x)).sum::<f64>() / a.len() as f64;
    let (mut h, mut pw, mut f, mut n_h) = (0.0, 0.0, 0.0, 0.0);
    let (mut pt, mut cm, mut vel, mut n_p) = (0.0, 0.0, 0.0, 0.0);
    let (mut ph, mut n) = (0.0, 0.0);
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
            let hit = mean(p, &[Contact, Timing, BatControl, Discipline, Recognition]);
            h += hit;
            pw += p.attrs.z(Power) - hit;
            f += mean(p, &[Hands, Range, Reads, ArmAccuracy, Footwork, Baserunning]);
            n_h += 1.0;
        }
    }
    let d = |x: f64, k: f64| if k > 0.0 { x / k } else { 0.0 };
    [d(h, n_h), d(pt, n_p), d(f, n_h), d(ph, n), d(pw, n_h), d(cm, n_p), d(vel, n_p)]
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
    fn offseasons_keep_the_world_whole() {
        let (world, levels, aging, careers, people) = ctx_parts();
        let ctx = Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people };
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 5);
        let start = pop.players().count();
        let mut again = pop.clone();
        for year in 2027..2030 {
            let report = offseason(&ctx, &mut pop, year, None, 5);
            assert!(report.drafted > 600 && report.signed > 1000, "{report:?}");
            assert!(report.retired + report.released > 1000, "{report:?}");
        }
        assert_eq!(pop.date, Date::new(2030, 2, 1));
        // Every roster as big as its level says, everyone once, with a person, a ceiling
        // and a career.
        let ids: Vec<u32> = pop.players().map(|p| p.id).collect();
        let unique: BTreeSet<u32> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "a player on two rosters");
        assert_eq!(ids.len(), start);
        for r in &pop.rosters {
            assert_eq!(r.players().count(), levels.get(&r.level).roster.max(26), "{}", r.club);
            assert_eq!((r.squad.hitters.len(), r.squad.starters.len(), r.squad.relievers.len()), (9, 5, 8), "{}", r.club);
        }
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
}
