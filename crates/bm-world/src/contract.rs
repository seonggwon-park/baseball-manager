//! Contracts, salaries and club budgets (`docs/contracts.md`).
//!
//! Each economy (the major-league organisations, the KBO, NPB) has its own currency, rules
//! and yearly raise (`data/people/contracts.json`), and each first team a payroll budget
//! (`data/leagues/budgets.json`). A player's worth is measured in wins above replacement
//! (WAR) on FanGraphs' scale. This stage keeps the books: every player has a contract and a
//! salary, but who stays and who goes is still decided by `career.rs` alone.

use crate::ability;
use crate::calendar::Date;
use crate::career::{organisations, Career, Org, OrgKind, Origin};
use crate::league::World;
use crate::player::Player;
use crate::populate::Population;
use bm_engine::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const RULES: &str = include_str!("../../../data/people/contracts.json");
const BUDGETS: &str = include_str!("../../../data/leagues/budgets.json");

/// Players on a major-league organisation's 40-man roster, the ones with major-league
/// contracts (Major League Rules 2026, Rule 2(a)(2)(A)).
pub const FORTY_MAN: usize = 40;

/// Registered (支配下) players an NPB club may hold (野球協約 2025; `npb_notes.md` D1).
pub const NPB_REGISTERED: usize = 70;

/// The three economies, each with its own currency, rules and budgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Economy {
    Mlb,
    Kbo,
    Npb,
}

impl Economy {
    pub const ALL: [Economy; 3] = [Economy::Mlb, Economy::Kbo, Economy::Npb];

    pub fn key(self) -> &'static str {
        match self {
            Economy::Mlb => "mlb",
            Economy::Kbo => "kbo",
            Economy::Npb => "npb",
        }
    }

    pub fn of(kind: OrgKind) -> Economy {
        match kind {
            OrgKind::Major => Economy::Mlb,
            OrgKind::Korean | OrgKind::KoreanIndependent | OrgKind::Military => Economy::Kbo,
            OrgKind::Japanese | OrgKind::JapaneseIndependent => Economy::Npb,
        }
    }

    /// The first-team level whose seasons count toward service.
    pub fn top_level(self) -> &'static str {
        self.key()
    }
}

// ---------------------------------------------------------------------------------------
// Rules

/// Amounts announced for some seasons. A season after the last one is raised by the
/// economy's rate for every year past it, one before the first lowered the same way, and
/// one between two announced values takes the earlier.
#[derive(Clone, Debug, Deserialize)]
pub struct Schedule {
    pub by_year: BTreeMap<i32, f64>,
}

impl Schedule {
    pub fn at(&self, year: i32, rate: f64) -> f64 {
        Self::lookup(&self.by_year, year, rate)
    }

    /// The schedule's rule on a bare table of announced values.
    pub fn lookup_at(by_year: &BTreeMap<i32, f64>, year: i32, rate: f64) -> f64 {
        Self::lookup(by_year, year, rate)
    }

