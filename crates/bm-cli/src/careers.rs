//! `bm careers`: make the world and run offseasons back to back without playing the
//! seasons in between (development does not depend on games yet), then compare each
//! level's talent and ages with the level table, and the prospects' outcomes with
//! published reach rates.

use bm_engine::Rng;
use bm_world::ability;
use bm_world::career::{offseason, talent_of, Offseason, Origin};
use bm_world::{
    populate_with, AgingTable, Attr, Budgets, CareerTable, ContractRules, Date, Economy, LevelTable, PeopleMaker, Population, Squad, Talent, World, N_ATTR,
};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const LEVELS: [&str; 11] = ["mlb", "aaa", "aa", "high_a", "single_a", "complex", "academy", "kbo", "kbo2", "npb", "npb2"];

/// One level's players at one moment: how many, mean ages of batters and pitchers, and
/// the talent offsets the domestic players show.
#[derive(Clone, Copy, Default)]
struct LevelStats {
    n: f64,
    age_b: f64,
    age_p: f64,
    talent: [f64; 8],
    /// Mean value of the position players and of the pitchers, runs per 162 games.
    value_h: f64,
    value_p: f64,
    /// Domestic batters and pitchers by whole years of age, from `HIST_FROM`.
    hist_b: [f64; HIST_N],
    hist_p: [f64; HIST_N],
}

const HIST_FROM: usize = 15;
const HIST_N: usize = 30;

impl LevelStats {
    fn add(&mut self, o: &LevelStats) {
        self.n += o.n;
        self.age_b += o.age_b;
        self.age_p += o.age_p;
        self.value_h += o.value_h;
        self.value_p += o.value_p;
        for i in 0..8 {
            self.talent[i] += o.talent[i];
        }
        for i in 0..HIST_N {
            self.hist_b[i] += o.hist_b[i];
            self.hist_p[i] += o.hist_p[i];
        }
    }

    fn scaled(&self, k: f64) -> LevelStats {
        LevelStats {
            n: self.n * k,
            age_b: self.age_b * k,
            age_p: self.age_p * k,
            talent: self.talent.map(|x| x * k),
            value_h: self.value_h * k,
            value_p: self.value_p * k,
            hist_b: self.hist_b.map(|x| x * k),
            hist_p: self.hist_p.map(|x| x * k),
        }
    }
}

/// Value of a position player in his best role, of a pitcher as a starter (so that the
/// mix of starters and relievers on a roster does not move a level's mean).
fn worth(p: &bm_world::Player) -> f64 {
    if p.is_pitcher() {
        ability::pitching(p, bm_world::Role::Starter)
    } else {
        ability::value(p)
    }
}

fn stats(pop: &Population) -> BTreeMap<&'static str, LevelStats> {
    let age = |id: u32| pop.date.days_since(pop.people[&id].born) as f64 / 365.2425;
    let mut out = BTreeMap::new();
    for key in LEVELS {
        let players: Vec<_> = pop.rosters.iter().filter(|r| r.level == key).flat_map(|r| r.players()).filter(|p| !pop.foreign.contains_key(&p.id)).collect();
        if players.is_empty() {
            continue;
        }
        let mean_age = |pitchers: bool| {
            let a: Vec<f64> = players.iter().filter(|p| p.is_pitcher() == pitchers).map(|p| age(p.id)).collect();
            a.iter().sum::<f64>() / a.len().max(1) as f64
        };
        let value = |pitchers: bool| {
            let v: Vec<f64> = players.iter().filter(|p| p.is_pitcher() == pitchers).map(|p| worth(p)).collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        };
        let hist = |pitchers: bool| {
            let mut h = [0.0; HIST_N];
            for p in players.iter().filter(|p| p.is_pitcher() == pitchers) {
                let i = (age(p.id).floor().max(0.0) as usize).saturating_sub(HIST_FROM).min(HIST_N - 1);
                h[i] += 1.0;
            }
            h
        };
        out.insert(
            key,
            LevelStats {
                n: players.len() as f64,
                age_b: mean_age(false),
                age_p: mean_age(true),
                talent: talent_of(players.iter().copied()),
                value_h: value(false),
                value_p: value(true),
                hist_b: hist(false),
                hist_p: hist(true),
            },
        );
    }
    out
}

