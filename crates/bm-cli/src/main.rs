//! `bm` — command line tools for the Baseball Manager engine.
//!
//! ```text
//! bm validate [--games N] [--seed S] [--threads T] [--clubs K]   compare simulated aggregates with MLB 2025;
//!             [--json FILE]                                       --json: pitches by type, count and result
//!             [--style none|mlb] [--set name=value]               AI manager preset, model constants
//!             [--zone human|kbo-abs|mlb-abs]                      who calls balls and strikes, for the same players (also for
//!                                                                 tactics, players, attributes, level and park): MLB's umpires,
//!                                                                 the KBO's ABS, or MLB's 2026 ABS zone on every pitch
//!             [--population world|engine]                         players drawn from attributes (default) or as physical profiles
//!             [--parks world|neutral]                             the 30 major-league parks of the world data (default) or one average park
//! bm tactics [--games N] [--seed S] [--json FILE]                 the games of `bm validate`: run expectancy by base-out state,
//!            [--shift-rule on|off]                                what bunts, intentional walks and steals returned, BABIP by
//!                                                                 batter hand; --shift-rule off: MLB's rules before 2023 (also
//!                                                                 for validate), when the AI managers use the full shift
//! bm attributes [--games N] [--only NAME]                         value of one standard deviation of each attribute
//!               [--by-position]                                   fielding value of the regular at each position (2 SD steps)
//! bm players [--clubs K] [--games G] [--json FILE]                season lines of individual players as CSV (G games per club);
//!                                                                 --json: the players themselves, one JSON object per line
//! bm level --level L [--talent h,p,f,x[,pw,cm[,v[,d]]]] [--vs-mlb] one level (aaa, aa, high_a, single_a, kbo, kbo2, npb, npb2,
//!          [--foreign h,p,f,x] [--cross-games N]                   mlb) in its own parks against its official rates; talent of
//!          [--vs-mlb-pitchers] [--ball-drag F]                     the foreign players of KBO and NPB clubs; wRC+ of the level's
//!                                                                  hitters (and pitchers) against major-leaguers; `pool` = only that
//! bm park --level L --fence l,lc,c,rc,r [--wall l,c,r | --wall h] one ballpark experiment: the level's clubs (levels.json talent
//!         [--elevation M] [--turf] [--talent ...] [--foreign ...]  unless given) play every game in one park (fence distances and
//!         [--games N] [--seed S] [--json FILE]                    wall heights in m); the same seed gives the same clubs
//!         [--balls FILE [--ev-shift MPH]]                         or measured batted balls flown through the park instead of games;
//!         [--temp C] [--humidity PCT]                             the air (default 24 C, 55 %; --temp also replaces the balls' own)
//! bm season [--seed S] [--until YYYY-MM-DD] [--seasons N]         generate the world and play a season of every league
//!           [--warmup K] [--levels FILE] [--careers FILE]         (N seasons with offseasons between them, after K offseasons
//!           [--aging FILE] [--contracts FILE] [--json FILE]       without games); --json: each level's rates in the last season;
//!           [--history H] [--game-logs DIR]                       --history: offseasons of the new world's past (careers.json);
//!                                                                 --game-logs: every game's batting and pitching lines as CSV
//!           [--repeat N [--leagues L,..] [--standings-out FILE]] the first season N times with the same rosters and schedule
//!                                                                 (only the games' luck changes; default leagues us-major,
//!                                                                 kr-top, jp-top): each club's wins, firsts and postseasons
//! bm careers [--years N] [--every K] [--from N]                   offseasons back to back: each level's talent and ages (every
//!            [--levels FILE] [--careers FILE] [--aging FILE]      K years and averaged from offseason N), and how the drafted
//!            [--contracts FILE] [--json FILE] [--out FILE]        and signed prospects turn out, from a new world with H
//!            [--history H]                                        offseasons of past (careers.json; 0: the first world itself); --json adds each level's
//!                                                                 attribute means and spreads, pitch grades, the stamina and
//!                                                                 quality of rotations and bullpens, how many regulars play
//!                                                                 their own position, and attribute correlations (--years 0:
//!                                                                 the new world); --out: every player's outcome of each
//!                                                                 offseason from N on (age, level, value, stayed or how he left);
//!                                                                 --attrs-out: every major leaguer's attributes before each one
//! bm contacts --out FILE [--games N] [--invert FILE]              balls in play with the swing that hit them (simulated), or the
//!             [--face-lead DEG] [--dir-shift DEG] [--swings]      impact parameter that reproduces each row's launch angle;
//!                                                                 --swings: every full swing instead of balls in play
//! bm we [--games N] [--seed S] [--json FILE]                     win expectancy of games between the AI managers by inning, half,
//!                                                                 outs, bases and score: the table behind bm-gym's reward
//!                                                                 (default FILE data/engine/win_expectancy.json)
//! bm re [--games N] [--seed S] [--json FILE]                     run expectancy by base state, outs and count before each pitch
//!                                                                 (innings 1-8 and the top of the 9th): the pitching
//!                                                                 environment's reward (default FILE data/engine/run_expectancy.json)
//! bm rollout [--states N] [--rollouts R] [--seed S] [--json FILE] the tactics of bm-gym's classes (bunt, steal, hit and run, ...)
//!                                                                 tried from copies of N plate appearances each, R rollouts per
//!                                                                 tactic, the AI managers finishing the game: change of the
//!                                                                 chance of winning and of the runs to the end of the inning
//! bm lineup [--level L] [--club K] [--games N] [--seed S]          one club of a level (levels.json talent) with other batting
//!           [--order p1,..,p9]... [--permutations K] [--json FILE] orders against the same opponents and random numbers: the
//!                                                                 engine's order, its reverse, given orders (places in the engine's
//!                                                                 order) and K random ones; runs per game, paired differences
//! bm game [--seed S]                                              play one game and print the box score
//! bm bench [--games N]                                            measure simulation speed
//! bm serve [--port P] [--static DIR]                              session API over HTTP for the interface in a browser
//! ```