    fn lookup(by_year: &BTreeMap<i32, f64>, year: i32, rate: f64) -> f64 {
        if let Some(v) = by_year.get(&year) {
            return *v;
        }
        let (Some((&first, &v0)), Some((&last, &v1))) = (by_year.iter().next(), by_year.iter().next_back()) else {
            return 0.0;
        };
        if year > last {
            v1 * (1.0 + rate).powi(year - last)
        } else if year < first {
            v0 / (1.0 + rate).powi(first - year)
        } else {
            by_year.range(..year).next_back().map_or(v0, |(_, v)| *v)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Valued<T> {
    value: T,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Raise {
    pub rate: f64,
}

/// Runs per 162 games (`ability::value`) of a replacement-level player: a position player
/// every day, a pitcher over an average pitcher's share of the innings.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Replacement {
    pub hitter: f64,
    pub pitcher: f64,
}

/// Pay above the minimum = scale x (WAR - `shift`)^`exponent`, the scale by season.
#[derive(Clone, Debug, Deserialize)]
pub struct Price {
    pub by_year: BTreeMap<i32, f64>,
    pub exponent: f64,
    #[serde(default)]
    pub shift: f64,
}

/// Salary above the minimum in arbitration years 1-3, as a share of free-agent pay above it.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ArbitrationShare {
    pub batter: [f64; 3],
    pub starter: [f64; 3],
    pub reliever: [f64; 3],
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MaxCut {
    pub previous: f64,
    pub two_before: f64,
}

/// NPB's reduction limit: more than `high` above `above`, `low` at or below it, needs the
/// player's consent.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct CutLimit {
    pub above: f64,
    pub high: f64,
    pub low: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Service {
    pub arbitration: u32,
    pub free_agency: u32,
}

/// Weekly pay of a first minor-league contract by level, and the season's weeks.
#[derive(Clone, Debug, Deserialize)]
pub struct MinorWeekly {
    pub aaa: BTreeMap<i32, f64>,
    pub aa: BTreeMap<i32, f64>,
    pub high_a: BTreeMap<i32, f64>,
    pub single_a: BTreeMap<i32, f64>,
    pub rookie: BTreeMap<i32, f64>,
    pub weeks: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TaxRates {
    pub first: [f64; 4],
    pub second: [f64; 4],
    pub third: [f64; 4],
}

/// The Competitive Balance Tax: marginal rates on the payroll above the threshold, in the
/// brackets above it, by the club's consecutive years over it.
#[derive(Clone, Debug, Deserialize)]
pub struct Tax {
    pub thresholds: BTreeMap<i32, f64>,
    pub brackets: Vec<f64>,
    pub rates: TaxRates,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Floor {
    pub by_year: BTreeMap<i32, f64>,
    pub rate: f64,
}

/// Seasons a KBO or NPB player needs for free agency.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct FaSeasons {
    #[serde(default)]
    pub high_school: u32,
    #[serde(default)]
    pub college: u32,
    #[serde(default)]
    pub domestic: u32,
    #[serde(default)]
    pub domestic_college: u32,
    #[serde(default)]
    pub international: u32,
    #[serde(default)]
    pub again: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EconomyRules {
    pub currency: String,
    pub raise: Raise,
    runs_per_win: Valued<f64>,
    pub replacement: Replacement,
    pub minimum: Schedule,
    #[serde(default)]
    pub development_minimum: Option<Schedule>,
    #[serde(default)]
    pub minor_rate_on_major_contract: Option<Schedule>,
    #[serde(default)]
    pub minor_weekly: Option<MinorWeekly>,
    #[serde(default)]
    pub price_per_war: Option<Price>,
    /// Club-set pay of reserved KBO and NPB players.
    #[serde(default)]
    pub pay: Option<Price>,
    #[serde(default)]
    pub arbitration_share: Option<ArbitrationShare>,
    #[serde(default)]
    pub max_cut: Option<MaxCut>,
    #[serde(default)]
    pub cut_limit: Option<CutLimit>,
    #[serde(default)]
    pub service: Option<Service>,
    #[serde(default)]
    pub tax: Option<Tax>,
    #[serde(default)]
    pub qualifying_offer: Option<Schedule>,
    #[serde(default)]
    pub cap: Option<Schedule>,
    #[serde(default)]
    pub floor: Option<Floor>,
    #[serde(default)]
    pub seasons_for_fa: Option<FaSeasons>,
}

impl EconomyRules {
    pub fn runs_per_win(&self) -> f64 {
        self.runs_per_win.value
    }

    pub fn minimum(&self, year: i32) -> f64 {
        self.minimum.at(year, self.raise.rate)
    }

    pub fn development_minimum(&self, year: i32) -> f64 {
        self.development_minimum.as_ref().map_or_else(|| self.minimum(year), |s| s.at(year, self.raise.rate))
    }

    /// A season's pay on a first minor-league contract at `level` (major-league
    /// organisations only; the development squads are paid the rookie scale).
    pub fn minor_pay(&self, level: &str, year: i32) -> f64 {
        let Some(w) = &self.minor_weekly else { return 0.0 };
        let (scale, weeks) = match level {
            "aaa" => (&w.aaa, "aaa"),
            "aa" => (&w.aa, "aa"),
            "high_a" => (&w.high_a, "high_a"),
            "single_a" => (&w.single_a, "single_a"),
            _ => (&w.rookie, "rookie"),
        };
        Schedule::lookup(scale, year, self.raise.rate) * w.weeks.get(weeks).copied().unwrap_or(0.0)
    }

    /// Pay above the minimum for `war` wins above replacement in `year`.
    fn above_minimum(price: &Price, war: f64, year: i32, rate: f64) -> f64 {
        Schedule::lookup(&price.by_year, year, rate) * (war - price.shift).max(0.0).powf(price.exponent)
    }
}

/// How often new free-agent contracts carry options and opt-outs, and their terms.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionRules {
    pub opt_out_multi: f64,
    /// (after which year, share) of the opt-outs.
    pub opt_out_after: Vec<(u32, f64)>,
    pub club_option_multi: f64,
    pub club_option_one: f64,
    /// Share of club options without a buyout, and the buyout's share of the option salary.
    pub no_buyout: f64,
    pub buyout: f64,
    /// The market premium a club counts on to replace a player when it weighs his option, and
    /// the one a player expects when he weighs an opt-out.
    pub club_lean: f64,
    pub player_lean: f64,
}

/// The rules of every economy and the WAR yardstick (`data/people/contracts.json`).
#[derive(Clone, Debug, Deserialize)]
pub struct ContractRules {
    economies: BTreeMap<Economy, EconomyRules>,
    /// The major-league free-agent market; without it no club signs another's players.
    #[serde(default)]
    pub market: Option<crate::market::MarketRules>,
    #[serde(default)]
    pub options: Option<OptionRules>,
}

/// A change to a coming draft from free agency: a compensation pick after a round, or a pick
/// of a round given up (BA Art. XX(B)(4)).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftChange {
    /// The offseason whose draft it changes.
    pub year: i32,
    pub org: String,
    pub round: u32,
    /// A pick added after `round`; otherwise the pick of `round` is given up.
    pub extra: bool,
}

impl ContractRules {
    pub fn bundled() -> ContractRules {
        Self::from_json(RULES).expect("bundled contracts.json")
    }

    pub fn from_json(text: &str) -> Result<ContractRules, String> {
        let r: ContractRules = serde_json::from_str(text).map_err(|e| e.to_string())?;
        for e in Economy::ALL {
            if !r.economies.contains_key(&e) {
                return Err(format!("contracts.json: no rules for {}", e.key()));
            }
        }
        Ok(r)
    }

    pub fn of(&self, e: Economy) -> &EconomyRules {
        &self.economies[&e]
    }

    /// Wins above replacement over a season with a first team of economy `e`: a position
    /// player playing every day, a pitcher over a starter's share of the innings if he has
    /// a starter's stamina and a reliever's if not (`ability.rs`).
    pub fn war(&self, p: &Player, e: Economy) -> f64 {
        let r = self.of(e);
        if p.is_pitcher() {
            let share = if ability::can_start(p) { ability::STARTER_SHARE } else { ability::RELIEVER_SHARE } / ability::STAFF_SHARE;
            (ability::value(p) - r.replacement.pitcher) * share / r.runs_per_win()
        } else {
            (ability::value(p) - r.replacement.hitter) / r.runs_per_win()
        }
    }

    /// A major-league free agent's yearly pay for `war` wins above replacement.
    pub fn free_agent_pay(&self, war: f64, year: i32) -> f64 {
        let r = self.of(Economy::Mlb);
        let min = r.minimum(year);
        match &r.price_per_war {
            Some(p) => min + EconomyRules::above_minimum(p, war, year, r.raise.rate),
            None => min,
        }
    }

    /// A major-league player's pay in arbitration year `year_no` (1-3): a share of his
    /// free-agent pay above the minimum, by his role.
    pub fn arbitration_pay(&self, p: &Player, war: f64, year: i32, year_no: u32) -> f64 {
        let r = self.of(Economy::Mlb);
        let min = r.minimum(year);
        let Some(share) = &r.arbitration_share else { return min };
        let row = if !p.is_pitcher() {
            share.batter
        } else if ability::can_start(p) {
            share.starter
        } else {
            share.reliever
        };
        let k = (year_no.clamp(1, 3) - 1) as usize;
        min + row[k] * (self.free_agent_pay(war, year) - min)
    }

    /// The pay a KBO or NPB club sets for a reserved player of `war` wins above
    /// replacement.
    pub fn club_pay(&self, e: Economy, war: f64, year: i32) -> f64 {
        let r = self.of(e);
        let min = r.minimum(year);
        match &r.pay {
            Some(p) => min + EconomyRules::above_minimum(p, war, year, r.raise.rate),
            None => min,
        }
    }

    /// The lowest salary NPB's reduction limit allows after `previous`, without the
    /// player's consent; and for a major-leaguer, the 20 % limit on a renewal.
    pub fn lowest_renewal(&self, e: Economy, previous: f64) -> f64 {
        let r = self.of(e);
        if let Some(c) = &r.cut_limit {
            let cut = if previous > c.above { c.high } else { c.low };
            return previous * (1.0 - cut);
        }
        if let Some(c) = &r.max_cut {
            return previous * (1.0 - c.previous);
        }
        0.0
    }
}

// ---------------------------------------------------------------------------------------
// Budgets

/// Each first team's payroll budget in its economy's currency (`data/leagues/budgets.json`).
#[derive(Clone, Debug, Deserialize)]
pub struct Budgets {
    year: i32,
    mlb: BTreeMap<String, f64>,
    kbo: BTreeMap<String, f64>,
    npb: BTreeMap<String, f64>,
}

impl Budgets {
    pub fn bundled() -> Budgets {
        serde_json::from_str(BUDGETS).expect("bundled budgets.json")
    }

    /// The budget of the organisation whose first team is `club` for `year`, raised by
    /// its economy's rate from the budgets' year.
    pub fn of(&self, club: &str, e: Economy, year: i32, rules: &ContractRules) -> Option<f64> {
        let table = match e {
            Economy::Mlb => &self.mlb,
            Economy::Kbo => &self.kbo,
            Economy::Npb => &self.npb,
        };
        table.get(club).map(|b| b * (1.0 + rules.of(e).raise.rate).powi(year - self.year))
    }
}

// ---------------------------------------------------------------------------------------
// Contracts

/// What a contract makes a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// A major-league contract: on the 40-man roster, in the majors or optioned down.
    Major,
    /// A minor-league contract with a major-league organisation.
    Minor,
    /// A KBO contract, or an NPB registered (支配下) one.
    Registered,
    /// An NPB development (育成) contract.
    Development,
    /// An independent or military club: not paid by the model.
    Other,
}

/// How a contract came about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terms {
    /// The first world's players, with contracts drawn when it was made.
    Initial,
    /// An amateur's first contract: the draft, an international or a development signing.
    Amateur,
    /// A reserved player's contract for the next season (pre-arbitration in the majors).
    Renewal,
    /// A major-leaguer's salary in arbitration.
    Arbitration,
    /// A free agent's contract, or one re-signed when free agency came due.
    FreeAgent,
    /// A foreign player of a KBO or NPB club; paid outside the budgets for now.
    Foreign,
}

/// A club option on the season after the guaranteed ones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClubOption {
    pub salary: f64,
    pub buyout: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    /// The organisation holding it: its first team's id.
    pub org: String,
    pub kind: Kind,
    pub terms: Terms,
    /// First season it covers.
    pub from: i32,
    /// Guaranteed salary of each season from `from` on, in the economy's currency.
    pub salary: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<ClubOption>,
    /// The player may leave after this season.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opt_out: Option<i32>,
}