/// Mean and standard deviation of every attribute (z-score) of a level's domestic position
/// players and pitchers: what the engine sees, beyond the talent parts.
#[derive(Clone, Default)]
struct AttrMoments {
    /// level -> [hitters, pitchers] -> attribute -> (mean, sd)
    m: BTreeMap<&'static str, [[(f64, f64); N_ATTR]; 2]>,
    /// level -> pitchers' pitch grades (usage-weighted, main fastball left out, as in
    /// `ability::pitching_rate`): (mean, sd)
    grades: BTreeMap<&'static str, (f64, f64)>,
    /// level -> mean stamina of the rotations' starters and of the bullpens' relievers
    staffs: BTreeMap<&'static str, (f64, f64)>,
    /// level -> lineups (the designated hitter left out): share of regulars at their own
    /// position, mean familiarity with the position played (0-1)
    fields: BTreeMap<&'static str, (f64, f64)>,
    /// level -> mean pitching rate (`ability::pitching_rate`) of the rotations and bullpens
    rates: BTreeMap<&'static str, (f64, f64)>,
    /// level -> correlations between attributes: hitters' contact-power, contact-discipline,
    /// power-discipline; pitchers' stuff-command, stuff-velocity
    corr: BTreeMap<&'static str, [f64; 5]>,
}