mod careers;
mod contacts;
mod level;
mod lineup;
mod players;
mod report;
mod season;
mod sensitivity;
mod serve;
mod tactics;
mod tally;
mod tuning;

use bm_engine::{play_game_styled, Club, GameResult, Level, ManagerStyle, MatchEnv, NoObserver, PitchKind, Rng, Rules};
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
    /// Draw the players from the attribute system instead of the engine's own population.
    world: bool,
    /// Play `validate` and `bench` in the major-league ballparks of the world data instead of one average park.
    parks: bool,
    only: Option<String>,
    /// `bm level`: the level, its talent offsets and whether to play it against major-leaguers.
    level: String,
    /// Whether `--level` was given (`bm lineup` defaults to the majors).
    level_given: bool,
    talent: bm_world::Talent,
    /// Talent of the foreign players of a KBO or NPB club.
    foreign: bm_world::Talent,
    vs_mlb: bool,
    /// Games per side of the comparison with major-leaguers (default: `--games`, at least 400),
    /// and whether to play the level's pitchers against them too.
    cross_games: Option<usize>,
    /// `bm rollout`: plate appearances per class and rollouts per tactic.
    states: usize,
    rollouts: usize,
    vs_mlb_pitchers: bool,
    /// `bm attributes`: fielding value position by position instead of the attribute table.
    by_position: bool,
    /// `bm level`: factor on the batted ball's drag in the level's own games.
    ball_drag: f64,
    /// `bm park`: the ballpark (fence distances and wall heights, m), its elevation and
    /// surface, and whether the talent was given on the command line.
    fence: Option<[f64; 5]>,
    wall: Option<[f64; 3]>,
    elevation: f64,
    turf: bool,
    talent_given: bool,
    foreign_given: bool,
    /// `bm park`: measured batted balls to fly through the park instead of games, and a
    /// change of their exit speed (mph).
    balls: Option<std::path::PathBuf>,
    ev_shift: f64,
    /// `bm park`: air temperature and humidity of the experiment.
    weather: level::Weather,
    /// `bm validate`, `bm tactics`: infield shifts restricted (MLB since 2023) or not.
    shift_rule: Option<bool>,
    /// `bm season`: play up to this date instead of the whole season.
    until: Option<bm_world::Date>,
    /// `bm careers`: offseasons to run, how often to print, other level and career files.
    years: usize,
    /// `bm season`: seasons to play in a row, after this many offseasons without games.
    seasons: usize,
    warmup: usize,
    /// `bm season`, `bm careers`: offseasons of the new world's past (default: careers.json).
    history: Option<u32>,
    every: usize,
    from: Option<usize>,
    levels_file: Option<std::path::PathBuf>,
    careers_file: Option<std::path::PathBuf>,
    contracts_file: Option<std::path::PathBuf>,
    aging_file: Option<std::path::PathBuf>,
    json: Option<std::path::PathBuf>,
    /// `bm contacts`: where to write the rows, and rows to run the collision model backwards on.
    out: Option<std::path::PathBuf>,
    attrs_out: Option<std::path::PathBuf>,
    /// `bm season`: directory for every game's box-score lines.
    game_logs: Option<std::path::PathBuf>,
    /// `bm lineup`: the club, batting orders to try and random ones.
    club: usize,
    orders: Vec<Vec<usize>>,
    permutations: usize,
    /// `bm season --repeat`: repeats of the first season, their leagues and where to write
    /// the records.
    repeat: usize,
    leagues: Option<Vec<String>>,
    standings_out: Option<std::path::PathBuf>,
    invert: Option<std::path::PathBuf>,
    face_lead: f64,
    dir_shift: f64,
    swings: bool,
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
        world: true,
        parks: true,
        only: None,
        level: "aaa".into(),
        level_given: false,
        talent: bm_world::Talent::MAJOR,
        foreign: bm_world::Talent::MAJOR,
        vs_mlb: false,
        cross_games: None,
        states: 120,
        rollouts: 200,
        vs_mlb_pitchers: false,
        by_position: false,
        ball_drag: 1.0,
        fence: None,
        wall: None,
        elevation: 0.0,
        turf: false,
        talent_given: false,
        foreign_given: false,
        balls: None,
        ev_shift: 0.0,
        weather: level::Weather::default(),
        shift_rule: None,
        until: None,
        years: 20,
        seasons: 1,
        warmup: 0,
        history: None,
        every: 5,
        from: None,
        levels_file: None,
        careers_file: None,
        contracts_file: None,
        aging_file: None,
        json: None,
        out: None,
        attrs_out: None,
        game_logs: None,
        club: 0,
        orders: Vec::new(),
        permutations: 0,
        repeat: 0,
        leagues: None,
        standings_out: None,
        invert: None,
        face_lead: 0.0,
        dir_shift: 0.0,
        swings: false,
        tuning: Tuning::new(),
    };
    let mut it = std::env::args().skip(1);
    if let Some(c) = it.next() {
        a.cmd = c;
    }
    while let Some(k) = it.next() {
        if k == "--vs-mlb" {
            a.vs_mlb = true;
            continue;
        }
        if k == "--swings" {
            a.swings = true;
            continue;
        }
        if k == "--by-position" {
            a.by_position = true;
            continue;
        }
        if k == "--vs-mlb-pitchers" {
            a.vs_mlb = true;
            a.vs_mlb_pitchers = true;
            continue;
        }
        if k == "--turf" {
            a.turf = true;
            continue;
        }
        let v = it.next().unwrap_or_default();
        match k.as_str() {
            "--games" => a.games = v.parse().expect("--games N"),
            "--years" => a.years = v.parse().expect("--years N"),
            "--ball-drag" => a.ball_drag = v.parse().expect("--ball-drag F"),
            "--fence" | "--wall" => {
                let x: Vec<f64> = v.split(',').map(|x| x.trim().parse().unwrap_or_else(|_| panic!("{k}: numbers in m, comma-separated"))).collect();
                if k == "--fence" {
                    a.fence = Some(
                        x.try_into()
                            .unwrap_or_else(|_: Vec<f64>| panic!("--fence needs five distances: left line, left-centre, centre, right-centre, right line")),
                    );
                } else {
                    a.wall = Some(match x.len() {
                        1 => [x[0]; 3],
                        3 => [x[0], x[1], x[2]],
                        _ => panic!("--wall needs one height or three (left, centre, right)"),
                    });
                }
            }
            "--elevation" => a.elevation = v.parse().expect("--elevation M"),
            "--balls" => a.balls = Some(v.into()),
            "--ev-shift" => a.ev_shift = v.parse().expect("--ev-shift MPH"),
            "--zone" => {
                a.tuning.zone = Some(bm_engine::ZonePreset::parse(&v).unwrap_or_else(|| {
                    eprintln!("unknown zone {v}; use human | kbo-abs | mlb-abs");
                    std::process::exit(2);
                }))
            }
            "--temp" => a.weather.temp_c = Some(v.parse().expect("--temp C")),
            "--humidity" => {
                a.weather.humidity = v.parse().expect("--humidity PCT");
                assert!((0.0..=100.0).contains(&a.weather.humidity), "--humidity is a percentage (0-100)");
            }
            "--shift-rule" => {
                a.shift_rule = Some(match v.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => panic!("--shift-rule on|off"),
                })
            }
            "--seasons" => a.seasons = v.parse().expect("--seasons N"),
            "--warmup" => a.warmup = v.parse().expect("--warmup N"),
            "--history" => a.history = Some(v.parse().expect("--history H")),
            "--every" => a.every = v.parse().expect("--every K"),
            "--from" => a.from = Some(v.parse().expect("--from N")),
            "--levels" => a.levels_file = Some(v.into()),
            "--careers" => a.careers_file = Some(v.into()),
            "--contracts" => a.contracts_file = Some(v.into()),
            "--json" => a.json = Some(v.into()),
            "--out" => a.out = Some(v.into()),
            "--attrs-out" => a.attrs_out = Some(v.into()),
            "--game-logs" => a.game_logs = Some(v.into()),
            "--repeat" => a.repeat = v.parse().expect("--repeat N"),
            "--club" => a.club = v.parse().expect("--club K"),
            "--order" => a.orders.push(v.split(',').map(|x| x.trim().parse().expect("--order p1,...,p9 (places 1-9)")).collect()),
            "--permutations" => a.permutations = v.parse().expect("--permutations K"),
            "--leagues" => a.leagues = Some(v.split(',').map(|s| s.trim().to_string()).collect()),
            "--standings-out" => a.standings_out = Some(v.into()),
            "--invert" => a.invert = Some(v.into()),
            "--face-lead" => a.face_lead = v.parse().expect("--face-lead DEG"),
            "--dir-shift" => a.dir_shift = v.parse().expect("--dir-shift DEG"),
            "--aging" => a.aging_file = Some(v.into()),
            "--seed" => a.seed = v.parse().expect("--seed S"),
            "--threads" => a.threads = v.parse::<usize>().expect("--threads T").max(1),
            "--clubs" => a.clubs = v.parse::<usize>().expect("--clubs K").max(2),
            "--port" => a.port = v.parse().expect("--port P"),
            "--population" => {
                a.world = match v.as_str() {
                    "world" => true,
                    "engine" => false,
                    other => {
                        eprintln!("unknown population {other}; use engine | world");
                        std::process::exit(2);
                    }
                }
            }
            "--parks" => {
                a.parks = match v.as_str() {
                    "world" => true,
                    "neutral" => false,
                    other => {
                        eprintln!("unknown parks {other}; use world | neutral");
                        std::process::exit(2);
                    }
                }
            }
            "--only" => a.only = Some(v),
            "--level" => {
                a.level = v;
                a.level_given = true;
            }
            "--cross-games" => a.cross_games = Some(v.parse().expect("--cross-games N")),
            "--states" => a.states = v.parse::<usize>().expect("--states N").max(1),
            "--rollouts" => a.rollouts = v.parse::<usize>().expect("--rollouts R").max(1),
            "--until" => a.until = Some(bm_world::Date::parse(&v).expect("--until YYYY-MM-DD")),
            "--talent" | "--foreign" => {
                let t: Vec<f32> = v
                    .split(',')
                    .map(|x| x.trim().parse().expect("--talent hitting,pitching,fielding,physical[,power,command[,velocity[,discipline]]]"))
                    .collect();
                if ![4, 6, 7, 8].contains(&t.len()) {
                    eprintln!("{k} needs four, six, seven or eight numbers: hitting,pitching,fielding,physical[,power,command[,velocity[,discipline]]]");
                    std::process::exit(2);
                }
                let extra = |i: usize| t.get(i).copied().unwrap_or(0.0);
                let talent = bm_world::Talent {
                    hitting: t[0],
                    pitching: t[1],
                    fielding: t[2],
                    physical: t[3],
                    power: extra(4),
                    command: extra(5),
                    velocity: extra(6),
                    discipline: extra(7),
                    spread: 1.0,
                    tradeoff: 0.0,
                };
                if k == "--talent" {
                    a.talent = talent;
                    a.talent_given = true;
                } else {
                    a.foreign = talent;
                    a.foreign_given = true;
                }
            }
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

fn make_clubs(n: usize, seed: u64, level: &Level, world: Option<&bm_world::Baseline>) -> Vec<Club> {
    let mut rng = Rng::new(seed ^ 0xC1B5);
    if let Some(base) = world {
        return (0..n).map(|i| bm_world::Squad::random(&mut rng, 1000 * (i as u32 + 1), &bm_world::Talent::MAJOR).to_engine_with(base)).collect();
    }
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

/// Run `games` games on `threads` threads, feeding every event to a [`Tally`]. Home club
/// `h` plays in `envs[h % envs.len()]`.
fn simulate(envs: &[MatchEnv], clubs: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize, observe: bool) -> Tally {
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
                    let mut t = Tally::default();
                    for _ in 0..n {
                        let (a, h) = pick_pair(&mut rng, clubs.len());
                        let env = &envs[h % envs.len()];
                        // Home runs are projected to field level in the air of the park they were hit in.
                        t.flight = Some(env.batted_flight);
                        t.park = Some(env.park.clone());
                        let dh = env.rules.dh;
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

/// Per class: each tactic's change of the chance of winning and of the runs to the end of the
/// inning against the first tactic (swinging away), mean over plate appearances ± standard error.
fn print_rollouts(res: &[bm_gym::StateResult]) {
    use bm_gym::rollout::mean_se;
    for (c, class) in bm_gym::CLASSES.iter().enumerate() {
        let rows: Vec<&bm_gym::StateResult> = res.iter().filter(|r| r.class == c).collect();
        if rows.is_empty() {
            continue;
        }
        let base_win = mean_se(&rows.iter().map(|r| r.win[0]).collect::<Vec<_>>()).0;
        let base_runs = mean_se(&rows.iter().map(|r| r.runs[0]).collect::<Vec<_>>()).0;
        println!();
        println!(
            "{} ({} plate appearances): {} wins {:.3}, runs to the end of the inning {:.3}",
            class.name,
            rows.len(),
            class.tactics[0].name(),
            base_win,
            base_runs
        );
        for (t, tactic) in class.tactics.iter().enumerate().skip(1) {
            let (dw, sw) = mean_se(&rows.iter().map(|r| r.win[t] - r.win[0]).collect::<Vec<_>>());
            let (dr, sr) = mean_se(&rows.iter().map(|r| r.runs[t] - r.runs[0]).collect::<Vec<_>>());
            println!("  {:<13} win {:+.4} ± {:.4}   runs {:+.3} ± {:.3}", tactic.name(), dw, sw, dr, sr);
        }
    }
}

fn main() {
    let mut args = parse_args();
    // Neutral MLB-like conditions (see `MatchEnv::neutral_mlb`) with any command-line overrides.
    let mut env = MatchEnv::new(
        &Atmosphere::new(24.0, 760.0, 55.0, 180.0),
        Wind::CALM,
        Park::mlb_average(),
        args.tuning.rules(Rules::mlb_2025()),
        args.tuning.params.clone(),
    );
    env.ball = args.tuning.ball;
    if let Some(z) = args.tuning.zone {
        println!("ball-strike calls: {} (--zone)", z.name());
    }
    if let Some(restricted) = args.shift_rule {
        // Without the restriction the AI managers shift as MLB clubs did in 2022.
        env.rules.shift_restricted = restricted;
        args.tuning.style.full_shift = !restricted;
    }
    let level: &Level = &args.tuning.level;
    let world = args.world.then_some(&args.tuning.baseline);
    let envs = if args.parks { level::park_envs(&["us-major"], &env.rules, &args.tuning).0 } else { vec![env.clone()] };
    match args.cmd.as_str() {
        "tactics" => {
            let clubs = make_clubs(args.clubs, args.seed, level, world);
            let t = tactics::run(&envs, &clubs, args.tuning.style, args.games, args.seed, args.threads);
            tactics::print(&t);
            if let Some(path) = &args.json {
                std::fs::write(path, serde_json::to_string_pretty(&t.json()).expect("json")).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
        "we" => {
            let league = bm_gym::League::mlb(args.seed);
            let start = Instant::now();
            let t = bm_gym::WinExpectancy::build(&league, args.tuning.style, args.games, args.seed, args.threads);
            println!("{} games, {} plate appearances, {:.1} s", t.games, t.plate_appearances(), start.elapsed().as_secs_f64());
            println!("home team's chance of winning (start of a plate appearance):");
            for (label, inning, top, outs, bases, lead) in [
                ("start of the game", 1, true, 0, 0, 0),
                ("top 9th, home up 1, nobody on, 0 out", 9, true, 0, 0, 1),
                ("bottom 9th, tied, nobody on, 0 out", 9, false, 0, 0, 0),
                ("bottom 9th, tied, runner on 1st, 0 out", 9, false, 0, 1, 0),
                ("bottom 9th, tied, runner on 2nd, 1 out", 9, false, 1, 2, 0),
                ("bottom 9th, down 1, bases loaded, 2 out", 9, false, 2, 7, -1),
                ("top 5th, home up 3, nobody on, 0 out", 5, true, 0, 0, 3),
            ] {
                println!("  {label:<44} {:.3}", t.home(inning, top, outs, bases, lead));
            }
            let path = args.json.clone().unwrap_or_else(|| "data/engine/win_expectancy.json".into());
            std::fs::write(&path, serde_json::to_string(&t).expect("json") + "\n").unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            println!("written {}", path.display());
        }
        "re" => {
            let league = bm_gym::League::mlb(args.seed);
            let start = Instant::now();
            let t = bm_gym::RunExpectancy::build(&league, args.tuning.style, args.games, args.seed, args.threads);
            println!("{} games, {} pitches, {:.1} s", t.games, t.pitches(), start.elapsed().as_secs_f64());
            println!("runs to the end of the half inning, before the pitch (rows: base state; columns: count):");
            print!("  {:<16}", "bases/outs");
            let counts = [(0, 0), (1, 0), (0, 1), (2, 0), (1, 1), (0, 2), (3, 0), (2, 1), (1, 2), (3, 1), (2, 2), (3, 2)];
            for (b, s) in counts {
                print!(" {b}-{s}  ");
            }
            println!();
            for bases in [0u8, 1, 2, 3, 4, 5, 6, 7] {
                for outs in 0..3u8 {
                    let label = format!(
                        "{}{}{} {} out",
                        if bases & 1 != 0 { '1' } else { '-' },
                        if bases & 2 != 0 { '2' } else { '-' },
                        if bases & 4 != 0 { '3' } else { '-' },
                        outs
                    );
                    print!("  {label:<16}");
                    for (b, s) in counts {
                        print!(" {:.3}", t.value(bases, outs, b, s));
                    }
                    println!();
                }
            }
            let path = args.json.clone().unwrap_or_else(|| "data/engine/run_expectancy.json".into());
            std::fs::write(&path, serde_json::to_string(&t).expect("json") + "\n").unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            println!("written {}", path.display());
        }
        "rollout" => {
            let league = std::sync::Arc::new(bm_gym::League::mlb(args.seed));
            let we = std::sync::Arc::new(bm_gym::WinExpectancy::bundled());
            let cfg = bm_gym::rollout::Config { states: args.states, rollouts: args.rollouts, seed: args.seed, threads: args.threads, sample: 0.25 };
            let start = Instant::now();
            let res = bm_gym::rollout::evaluate(league, we, &cfg);
            println!("{} plate appearances, {} rollouts per tactic, {:.0} s", res.len(), args.rollouts, start.elapsed().as_secs_f64());
            print_rollouts(&res);
            if let Some(path) = &args.json {
                let rows: Vec<serde_json::Value> = res
                    .iter()
                    .map(|r| {
                        let mut v = serde_json::to_value(r).expect("json");
                        v["class_name"] = bm_gym::CLASSES[r.class].name.into();
                        v["tactics"] = bm_gym::CLASSES[r.class].tactics.iter().map(|t| t.name()).collect::<Vec<_>>().into();
                        v
                    })
                    .collect();
                std::fs::write(path, serde_json::to_string(&rows).expect("json") + "\n").unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
        "validate" => {
            let clubs = make_clubs(args.clubs, args.seed, level, world);
            let start = Instant::now();
            let t = simulate(&envs, &clubs, args.tuning.style, args.games, args.seed, args.threads, true);
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
            if let Some(path) = args.json.as_deref() {
                // Pitches by type, count and result, for a run value by pitch type.
                let kinds: serde_json::Map<String, serde_json::Value> =
                    PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.pitch_classes[k.index()]))).collect();
                let zone: serde_json::Map<String, serde_json::Value> =
                    PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.pitch_classes_zone[k.index()]))).collect();
                let dist: serde_json::Map<String, serde_json::Value> =
                    PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.kinds[k.index()].dist))).collect();
                let doc = serde_json::json!({
                    "classes": tally::PITCH_CLASSES,
                    "counts": "balls * 3 + strikes",
                    "pitches": kinds,
                    "in_zone": zone,
                    "dist_edges": tally::KIND_DIST_EDGES,
                    "swing_by_dist": dist,
                    "speed_whiff": PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.kinds[k.index()].speed_whiff))).collect::<serde_json::Map<String, serde_json::Value>>(),
                    "fb_whiff": PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.kinds[k.index()].fb_whiff))).collect::<serde_json::Map<String, serde_json::Value>>(),
                    "edge": PitchKind::ALL.iter().map(|k| (k.code().to_string(), serde_json::json!(t.kinds[k.index()].edge))).collect::<serde_json::Map<String, serde_json::Value>>(),
                    "regions": tally::REGION_NAMES,
                    "region_classes": t.region_classes,
                    "region_bip": t.region_bip,
                    "region_swing": t.region_swing,
                    "region_miss": t.region_miss,
                    "kind_zone_miss": t.kind_zone_miss,
                    "kind_region": t.kind_region,
                    "region_contact": t.region_contact,
                    "height_geom": t.height_geom,
                    "depth_fine_from_in": -60,
                    "depth_fine": t.geo.depth_fine.iter().flatten().collect::<Vec<_>>(),
                });
                std::fs::write(path, serde_json::to_string_pretty(&doc).expect("json")).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
        "contacts" => {
            let out = args.out.as_deref().expect("bm contacts needs --out FILE");
            match args.invert.as_deref() {
                Some(input) => contacts::invert(&envs[0], input, out, args.face_lead, args.dir_shift),
                None => {
                    let clubs = make_clubs(args.clubs, args.seed, level, world);
                    contacts::dump(&envs, &clubs, args.tuning.style, args.games, args.seed, args.threads, out, args.swings);
                }
            }
        }
        "bench" => {
            let clubs = make_clubs(args.clubs, args.seed, level, world);
            for threads in [1, args.threads] {
                let start = Instant::now();
                let t = simulate(&envs, &clubs, args.tuning.style, args.games, args.seed, threads, false);
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
            let clubs = make_clubs(args.clubs, args.seed, level, world);
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
        "attributes" if args.by_position => {
            sensitivity::run_positions(&env, &args.tuning.baseline, args.tuning.style, args.games, args.seed, args.threads, args.clubs.min(60), 2.0)
        }
        "attributes" => {
            sensitivity::run(&env, &args.tuning.baseline, args.tuning.style, args.games, args.seed, args.threads, args.clubs.min(60), args.only.as_deref())
        }
        "level" => {
            let vs = args.vs_mlb.then(|| level::VsMlb { games: args.cross_games.unwrap_or(args.games.max(400)), pitchers: args.vs_mlb_pitchers });
            level::run(&args.level, args.talent, args.foreign, args.games, args.seed, args.threads, &args.tuning, vs, args.ball_drag)
        }
        "lineup" => {
            let levels = bm_world::LevelTable::bundled();
            let name = if args.level_given { args.level.as_str() } else { "mlb" };
            let profile = levels.levels.get(name).unwrap_or_else(|| panic!("unknown level '{name}'"));
            let talent = if args.talent_given { args.talent } else { profile.talent() };
            let foreign = if args.foreign_given { args.foreign } else { profile.foreign.as_ref().map_or(bm_world::Talent::MAJOR, |f| f.talent()) };
            let drag = if args.ball_drag != 1.0 { args.ball_drag } else { profile.ball_drag };
            lineup::run(
                name,
                talent,
                foreign,
                drag,
                args.club,
                &args.orders,
                args.permutations,
                args.games,
                args.seed,
                args.threads,
                &args.tuning,
                args.json.as_deref(),
            );
        }
        "park" => {
            let levels = bm_world::LevelTable::bundled();
            let profile = levels.levels.get(&args.level).unwrap_or_else(|| panic!("unknown level '{}'", args.level));
            let talent = if args.talent_given { args.talent } else { profile.talent() };
            let foreign = if args.foreign_given { args.foreign } else { profile.foreign.as_ref().map_or(bm_world::Talent::MAJOR, |f| f.talent()) };
            let drag = if args.ball_drag != 1.0 { args.ball_drag } else { profile.ball_drag };
            let ballpark = bm_world::league::Ballpark {
                name: bm_world::league::Text { ko: "실험 구장".into(), en: "Experimental park".into() },
                capacity: None,
                elevation_m: Some(args.elevation),
                roof: bm_world::league::Roof::Open,
                surface: if args.turf { bm_world::league::Surface::Turf } else { bm_world::league::Surface::Grass },
                fence_m: args.fence.expect("bm park needs --fence l,lc,c,rc,r (m)"),
                wall_m: args.wall.unwrap_or([2.4; 3]),
                estimated: Vec::new(),
                batted_drag: 1.0,
            };
            match &args.balls {
                Some(balls) => level::park_balls(&args.level, &ballpark, balls, args.ev_shift, &args.tuning, drag, args.weather, args.json.as_deref()),
                None => level::park(
                    &args.level,
                    talent,
                    foreign,
                    &ballpark,
                    args.games,
                    args.seed,
                    args.threads,
                    &args.tuning,
                    drag,
                    args.weather,
                    args.json.as_deref(),
                ),
            }
        }
        "season" => season::run(
            &args.tuning,
            args.seed,
            args.threads,
            &season::Options {
                until: args.until,
                seasons: args.seasons,
                warmup: args.warmup,
                history: args.history,
                levels_file: args.levels_file.as_deref(),
                careers_file: args.careers_file.as_deref(),
                contracts_file: args.contracts_file.as_deref(),
                aging_file: args.aging_file.as_deref(),
                json: args.json.as_deref(),
                out: args.out.as_deref(),
                game_logs: args.game_logs.as_deref(),
                repeat: args.repeat,
                leagues: args.leagues.clone(),
                standings_out: args.standings_out.as_deref(),
            },
        ),
        "careers" => careers::run(
            args.years,
            args.history,
            args.seed,
            args.every,
            args.from,
            args.levels_file.as_deref(),
            args.careers_file.as_deref(),
            args.aging_file.as_deref(),
            args.contracts_file.as_deref(),
            args.json.as_deref(),
            args.out.as_deref(),
            args.attrs_out.as_deref(),
        ),
        "players" => {
            players::run(&env, &args.tuning.baseline, args.tuning.style, args.seed, args.threads, args.clubs.min(60), args.games.min(400), args.json.as_deref())
        }
        "game" => {
            let clubs = make_clubs(2, args.seed, level, world);
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
            eprintln!(
                "unknown command {other}; use validate | tactics | attributes | players | level | park | season | careers | game | bench | population | serve"
            );
            std::process::exit(2);
        }
    }
}