impl Contract {
    /// Salaries are whole units of the currency, which also survive a save exactly.
    pub fn new(org: &str, kind: Kind, terms: Terms, year: i32, salary: &[f64]) -> Contract {
        Contract { org: org.to_string(), kind, terms, from: year, salary: salary.iter().map(|s| s.round()).collect(), option: None, opt_out: None }
    }

    pub fn one_season(org: &str, kind: Kind, terms: Terms, year: i32, salary: f64) -> Contract {
        Contract::new(org, kind, terms, year, &[salary])
    }

    /// Last season it guarantees.
    pub fn last(&self) -> i32 {
        self.from + self.salary.len() as i32 - 1
    }

    /// Salary for `year`, if the contract covers it.
    pub fn pay(&self, year: i32) -> Option<f64> {
        if year < self.from {
            return None;
        }
        self.salary.get((year - self.from) as usize).copied()
    }

    pub fn covers(&self, year: i32) -> bool {
        year >= self.from && year <= self.last()
    }
}

// ---------------------------------------------------------------------------------------
// Service

/// Seasons a player started on a first team of `level` before `year`, with those he had
/// before the world began.
pub fn first_team_seasons(c: &Career, level: &str, year: i32) -> u32 {
    c.prior + c.seasons.iter().filter(|s| s.level == level && s.year < year).count() as u32
}

