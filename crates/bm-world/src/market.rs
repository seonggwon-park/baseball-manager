//! The free-agent markets (`docs/contracts.md`, stages 2-4): one for the major-league
//! organisations (with the KBO's and NPB's posted players and free agents who try the
//! majors), one for the KBO and its independent club, one for NPB and its farm-only clubs.
//!
//! Every offseason the free agents (players whose contract ran out with the service for
//! free agency, players their organisation did not keep, minor-leaguers whose contract ran
//! out) go to market, best first. Each organisation values a player by the wins he adds to its
//! roster (his WAR against the player he would push out of its 26, `contract.rs`), within
//! its budget; a player asks the free-agent price of his own WAR, more when several
//! organisations want him (decision 2026-11-18: base price plus a premium for competing
//! bids), and takes the best offer, his own organisation's counting for a little more. The
//! length of a contract follows his age and WAR. Those no organisation wants on a
//! major-league contract may sign a minor-league one as Triple-A depth; the rest leave.

use crate::ability;
use crate::contract::{expected_years, ClubOption, ContractRules, Economy, OptionRules, Posting};
use crate::player::Player;
use bm_engine::Rng;
use serde::Deserialize;

/// What a market's behaviour rests on (`contracts.json` market, one per economy): each value
/// is fitted to the measured outcomes of free agents (`docs/contracts.md`, calibration
/// targets). The qualifying offer and its compensation are the majors' only; the declaration
/// threshold the KBO's and NPB's.
#[derive(Clone, Debug, Deserialize)]
pub struct MarketRules {
    /// Rise of the offer per competing organisation beyond the first, as a share of the
    /// asking price, and the most competitors that count.
    pub premium: f64,
    pub max_bidders: usize,
    /// Extra weight a player puts on his own organisation's offer.
    pub home: f64,
    /// Spread of a player's taste among offers (log scale).
    pub taste: f64,
    /// Share of its budget an organisation may go past.
    pub slack: f64,
    /// How much more an organisation with a bigger budget offers a free agent it wants as a
    /// regular: its offer is the asking price (with the premium for competing bids) times
    /// (its budget / the market's median budget) ^ `budget_power`, so a richer club outbids a
    /// poorer one for the same player (decision 2026-11-21: in the majors payroll explains a
    /// fifth to a third of the variation in winning, Abramowitz 2006). 0: every organisation
    /// offers the same and the player's taste decides.
    #[serde(default)]
    pub budget_power: f64,
    /// A bench player's playing time as a share of a regular's.
    pub bench_share: f64,
    /// How much of the wins of the players a signing pushes down the roster an organisation
    /// counts as lost (1: all, the pure marginal value; 0: none, the player's own WAR in his
    /// place). A displaced regular still covers injuries, moves to another position or is
    /// traded, so the market prices players near their own WAR (FanGraphs' price per WAR).
    pub displaced: f64,
    /// Minor-league free agents an organisation signs at most as depth in one offseason.
    pub depth_signings: usize,
    /// Factor on the retirement chances (`careers.json` retirement) of major-league
    /// organisations' players: the market decides most exits, players retire on their own
    /// less (decision 2026-11-18).
    pub retirement: f64,
    /// An organisation makes the qualifying offer to a free agent whose price is at least
    /// this share of it.
    #[serde(default)]
    pub qualifying_offer: f64,
    /// Organisations taken to receive revenue sharing: the ones with the smallest budgets.
    #[serde(default)]
    pub revenue_sharing_payees: usize,
    /// Total guarantee (2026, raised with the economy) from which a payee losing a qualified
    /// free agent picks after the first round.
    #[serde(default)]
    pub compensation_guarantee: f64,
    /// A KBO or NPB player with the seasons for free agency declares it when the gain he
    /// expects from the market (log of the price he expects over his club's pay) is above
    /// this (decision 2026-11-18: each league's preference for the old club, fitted to the
    /// declaration rates).
    #[serde(default)]
    pub declare: f64,
    /// A KBO or NPB free agent's price over the pay his club sets for a reserved player of his
    /// WAR, above the minimum (`price`): clubs pay reserved players less than the market would
    /// (Scully 1974).
    #[serde(default)]
    pub markup: f64,
    /// A KBO or NPB player who may go to the majors (posted, or a free agent with the right)
    /// wants to when the log of the pay he expects there over the pay he expects at home, plus
    /// his taste, is above this.
    #[serde(default)]
    pub abroad: f64,
}

