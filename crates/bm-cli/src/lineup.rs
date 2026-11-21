//! `bm lineup`: what a batting order is worth. One club of a level, made as `bm park` makes
//! them (levels.json talent), plays the same games against the level's other clubs with
//! different batting orders: the same opponents, parks, starting pitchers and random numbers
//! for every order, so the difference between two orders is measured game by game (common
//! random numbers) and only the order differs.
//!
//! Orders are written as places in the engine's own order (1 = the hitter it bats first: the
//! best by bat speed, barrel control and eye, `Club::team_for_game`). The AI manager still
//! makes its substitutions, so a pinch hitter takes the slot of the hitter he replaces.
//!
//! The common random numbers help little: a different order sends a different batter to the
//! plate at once and the games' random streams part from there. Two orders' paired difference
//! has a standard error of about 0.085 runs a game over 2,000 games (MLB level), so a gap of
//! 10 runs over 162 games (0.06 a game) needs some 30,000 games per order to show.

use crate::level;
use crate::tuning::Tuning;
use bm_engine::{play_game_styled, NoObserver, Rng};
use bm_world::Talent;

/// What the command measures for one order.
struct Tally {
    runs: Vec<f64>,
    allowed: f64,
    wins: f64,
    decided: f64,
}

/// A random order of `n` places, the others fixed (Fisher-Yates).
fn shuffled(rng: &mut Rng, n: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (1..=9).collect();
    for i in (1..n).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
    v
}

fn name(order: &[usize]) -> String {
    order.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("-")
}