/// Seasons a KBO or NPB player needs for free agency, by where he came from.
fn seasons_for_fa(rules: &EconomyRules, c: &Career) -> u32 {
    let s = rules.seasons_for_fa.unwrap_or_default();
    let college = matches!(&c.origin, Origin::Draft { class, .. } | Origin::Signed { class, .. } if class.contains("college") || class.contains("corporate"));
    if s.domestic > 0 {
        if college && s.domestic_college > 0 {
            s.domestic_college
        } else {
            s.domestic
        }
    } else if college && s.college > 0 {
        s.college
    } else {
        s.high_school.max(1)
    }
}

/// Expected length in years of a free agent's contract by his age and his WAR (2021-22 to
/// 2025-26 major-league free agents, `contracts_markets.md` 3C): players of 2 WAR or more
/// got 4.4 years at 28-29, 5.0 at 30, 4.1 at 31, 2.9 at 32, 2.5 at 33, 1.9 at 34 and about
/// 1.75 from 35 on; players of 1-2 WAR multi-year deals about 40 % of the time; nearly all
/// others one year. A starting point for the market calibration.
pub fn expected_years(age: f64, war: f64) -> f64 {
    let top = match age {
        a if a < 30.0 => 4.4,
        a if a < 31.0 => 5.0,
        a if a < 32.0 => 4.1,
        a if a < 33.0 => 2.9,
        a if a < 34.0 => 2.5,
        a if a < 35.0 => 1.9,
        _ => 1.75,
    };
    if war >= 2.0 {
        top
    } else if war >= 1.0 {
        1.0 + 0.4 * (top - 1.0).min(1.0)
    } else {
        1.0
    }
}

