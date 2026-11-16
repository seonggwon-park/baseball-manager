//! `bm level`: a league below the majors, or in Korea or Japan, played by players generated
//! at a given talent level in the league's own ballparks and under its rules, compared with
//! the league's official rates (`docs/research/world/levels.md`).
//!
//! With `--vs-mlb` the level's hitters bat against major-league pitching and the level's
//! pitchers face major-league hitters, in a neutral park; the runs per plate appearance are
//! compared with those of major-leaguers against major-leaguers. Published translations
//! give the hitters' side (FanGraphs: wRC+ 35 points lower at Triple-A, 48 at Double-A,
//! 60 at High-A, 70 at Single-A).

use crate::tuning::Tuning;
use bm_engine::{play_game_styled, BattedInfo, Club, MatchEnv, Observer, PitchEvent, PitchKind, PitchResult, Rng, Rules, TeamBox};
use bm_physics::units::MPH;
use bm_physics::{Atmosphere, Park, Wind};
use bm_world::{Squad, Talent, World};

/// Official rates of a league-season: runs per team-game, AVG, OBP, SLG, and K, BB and HR
/// per plate appearance (%).
#[derive(Clone, Copy)]
struct Rates {
    r: f64,
    avg: f64,
    obp: f64,
    slg: f64,
    k: f64,
    bb: f64,
    hr: f64,
}

/// `docs/research/world/levels.md` section 2.1: 2025 and 2026.
// An OBP of .318 is not 1/pi.
#[allow(clippy::approx_constant)]
fn targets(level: &str) -> Option<[Rates; 2]> {
    let r = |r, avg, obp, slg, k, bb, hr| Rates { r, avg, obp, slg, k, bb, hr };
    Some(match level {
        "mlb" => [r(4.45, 0.245, 0.315, 0.404, 22.2, 8.4, 3.09), r(4.48, 0.244, 0.318, 0.400, 22.1, 8.9, 3.03)],
        "aaa" => [r(5.24, 0.258, 0.347, 0.421, 22.6, 10.9, 2.87), r(5.31, 0.258, 0.349, 0.422, 22.3, 11.2, 2.92)],
        "aa" => [r(4.29, 0.235, 0.323, 0.360, 23.5, 10.3, 2.01), r(5.24, 0.249, 0.340, 0.415, 24.4, 10.9, 3.08)],
        "high_a" => [r(4.51, 0.233, 0.330, 0.358, 23.7, 11.0, 1.90), r(5.54, 0.249, 0.351, 0.413, 24.2, 12.0, 2.91)],
        "single_a" => [r(4.82, 0.237, 0.340, 0.345, 23.5, 11.9, 1.42), r(5.57, 0.248, 0.354, 0.395, 24.4, 12.5, 2.32)],
        "kbo" => [r(4.73, 0.262, 0.338, 0.389, 19.7, 9.1, 2.13), r(5.07, 0.268, 0.346, 0.404, 19.5, 9.4, 2.41)],
        "kbo2" => [r(6.22, 0.286, 0.381, 0.427, 17.8, 12.0, 2.15), r(5.44, 0.270, 0.362, 0.383, 16.9, 11.1, 1.51)],
        "npb" => [r(3.29, 0.244, 0.305, 0.351, 19.4, 7.1, 1.71), r(3.59, 0.240, 0.305, 0.363, 21.0, 7.6, 2.23)],
        "npb2" => [r(3.80, 0.250, 0.321, 0.343, 18.2, 8.5, 1.07), r(4.11, 0.253, 0.327, 0.364, 19.1, 8.8, 1.55)],
        _ => return None,
    })
}

/// League ids of the world data that make up a level, and the rules played there.
fn leagues(level: &str) -> Option<(Vec<&'static str>, Rules)> {
    Some(match level {
        "mlb" => (vec!["us-major"], Rules::mlb_2025()),
        "aaa" => (vec!["us-aaa-east", "us-aaa-west"], Rules::mlb_2025()),
        "aa" => (vec!["us-aa-northeast", "us-aa-south", "us-aa-central"], Rules::mlb_2025()),
        "high_a" => (vec!["us-higha-midwest", "us-higha-northwest", "us-higha-atlantic"], Rules::mlb_2025()),
        "single_a" => (vec!["us-a-california", "us-a-carolina", "us-a-florida"], Rules::mlb_2025()),
        "kbo" => (vec!["kr-top"], Rules::kbo_2025()),
        "kbo2" => (vec!["kr-second"], Rules::kbo_2025()),
        // The Central League uses the designated hitter from 2027.
        "npb" => (vec!["jp-top"], Rules::npb_2025(true)),
        "npb2" => (vec!["jp-second"], Rules::npb_2025(true)),
        _ => return None,
    })
}

