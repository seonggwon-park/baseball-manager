//! `bm` — command line tools for the Baseball Manager engine.
//!
//! ```text
//! bm validate [--games N] [--seed S] [--threads T] [--clubs K]   compare simulated aggregates with MLB 2025
//!             [--style none|mlb] [--set name=value]               AI manager preset, model constants
//! bm game [--seed S]                                              play one game and print the box score
//! bm bench [--games N]                                            measure simulation speed
//! bm serve [--port P] [--static DIR]                              session API over HTTP for the interface in a browser
//! ```

mod report;
mod serve;
mod tally;
mod tuning;

use bm_engine::{play_game_styled, Club, GameResult, Level, ManagerStyle, MatchEnv, NoObserver, Rng, Rules};
use bm_physics::{Atmosphere, Park, Wind};
use std::time::Instant;
use tally::Tally;
use tuning::Tuning;

struct Args {
    cmd: String,
    port: u16,
    static_dir: Option<std::path::PathBuf>,
    games: usize,
    seed: u64,
    threads: usize,
    clubs: usize,
    tuning: Tuning,
}

fn parse_args() -> Args {
    let mut a = Args {
        cmd: "validate".into(),
        port: 8787,
        static_dir: None,
        games: 2000,
        seed: 2025,
        threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
        clubs: 300,
        tuning: Tuning::new(),
    };
    let mut it = std::env::args().skip(1);
    if let Some(c) = it.next() {
        a.cmd = c;
    }
    while let Some(k) = it.next() {
        let v = it.next().unwrap_or_default();
        match k.as_str() {
            "--games" => a.games = v.parse().expect("--games N"),
            "--seed" => a.seed = v.parse().expect("--seed S"),
            "--threads" => a.threads = v.parse::<usize>().expect("--threads T").max(1),
            "--clubs" => a.clubs = v.parse::<usize>().expect("--clubs K").max(2),
            "--port" => a.port = v.parse().expect("--port P"),
            "--static" => a.static_dir = Some(v.into()),
            "--style" => {
                a.tuning.style = match v.as_str() {
                    "none" => ManagerStyle::none(),
                    "mlb" => ManagerStyle::mlb(),
                    other => {
                        eprintln!("unknown style {other}; use none | mlb");
                        std::process::exit(2);
                    }
                }
            }
            "--set" => {
                let (name, value) = v.split_once('=').expect("--set name=value");
                let value: f64 = value.parse().expect("--set name=value needs a number");
                if let Err(e) = a.tuning.set(name, value) {
                    eprintln!("{e}");
                    std::process::exit(2);
                }
            }
            other => {
                eprintln!("unknown option {other}");
                std::process::exit(2);
            }
        }
    }
    a
}

fn make_clubs(n: usize, seed: u64, level: &Level) -> Vec<Club> {
    let mut rng = Rng::new(seed ^ 0xC1B5);
    (0..n).map(|i| Club::random(&mut rng, 1000 * (i as u32 + 1), level)).collect()
}

fn pick_pair(rng: &mut Rng, n: usize) -> (usize, usize) {
    let a = rng.below(n);
    let mut b = rng.below(n - 1);
    if b >= a {
        b += 1;
    }
    (a, b)
}

/// Run `games` games on `threads` threads, feeding every event to a [`Tally`].
fn simulate(env: &MatchEnv, clubs: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize, observe: bool) -> Tally {
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let mut total = Tally::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .enumerate()
            .map(|(i, mut rng)| {
                let n = per.min(games.saturating_sub(i * per));
                scope.spawn(move || {
                    let mut t = Tally { flight: Some(env.batted_flight), ..Default::default() };
                    let dh = env.rules.dh;
                    for _ in 0..n {
                        let (a, h) = pick_pair(&mut rng, clubs.len());
                        let away = clubs[a].team_for_game(rng.below(5), dh);
                        let home = clubs[h].team_for_game(rng.below(5), dh);
                        let styles = [style; 2];
                        let res = if observe {
                            play_game_styled(env, &away, &home, styles, &mut rng, &mut t)
                        } else {
                            play_game_styled(env, &away, &home, styles, &mut rng, &mut NoObserver)
                        };
                        t.games += 1.0;
                        t.team_games += 2.0;
                        t.innings += res.innings as f64;
                        t.runs += (res.score[0] + res.score[1]) as f64;
                        if res.score[0] == res.score[1] {
                            t.ties += 1.0;
                        }
                    }
                    t
                })
            })
            .collect();
        for h in handles {
            total.merge(&h.join().expect("simulation thread panicked"));
        }
    });
    total
}