fn draw_years(age: f64, war: f64, rng: &mut Rng) -> usize {
    (expected_years(age, war) + rng.gauss(0.0, 0.7)).round().clamp(1.0, 8.0) as usize
}

// ---------------------------------------------------------------------------------------
// Bookkeeping

/// Where a player stands in his organisation for `year`.
struct Placed<'a> {
    org: &'a Org,
    level: &'a str,
    player: &'a Player,
    /// On the 40-man roster of a major-league organisation, or registered (支配下) in NPB.
    upper: bool,
    /// His time with the first team as a share of a regular's: 1 in its lineup, rotation or
    /// bullpen, `BENCH_SHARE` on its bench, 0 below it.
    share: f64,
}

/// A bench player's time as a share of a regular's (as for the replacement levels in
/// `contracts.json`).
pub const BENCH_SHARE: f64 = 0.1;

/// Every rostered player with his organisation, level and whether he holds an upper-tier
/// contract: in a major-league organisation the first team and the next best to 40 (by
/// value in standard deviations), in NPB the best 70.
fn placements<'a>(orgs: &'a [Org], pop: &'a Population) -> Vec<Placed<'a>> {
    let index: BTreeMap<&str, &crate::populate::Roster> = pop.rosters.iter().map(|r| (r.club.as_str(), r)).collect();
    let mut out = Vec::new();
    for org in orgs {
        let mut here: Vec<(usize, &str, &Player)> = Vec::new();
        let mut share: BTreeMap<u32, f64> = BTreeMap::new();
        for (i, (club, key)) in org.levels.iter().enumerate() {
            if let Some(r) = index.get(club.as_str()) {
                here.extend(r.players().map(|p| (i, key.as_str(), p)));
                if i == 0 {
                    let s = &r.squad;
                    share.extend(s.hitters.iter().map(|(p, _)| (p.id, 1.0)));
                    share.extend(s.starters.iter().chain(&s.relievers).map(|p| (p.id, 1.0)));
                    share.extend(s.bench.iter().map(|(p, _)| (p.id, BENCH_SHARE)));
                }
            }
        }
        let upper: BTreeSet<u32> = match org.kind {
            OrgKind::Major | OrgKind::Japanese => {
                let cap = if org.kind == OrgKind::Major { FORTY_MAN } else { NPB_REGISTERED };
                let mut set: BTreeSet<u32> = here.iter().filter(|(i, _, _)| *i == 0).map(|(_, _, p)| p.id).collect();
                let mut rest: Vec<(f64, u32)> =
                    here.iter().filter(|(i, _, _)| *i > 0).map(|(_, _, p)| (ability::value(p) / ability::per_sd(p), p.id)).collect();
                rest.sort_by(|a, b| b.0.total_cmp(&a.0));
                let room = cap.saturating_sub(set.len());
                set.extend(rest.iter().take(room).map(|x| x.1));
                set
            }
            _ => here.iter().map(|(_, _, p)| p.id).collect(),
        };
        for (_, level, p) in here {
            out.push(Placed { org, level, player: p, upper: upper.contains(&p.id), share: share.get(&p.id).copied().unwrap_or(0.0) });
        }
    }
    out
}

