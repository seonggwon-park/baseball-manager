//! `bm season`: generate the world, play a whole season of every league pitch by pitch,
//! and print each league's rates next to its official 2025-2026 averages, the
//! major-league standings and the time it took. With `--seasons N`, play N seasons with
//! an offseason (development, retirements, drafts, rosters) between them.

use crate::tuning::Tuning;
use bm_world::season::{Batting, Play};
use bm_world::{populate, Date, LevelTable, Season, World};
use std::collections::BTreeMap;
use std::time::Instant;

/// Official 2025-2026 averages of a league (docs/research/world/levels.md 2.1): R/G, AVG,
/// OBP, SLG, K %, BB %, HR %.
// An OBP of .318 is not 1/pi.
#[allow(clippy::approx_constant)]
fn official(league: &str) -> Option<[f64; 7]> {
    let mean = |a: [f64; 7], b: [f64; 7]| std::array::from_fn(|i| (a[i] + b[i]) / 2.0);
    Some(match league {
        "us-major" => mean([4.45, 0.245, 0.315, 0.404, 22.2, 8.4, 3.09], [4.48, 0.244, 0.318, 0.400, 22.1, 8.9, 3.03]),
        l if l.starts_with("us-aaa") => mean([5.24, 0.258, 0.347, 0.421, 22.6, 10.9, 2.87], [5.31, 0.258, 0.349, 0.422, 22.3, 11.2, 2.92]),
        l if l.starts_with("us-aa-") => mean([4.29, 0.235, 0.323, 0.360, 23.5, 10.3, 2.01], [5.24, 0.249, 0.340, 0.415, 24.4, 10.9, 3.08]),
        l if l.starts_with("us-higha") => mean([4.51, 0.233, 0.330, 0.358, 23.7, 11.0, 1.90], [5.54, 0.249, 0.351, 0.413, 24.2, 12.0, 2.91]),
        l if l.starts_with("us-a-") => mean([4.82, 0.237, 0.340, 0.345, 23.5, 11.9, 1.42], [5.57, 0.248, 0.354, 0.395, 24.4, 12.5, 2.32]),
        "kr-top" => mean([4.73, 0.262, 0.338, 0.389, 19.7, 9.1, 2.13], [5.07, 0.268, 0.346, 0.404, 19.5, 9.4, 2.41]),
        "kr-second" => mean([6.22, 0.286, 0.381, 0.427, 17.8, 12.0, 2.15], [5.44, 0.270, 0.362, 0.383, 16.9, 11.1, 1.51]),
        "jp-top" => mean([3.29, 0.244, 0.305, 0.351, 19.4, 7.1, 1.71], [3.59, 0.240, 0.305, 0.363, 21.0, 7.6, 2.23]),
        "jp-second" => mean([3.80, 0.250, 0.321, 0.343, 18.2, 8.5, 1.07], [4.11, 0.253, 0.327, 0.364, 19.1, 8.8, 1.55]),
        _ => return None,
    })
}

/// League totals from the players' lines: every player belongs to one club all season.
fn totals<'a>(world: &'a World, pop: &bm_world::Population, season: &Season) -> (BTreeMap<&'a str, Batting>, BTreeMap<&'a str, u32>) {
    let league_of: BTreeMap<&str, &str> = world.clubs().map(|c| (c.id.as_str(), c.league.as_str())).collect();
    let mut player_league: BTreeMap<u32, &str> = BTreeMap::new();
    for r in &pop.rosters {
        if let Some(l) = league_of.get(r.club.as_str()) {
            for p in r.players() {
                player_league.insert(p.id, l);
            }
        }
    }
    let mut totals: BTreeMap<&str, Batting> = BTreeMap::new();
    for (id, b) in &season.batting {
        if let Some(l) = player_league.get(id) {
            let t = totals.entry(l).or_default();
            t.pa += b.pa;
            t.ab += b.ab;
            t.h += b.h;
            t.doubles += b.doubles;
            t.triples += b.triples;
            t.hr += b.hr;
            t.bb += b.bb;
            t.hbp += b.hbp;
            t.so += b.so;
            t.sf += b.sf;
            t.r += b.r;
        }
    }
    let mut team_games: BTreeMap<&str, u32> = BTreeMap::new();
    for (club, s) in &season.standings {
        if let Some(l) = league_of.get(club.as_str()) {
            *team_games.entry(l).or_default() += s.won + s.lost + s.tied;
        }
    }
    (totals, team_games)
}

/// R/G, AVG, OBP, SLG, K %, BB %, HR % of a league's totals.
fn rates(t: &Batting, games: u32) -> [f64; 7] {
    let pa = t.pa.max(1) as f64;
    [t.r as f64 / games.max(1) as f64, t.avg(), t.obp(), t.slg(), 100.0 * t.so as f64 / pa, 100.0 * t.bb as f64 / pa, 100.0 * t.hr as f64 / pa]
}

