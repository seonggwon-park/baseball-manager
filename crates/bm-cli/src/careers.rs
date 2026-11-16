//! `bm careers`: make the world and run offseasons back to back without playing the
//! seasons in between (development does not depend on games yet), then compare each
//! level's talent and ages with the level table, and the prospects' outcomes with
//! published reach rates.

use bm_engine::Rng;
use bm_world::ability;
use bm_world::career::{offseason, talent_of, Offseason, Origin};
use bm_world::{populate_with, AgingTable, Attr, CareerTable, Date, LevelTable, PeopleMaker, Population, Squad, Talent, World, N_ATTR};
use std::collections::BTreeMap;
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
}

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
    }

    fn scaled(&self, k: f64) -> LevelStats {
        LevelStats {
            n: self.n * k,
            age_b: self.age_b * k,
            age_p: self.age_p * k,
            talent: self.talent.map(|x| x * k),
            value_h: self.value_h * k,
            value_p: self.value_p * k,
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
        out.insert(
            key,
            LevelStats {
                n: players.len() as f64,
                age_b: mean_age(false),
                age_p: mean_age(true),
                talent: talent_of(players.iter().copied()),
                value_h: value(false),
                value_p: value(true),
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
}

impl AttrMoments {
    fn of(pop: &Population) -> AttrMoments {
        let mut m = BTreeMap::new();
        for key in LEVELS {
            let players: Vec<_> =
                pop.rosters.iter().filter(|r| r.level == key).flat_map(|r| r.players()).filter(|p| !pop.foreign.contains_key(&p.id)).collect();
            if players.is_empty() {
                continue;
            }
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
        AttrMoments { m }
    }

    fn add(&mut self, o: &AttrMoments, k: f64) {
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
        self.m.iter().map(|(k, v)| (k.to_string(), serde_json::json!({"h": side(&v[0]), "p": side(&v[1])}))).collect::<serde_json::Map<_, _>>().into()
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

/// Run `years` offseasons, print the levels every `every` years and their average from
/// offseason `from` on.
#[allow(clippy::too_many_arguments)]
pub fn run(
    years: usize,
    seed: u64,
    every: usize,
    from: Option<usize>,
    levels_file: Option<&Path>,
    careers_file: Option<&Path>,
    aging_file: Option<&Path>,
    json: Option<&Path>,
) {
    let read = |p: &Path| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let world = World::bundled();
    let levels = levels_file.map_or_else(LevelTable::bundled, |p| LevelTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let aging = aging_file.map_or_else(AgingTable::bundled, |p| AgingTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let careers = careers_file.map_or_else(CareerTable::bundled, |p| CareerTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let people = PeopleMaker::new();
    let ctx = Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people };
    let t0 = Instant::now();
    let targets = value_targets(&levels);
    let mut pop = populate_with(&world, &levels, &aging, &careers, Date::new(2027, 2, 1), seed);
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
    for y in 0..years {
        let year = 2027 + y as i32;
        let r = offseason(&ctx, &mut pop, year, None, seed);
        last = year;
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
                "\nAfter the {year} offseason ({} players, {} people so far): retired {}, released {}, drafted {}, signed {}, foreign {} in / {} out; first-team spread {:.2}",
                pop.players().count(),
                pop.people.len(),
                r.retired,
                r.released,
                r.drafted,
                r.signed,
                r.foreign_signed,
                r.foreign_replaced,
                spread(&pop, &world)
            );
            print_table(&stats(&pop), &levels, &targets);
        }
    }
    let mean: BTreeMap<&'static str, LevelStats> = sum.iter().map(|(k, s)| (*k, s.scaled(1.0 / counted))).collect();
    println!("\nAverage after offseasons {} to {years} (first-team spread {:.2}):", from + 1, spreads / counted);
    print_table(&mean, &levels, &targets);
    let summary = outcomes(&pop, last, 7);
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
                    }),
                )
            })
            .collect();
        let mut attrs = AttrMoments::default();
        for m in &moments {
            attrs.add(m, 1.0 / moments.len() as f64);
        }
        let out = serde_json::json!({ "levels": levels_json, "spread": spreads / counted, "outcomes": summary, "attrs": attrs.json() });
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}