impl AttrMoments {
    fn of(pop: &Population) -> AttrMoments {
        let mut m = BTreeMap::new();
        let mut grades = BTreeMap::new();
        let mut staffs = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut rates = BTreeMap::new();
        let mut corr = BTreeMap::new();
        for key in LEVELS {
            let players: Vec<_> =
                pop.rosters.iter().filter(|r| r.level == key).flat_map(|r| r.players()).filter(|p| !pop.foreign.contains_key(&p.id)).collect();
            if players.is_empty() {
                continue;
            }
            let g: Vec<f64> = players
                .iter()
                .filter_map(|p| p.pitching.as_ref())
                .filter_map(|pit| {
                    let main = bm_world::convert::main_fastball(&pit.pitches);
                    let (s, w) = pit
                        .pitches
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| Some(*i) != main)
                        .fold((0.0, 0.0), |a, (_, k)| (a.0 + (k.usage * k.grade) as f64, a.1 + k.usage as f64));
                    (w > 0.0).then(|| s / w)
                })
                .collect();
            let stamina = |which: &dyn Fn(&bm_world::Roster) -> Vec<f64>| {
                let v: Vec<f64> = pop.rosters.iter().filter(|r| r.level == key).flat_map(which).collect();
                v.iter().sum::<f64>() / v.len().max(1) as f64
            };
            let rot = stamina(&|r| r.squad.starters.iter().map(|p| p.attrs.z(Attr::Stamina)).collect());
            let pen = stamina(&|r| r.squad.relievers.iter().map(|p| p.attrs.z(Attr::Stamina)).collect());
            staffs.insert(key, (rot, pen));
            let rot_q = stamina(&|r| r.squad.starters.iter().map(ability::pitching_rate).collect());
            let pen_q = stamina(&|r| r.squad.relievers.iter().map(ability::pitching_rate).collect());
            rates.insert(key, (rot_q, pen_q));
            let correlation = |pitchers: bool, a: Attr, b: Attr| {
                let v: Vec<(f64, f64)> = players.iter().filter(|p| p.is_pitcher() == pitchers).map(|p| (p.attrs.z(a), p.attrs.z(b))).collect();
                let n = v.len().max(1) as f64;
                let (ma, mb) = (v.iter().map(|x| x.0).sum::<f64>() / n, v.iter().map(|x| x.1).sum::<f64>() / n);
                let cov = v.iter().map(|x| (x.0 - ma) * (x.1 - mb)).sum::<f64>() / n;
                let (sa, sb) = (v.iter().map(|x| (x.0 - ma).powi(2)).sum::<f64>() / n, v.iter().map(|x| (x.1 - mb).powi(2)).sum::<f64>() / n);
                cov / (sa * sb).sqrt().max(1e-9)
            };
            corr.insert(
                key,
                [
                    correlation(false, Attr::Contact, Attr::Power),
                    correlation(false, Attr::Contact, Attr::Discipline),
                    correlation(false, Attr::Power, Attr::Discipline),
                    correlation(true, Attr::Stuff, Attr::Command),
                    correlation(true, Attr::Stuff, Attr::Velocity),
                ],
            );
            let slots: Vec<(bool, f64)> = pop
                .rosters
                .iter()
                .filter(|r| r.level == key)
                .flat_map(|r| r.squad.hitters.iter())
                .filter_map(|(p, pos)| pos.map(|pos| (p.position == pos, p.familiarity(pos))))
                .collect();
            let n = slots.len().max(1) as f64;
            fields.insert(key, (slots.iter().filter(|s| s.0).count() as f64 / n, slots.iter().map(|s| s.1).sum::<f64>() / n));
            let gm = g.iter().sum::<f64>() / g.len().max(1) as f64;
            grades.insert(key, (gm, (g.iter().map(|x| (x - gm).powi(2)).sum::<f64>() / g.len().max(1) as f64).sqrt()));
            let mut out = [[(0.0, 0.0); N_ATTR]; 2];
            for (side, pitchers) in [false, true].into_iter().enumerate() {
                let group: Vec<_> = players.iter().filter(|p| p.is_pitcher() == pitchers).collect();
                let n = group.len().max(1) as f64;
                for (i, a) in Attr::ALL.into_iter().enumerate() {
                    let mean = group.iter().map(|p| p.attrs.z(a)).sum::<f64>() / n;
                    let var = group.iter().map(|p| (p.attrs.z(a) - mean).powi(2)).sum::<f64>() / n;
                    out[side][i] = (mean, var.sqrt());
                }
            }
            m.insert(key, out);
        }
        AttrMoments { m, grades, staffs, fields, rates, corr }
    }

    fn add(&mut self, o: &AttrMoments, k: f64) {
        for (key, g) in &o.grades {
            let e = self.grades.entry(key).or_insert((0.0, 0.0));
            e.0 += k * g.0;
            e.1 += k * g.1;
        }
        for (key, g) in &o.staffs {
            let e = self.staffs.entry(key).or_insert((0.0, 0.0));
            e.0 += k * g.0;
            e.1 += k * g.1;
        }
        for (key, g) in &o.fields {
            let e = self.fields.entry(key).or_insert((0.0, 0.0));
            e.0 += k * g.0;
            e.1 += k * g.1;
        }
        for (key, g) in &o.rates {
            let e = self.rates.entry(key).or_insert((0.0, 0.0));
            e.0 += k * g.0;
            e.1 += k * g.1;
        }
        for (key, c) in &o.corr {
            let e = self.corr.entry(key).or_insert([0.0; 5]);
            for i in 0..5 {
                e[i] += k * c[i];
            }
        }
        for (key, sides) in &o.m {
            let e = self.m.entry(key).or_insert([[(0.0, 0.0); N_ATTR]; 2]);
            for s in 0..2 {
                for i in 0..N_ATTR {
                    e[s][i].0 += k * sides[s][i].0;
                    e[s][i].1 += k * sides[s][i].1;
                }
            }
        }
    }

    fn json(&self) -> serde_json::Value {
        let side = |v: &[(f64, f64); N_ATTR]| -> serde_json::Map<String, serde_json::Value> {
            Attr::ALL.iter().zip(v).map(|(a, (m, s))| (a.key().to_string(), serde_json::json!([m, s]))).collect()
        };
        self.m
            .iter()
            .map(|(k, v)| {
                let mut p = side(&v[1]);
                if let Some(g) = self.grades.get(k) {
                    p.insert("grades".to_string(), serde_json::json!([g.0, g.1]));
                }
                if let Some(s) = self.staffs.get(k) {
                    p.insert("rotation_stamina".to_string(), serde_json::json!([s.0, 0.0]));
                    p.insert("bullpen_stamina".to_string(), serde_json::json!([s.1, 0.0]));
                }
                if let Some(s) = self.rates.get(k) {
                    p.insert("rotation_rate".to_string(), serde_json::json!([s.0, 0.0]));
                    p.insert("bullpen_rate".to_string(), serde_json::json!([s.1, 0.0]));
                }
                if let Some(c) = self.corr.get(k) {
                    p.insert("corr".to_string(), serde_json::json!([c[3], c[4]]));
                }
                let mut h = side(&v[0]);
                if let Some(c) = self.corr.get(k) {
                    h.insert("corr".to_string(), serde_json::json!([c[0], c[1], c[2]]));
                }
                if let Some(f) = self.fields.get(k) {
                    h.insert("own_position".to_string(), serde_json::json!([f.0, 0.0]));
                    h.insert("familiarity".to_string(), serde_json::json!([f.1, 0.0]));
                }
                (k.to_string(), serde_json::json!({"h": h, "p": p}))
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
}

/// What a level's players would be worth if they were drawn at its talent: mean value of
/// position players and pitchers, and how much each changes per standard deviation of
/// talent (all of hitting, pitching, fielding and physical together).
#[derive(Clone, Copy)]
struct ValueTarget {
    h: f64,
    p: f64,
    slope_h: f64,
    slope_p: f64,
}

fn value_targets(levels: &LevelTable) -> BTreeMap<&'static str, ValueTarget> {
    let mean = |t: &Talent| -> (f64, f64) {
        let mut rng = Rng::new(77);
        let (mut h, mut p, mut nh, mut np) = (0.0, 0.0, 0.0, 0.0);
        for i in 0..120 {
            let s = Squad::random(&mut rng, 1000 * (i + 1), t);
            for (x, _) in s.hitters.iter().chain(&s.bench) {
                h += worth(x);
                nh += 1.0;
            }
            for x in s.starters.iter().chain(&s.relievers) {
                p += worth(x);
                np += 1.0;
            }
        }
        (h / nh, p / np)
    };
    let mut out = BTreeMap::new();
    for key in LEVELS {
        let t = levels.get(key).talent();
        let (h, p) = mean(&t);
        let up = Talent { hitting: t.hitting + 0.5, pitching: t.pitching + 0.5, fielding: t.fielding + 0.5, physical: t.physical + 0.5, ..t };
        let (h1, p1) = mean(&up);
        out.insert(key, ValueTarget { h, p, slope_h: (h1 - h) / 0.5, slope_p: (p1 - p) / 0.5 });
    }
    out
}

fn print_table(rows: &BTreeMap<&'static str, LevelStats>, levels: &LevelTable, targets: &BTreeMap<&'static str, ValueTarget>) {
    println!("  level       n   age B  age P   hitting  pitching fielding physical  power  command  veloc  discip | value H  value P  (level table below; gap in SD)");
    for key in LEVELS {
        let Some(s) = rows.get(key) else { continue };
        let t = s.talent;
        println!(
            "  {:<9} {:5.0}  {:5.2}  {:5.2}   {:+7.2}  {:+7.2}  {:+7.2}  {:+7.2}  {:+6.2}  {:+6.2}  {:+5.2}  {:+6.2} | {:+7.1}  {:+7.1}",
            key, s.n, s.age_b, s.age_p, t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], s.value_h, s.value_p
        );
        let target = levels.get(key);
        let g: [f32; 8] = std::array::from_fn(|i| target.talent.get(i).copied().unwrap_or(0.0));
        let v = targets[key];
        println!(
            "  {:<9} {:5}  {:5.2}  {:5.2}   {:+7.2}  {:+7.2}  {:+7.2}  {:+7.2}  {:+6.2}  {:+6.2}  {:+5.2}  {:+6.2} | {:+7.1}  {:+7.1}  gap {:+5.2} {:+5.2}",
            "",
            "",
            target.batters.mean,
            target.pitchers.mean,
            g[0],
            g[1],
            g[2],
            g[3],
            g[4],
            g[5],
            g[6],
            g[7],
            v.h,
            v.p,
            (s.value_h - v.h) / v.slope_h,
            (s.value_p - v.p) / v.slope_p
        );
    }
}