/// A new contract for a player who has none with his organisation for `year`, or whose
/// contract has run out: by his economy, tier and service.
#[allow(clippy::too_many_arguments)]
fn new_contract(
    rules: &ContractRules,
    pl: &Placed,
    career: Option<&Career>,
    previous: Option<&Contract>,
    age: f64,
    year: i32,
    foreign: bool,
    rng: &mut Rng,
) -> Contract {
    let e = Economy::of(pl.org.kind);
    let r = rules.of(e);
    let org = pl.org.id.as_str();
    let p = pl.player;
    let war = rules.war(p, e);
    let amateur = career.is_some_and(|c| c.seasons.iter().all(|s| s.year >= year) && c.prior == 0 && !matches!(c.origin, Origin::Initial));
    if foreign {
        return Contract::one_season(org, Kind::Registered, Terms::Foreign, year, 0.0);
    }
    match pl.org.kind {
        OrgKind::Major => {
            if !pl.upper {
                let pay = r.minor_pay(pl.level, year);
                return Contract::one_season(org, Kind::Minor, if amateur { Terms::Amateur } else { Terms::Renewal }, year, pay);
            }
            let service = career.map_or(0, |c| first_team_seasons(c, "mlb", year));
            let svc = r.service.unwrap_or(Service { arbitration: 3, free_agency: 6 });
            if service < svc.arbitration {
                Contract::one_season(org, Kind::Major, Terms::Renewal, year, r.minimum(year))
            } else if service < svc.free_agency {
                // Arbitration pays for the wins a player gives in the time he gets (Clemens's
                // shares are of pay per WAR produced).
                let mut pay = rules.arbitration_pay(p, war * pl.share, year, service - svc.arbitration + 1);
                if let Some(prev) = previous.and_then(|c| c.pay(year - 1)) {
                    pay = pay.max(rules.lowest_renewal(e, prev));
                }
                Contract::one_season(org, Kind::Major, Terms::Arbitration, year, pay)
            } else {
                // Free agency came due: re-signed at the free-agent price for now (the market
                // that lets him go elsewhere is the next stage).
                let n = draw_years(age, war, rng);
                let pay = rules.free_agent_pay(war, year);
                Contract::new(org, Kind::Major, Terms::FreeAgent, year, &vec![pay; n])
            }
        }
        OrgKind::Korean | OrgKind::Japanese => {
            let kind = if pl.upper { Kind::Registered } else { Kind::Development };
            if kind == Kind::Development {
                return Contract::one_season(org, kind, if amateur { Terms::Amateur } else { Terms::Renewal }, year, r.development_minimum(year));
            }
            let seasons = career.map_or(0, |c| first_team_seasons(c, e.top_level(), year));
            let due = career.is_some_and(|c| seasons >= seasons_for_fa(r, c));
            let mut pay = rules.club_pay(e, war, year);
            if let Some(prev) = previous.filter(|c| c.org == org).and_then(|c| c.pay(year - 1)) {
                pay = pay.max(rules.lowest_renewal(e, prev));
            }
            if due && !amateur {
                let n = draw_years(age, war, rng).min(4);
                Contract::new(org, kind, Terms::FreeAgent, year, &vec![pay; n])
            } else {
                Contract::one_season(org, kind, if amateur { Terms::Amateur } else { Terms::Renewal }, year, pay)
            }
        }
        _ => Contract::one_season(org, Kind::Other, if amateur { Terms::Amateur } else { Terms::Renewal }, year, 0.0),
    }
}

/// Give every rostered player a contract for `year`: those whose contract with their
/// organisation still covers it keep it (a minor-leaguer added to the 40-man gets a
/// major-league contract at the minimum); everyone else gets a new one. Contracts of
/// players who left the world go; those of players on military duty stay.
pub fn settle(world: &World, pop: &mut Population, rules: &ContractRules, year: i32, seed: u64) {
    let orgs = organisations(world);
    let mut rng = Rng::new(seed ^ 0xC0_47AC7 ^ (year as u64).wrapping_mul(0x9E37_79B9));
    let on_date = Date::new(year, 4, 1);
    let mut next: BTreeMap<u32, Contract> = BTreeMap::new();
    for pl in placements(&orgs, pop) {
        let id = pl.player.id;
        let old = pop.contracts.get(&id);
        let keep = old.filter(|c| c.org == pl.org.id && c.covers(year));
        let c = match keep {
            Some(c) if c.kind == Kind::Minor && pl.upper => {
                Contract::one_season(&pl.org.id, Kind::Major, Terms::Renewal, year, rules.of(Economy::Mlb).minimum(year))
            }
            Some(c) if c.kind == Kind::Development && pl.upper => {
                let e = Economy::of(pl.org.kind);
                Contract::one_season(&pl.org.id, Kind::Registered, Terms::Renewal, year, rules.of(e).minimum(year))
            }
            Some(c) => c.clone(),
            None => {
                let age = pop.people.get(&id).map_or(25.0, |p| on_date.days_since(p.born) as f64 / 365.2425);
                new_contract(rules, &pl, pop.careers.get(&id), old, age, year, pop.foreign.contains_key(&id), &mut rng)
            }
        };
        next.insert(id, c);
    }
    for id in pop.away.keys() {
        if let Some(c) = pop.contracts.get(id) {
            next.insert(*id, c.clone());
        }
    }
    pop.contracts = next;
}