/// Batting totals from box scores.
#[derive(Clone, Copy, Default)]
struct Line {
    team_games: f64,
    runs: f64,
    pa: f64,
    ab: f64,
    h: f64,
    doubles: f64,
    triples: f64,
    tb: f64,
    hr: f64,
    bb: f64,
    ibb: f64,
    hbp: f64,
    so: f64,
    sf: f64,
}

impl Line {
    fn add(&mut self, b: &TeamBox, runs: u32) {
        self.team_games += 1.0;
        self.runs += runs as f64;
        for l in &b.batting {
            self.pa += l.pa as f64;
            self.ab += l.ab as f64;
            self.h += l.h as f64;
            self.doubles += l.doubles as f64;
            self.triples += l.triples as f64;
            self.tb += (l.h + l.doubles + 2 * l.triples + 3 * l.hr) as f64;
            self.hr += l.hr as f64;
            self.bb += l.bb as f64;
            self.ibb += l.ibb as f64;
            self.hbp += l.hbp as f64;
            self.so += l.so as f64;
            self.sf += l.sf as f64;
        }
    }

    fn merge(&mut self, o: &Line) {
        self.team_games += o.team_games;
        self.runs += o.runs;
        self.pa += o.pa;
        self.ab += o.ab;
        self.h += o.h;
        self.doubles += o.doubles;
        self.triples += o.triples;
        self.tb += o.tb;
        self.hr += o.hr;
        self.bb += o.bb;
        self.ibb += o.ibb;
        self.hbp += o.hbp;
        self.so += o.so;
        self.sf += o.sf;
    }

    fn rates(&self) -> Rates {
        Rates {
            r: self.runs / self.team_games,
            avg: self.h / self.ab,
            obp: (self.h + self.bb + self.hbp) / (self.ab + self.bb + self.hbp + self.sf),
            slg: self.tb / self.ab,
            k: 100.0 * self.so / self.pa,
            bb: 100.0 * self.bb / self.pa,
            hr: 100.0 * self.hr / self.pa,
        }
    }

    fn runs_per_pa(&self) -> f64 {
        self.runs / self.pa
    }

    /// wOBA with FanGraphs' 2025 linear weights.
    fn woba(&self) -> f64 {
        let singles = self.h - self.doubles - self.triples - self.hr;
        let num = 0.691 * (self.bb - self.ibb) + 0.722 * self.hbp + 0.882 * singles + 1.252 * self.doubles + 1.584 * self.triples + 2.037 * self.hr;
        num / (self.ab + self.bb - self.ibb + self.sf + self.hbp)
    }

    /// wRC+ of these batters in the run environment of `league` (100 = its average):
    /// runs above average per PA from the wOBA gap (wOBA scale 1.232, FanGraphs 2025),
    /// over the league's runs per PA. FanGraphs' level translations are in these units.
    fn wrc_plus(&self, league: &Line) -> f64 {
        let r = league.runs_per_pa();
        100.0 * ((self.woba() - league.woba()) / 1.232 + r) / r
    }
}

/// Four-seam velocity and exit velocity, for comparison with the tracked levels.
#[derive(Clone, Copy, Default)]
struct Speeds {
    ff: (f64, f64),
    ev: (f64, f64),
}

impl Observer for Speeds {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.thrown.kind == PitchKind::FourSeam {
            self.ff.0 += e.thrown.spec.speed / MPH;
            self.ff.1 += 1.0;
        }
        // Statcast's average exit velocity is over balls in play.
        if let (PitchResult::InPlay, Some(BattedInfo { exit_speed, bunt: false, .. })) = (e.result, e.batted) {
            self.ev.0 += exit_speed / MPH;
            self.ev.1 += 1.0;
        }
    }
}

struct Env {
    clubs: Vec<Club>,
    /// League index of every club, and one match environment per club (its ballpark).
    league: Vec<usize>,
    envs: Vec<MatchEnv>,
}

/// One match environment for every club of the given leagues, in the order of the world
/// data: its ballpark and elevation, the league's rules, average game-time weather and no
/// wind. Returns the league index of every club alongside.
pub fn park_envs(ids: &[&str], rules: &Rules, tuning: &Tuning) -> (Vec<MatchEnv>, Vec<usize>) {
    let world = World::bundled();
    let (mut envs, mut league) = (Vec::new(), Vec::new());
    for (li, id) in ids.iter().enumerate() {
        for c in world.clubs().filter(|c| c.league == *id) {
            let mut env =
                MatchEnv::new(&Atmosphere::new(24.0, 760.0, 55.0, c.park.elevation()), Wind::CALM, c.park.field(), rules.clone(), tuning.params.clone());
            env.ball = tuning.ball;
            envs.push(env);
            league.push(li);
        }
    }
    (envs, league)
}