fn league_table(world: &World, pop: &bm_world::Population, season: &Season) {
    let (totals, team_games) = totals(world, pop, season);
    println!("\nleague                 R/G     AVG    OBP    SLG     K%    BB%    HR%   (official 2025-26 mean below)");
    for league in world.leagues() {
        let (Some(t), Some(g)) = (totals.get(league.id.as_str()), team_games.get(league.id.as_str())) else { continue };
        let r = rates(t, *g);
        println!("  {:<20} {:5.2}   {:.3}  {:.3}  {:.3}  {:5.1}  {:5.1}  {:5.2}", league.id, r[0], r[1], r[2], r[3], r[4], r[5], r[6]);
        if let Some(o) = official(&league.id) {
            println!("  {:<20} {:5.2}   {:.3}  {:.3}  {:.3}  {:5.1}  {:5.1}  {:5.2}", "", o[0], o[1], o[2], o[3], o[4], o[5], o[6]);
        }
    }
}

/// Play `seasons` seasons of every league, with an offseason between them.
pub fn run(tuning: &Tuning, seed: u64, threads: usize, until: Option<Date>, seasons: usize) {
    let world = World::bundled();
    let levels = LevelTable::bundled();
    let aging = bm_world::AgingTable::bundled();
    let careers = bm_world::CareerTable::bundled();
    let people = bm_world::PeopleMaker::new();
    let ctx = bm_world::Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people };
    let t0 = Instant::now();
    let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), seed);
    let t_pop = t0.elapsed().as_secs_f64();
    let play = Play { params: tuning.params.clone(), baseline: tuning.baseline.clone(), style: tuning.style, threads, ..Play::default() }.with_balls(&levels);
    let seasons = seasons.max(1);
    for k in 0..seasons {
        let year = 2027 + k as i32;
        let mut season = Season::new(&world, year, seed.wrapping_add(k as u64));
        let t1 = Instant::now();
        match until.filter(|_| seasons == 1) {
            Some(d) => season.play_until(&world, &pop, &play, d),
            None => season.play_out(&world, &pop, &play),
        }
        let secs = t1.elapsed().as_secs_f64();
        println!(
            "\n{year}: {} players{}; {} of {} games played in {:.1} s on {threads} threads ({:.0} games/s)",
            pop.players().count(),
            if k == 0 { format!(" generated in {t_pop:.2} s") } else { String::new() },
            season.played,
            season.fixtures.len(),
            secs,
            season.played as f64 / secs
        );
        if seasons == 1 || k == 0 || k + 1 == seasons {
            league_table(&world, &pop, &season);
        } else {
            let (t, g) = totals(&world, &pop, &season);
            let line: Vec<String> = ["us-major", "kr-top", "jp-top"]
                .iter()
                .filter_map(|l| t.get(l).zip(g.get(l)).map(|(t, g)| (l, rates(t, *g))))
                .map(|(l, r)| format!("{l} {:.2} R/G, {:.3}/{:.3}/{:.3}, K {:.1} BB {:.1} HR {:.2}", r[0], r[1], r[2], r[3], r[4], r[5], r[6]))
                .collect();
            println!("  {}", line.join(" | "));
        }
        if k + 1 == seasons {
            standings(&world, &season);
        } else {
            let r = bm_world::offseason(&ctx, &mut pop, year, Some(&season.standings), seed);
            println!(
                "  offseason: retired {}, released {}, drafted {}, signed {}, foreign {} in / {} out",
                r.retired, r.released, r.drafted, r.signed, r.foreign_signed, r.foreign_replaced
            );
        }
    }
}

fn standings(world: &World, season: &Season) {
    // Major-league standings by division.
    if let Some(major) = world.league("us-major") {
        println!("\nmajor-league standings");
        for div in &major.divisions {
            let mut rows: Vec<(&str, &bm_world::Standing)> = world
                .clubs()
                .filter(|c| c.division.as_deref() == Some(div.id.as_str()))
                .filter_map(|c| season.standings.get(&c.id).map(|s| (c.name.en.as_str(), s)))
                .collect();
            rows.sort_by(|a, b| b.1.pct().partial_cmp(&a.1.pct()).unwrap_or(std::cmp::Ordering::Equal));
            println!("  {}", div.id);
            for (name, s) in rows {
                println!("    {:<28} {:3}-{:<3} {:.3}  RS {:4} RA {:4}", name, s.won, s.lost, s.pct(), s.runs_for, s.runs_against);
            }
        }
    }
}