/// Mean and standard error.
fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len().max(1) as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, (var / n).sqrt())
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    level_name: &str,
    talent: Talent,
    foreign: Talent,
    drag: f64,
    club: usize,
    custom: &[Vec<usize>],
    permutations: usize,
    games: usize,
    seed: u64,
    threads: usize,
    tuning: &Tuning,
    json: Option<&std::path::Path>,
) {
    let env = level::build(level_name, &talent, &foreign, tuning, seed, drag);
    assert!(club < env.clubs.len(), "--club {club}: the level has {} clubs (0-{})", env.clubs.len(), env.clubs.len() - 1);
    let others: Vec<usize> = (0..env.clubs.len()).filter(|&c| c != club && env.league[c] == env.league[club]).collect();
    let dh = env.envs[club].rules.dh;
    // In a league without the designated hitter the pitcher keeps the ninth place.
    let movable = if dh { 9 } else { 8 };
    let mut orders: Vec<Vec<usize>> = vec![(1..=9).collect(), {
        let mut r: Vec<usize> = (1..=movable).rev().collect();
        r.extend(movable + 1..=9);
        r
    }];
    for o in custom {
        let mut sorted = o.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (1..=9).collect::<Vec<_>>(), "--order needs the places 1-9 once each: {}", name(o));
        assert!(dh || o[8] == 9, "--order: without the designated hitter the pitcher (9) bats ninth");
        orders.push(o.clone());
    }
    let mut prng = Rng::new(seed ^ 0x0D_E5);
    for _ in 0..permutations {
        orders.push(shuffled(&mut prng, movable));
    }
    let threads = threads.max(1);
    let per = games.div_ceil(threads);
    let style = tuning.style;
    let results: Vec<Vec<(u32, u32)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (env, others, orders) = (&env, &others, &orders);
                scope.spawn(move || {
                    let mut out = Vec::new();
                    for g in t * per..((t + 1) * per).min(games) {
                        let opp = others[g % others.len()];
                        let home = g % 2 == 0;
                        let start = (g / 2) % 5;
                        let them = env.clubs[opp].team_for_game(start, dh);
                        let menv = &env.envs[if home { club } else { opp }];
                        let base = env.clubs[club].team_for_game(start, dh);
                        let row: Vec<(u32, u32)> = orders
                            .iter()
                            .map(|o| {
                                let mut us = base.clone();
                                for (slot, &place) in o.iter().enumerate() {
                                    us.lineup[slot] = base.lineup[place - 1];
                                }
                                let mut rng = Rng::new(seed ^ 0x11_9E_00 ^ (g as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
                                let (away, homet) = if home { (&them, &us) } else { (&us, &them) };
                                let res = play_game_styled(menv, away, homet, [style; 2], &mut rng, &mut NoObserver);
                                let (s_us, s_them) = if home { (res.score[1], res.score[0]) } else { (res.score[0], res.score[1]) };
                                (s_us, s_them)
                            })
                            .collect();
                        out.push(row);
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("lineup thread panicked")).collect()
    });
    let mut tallies: Vec<Tally> = orders.iter().map(|_| Tally { runs: Vec::with_capacity(games), allowed: 0.0, wins: 0.0, decided: 0.0 }).collect();
    for row in &results {
        for (k, &(us, them)) in row.iter().enumerate() {
            let t = &mut tallies[k];
            t.runs.push(us as f64);
            t.allowed += them as f64;
            if us != them {
                t.decided += 1.0;
                t.wins += f64::from(us > them);
            }
        }
    }
    let n = results.len() as f64;
    let base_runs = tallies[0].runs.clone();
    println!(
        "lineup: level {level_name}, club {club} of {}, {} games against {} clubs ({}), {} orders; places are the engine's order (1 bats first)",
        env.clubs.len(),
        results.len(),
        others.len(),
        if dh { "designated hitter" } else { "pitchers bat ninth" },
        orders.len()
    );
    println!("  order               runs/game  vs engine's order (paired)   per 162 games   allowed/game  win %");
    let mut rows = Vec::new();
    for (k, t) in tallies.iter().enumerate() {
        let (m, _) = mean_se(&t.runs);
        let diff: Vec<f64> = t.runs.iter().zip(&base_runs).map(|(a, b)| a - b).collect();
        let (d, se) = mean_se(&diff);
        let label = match k {
            0 => "engine".to_string(),
            1 => "reverse".to_string(),
            _ if k < 2 + custom.len() => "custom".to_string(),
            _ => "random".to_string(),
        };
        rows.push((k, label, m, d, se, t.allowed / n, 100.0 * t.wins / t.decided.max(1.0)));
    }
    let print = |r: &(usize, String, f64, f64, f64, f64, f64)| {
        println!(
            "  {:<19} {:>8.3}   {:+.3} +- {:.3}               {:+6.1}       {:>8.3}    {:5.1}   ({})",
            name(&orders[r.0]),
            r.2,
            r.3,
            r.4,
            162.0 * r.3,
            r.5,
            r.6,
            r.1
        );
    };
    for r in rows.iter().take(2 + custom.len()) {
        print(r);
    }
    if permutations > 0 {
        let mut random: Vec<_> = rows.iter().skip(2 + custom.len()).collect();
        random.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        let shown = random.len().min(5);
        println!("  {permutations} random orders, the best and worst {shown}:");
        for r in random.iter().take(shown) {
            print(r);
        }
        if random.len() > shown {
            println!("  ...");
            for r in random.iter().skip(random.len().saturating_sub(shown).max(shown)) {
                print(r);
            }
        }
        // Each game's mean over the random orders less the engine's order, so that the
        // standard error counts the games they share.
        let first = 2 + custom.len();
        let per_game: Vec<f64> =
            (0..base_runs.len()).map(|g| tallies[first..].iter().map(|t| t.runs[g]).sum::<f64>() / permutations as f64 - base_runs[g]).collect();
        let (m, se) = mean_se(&per_game);
        println!("  random orders: mean {:+.3} +- {:.3} runs/game against the engine's order ({:+.1} per 162 games)", m, se, 162.0 * m);
    }
    if let Some(path) = json {
        let doc = serde_json::json!({
            "level": level_name, "club": club, "games": results.len(), "seed": seed,
            "orders": rows.iter().map(|r| serde_json::json!({
                "order": orders[r.0], "kind": r.1, "runs_per_game": r.2, "vs_engine": r.3, "vs_engine_se": r.4,
                "allowed_per_game": r.5, "win_pct": r.6,
            })).collect::<Vec<_>>(),
        });
        std::fs::write(path, serde_json::to_string_pretty(&doc).expect("json")).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_orders_are_permutations_that_keep_the_fixed_places() {
        let mut rng = Rng::new(3);
        for movable in [8, 9] {
            for _ in 0..50 {
                let o = shuffled(&mut rng, movable);
                let mut s = o.clone();
                s.sort_unstable();
                assert_eq!(s, (1..=9).collect::<Vec<_>>());
                assert!(movable == 9 || o[8] == 9);
            }
        }
    }
}