/// Spread of the first teams' strength: standard deviation over the major-league clubs of
/// the mean value of their 26 players, runs per 162 games per player.
/// Counts of several markets, "a/b/c".
fn slashed(v: &[usize]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("/")
}

fn spread(pop: &Population, world: &World) -> f64 {
    let means: Vec<f64> = world
        .first_teams()
        .filter(|c| c.country == "US")
        .filter_map(|c| pop.roster(&c.id))
        .map(|r| r.players().map(bm_world::ability::value).sum::<f64>() / 26.0)
        .collect();
    let m = means.iter().sum::<f64>() / means.len() as f64;
    (means.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / means.len() as f64).sqrt()
}

/// Published reach rates of signed draftees by round, 1996-2011 drafts (Karcher 2017;
/// docs/research/world/aging.md 4B): (first round, last round, share that played in MLB).
const REACH: [(u32, u32, f64); 8] =
    [(1, 1, 0.667), (2, 2, 0.494), (3, 3, 0.397), (4, 4, 0.350), (5, 5, 0.333), (6, 10, 0.20), (11, 15, 0.127), (16, 20, 0.099)];

fn outcomes(pop: &Population, last_year: i32, follow: i32) -> serde_json::Value {
    println!(
        "\nProspects drafted or signed by {}, followed {follow} or more years (reached = among the organisation's {} best):",
        last_year - follow,
        bm_world::career::USED_PER_SEASON
    );
    let mut by_round: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
    let mut intl = (0.0f64, 0.0f64);
    let mut debut = (0.0f64, 0.0f64);
    for (id, c) in &pop.careers {
        let joined = match &c.origin {
            Origin::Draft { year, .. } | Origin::Signed { year, .. } | Origin::Foreign { year } => *year,
            Origin::Initial => continue,
        };
        if let Some(y) = c.majors {
            if !matches!(c.origin, Origin::Foreign { .. }) {
                debut.0 += Date::new(y, 7, 1).days_since(pop.people[id].born) as f64 / 365.2425;
                debut.1 += 1.0;
            }
        }
        if joined > last_year - follow {
            continue;
        }
        let reached = f64::from(u8::from(c.majors.is_some()));
        match &c.origin {
            Origin::Draft { round, class, .. } if class.starts_with("us_") => {
                if let Some(k) = REACH.iter().position(|(a, b, _)| (*a..=*b).contains(round)) {
                    let e = by_round.entry(k).or_default();
                    e.0 += reached;
                    e.1 += 1.0;
                }
            }
            Origin::Signed { class, .. } if class == "international" => {
                intl.0 += reached;
                intl.1 += 1.0;
            }
            _ => {}
        }
    }
    println!("  round      players   reached   published (played in MLB)");
    for (k, (a, b, real)) in REACH.iter().enumerate() {
        let (r, n) = by_round.get(&k).copied().unwrap_or_default();
        let label = if a == b { format!("{a}") } else { format!("{a}-{b}") };
        println!("  {:<9} {:7.0}   {:6.1} %   {:5.1} %", label, n, 100.0 * r / n.max(1.0), 100.0 * real);
    }
    println!(
        "  international amateurs: {:.0} signed, {:.1} % reached (Dominican signees 2004-2012: 5.4 %, Perez Nero 2019)",
        intl.1,
        100.0 * intl.0 / intl.1.max(1.0)
    );
    println!("  age on 1 July of the first season among them: {:.2} (MLB debuts 2015-2019: 25.0-25.3, Justice 2020)", debut.0 / debut.1.max(1.0));
    let rounds: Vec<serde_json::Value> = REACH
        .iter()
        .enumerate()
        .map(|(k, (a, b, real))| {
            let (r, n) = by_round.get(&k).copied().unwrap_or_default();
            serde_json::json!({"from": a, "to": b, "n": n, "reached": r / n.max(1.0), "published": real})
        })
        .collect();
    serde_json::json!({"rounds": rounds, "international": intl.0 / intl.1.max(1.0), "debut_age": debut.0 / debut.1.max(1.0)})
}

