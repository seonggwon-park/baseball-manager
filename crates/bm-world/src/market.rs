//! The free-agent market of the major-league organisations (`docs/contracts.md`, stage 2).
//!
//! Every offseason the free agents (players whose contract ran out after six years of
//! service, players their organisation did not keep, minor-leaguers whose contract ran out)
//! go to market, best first. Each organisation values a player by the wins he adds to its
//! roster (his WAR against the player he would push out of its 26, `contract.rs`), within
//! its budget; a player asks the free-agent price of his own WAR, more when several
//! organisations want him (decision 2026-11-18: base price plus a premium for competing
//! bids), and takes the best offer, his own organisation's counting for a little more. The
//! length of a contract follows his age and WAR. Those no organisation wants on a
//! major-league contract may sign a minor-league one as Triple-A depth; the rest leave.

use crate::ability;
use crate::contract::{expected_years, ContractRules, Economy};
use crate::player::Player;
use bm_engine::Rng;
use serde::Deserialize;

/// What the market's behaviour rests on (`contracts.json` market): each value is fitted to
/// the measured outcomes of free agents (`docs/contracts.md`, calibration targets).
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
}

pub struct FreeAgent {
    pub player: Player,
    pub age: f64,
    /// The organisation (index) he played for.
    pub from: Option<usize>,
    pub why: Why,
}

/// A bidder's roster for valuing a free agent: the WAR of its position players (playing
/// every day) and of its pitchers as starters (if they can start) and as relievers.
#[derive(Clone, Debug, Default)]
pub struct Depth {
    pub hitters: Vec<f64>,
    pub starters: Vec<f64>,
    pub relievers: Vec<f64>,
}

fn war_as(rules: &ContractRules, p: &Player, starter: bool) -> f64 {
    let r = rules.of(Economy::Mlb);
    let share = if starter { ability::STARTER_SHARE } else { ability::RELIEVER_SHARE } / ability::STAFF_SHARE;
    (ability::value(p) - r.replacement.pitcher) * share / r.runs_per_win()
}