/// Why a player is on the market.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// Six years of service and his contract ran out.
    Service,
    /// His organisation did not keep him (released, or not tendered a contract).
    LetGo,
    /// His first minor-league contract ran out (six or seven seasons).
    MinorContract,
    /// His KBO or NPB club posted him: he signs a major-league contract, whose posting fee
    /// the signing organisation pays, or goes back to his club.
    Posted,
    /// A KBO or NPB player back from the majors, on his league's market.
    Returning,
}

pub struct FreeAgent {
    pub player: Player,
    pub age: f64,
    /// The organisation (index) he played for.
    pub from: Option<usize>,
    pub why: Why,
    /// He turned down his organisation's qualifying offer.
    pub qualified: bool,
    /// What another club signing him owes his old one in cash, when it owes compensation at
    /// all (a KBO or NPB free agent of a compensated grade or rank; nothing in cash when the
    /// old club takes a draft pick). Such a player signs with another club only on a
    /// first-team contract.
    pub compensation: Option<f64>,
    /// A signing by another club counts toward its limit of other clubs' free agents.
    pub counts: bool,
    /// A KBO or NPB player in the majors' market who would rather go home than take a
    /// minor-league contract (`career.rs` sends him to his league's market if nobody offers a
    /// major-league one).
    pub homeward: bool,
    /// Only his old club may sign him (a KBO player back from a posting: his club holds his
    /// rights).
    pub exclusive: bool,
}

impl FreeAgent {
    /// A player on the market with nothing owed for him.
    pub fn new(player: Player, age: f64, from: Option<usize>, why: Why) -> FreeAgent {
        FreeAgent { player, age, from, why, qualified: false, compensation: None, counts: false, homeward: false, exclusive: false }
    }

    /// What organisation `k` pays on top of his salary if it signs him for `pay` a year over
    /// `years`: the posting fee, or his old club's compensation in cash.
    fn owed(&self, k: usize, pay: f64, years: usize, posting: Option<&Posting>) -> f64 {
        if self.why == Why::Posted {
            return posting.map_or(0.0, |p| p.fee(pay * years as f64));
        }
        if self.from == Some(k) {
            0.0
        } else {
            self.compensation.unwrap_or(0.0)
        }
    }

    /// Whether organisation `k` may sign him.
    fn may_sign(&self, k: usize) -> bool {
        !self.exclusive || self.from == Some(k)
    }
}

/// A bidder's roster for valuing a free agent: the WAR (in its economy) of its position
/// players (playing every day) and of its pitchers as starters (if they can start) and as
/// relievers.
#[derive(Clone, Debug)]
pub struct Depth {
    pub economy: Economy,
    pub hitters: Vec<f64>,
    pub starters: Vec<f64>,
    pub relievers: Vec<f64>,
}

fn war_as(rules: &ContractRules, e: Economy, p: &Player, starter: bool) -> f64 {
    let r = rules.of(e);
    let share = if starter { ability::STARTER_SHARE } else { ability::RELIEVER_SHARE } / ability::STAFF_SHARE;
    (ability::value(p) - r.replacement.pitcher) * share / r.runs_per_win()
}

impl Depth {
    pub fn new(economy: Economy) -> Depth {
        Depth { economy, hitters: Vec::new(), starters: Vec::new(), relievers: Vec::new() }
    }