/// Payrolls against budgets and the spread of salaries in each economy for `year`, printed
/// and returned for `--json`.
fn payroll_summary(world: &World, pop: &Population, rules: &ContractRules, year: i32) -> serde_json::Value {
    use bm_world::contract::{payrolls, Kind, Terms};
    let budgets = Budgets::bundled();
    let rows = payrolls(world, pop, rules, &budgets, year);
    let pct = |v: &[f64], q: f64| -> f64 {
        if v.is_empty() {
            return 0.0;
        }
        v[((v.len() - 1) as f64 * q).round() as usize]
    };
    let mut out = serde_json::Map::new();
    println!("\nPayrolls and salaries for {year} (major-league contracts in the majors' system, registered domestic players in the KBO and NPB):");
    for e in Economy::ALL {
        let mine: Vec<_> = rows.iter().filter(|r| r.economy == e).collect();
        let mut totals: Vec<f64> = mine.iter().map(|r| r.total).collect();
        totals.sort_by(f64::total_cmp);
        let budget: f64 = mine.iter().filter_map(|r| r.budget).sum();
        let sum: f64 = totals.iter().sum();
        let ratio: Vec<f64> = mine.iter().filter_map(|r| r.budget.map(|b| r.total / b)).collect();
        let mut pay: Vec<f64> = pop
            .contracts
            .values()
            .filter(|c| matches!(c.kind, Kind::Major | Kind::Registered) && c.terms != Terms::Foreign && rows.iter().any(|r| r.org == c.org && r.economy == e))
            .filter_map(|c| c.pay(year))
            .collect();
        pay.sort_by(f64::total_cmp);
        let min = rules.of(e).minimum(year);
        let at_min = pay.iter().filter(|x| **x <= min * 1.0001).count() as f64 / pay.len().max(1) as f64;
        let mean = pay.iter().sum::<f64>() / pay.len().max(1) as f64;
        let by_terms = |terms: Terms| -> f64 {
            pop.contracts
                .values()
                .filter(|c| c.terms == terms && matches!(c.kind, Kind::Major | Kind::Registered) && rows.iter().any(|r| r.org == c.org && r.economy == e))
                .filter_map(|c| c.pay(year))
                .sum::<f64>()
                / sum.max(1.0)
        };
        let shares = [Terms::Initial, Terms::Amateur, Terms::Renewal, Terms::Arbitration, Terms::FreeAgent].map(by_terms);
        println!(
            "  {:3}: {} clubs, payroll {:.4e} against budgets {:.4e} ({:.2}); club payroll min {:.4e} median {:.4e} max {:.4e}; {} salaries, mean {:.4e}, median {:.4e}, p90 {:.4e}, {:.0} % at the minimum {:.4e}; share of pay initial/amateur/renewal/arbitration/free agent {:.2}/{:.2}/{:.2}/{:.2}/{:.2}",
            e.key(),
            mine.len(),
            sum,
            budget,
            sum / budget.max(1.0),
            pct(&totals, 0.0),
            pct(&totals, 0.5),
            pct(&totals, 1.0),
            pay.len(),
            mean,
            pct(&pay, 0.5),
            pct(&pay, 0.9),
            100.0 * at_min,
            min,
            shares[0],
            shares[1],
            shares[2],
            shares[3],
            shares[4]
        );
        out.insert(
            e.key().to_string(),
            serde_json::json!({
                "payroll": sum, "budget": budget, "club_min": pct(&totals, 0.0), "club_median": pct(&totals, 0.5), "club_max": pct(&totals, 1.0),
                "ratio_mean": ratio.iter().sum::<f64>() / ratio.len().max(1) as f64, "salaries": pay.len(), "mean": mean, "median": pct(&pay, 0.5),
                "p10": pct(&pay, 0.1), "p90": pct(&pay, 0.9), "at_minimum": at_min, "shares": shares,
            }),
        );
    }
    serde_json::Value::Object(out)
}