impl Depth {
    pub fn add(&mut self, rules: &ContractRules, p: &Player) {
        if p.is_pitcher() {
            if ability::can_start(p) {
                self.starters.push(war_as(rules, p, true));
            }
            self.relievers.push(war_as(rules, p, false));
        } else {
            self.hitters.push(rules.war(p, Economy::Mlb));
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
        let own = self.place(rules, p, bench_share) * rules.war(p, Economy::Mlb);
        displaced * change + (1.0 - displaced) * own
    }

    /// The share of a regular's time `p` would get here: 1 among the nine regulars or the
    /// rotation and bullpen, `bench_share` on the bench, 0 outside the 26.
    pub fn place(&self, rules: &ContractRules, p: &Player, bench_share: f64) -> f64 {
        let better = |v: &[f64], x: f64| v.iter().filter(|w| **w > x).count();
        if p.is_pitcher() {
            if ability::can_start(p) && better(&self.starters, war_as(rules, p, true)) < 5 {
                return 1.0;
            }
            if better(&self.relievers, war_as(rules, p, false)) < 13 {
                return 1.0;
            }
            0.0
        } else {
            match better(&self.hitters, rules.war(p, Economy::Mlb)) {
                0..=8 => 1.0,
                9..=12 => bench_share,
                _ => 0.0,
            }
        }
    }
}

/// A major-league organisation in the market.
pub struct Bidder {
    pub depth: Depth,
    /// Salaries already owed for the next season (major-league contracts).
    pub committed: f64,
    pub budget: f64,
    pub depth_signed: usize,
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
}

/// The market's result: signings, and the players nobody signed.
pub struct Outcome {
    pub signed: Vec<Signing>,
    pub unsigned: Vec<FreeAgent>,
}

/// Run the market for `year` (the season the contracts begin).
pub fn run(rules: &ContractRules, m: &MarketRules, mut agents: Vec<FreeAgent>, bidders: &mut [Bidder], year: i32, rng: &mut Rng) -> Outcome {
    let r = rules.of(Economy::Mlb);
    let price = |war: f64| rules.free_agent_pay(war, year);
    // Best first.
    agents.sort_by(|a, b| rules.war(&b.player, Economy::Mlb).total_cmp(&rules.war(&a.player, Economy::Mlb)));
    let mut signed = Vec::new();
    let mut left = Vec::new();
    for fa in agents {
        let war = rules.war(&fa.player, Economy::Mlb);
        let ask = price(war);
        let years = (expected_years(fa.age, war) + rng.gauss(0.0, 0.5)).round().clamp(1.0, 8.0) as usize;
        // Who wants him: every organisation he would make the 26 of. A regular (in the
        // lineup, the rotation or the bullpen) is offered his price, a bench player that share
        // of it (not less than the minimum); the price of a regular rises with the number of
        // organisations that want him as one. An organisation offers what its budget leaves.
        let min = r.minimum(year);
        let wants: Vec<(usize, f64)> = bidders
            .iter()
            .enumerate()
            .filter_map(|(k, b)| {
                let place = b.depth.place(rules, &fa.player, m.bench_share);
                (place > 0.0).then_some((k, place))
            })
            .collect();
        let regulars = wants.iter().filter(|(_, place)| *place >= 1.0).count().min(m.max_bidders);
        let premium = 1.0 + m.premium * regulars.saturating_sub(1) as f64;
        let offers: Vec<(usize, f64)> = wants
            .iter()
            .filter_map(|&(k, place)| {
                let pay = if place >= 1.0 { ask * premium } else { (place * ask).max(min) };
                let b = &bidders[k];
                (b.committed + pay <= b.budget * (1.0 + m.slack)).then_some((k, pay))
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
            let b = &mut bidders[k];
            b.committed += pay;
            b.depth.add(rules, &fa.player);
            // A bench player signs for a year.
            let years = if pay < ask { 1 } else { years };
            signed.push(Signing { player: fa.player, org: k, major: true, pay, years, why: fa.why });
            continue;
        }
        left.push(fa);
    }
    // Minor-league deals: Triple-A depth for the organisations he would improve most.
    let mut unsigned = Vec::new();
    for fa in left {
        let best = bidders
            .iter()
            .enumerate()
            .filter(|(_, b)| b.depth_signed < m.depth_signings)
            .map(|(k, b)| (k, depth_gain(rules, &b.depth, &fa.player)))
            .filter(|(_, g)| *g > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1));
        match best {
            Some((k, _)) => {
                let b = &mut bidders[k];
                b.depth_signed += 1;
                b.depth.add(rules, &fa.player);
                let pay = r.minor_pay("aaa", year);
                signed.push(Signing { player: fa.player, org: k, major: false, pay, years: 1, why: fa.why });
            }
            None => unsigned.push(fa),
        }
    }
    Outcome { signed, unsigned }
}

/// How much a player improves an organisation's depth beyond its 26: whether he would rank
/// among its best 54 of his kind (the majors and Triple-A), by how much over the last of them.
fn depth_gain(rules: &ContractRules, d: &Depth, p: &Player) -> f64 {
    let (pool, mine, n) = if p.is_pitcher() { (&d.relievers, war_as(rules, p, false), 27) } else { (&d.hitters, rules.war(p, Economy::Mlb), 27) };
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
        let mut d = Depth::default();
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
    fn a_free_agent_signs_where_there_is_room_and_nobody_signs_without_it() {
        let rules = ContractRules::bundled();
        let m = rules.market.clone().expect("market rules");
        let star = || FreeAgent { player: hitter(500, 1.0), age: 29.0, from: Some(0), why: Why::Service };
        let roster = || {
            let mut d = Depth::default();
            for i in 0..13 {
                d.add(&rules, &hitter(i, 0.0));
            }
            d
        };
        // The first organisation has spent its budget; the second has room.
        let mut bidders = vec![
            Bidder { depth: roster(), committed: 2.0e8, budget: 1.0e8, depth_signed: 0 },
            Bidder { depth: roster(), committed: 0.0, budget: 2.0e8, depth_signed: 0 },
        ];
        let out = run(&rules, &m, vec![star()], &mut bidders, 2027, &mut Rng::new(2));
        assert_eq!(out.signed.len(), 1);
        assert_eq!(out.signed[0].org, 1);
        assert!(out.signed[0].major && out.signed[0].pay >= rules.free_agent_pay(rules.war(&hitter(500, 1.0), Economy::Mlb), 2027) - 1.0);
        // With no room anywhere he gets at best a minor-league deal.
        let mut broke = vec![Bidder { depth: roster(), committed: 3.0e8, budget: 1.0e8, depth_signed: 0 }];
        let out = run(&rules, &m, vec![star()], &mut broke, 2027, &mut Rng::new(3));
        assert!(out.signed.iter().all(|s| !s.major));
    }
}