/// Contracts for the first world: first-team players get the seasons they would have
/// played before it began (debut age about 25 in the majors, Justice 2020; about 22 in
/// the KBO and NPB, an assumption), and then a contract as `settle` would give it, a
/// free agent's with a random number of its seasons already played.
pub fn initial(world: &World, pop: &mut Population, rules: &ContractRules, seed: u64) {
    let year = pop.date.year();
    let mut rng = Rng::new(seed ^ 0x1717_C047);
    let orgs = organisations(world);
    let first_teams: BTreeMap<&str, Economy> = orgs.iter().map(|o| (o.levels[0].0.as_str(), Economy::of(o.kind))).collect();
    let end_of_last = Date::new(year - 1, 10, 1);
    let rosters: Vec<(String, Vec<u32>)> = pop.rosters.iter().map(|r| (r.club.clone(), r.players().map(|p| p.id).collect())).collect();
    for (club, ids) in rosters {
        let Some(&e) = first_teams.get(club.as_str()) else { continue };
        let debut = if e == Economy::Mlb { 25.0 } else { 22.0 };
        for id in ids {
            if pop.foreign.contains_key(&id) {
                continue;
            }
            let age = pop.people.get(&id).map_or(25.0, |p| end_of_last.days_since(p.born) as f64 / 365.2425);
            let seasons = (age - (debut + rng.gauss(0.0, 2.0))).floor().clamp(0.0, 20.0) as u32;
            if let Some(c) = pop.careers.get_mut(&id) {
                c.prior = seasons;
            }
        }
    }
    pop.contracts.clear();
    settle(world, pop, rules, year, seed);
    // A free agent's contract of the first world began some seasons ago.
    for c in pop.contracts.values_mut() {
        if c.terms == Terms::FreeAgent && c.salary.len() > 1 {
            let played = rng.below(c.salary.len()) as i32;
            c.from -= played;
        }
        if c.terms != Terms::Foreign {
            c.terms = if c.terms == Terms::FreeAgent { Terms::FreeAgent } else { Terms::Initial };
        }
    }
}

// ---------------------------------------------------------------------------------------
// Payrolls

/// One organisation's payroll for a season.
#[derive(Clone, Debug)]
pub struct Payroll {
    pub org: String,
    pub economy: Economy,
    /// Salaries counted against the budget: major-league contracts in the majors' system,
    /// registered domestic players in the KBO and NPB (as the budgets are measured).
    pub total: f64,
    pub budget: Option<f64>,
    pub players: usize,
}