/// One rostered player before an offseason.
struct Before {
    id: u32,
    level: String,
    /// Where he stood on the roster: lineup, bench, starter, reliever or reserve.
    slot: &'static str,
    pitcher: bool,
    /// Value in runs per 162 games (`ability::value`) and in standard deviations
    /// (`ability::per_sd`).
    value: f64,
    value_sd: f64,
    /// A KBO or NPB club's foreign player.
    foreign: bool,
    /// Wins above replacement with his economy's first team (`contract::ContractRules::war`).
    war: f64,
    /// His contract for the season just played: kind, terms and salary.
    contract: String,
}

/// Every rostered player before an offseason.
fn snapshot(pop: &Population, rules: &ContractRules) -> Vec<Before> {
    let year = pop.date.year();
    let mut out = Vec::new();
    for r in &pop.rosters {
        let s = &r.squad;
        let groups: [(&'static str, Vec<&bm_world::Player>); 5] = [
            ("lineup", s.hitters.iter().map(|(p, _)| p).collect()),
            ("bench", s.bench.iter().map(|(p, _)| p).collect()),
            ("reserve", r.reserves.iter().map(|(p, _)| p).collect()),
            ("starter", s.starters.iter().collect()),
            ("reliever", s.relievers.iter().collect()),
        ];
        for (slot, players) in groups {
            for p in players {
                let value = ability::value(p);
                let economy = match bm_world::populate::peak_level(&r.level) {
                    "kbo" => Economy::Kbo,
                    "npb" => Economy::Npb,
                    _ => Economy::Mlb,
                };
                let service = pop.careers.get(&p.id).map_or(0, |c| bm_world::contract::first_team_seasons(c, economy.top_level(), year + 1));
                let contract = pop.contracts.get(&p.id).map_or_else(
                    || format!(",,,,,{service}"),
                    |c| format!("{},{:?},{:?},{},{:.0},{service}", c.org, c.kind, c.terms, c.last() - year, c.pay(year).unwrap_or(f64::NAN)),
                );
                out.push(Before {
                    id: p.id,
                    level: r.level.clone(),
                    slot,
                    pitcher: p.is_pitcher(),
                    value,
                    value_sd: value / ability::per_sd(p),
                    foreign: pop.foreign.contains_key(&p.id),
                    war: rules.war(p, economy),
                    contract,
                });
            }
        }
    }
    out
}

/// What became of the players of `before` in the offseason after the `year` season, one CSV
/// row each: age on 1 November, level and value before, and the outcome (stayed, with the
/// new level; away on military duty; or how he left).
fn write_outcomes(out: &mut impl std::io::Write, pop: &Population, before: &[Before], year: i32) {
    let now: BTreeMap<u32, &str> = pop.rosters.iter().flat_map(|r| r.players().map(move |p| (p.id, r.level.as_str()))).collect();
    let nov = Date::new(year, 11, 1);
    let after = |id: u32| -> String {
        pop.contracts.get(&id).map_or_else(
            || ",,,,".to_string(),
            |c| format!("{},{:?},{:?},{},{:.0}", c.org, c.kind, c.terms, c.last() - year, c.pay(year + 1).unwrap_or(f64::NAN)),
        )
    };
    for b in before {
        let id = &b.id;
        let age = pop.people.get(id).map_or(f64::NAN, |p| nov.days_since(p.born) as f64 / 365.2425);
        let left = pop.careers.get(id).and_then(|c| c.left).filter(|(y, _)| *y == year);
        let (outcome, new_level) = match (left, now.get(id)) {
            (Some((_, exit)), _) => (format!("{exit:?}"), ""),
            (None, Some(l)) => ("Stayed".to_string(), *l),
            (None, None) if pop.away.contains_key(id) => ("Away".to_string(), ""),
            _ => ("Unknown".to_string(), ""),
        };
        writeln!(
            out,
            "{year},{id},{age:.2},{},{},{},{:.2},{:.3},{},{:.3},{},{outcome},{new_level},{}",
            b.level,
            b.slot,
            u8::from(b.pitcher),
            b.value,
            b.value_sd,
            u8::from(b.foreign),
            b.war,
            b.contract,
            after(*id)
        )
        .expect("write --out");
    }
}

/// Run `years` offseasons, print the levels every `every` years and their average from
/// offseason `from` on; with `out`, write every player's outcome of each offseason from
/// `from` on as CSV.
#[allow(clippy::too_many_arguments)]
pub fn run(
    years: usize,
    seed: u64,
    every: usize,
    from: Option<usize>,
    levels_file: Option<&Path>,
    careers_file: Option<&Path>,
    aging_file: Option<&Path>,
    contracts_file: Option<&Path>,
    json: Option<&Path>,
    out: Option<&Path>,
) {
    let read = |p: &Path| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let world = World::bundled();
    let levels = levels_file.map_or_else(LevelTable::bundled, |p| LevelTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let aging = aging_file.map_or_else(AgingTable::bundled, |p| AgingTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let careers = careers_file.map_or_else(CareerTable::bundled, |p| CareerTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let people = PeopleMaker::new();
    let contracts =
        contracts_file.map_or_else(ContractRules::bundled, |p| ContractRules::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let ctx = Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people, contracts: &contracts };
    let t0 = Instant::now();
    let targets = value_targets(&levels);
    let mut pop = populate_with(&world, &levels, &aging, &careers, Date::new(2027, 2, 1), seed);
    if contracts_file.is_some() {
        bm_world::contract::initial(&world, &mut pop, &contracts, seed);
    }
    println!("World of {} players on {} (first-team spread {:.2} runs per player)", pop.players().count(), pop.date, spread(&pop, &world));
    print_table(&stats(&pop), &levels, &targets);
    let from = from.unwrap_or(years / 2).min(years.saturating_sub(1));
    let mut sum: BTreeMap<&'static str, LevelStats> = BTreeMap::new();
    let mut spreads = 0.0;
    let mut counted = 0.0;
    let mut last = 2027;
    let mut moments: Vec<AttrMoments> = Vec::new();
    if years == 0 {
        moments.push(AttrMoments::of(&pop));
    }
    let mut out = out.map(|p| {
        let mut f = std::io::BufWriter::new(std::fs::File::create(p).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
        writeln!(f, "year,id,age,level,slot,pitcher,value,value_sd,foreign,war,org,kind,terms,years_left,salary,service,outcome,new_level,new_org,new_kind,new_terms,new_years,new_salary").expect("write --out");
        f
    });
    for y in 0..years {
        let year = 2027 + y as i32;
        let before = if out.is_some() && y >= from { snapshot(&pop, &contracts) } else { Vec::new() };
        let r = offseason(&ctx, &mut pop, year, None, seed);
        last = year;
        if let Some(f) = out.as_mut().filter(|_| y >= from) {
            write_outcomes(f, &pop, &before, year);
        }
        if y >= from {
            for (k, s) in stats(&pop) {
                sum.entry(k).or_default().add(&s);
            }
            spreads += spread(&pop, &world);
            counted += 1.0;
            moments.push(AttrMoments::of(&pop));
        }
        if every > 0 && ((y + 1) % every == 0 || y + 1 == years) {
            println!(
                "\nAfter the {year} offseason ({} players, {} people so far): retired {}, released {}, minor-league free agents {}, drafted {}, signed {}, foreign {} in / {} out, military service {} in ({} with Sangmu) / {} back, {} exempt; markets (MLB/KBO/NPB) {} free agents: {} first-team and {} depth contracts, {} unsigned (MLB before it: {} of {} opt-outs used, club options {} exercised / {} declined, {} not tendered, {} qualifying offers, {} accepted, {} compensation picks; KBO/NPB free agency: {} eligible, {} declared, {} stayed, {} moved, {} compensation players, {} NPB compensation picks); first-team spread {:.2}",
                pop.players().count(),
                pop.people.len(),
                r.retired,
                r.released,
                r.minor_free_agents,
                r.drafted,
                r.signed,
                r.foreign_signed,
                r.foreign_replaced,
                r.enlisted,
                r.sangmu,
                r.discharged,
                r.exempt,
                slashed(&r.free_agents),
                slashed(&r.signed_major),
                slashed(&r.signed_minor),
                slashed(&r.unsigned),
                r.opted_out,
                r.opt_outs_due,
                r.options_exercised,
                r.options_declined,
                r.non_tendered,
                r.qualifying_offers,
                r.qualifying_accepted,
                r.compensation_picks,
                slashed(&r.fa_eligible),
                slashed(&r.fa_declared),
                slashed(&r.fa_stayed),
                slashed(&r.fa_moved),
                r.compensation_players,
                r.npb_compensation_picks,
                spread(&pop, &world)
            );
            print_table(&stats(&pop), &levels, &targets);
        }
    }
    let mean: BTreeMap<&'static str, LevelStats> = sum.iter().map(|(k, s)| (*k, s.scaled(1.0 / counted))).collect();
    println!("\nAverage after offseasons {} to {years} (first-team spread {:.2}):", from + 1, spreads / counted);
    print_table(&mean, &levels, &targets);
    let summary = outcomes(&pop, last, 7);
    let money = payroll_summary(&world, &pop, &contracts, pop.date.year());
    println!("\n{years} offseasons in {:.1} s", t0.elapsed().as_secs_f64());
    if let Some(path) = json {
        let levels_json: serde_json::Map<String, serde_json::Value> = mean
            .iter()
            .map(|(k, s)| {
                let v = targets[k];
                let target = levels.get(k);
                (
                    k.to_string(),
                    serde_json::json!({
                        "n": s.n, "age_b": s.age_b, "age_p": s.age_p, "talent": s.talent,
                        "target_age_b": target.batters.mean, "target_age_p": target.pitchers.mean,
                        "target_talent": (0..8).map(|i| target.talent.get(i).copied().unwrap_or(0.0)).collect::<Vec<f32>>(),
                        "gap_h": (s.value_h - v.h) / v.slope_h, "gap_p": (s.value_p - v.p) / v.slope_p,
                        "hist_from": HIST_FROM, "hist_b": s.hist_b, "hist_p": s.hist_p,
                    }),
                )
            })
            .collect();
        let mut attrs = AttrMoments::default();
        for m in &moments {
            attrs.add(m, 1.0 / moments.len() as f64);
        }
        let out = serde_json::json!({ "levels": levels_json, "spread": spreads / counted, "outcomes": summary, "attrs": attrs.json(), "money": money });
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}