fn print_box(res: &GameResult) {
    let names = ["Away", "Home"];
    print!("      ");
    for i in 1..=res.line_score.len() {
        print!("{i:>3}");
    }
    println!("    R   H   E");
    for (t, name) in names.iter().enumerate() {
        print!("{name:<6}");
        for (i, inn) in res.line_score.iter().enumerate() {
            // A home side that is ahead does not bat in the last inning.
            let not_played = t == 1 && i + 1 == res.line_score.len() && res.score[1] > res.score[0] && inn[1] == 0;
            if not_played {
                print!("  x");
            } else {
                print!("{:>3}", inn[t]);
            }
        }
        println!("  {:>3} {:>3} {:>3}", res.score[t], res.boxes[t].hits, res.boxes[1 - t].errors);
    }
    for (t, name) in names.iter().enumerate() {
        println!("\n{name} batting      PA AB  R  H 2B 3B HR RBI BB SO");
        for (i, b) in res.boxes[t].batting.iter().enumerate() {
            if b.pa > 0 {
                println!(
                    "  #{i:<2}            {:>3}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}{:>4}{:>3}{:>3}",
                    b.pa, b.ab, b.r, b.h, b.doubles, b.triples, b.hr, b.rbi, b.bb, b.so
                );
            }
        }
        println!("{name} pitching      IP  H  R ER BB SO HR  NP");
        for (i, p) in res.boxes[t].pitching.iter().enumerate() {
            if p.appeared {
                println!(
                    "  #{i:<2}           {:>2}.{} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>3}",
                    p.outs / 3,
                    p.outs % 3,
                    p.h,
                    p.r,
                    p.er,
                    p.bb,
                    p.so,
                    p.hr,
                    p.pitches
                );
            }
        }
    }
    let show = |label: &str, x: Option<(usize, usize)>| {
        if let Some((t, p)) = x {
            println!("{label}: {} #{p}", names[t]);
        }
    };
    println!();
    show("W", res.win);
    show("L", res.loss);
    show("S", res.save);
}

fn main() {
    let args = parse_args();
    // Neutral MLB-like conditions (see `MatchEnv::neutral_mlb`) with any command-line overrides.
    let mut env = MatchEnv::new(&Atmosphere::new(24.0, 760.0, 55.0, 180.0), Wind::CALM, Park::mlb_average(), Rules::mlb_2025(), args.tuning.params.clone());
    env.ball = args.tuning.ball;
    let level: &Level = &args.tuning.level;
    match args.cmd.as_str() {
        "validate" => {
            let clubs = make_clubs(args.clubs, args.seed, level);
            let start = Instant::now();
            let t = simulate(&env, &clubs, args.tuning.style, args.games, args.seed, args.threads, true);
            let secs = start.elapsed().as_secs_f64();
            println!(
                "Simulated {:.0} games ({:.0} pitches) in {:.2} s on {} threads: {:.0} games/s, {:.1} ms per game per thread",
                t.games,
                t.pitches,
                secs,
                args.threads,
                t.games / secs,
                1000.0 * secs * args.threads as f64 / t.games
            );
            report::print(&t);
        }
        "bench" => {
            let clubs = make_clubs(args.clubs, args.seed, level);
            for threads in [1, args.threads] {
                let start = Instant::now();
                let t = simulate(&env, &clubs, args.tuning.style, args.games, args.seed, threads, false);
                let secs = start.elapsed().as_secs_f64();
                println!(
                    "{threads} thread(s): {:.0} games in {:.2} s = {:.0} games/s ({:.2} ms per game per thread)",
                    t.games,
                    secs,
                    t.games / secs,
                    1000.0 * secs * threads as f64 / t.games
                );
            }
            println!("A season of 13,800 games (MLB + minors + KBO + NPB with farm leagues) at the multi-thread rate above takes the corresponding fraction of that time.");
        }
        "population" => {
            // Averages of the physical profiles in the random population (a sanity check
            // that two seeds describe the same league).
            let clubs = make_clubs(args.clubs, args.seed, level);
            let mut m = [tally::Mean::default(); 10];
            for c in &clubs {
                for h in &c.hitters {
                    m[0].add(h.batter.bat_speed / bm_physics::units::MPH);
                    m[1].add(h.batter.barrel_sd * 1000.0);
                    m[2].add(h.batter.timing_sd * 1000.0);
                    m[3].add(h.batter.eye);
                    m[4].add(h.batter.attack_angle.to_degrees());
                    m[5].add(h.batter.sprint / bm_physics::units::FT);
                }
                for p in c.starters.iter().chain(&c.relievers) {
                    let prof = p.pitcher.as_ref().unwrap();
                    let fb = prof.pitches.iter().map(|d| d.shape.speed).fold(0.0, f64::max);
                    m[6].add(fb / bm_physics::units::MPH);
                    m[7].add(prof.pitches[0].command / bm_physics::units::INCH);
                    m[8].add(prof.pitches.len() as f64);
                    m[9].add(prof.stamina);
                }
            }
            let names = [
                "bat speed (mph)",
                "barrel SD (mm)",
                "timing SD (ms)",
                "eye",
                "attack angle (deg)",
                "sprint (ft/s)",
                "fastest pitch (mph)",
                "command (in)",
                "pitches in arsenal",
                "stamina (pitches)",
            ];
            for (n, x) in names.iter().zip(&m) {
                println!("  {:<22} mean {:8.3}  sd {:7.3}  n {:.0}", n, x.mean(), x.sd(), x.n);
            }
        }
        "game" => {
            let clubs = make_clubs(2, args.seed, level);
            let mut rng = Rng::new(args.seed);
            let away = clubs[0].team_for_game(0, env.rules.dh);
            let home = clubs[1].team_for_game(0, env.rules.dh);
            let res = play_game_styled(&env, &away, &home, [args.tuning.style; 2], &mut rng, &mut NoObserver);
            print_box(&res);
        }
        "serve" => {
            if let Err(e) = serve::run(args.port, args.static_dir) {
                eprintln!("server error: {e}");
                std::process::exit(1);
            }
        }
        other => {
            eprintln!("unknown command {other}; use validate | game | bench | population | serve");
            std::process::exit(2);
        }
    }
}