/// Foreign players a first-team club of the level uses: starting pitchers, relievers and
/// lineup slots (0 = catcher ... 8 = designated hitter). KBO clubs field three foreign
/// players, usually two starters and a hitter, plus an Asia-quota player, usually a
/// pitcher; NPB allows four on the active roster, at most three pitchers or three hitters.
fn foreign_slots(level: &str) -> (usize, usize, &'static [usize]) {
    match level {
        "kbo" => (2, 1, &[8]),
        "npb" => (2, 0, &[1, 8]),
        _ => (0, 0, &[]),
    }
}

fn build(level: &str, talent: &Talent, foreign: &Talent, tuning: &Tuning, seed: u64) -> Env {
    let (ids, rules) = leagues(level).unwrap_or_else(|| panic!("unknown level '{level}'"));
    let (envs, league) = park_envs(&ids, &rules, tuning);
    let mut rng = Rng::new(seed ^ 0x1E7E1);
    let (fs, fr, fh) = foreign_slots(level);
    let clubs = (0..envs.len())
        .map(|i| {
            let id = 1000 * (i as u32 + 1);
            let mut squad = Squad::random(&mut rng, id, talent);
            if fs + fr + fh.len() > 0 {
                squad.import_from(&Squad::random(&mut rng, id + 500, foreign), fs, fr, fh);
            }
            squad.to_engine_with(&tuning.baseline)
        })
        .collect();
    Env { clubs, league, envs }
}

/// The level's own league: random pairs of the same league, at the home club's park.
fn own_league(env: &Env, games: usize, seed: u64, threads: usize, style: bm_engine::ManagerStyle) -> (Line, Speeds) {
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let rngs: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let mut total = (Line::default(), Speeds::default());
    std::thread::scope(|scope| {
        let handles: Vec<_> = rngs
            .into_iter()
            .enumerate()
            .map(|(t, mut rng)| {
                let n = per.min(games.saturating_sub(t * per));
                scope.spawn(move || {
                    let (mut line, mut speeds) = (Line::default(), Speeds::default());
                    for _ in 0..n {
                        let h = rng.below(env.clubs.len());
                        let same: Vec<usize> = (0..env.clubs.len()).filter(|&c| c != h && env.league[c] == env.league[h]).collect();
                        let a = same[rng.below(same.len())];
                        let menv = &env.envs[h];
                        let dh = menv.rules.dh;
                        let away = env.clubs[a].team_for_game(rng.below(5), dh);
                        let home = env.clubs[h].team_for_game(rng.below(5), dh);
                        let res = play_game_styled(menv, &away, &home, [style; 2], &mut rng, &mut speeds);
                        line.add(&res.boxes[0], res.score[0]);
                        line.add(&res.boxes[1], res.score[1]);
                    }
                    (line, speeds)
                })
            })
            .collect();
        for h in handles {
            let (l, s) = h.join().expect("simulation thread panicked");
            total.0.merge(&l);
            total.1.ff.0 += s.ff.0;
            total.1.ff.1 += s.ff.1;
            total.1.ev.0 += s.ev.0;
            total.1.ev.1 += s.ev.1;
        }
    });
    total
}

/// Runs per plate appearance of `bat` hitters against `pitch` pitchers in a neutral park.
fn cross(bat: &[Club], pitch: &[Club], games: usize, seed: u64, threads: usize, tuning: &Tuning) -> Line {
    let mut env = MatchEnv::new(&Atmosphere::new(24.0, 760.0, 55.0, 180.0), Wind::CALM, Park::mlb_average(), Rules::mlb_2025(), tuning.params.clone());
    env.ball = tuning.ball;
    let style = tuning.style;
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let rngs: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let env = &env;
    let mut total = Line::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = rngs
            .into_iter()
            .enumerate()
            .map(|(t, mut rng)| {
                let n = per.min(games.saturating_sub(t * per));
                scope.spawn(move || {
                    let mut line = Line::default();
                    for g in 0..n {
                        let b = &bat[rng.below(bat.len())];
                        let p = &pitch[rng.below(pitch.len())];
                        let (bt, pt) = (b.team_for_game(rng.below(5), true), p.team_for_game(rng.below(5), true));
                        // Alternate home and away so that the last-inning rule falls on both sides.
                        if g % 2 == 0 {
                            let res = play_game_styled(env, &bt, &pt, [style; 2], &mut rng, &mut bm_engine::NoObserver);
                            line.add(&res.boxes[0], res.score[0]);
                        } else {
                            let res = play_game_styled(env, &pt, &bt, [style; 2], &mut rng, &mut bm_engine::NoObserver);
                            line.add(&res.boxes[1], res.score[1]);
                        }
                    }
                    line
                })
            })
            .collect();
        for h in handles {
            total.merge(&h.join().expect("simulation thread panicked"));
        }
    });
    total
}