    pub fn add(&mut self, rules: &ContractRules, p: &Player) {
        let e = self.economy;
        if p.is_pitcher() {
            if ability::can_start(p) {
                self.starters.push(war_as(rules, e, p, true));
            }
            self.relievers.push(war_as(rules, e, p, false));
        } else {
            self.hitters.push(rules.war(p, e));
        }
    }

    /// Wins above replacement of the best 26: nine regulars and four on the bench, five
    /// starters and eight relievers (a starter in the rotation does not also pitch in relief;
    /// near enough, the best relief WAR not already used by the rotation).
    pub fn value(&self, bench_share: f64) -> f64 {
        let top = |v: &[f64], n: usize| -> Vec<f64> {
            let mut v = v.to_vec();
            v.sort_by(|a, b| b.total_cmp(a));
            v.truncate(n);
            v
        };
        let h = top(&self.hitters, 13);
        let hit: f64 = h.iter().take(9).sum::<f64>() + bench_share * h.iter().skip(9).sum::<f64>();
        let rot = top(&self.starters, 5);
        let rotation: f64 = rot.iter().sum();
        // Relievers: the pitchers' relief WAR, less as many of the best as the rotation took.
        let relief = top(&self.relievers, 8 + rot.len());
        let bullpen: f64 = relief.iter().skip(rot.len()).sum();
        hit + rotation + bullpen
    }

    /// Wins `p` adds to this roster, counting `displaced` of the wins the players he pushes
    /// down lose: between his own WAR in the place he takes (`displaced` 0) and the change of
    /// the 26's WAR (1).
    pub fn marginal(&self, rules: &ContractRules, p: &Player, bench_share: f64, displaced: f64) -> f64 {
        let mut with = self.clone();
        with.add(rules, p);
        let change = with.value(bench_share) - self.value(bench_share);
        let own = self.place(rules, p, bench_share) * rules.war(p, self.economy);
        displaced * change + (1.0 - displaced) * own
    }

    /// The share of a regular's time `p` would get here: 1 among the nine regulars or the
    /// rotation and bullpen, `bench_share` on the bench, 0 outside the 26.
    pub fn place(&self, rules: &ContractRules, p: &Player, bench_share: f64) -> f64 {
        let better = |v: &[f64], x: f64| v.iter().filter(|w| **w > x).count();
        let e = self.economy;
        if p.is_pitcher() {
            if ability::can_start(p) && better(&self.starters, war_as(rules, e, p, true)) < 5 {
                return 1.0;
            }
            if better(&self.relievers, war_as(rules, e, p, false)) < 13 {
                return 1.0;
            }
            0.0
        } else {
            match better(&self.hitters, rules.war(p, e)) {
                0..=8 => 1.0,
                9..=12 => bench_share,
                _ => 0.0,
            }
        }
    }
}

/// An organisation in the market.
pub struct Bidder {
    pub depth: Depth,
    /// Salaries already owed for the next season (major-league contracts; registered domestic
    /// players in the KBO and NPB).
    pub committed: f64,
    pub budget: f64,
    pub depth_signed: usize,
    /// Free agents from other organisations it may still sign (the KBO's limit; `None` for
    /// no limit).
    pub outside_left: Option<usize>,
    /// It takes depth signings only (independent clubs: no budget in the model).
    pub depth_only: bool,
}

/// One signing.
pub struct Signing {
    pub player: Player,
    pub org: usize,
    pub major: bool,
    /// Yearly salary and years.
    pub pay: f64,
    pub years: usize,
    pub why: Why,
    pub from: Option<usize>,
    pub qualified: bool,
    /// A club option on the year after, and the year after which he may opt out (counted
    /// from the first season: 1 = after the first).
    pub option: Option<ClubOption>,
    pub opt_out_after: Option<u32>,
}

/// The market's result: signings, and the players nobody signed.
pub struct Outcome {
    pub signed: Vec<Signing>,
    pub unsigned: Vec<FreeAgent>,
}