pub fn payrolls(world: &World, pop: &Population, rules: &ContractRules, budgets: &Budgets, year: i32) -> Vec<Payroll> {
    let orgs = organisations(world);
    let mut by_org: BTreeMap<&str, (f64, usize)> = BTreeMap::new();
    for c in pop.contracts.values() {
        let counted = matches!(c.kind, Kind::Major | Kind::Registered) && c.terms != Terms::Foreign;
        if counted {
            if let Some(pay) = c.pay(year) {
                let e = by_org.entry(c.org.as_str()).or_default();
                e.0 += pay;
                e.1 += 1;
            }
        }
    }
    orgs.iter()
        .filter(|o| matches!(o.kind, OrgKind::Major | OrgKind::Korean | OrgKind::Japanese))
        .map(|o| {
            let e = Economy::of(o.kind);
            let (total, players) = by_org.get(o.id.as_str()).copied().unwrap_or_default();
            Payroll { org: o.id.clone(), economy: e, total, budget: budgets.of(&o.id, e, year, rules), players }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedules_follow_announced_values_then_the_raise() {
        let s = Schedule { by_year: [(2026, 100.0), (2028, 120.0)].into_iter().collect() };
        assert_eq!(s.at(2026, 0.05), 100.0);
        assert_eq!(s.at(2027, 0.05), 100.0);
        assert_eq!(s.at(2028, 0.05), 120.0);
        assert!((s.at(2030, 0.05) - 120.0 * 1.05 * 1.05).abs() < 1e-9);
        assert!((s.at(2025, 0.05) - 100.0 / 1.05).abs() < 1e-9);
    }

    #[test]
    fn the_bundled_rules_and_budgets_load() {
        let r = ContractRules::bundled();
        assert_eq!(r.of(Economy::Mlb).minimum(2026), 780_000.0);
        // After the agreement's last season the minimum rises at the economy's rate.
        assert!((r.of(Economy::Mlb).minimum(2027) - 780_000.0 * 1.047).abs() < 1.0);
        assert_eq!(r.of(Economy::Kbo).minimum(2027), 33_000_000.0);
        assert_eq!(r.of(Economy::Npb).minimum(2026), 4_200_000.0);
        assert!((r.of(Economy::Npb).minimum(2027) - 4_200_000.0 * 1.037).abs() < 1.0);
        let b = Budgets::bundled();
        assert!(b.of("los-angeles-palms", Economy::Mlb, 2026, &r).is_some_and(|x| x > 4.0e8));
        assert!(b.of("seoul-stallions", Economy::Kbo, 2026, &r).is_some_and(|x| (x - 4.911e9).abs() < 1.0));
        // The free-agent price grows with WAR faster than linearly.
        let one = r.free_agent_pay(1.0, 2026) - 780_000.0;
        let three = r.free_agent_pay(3.0, 2026) - 780_000.0;
        assert!(three > 3.0 * one);
    }

    #[test]
    fn every_rostered_player_has_a_contract_before_and_after_an_offseason() {
        use crate::career::{offseason, CareerTable, Offseason};
        use crate::develop::AgingTable;
        use crate::people::PeopleMaker;
        use crate::populate::{populate, LevelTable};
        let (world, levels) = (World::bundled(), LevelTable::bundled());
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 3);
        let rules = ContractRules::bundled();
        let check = |pop: &Population, year: i32| {
            let orgs = organisations(&world);
            for org in &orgs {
                for (i, (club, _)) in org.levels.iter().enumerate() {
                    let Some(r) = pop.roster(club) else { continue };
                    for p in r.players() {
                        let c = pop.contracts.get(&p.id).unwrap_or_else(|| panic!("{} of {club} has no contract", p.id));
                        assert!(c.covers(year) && c.org == org.id, "{year}: {c:?}");
                        // Every major-league first team is on major-league contracts.
                        if org.kind == OrgKind::Major && i == 0 {
                            assert_eq!(c.kind, Kind::Major);
                            assert!(c.pay(year).unwrap() >= rules.of(Economy::Mlb).minimum(year) - 0.5);
                        }
                    }
                }
            }
        };
        check(&pop, 2027);
        let (aging, careers, people) = (AgingTable::bundled(), CareerTable::bundled(), PeopleMaker::new());
        let ctx = Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people, contracts: &rules };
        offseason(&ctx, &mut pop, 2027, None, 3);
        check(&pop, 2028);
        // The majors' payroll is of the order of the budgets: within a factor of two.
        let budgets = Budgets::bundled();
        let rows = payrolls(&world, &pop, &rules, &budgets, 2028);
        let (pay, budget): (f64, f64) =
            rows.iter().filter(|r| r.economy == Economy::Mlb).fold((0.0, 0.0), |a, r| (a.0 + r.total, a.1 + r.budget.unwrap_or(0.0)));
        assert!(pay > 0.5 * budget && pay < 2.0 * budget, "{pay:.3e} against {budget:.3e}");
    }

    #[test]
    fn npb_cuts_are_limited_and_major_league_renewals_too() {
        let r = ContractRules::bundled();
        assert_eq!(r.lowest_renewal(Economy::Npb, 200_000_000.0), 120_000_000.0);
        assert_eq!(r.lowest_renewal(Economy::Npb, 100_000_000.0), 75_000_000.0);
        assert_eq!(r.lowest_renewal(Economy::Mlb, 10_000_000.0), 8_000_000.0);
        assert_eq!(r.lowest_renewal(Economy::Kbo, 300_000_000.0), 0.0);
    }
}