#[allow(clippy::too_many_arguments)]
/// What to play against major-leaguers: `games` games each for the major-league baseline,
/// the level's hitters and, if asked, the level's pitchers.
pub struct VsMlb {
    pub games: usize,
    pub pitchers: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn run(level: &str, talent: Talent, foreign: Talent, games: usize, seed: u64, threads: usize, tuning: &Tuning, vs_mlb: Option<VsMlb>) {
    let mut rng = Rng::new(seed ^ 0xA11);
    let mlb: Vec<Club> = (0..30).map(|i| Squad::random(&mut rng, 900_000 + 1000 * i, &Talent::MAJOR).to_engine_with(&tuning.baseline)).collect();
    let against_mlb = |clubs: &[Club], v: &VsMlb| {
        let base = cross(&mlb, &mlb, v.games, seed ^ 1, threads, tuning);
        let hitters = cross(clubs, &mlb, v.games, seed ^ 2, threads, tuning);
        print!(
            "  against major-leaguers (neutral park): hitters wRC+ {:.1} ({:.0} % of the major-league runs per PA)",
            hitters.wrc_plus(&base),
            100.0 * hitters.runs_per_pa() / base.runs_per_pa()
        );
        if v.pitchers {
            let pitchers = cross(&mlb, clubs, v.games, seed ^ 3, threads, tuning);
            print!(", pitchers allow wRC+ {:.1} ({:.0} %)", pitchers.wrc_plus(&base), 100.0 * pitchers.runs_per_pa() / base.runs_per_pa());
        }
        println!();
    };
    if level == "pool" {
        // Players of one talent level only, against major-leaguers: for the foreign players of
        // the Asian leagues, whose level is set by their translated major-league production.
        println!(
            "pool: 30 clubs, talent hitting {:+.2} pitching {:+.2} fielding {:+.2} physical {:+.2} power {:+.2} command {:+.2}",
            talent.hitting, talent.pitching, talent.fielding, talent.physical, talent.power, talent.command
        );
        let clubs: Vec<Club> = (0..30).map(|i| Squad::random(&mut rng, 1000 * (i + 1), &talent).to_engine_with(&tuning.baseline)).collect();
        against_mlb(&clubs, &vs_mlb.unwrap_or(VsMlb { games: games.max(400), pitchers: true }));
        return;
    }
    let target = targets(level).unwrap_or_else(|| panic!("unknown level '{level}': use aaa, aa, high_a, single_a, kbo, kbo2, npb, npb2, mlb or pool"));
    let env = build(level, &talent, &foreign, tuning, seed);
    let (line, speeds) = own_league(&env, games, seed, threads, tuning.style);
    let s = line.rates();
    println!(
        "level {level}: {} clubs, talent hitting {:+.2} pitching {:+.2} fielding {:+.2} physical {:+.2} power {:+.2} command {:+.2}; {games} games",
        env.clubs.len(),
        talent.hitting,
        talent.pitching,
        talent.fielding,
        talent.physical,
        talent.power,
        talent.command
    );
    let (fs, fr, fh) = foreign_slots(level);
    if fs + fr + fh.len() > 0 {
        println!(
            "  foreign players ({fs} starters, {fr} relievers, {} hitters per club): hitting {:+.2} pitching {:+.2} fielding {:+.2} physical {:+.2}",
            fh.len(),
            foreign.hitting,
            foreign.pitching,
            foreign.fielding,
            foreign.physical
        );
    }
    println!("                 R/G     AVG    OBP    SLG     K%    BB%   HR/PA");
    let show = |name: &str, r: &Rates| {
        println!("  {name:<12} {:5.2}   {:.3}  {:.3}  {:.3}  {:5.1}  {:5.1}  {:5.2}", r.r, r.avg, r.obp, r.slg, r.k, r.bb, r.hr);
    };
    show("sim", &s);
    show("2025", &target[0]);
    show("2026", &target[1]);
    println!("  four-seam {:.1} mph, exit velocity {:.1} mph (balls in play)", speeds.ff.0 / speeds.ff.1.max(1.0), speeds.ev.0 / speeds.ev.1.max(1.0));
    if let Some(v) = vs_mlb {
        // The translations describe the league's own players: leave the foreign ones out.
        let domestic: Vec<Club> = if fs + fr + fh.len() > 0 {
            (0..env.clubs.len()).map(|i| Squad::random(&mut rng, 500_000 + 1000 * i as u32, &talent).to_engine_with(&tuning.baseline)).collect()
        } else {
            env.clubs.clone()
        };
        against_mlb(&domestic, &v);
    }
}