/// The price of a free agent of `war` in economy `e` for `year`: the majors' free-agent
/// price; in the KBO and NPB the club pay of his WAR, raised above the minimum by the market's
/// markup.
pub fn price(rules: &ContractRules, m: &MarketRules, e: Economy, war: f64, year: i32) -> f64 {
    match e {
        Economy::Mlb => rules.free_agent_pay(war, year),
        _ => {
            let min = rules.of(e).minimum(year);
            min + (1.0 + m.markup) * (rules.club_pay(e, war, year) - min)
        }
    }
}

/// Longest free-agent contract of an economy: eight years in the majors; four in the KBO and
/// NPB (the KBO's longest deals are four years with options, `contracts_kbo.md` 9.2).
fn longest(e: Economy) -> f64 {
    if e == Economy::Mlb {
        8.0
    } else {
        4.0
    }
}

/// Run the market of economy `e` for `year` (the season the contracts begin).
pub fn run(rules: &ContractRules, m: &MarketRules, e: Economy, mut agents: Vec<FreeAgent>, bidders: &mut [Bidder], year: i32, rng: &mut Rng) -> Outcome {
    let r = rules.of(e);
    let price = |war: f64| price(rules, m, e, war, year);
    let posting = rules.of(Economy::Mlb).posting.as_ref();
    // Best first.
    agents.sort_by(|a, b| rules.war(&b.player, e).total_cmp(&rules.war(&a.player, e)));
    let mut signed = Vec::new();
    let mut left = Vec::new();
    // The market's median budget, the yardstick of `budget_power`.
    let mut budgets: Vec<f64> = bidders.iter().filter(|b| !b.depth_only && b.budget > 0.0).map(|b| b.budget).collect();
    budgets.sort_by(f64::total_cmp);
    let median = budgets.get(budgets.len() / 2).copied().unwrap_or(1.0);
    let reach = |b: &Bidder| if m.budget_power == 0.0 || b.budget <= 0.0 { 1.0 } else { (b.budget / median).powf(m.budget_power) };
    for fa in agents {
        let war = rules.war(&fa.player, e);
        let ask = price(war);
        let years = (expected_years(fa.age, war) + rng.gauss(0.0, 0.5)).round().clamp(1.0, longest(e)) as usize;
        // Who wants him: every organisation he would make the 26 of. A regular (in the
        // lineup, the rotation or the bullpen) is offered his price, a bench player that share
        // of it (not less than the minimum); the price of a regular rises with the number of
        // organisations that want him as one. An organisation offers what its budget leaves.
        let min = r.minimum(year);
        let outside = |k: usize| fa.from != Some(k) && fa.counts;
        let wants: Vec<(usize, f64)> = bidders
            .iter()
            .enumerate()
            .filter(|(k, b)| !b.depth_only && fa.may_sign(*k) && !(outside(*k) && b.outside_left == Some(0)))
            .filter_map(|(k, b)| {
                let place = b.depth.place(rules, &fa.player, m.bench_share);
                (place > 0.0).then_some((k, place))
            })
            .collect();
        let regulars = wants.iter().filter(|(_, place)| *place >= 1.0).count().min(m.max_bidders);
        let premium = 1.0 + m.premium * regulars.saturating_sub(1) as f64;
        // A bench player signs for a year.
        let years_at = |pay: f64| if pay < ask { 1 } else { years };
        let offers: Vec<(usize, f64)> = wants
            .iter()
            .filter_map(|&(k, place)| {
                let b = &bidders[k];
                let pay = if place >= 1.0 { ask * premium * reach(b) } else { (place * ask).max(min) };
                // A club signing another's free agent also pays his compensation, or the
                // posting fee of a posted player.
                (b.committed + pay + fa.owed(k, pay, years_at(pay), posting) <= b.budget * (1.0 + m.slack)).then_some((k, pay))
            })
            .collect();
        let choice = offers
            .iter()
            .map(|&(k, pay)| {
                let home = if fa.from == Some(k) { m.home } else { 0.0 };
                (k, pay, pay.ln() + home + rng.gauss(0.0, m.taste))
            })
            .max_by(|a, b| a.2.total_cmp(&b.2));
        if let Some((k, pay, _)) = choice {
            let years = years_at(pay);
            let owed = fa.owed(k, pay, years, posting);
            let b = &mut bidders[k];
            b.committed += pay + owed;
            b.depth.add(rules, &fa.player);
            if outside(k) {
                if let Some(n) = b.outside_left.as_mut() {
                    *n = n.saturating_sub(1);
                }
            }
            // Options and opt-outs follow the majors' contracts (`contracts_options.md`); the
            // KBO's and NPB's are not measured.
            let (option, opt_out_after) = if e == Economy::Mlb { clauses(rules.options.as_ref(), pay, years, rng) } else { (None, None) };
            signed.push(Signing {
                player: fa.player,
                org: k,
                major: true,
                pay,
                years,
                why: fa.why,
                from: fa.from,
                qualified: fa.qualified,
                option,
                opt_out_after,
            });
            continue;
        }
        left.push(fa);
    }
    // Depth deals (minor-league contracts in the majors' system; in the KBO and NPB a second
    // team's place, an NPB development contract, or an independent club) for the
    // organisations he would improve most. A free agent his old club is owed compensation for
    // takes one only from it; a posted player and one who would rather go home take none.
    let mut unsigned = Vec::new();
    for fa in left {
        if fa.why == Why::Posted || fa.homeward {
            unsigned.push(fa);
            continue;
        }
        let best = bidders
            .iter()
            .enumerate()
            .filter(|(_, b)| b.depth_signed < m.depth_signings)
            .filter(|(k, _)| fa.may_sign(*k) && (fa.from == Some(*k) || fa.compensation.is_none()))
            .map(|(k, b)| (k, depth_gain(rules, &b.depth, &fa.player)))
            .filter(|(_, g)| *g > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1));
        match best {
            Some((k, _)) => {
                let b = &mut bidders[k];
                b.depth_signed += 1;
                b.depth.add(rules, &fa.player);
                let pay = match e {
                    _ if b.depth_only => 0.0,
                    Economy::Mlb => r.minor_pay("aaa", year),
                    Economy::Npb => r.development_minimum(year),
                    Economy::Kbo => r.minimum(year),
                };
                signed.push(Signing {
                    player: fa.player,
                    org: k,
                    major: false,
                    pay,
                    years: 1,
                    why: fa.why,
                    from: fa.from,
                    qualified: fa.qualified,
                    option: None,
                    opt_out_after: None,
                });
            }
            None => unsigned.push(fa),
        }
    }
    Outcome { signed, unsigned }
}

/// The clauses of a new major-league contract: an opt-out (multi-year deals) and a club
/// option on the year after (`contracts.json` options).
fn clauses(o: Option<&OptionRules>, pay: f64, years: usize, rng: &mut Rng) -> (Option<ClubOption>, Option<u32>) {
    let Some(o) = o else { return (None, None) };
    let mut opt_out = None;
    if years >= 2 && rng.chance(o.opt_out_multi) {
        let weights: Vec<f64> = o.opt_out_after.iter().map(|(_, w)| *w).collect();
        let after = o.opt_out_after[rng.pick_weighted(&weights)].0;
        opt_out = Some(after.min(years as u32 - 1).max(1));
    }
    let share = if years >= 2 { o.club_option_multi } else { o.club_option_one };
    let option = rng.chance(share).then(|| ClubOption { salary: pay.round(), buyout: if rng.chance(o.no_buyout) { 0.0 } else { (o.buyout * pay).round() } });
    (option, opt_out)
}

/// How much a player improves an organisation's depth beyond its 26: whether he would rank
/// among its best 54 of his kind (the majors and Triple-A, or a first and second team), by
/// how much over the last of them.
fn depth_gain(rules: &ContractRules, d: &Depth, p: &Player) -> f64 {
    let (pool, mine, n) = if p.is_pitcher() { (&d.relievers, war_as(rules, d.economy, p, false), 27) } else { (&d.hitters, rules.war(p, d.economy), 27) };
    let mut v = pool.clone();
    v.sort_by(|a, b| b.total_cmp(a));
    match v.get(n - 1) {
        Some(last) => mine - last,
        None => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::{pitcher, position_player, Talent};
    use crate::player::{Position, Role};

    fn hitter(id: u32, shift: f32) -> Player {
        let mut rng = Rng::new(u64::from(id));
        let mut p = position_player(&mut rng, id, Some(Position::Left), &Talent::MAJOR);
        p.attrs = Default::default();
        for a in crate::attr::Attr::ALL {
            p.attrs[a] = shift;
        }
        p
    }

    #[test]
    fn a_strong_hitter_is_a_regular_and_a_weak_one_is_left_out() {
        let rules = ContractRules::bundled();
        let mut d = Depth::new(Economy::Mlb);
        for i in 0..13 {
            d.add(&rules, &hitter(i, 0.0));
        }
        assert_eq!(d.place(&rules, &hitter(100, 1.0), 0.1), 1.0);
        assert_eq!(d.place(&rules, &hitter(101, -2.0), 0.1), 0.0);
        let mut rng = Rng::new(1);
        let p = pitcher(&mut rng, 200, Role::Starter, &Talent::MAJOR);
        // An empty staff takes any pitcher.
        assert_eq!(d.place(&rules, &p, 0.1), 1.0);
    }

    #[test]
    fn a_richer_club_outbids_a_poorer_one_when_budgets_count() {
        let rules = ContractRules::bundled();
        let roster = || {
            let mut d = Depth::new(Economy::Mlb);
            for i in 0..13 {
                d.add(&rules, &hitter(i, 0.0));
            }
            d
        };
        // Three clubs with room want him; the richest has three times the median budget.
        let richest = |power: f64, seed: u64| {
            let m = MarketRules { budget_power: power, home: 0.0, ..rules.market[&Economy::Mlb].clone() };
            let mut bidders: Vec<Bidder> = [1.0e8, 1.5e8, 4.5e8]
                .iter()
                .map(|&budget| Bidder { depth: roster(), committed: 0.0, budget, depth_signed: 0, outside_left: None, depth_only: false })
                .collect();
            let out = run(&rules, &m, Economy::Mlb, vec![FreeAgent::new(hitter(500, 1.0), 29.0, None, Why::Service)], &mut bidders, 2027, &mut Rng::new(seed));
            (out.signed[0].org == 2, out.signed[0].pay)
        };
        let share = |power: f64| (0..200).filter(|&s| richest(power, s).0).count();
        // Equal offers: the richest club wins about a third of the time; offers that grow with
        // the budget: nearly always, and he is paid more.
        let (even, rich) = (share(0.0), share(1.0));
        assert!((40..100).contains(&even), "{even} of 200 with equal offers");
        assert!(rich > 180, "{rich} of 200 with budget_power 1");
        // Three times the median budget, three times the offer.
        let (won, pay) = richest(1.0, 1);
        assert!(won && (pay / richest(0.0, 1).1 - 3.0).abs() < 0.01, "{pay}");
    }

    #[test]
    fn a_free_agent_signs_where_there_is_room_and_nobody_signs_without_it() {
        let rules = ContractRules::bundled();
        let m = rules.market[&Economy::Mlb].clone();
        let star = || FreeAgent::new(hitter(500, 1.0), 29.0, Some(0), Why::Service);
        let roster = || {
            let mut d = Depth::new(Economy::Mlb);
            for i in 0..13 {
                d.add(&rules, &hitter(i, 0.0));
            }
            d
        };
        // The first organisation has spent its budget; the second has room.
        let mut bidders = vec![
            Bidder { depth: roster(), committed: 2.0e8, budget: 1.0e8, depth_signed: 0, outside_left: None, depth_only: false },
            Bidder { depth: roster(), committed: 0.0, budget: 2.0e8, depth_signed: 0, outside_left: None, depth_only: false },
        ];
        let out = run(&rules, &m, Economy::Mlb, vec![star()], &mut bidders, 2027, &mut Rng::new(2));
        assert_eq!(out.signed.len(), 1);
        assert_eq!(out.signed[0].org, 1);
        assert!(out.signed[0].major && out.signed[0].pay >= rules.free_agent_pay(rules.war(&hitter(500, 1.0), Economy::Mlb), 2027) - 1.0);
        // With no room anywhere he gets at best a minor-league deal.
        let mut broke = vec![Bidder { depth: roster(), committed: 3.0e8, budget: 1.0e8, depth_signed: 0, outside_left: None, depth_only: false }];
        let out = run(&rules, &m, Economy::Mlb, vec![star()], &mut broke, 2027, &mut Rng::new(3));
        assert!(out.signed.iter().all(|s| !s.major));
    }

    #[test]
    fn compensation_and_the_limit_on_outside_free_agents_bind() {
        let rules = ContractRules::bundled();
        let m = rules.market[&Economy::Kbo].clone();
        let roster = || {
            let mut d = Depth::new(Economy::Kbo);
            for i in 0..13 {
                d.add(&rules, &hitter(i, -0.5));
            }
            d
        };
        let bidder = |committed: f64, outside_left: Option<usize>| Bidder {
            depth: roster(),
            committed,
            budget: 1.0e10,
            depth_signed: 0,
            outside_left,
            depth_only: false,
        };
        let agent = |id: u32, from: Option<usize>, owed: Option<f64>| FreeAgent {
            compensation: owed,
            counts: true,
            ..FreeAgent::new(hitter(id, 1.0), 29.0, from, Why::Service)
        };
        // Club 1 may sign no more of other clubs' free agents: the player stays with club 0.
        let mut bidders = vec![bidder(0.0, Some(1)), bidder(0.0, Some(0))];
        let out = run(&rules, &m, Economy::Kbo, vec![agent(600, Some(0), Some(1.0e8))], &mut bidders, 2027, &mut Rng::new(4));
        assert_eq!(out.signed.len(), 1);
        assert_eq!(out.signed[0].org, 0);
        // A club that signs another's free agent pays his compensation from its budget: club 1
        // has room for his pay but not for the compensation as well (his own club has none).
        let ask = price(&rules, &m, Economy::Kbo, rules.war(&hitter(601, 1.0), Economy::Kbo), 2027);
        let full = 1.0e10 * (1.0 + m.slack);
        let signs_with_1 = |owed: Option<f64>| {
            let mut bidders = vec![bidder(full, Some(1)), bidder(full - 1.5 * ask, Some(1))];
            let out = run(&rules, &m, Economy::Kbo, vec![agent(601, Some(0), owed)], &mut bidders, 2027, &mut Rng::new(5));
            out.signed.iter().any(|s| s.major && s.org == 1)
        };
        assert!(signs_with_1(Some(0.0)));
        assert!(!signs_with_1(Some(2.0 * ask)));
        // Nobody can afford him: only his own club may give him a depth place, while a player
        // with nothing owed for him may take one anywhere.
        let mut bidders = vec![bidder(2.0e10, Some(1)), bidder(2.0e10, Some(1))];
        bidders[0].depth_signed = m.depth_signings;
        let weak = |id: u32, owed: Option<f64>| FreeAgent {
            compensation: owed,
            counts: owed.is_some(),
            ..FreeAgent::new(hitter(id, -0.4), 29.0, Some(0), Why::Service)
        };
        let out = run(&rules, &m, Economy::Kbo, vec![weak(602, Some(0.0)), weak(603, None)], &mut bidders, 2027, &mut Rng::new(6));
        assert!(out.signed.iter().all(|s| !s.major));
        assert!(out.signed.iter().all(|s| s.player.id == 603 && s.org == 1), "{:?}", out.signed.iter().map(|s| (s.player.id, s.org)).collect::<Vec<_>>());
        assert_eq!(out.unsigned.len(), 1);
    }
}
