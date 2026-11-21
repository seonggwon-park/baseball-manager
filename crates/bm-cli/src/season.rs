//! `bm season`: generate the world, play a whole season of every league pitch by pitch,
//! and print each league's rates next to its official 2025-2026 averages, the
//! major-league standings and the time it took. With `--seasons N`, play N seasons with
//! an offseason (development, retirements, drafts, rosters) between them.

use crate::tuning::Tuning;
use bm_world::season::{Batting, Play};
use bm_world::{Date, LevelTable, Season, World};
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
/// What `bm season` plays: how many seasons, after how many offseasons without games, and
/// with which level and career files.
pub struct Options<'a> {
    pub until: Option<Date>,
    pub seasons: usize,
    pub warmup: usize,
    /// Offseasons of the new world's past (`careers.json` history if `None`).
    pub history: Option<u32>,
    pub levels_file: Option<&'a std::path::Path>,
    pub careers_file: Option<&'a std::path::Path>,
    pub contracts_file: Option<&'a std::path::Path>,
    pub aging_file: Option<&'a std::path::Path>,
    /// Write each level's rates in the last season played here.
    pub json: Option<&'a std::path::Path>,
    /// Write every player's line of the last season played here (CSV): his level and role,
    /// his value (`ability::value`) and his batting and pitching.
    pub out: Option<&'a std::path::Path>,
    /// Write every game's box-score lines of every season played into this directory
    /// (`batting.csv`, `pitching.csv`; [`GameLogs`]).
    pub game_logs: Option<&'a std::path::Path>,
    /// Play the first season this many times with the same rosters and schedule instead
    /// ([`repeat`]); 0 = play the seasons as usual.
    pub repeat: usize,
    /// Leagues whose games the repeats play (default: the three first-team leagues).
    pub leagues: Option<Vec<String>>,
    /// Write every club's record in every repeat here (CSV).
    pub standings_out: Option<&'a std::path::Path>,
}

/// The first-team leagues whose postseason [`postseason`] knows.
const FIRST_TEAMS: [&str; 3] = ["us-major", "kr-top", "jp-top"];

/// Clubs of the leagues of `world` as `standings` ranks them: first place in each division (or
/// in the league if it has none) and a place in the postseason, by winning percentage with the
/// run difference breaking ties (the leagues break them by head-to-head records first; this is
/// a simplification). The postseason: in MLB, per league, the three division winners and the
/// three best other records (12 clubs since 2022, MLB.com "MLB's postseason format,
/// explained"); in the KBO the top five (the fourth and fifth meet in the wild-card games; KBO
/// press release of 2025-09-28); in NPB the top three of each league (the Climax Series, NPB
/// 2026 notice). Other leagues get `None`.
fn postseason(world: &World, clubs: &[&bm_world::league::Club], standings: &BTreeMap<String, bm_world::Standing>) -> Vec<(bool, Option<bool>)> {
    let key = |i: usize| {
        let s = standings.get(&clubs[i].id).copied().unwrap_or_default();
        (s.pct(), s.runs_for as i64 - s.runs_against as i64)
    };
    let ranked = |mut v: Vec<usize>| {
        v.sort_by(|&a, &b| key(b).partial_cmp(&key(a)).unwrap_or(std::cmp::Ordering::Equal));
        v
    };
    let group_of = |i: usize| clubs[i].division.clone().unwrap_or_else(|| clubs[i].league.clone());
    let mut out = vec![(false, None); clubs.len()];
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for i in 0..clubs.len() {
        groups.entry(group_of(i)).or_default().push(i);
    }
    for v in groups.into_values() {
        out[ranked(v)[0]].0 = true;
    }
    // Sub-leagues of MLB's divisions (American, National).
    let sub = |i: usize| {
        let league = world.league(&clubs[i].league);
        let div = clubs[i].division.as_deref();
        league.and_then(|l| l.divisions.iter().find(|d| Some(d.id.as_str()) == div)).and_then(|d| d.group.clone()).unwrap_or_default()
    };
    let mut fields: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for (i, c) in clubs.iter().enumerate() {
        let field = match c.league.as_str() {
            "us-major" => sub(i),
            "jp-top" => group_of(i),
            "kr-top" => String::new(),
            _ => continue,
        };
        fields.entry((c.league.clone(), field)).or_default().push(i);
    }
    for ((league, _), v) in fields {
        let order = ranked(v);
        let mut seats: Vec<usize> = match league.as_str() {
            "us-major" => {
                let winners: Vec<usize> = order.iter().copied().filter(|&i| out[i].0).collect();
                let others = order.iter().copied().filter(|&i| !out[i].0).take(3);
                winners.into_iter().chain(others).collect()
            }
            "kr-top" => order.iter().copied().take(5).collect(),
            _ => order.iter().copied().take(3).collect(),
        };
        seats.sort_unstable();
        for &i in &order {
            out[i].1 = Some(seats.binary_search(&i).is_ok());
        }
    }
    out
}

/// `bm season --repeat N`: the first season played N times with the same rosters, the same
/// schedule and no offseason, only the games' random numbers changing (each repeat's seed is
/// the season's seed plus its number), for the leagues of `o.leagues`. Prints each club's
/// mean record, the spread of its wins, and how often it finished first and reached the
/// postseason; with `o.standings_out`, every club's record in every repeat.
fn repeat(world: &World, pop: &bm_world::Population, play: &Play, year: i32, seed: u64, o: &Options) {
    use std::io::Write;
    let leagues: Vec<String> = o.leagues.clone().unwrap_or_else(|| FIRST_TEAMS.iter().map(|s| s.to_string()).collect());
    let mut base = Season::new(world, year, seed);
    base.fixtures.retain(|f| leagues.contains(&f.league));
    let clubs: Vec<&bm_world::league::Club> = world.clubs().filter(|c| leagues.contains(&c.league)).collect();
    let mut out = o.standings_out.map(|p| {
        let mut f = std::io::BufWriter::new(std::fs::File::create(p).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
        writeln!(f, "repeat,league,division,club,won,lost,tied,runs_for,runs_against,first,postseason").expect("standings");
        f
    });
    let (mut records, mut firsts, mut posts) = (vec![Vec::new(); clubs.len()], vec![0usize; clubs.len()], vec![0usize; clubs.len()]);
    let t0 = Instant::now();
    for r in 0..o.repeat {
        let mut s = base.clone();
        s.seed = seed.wrapping_add(r as u64);
        s.play_out(world, pop, play);
        let places = postseason(world, &clubs, &s.standings);
        for (i, c) in clubs.iter().enumerate() {
            let st = s.standings.get(&c.id).copied().unwrap_or_default();
            records[i].push(st);
            firsts[i] += usize::from(places[i].0);
            posts[i] += usize::from(places[i].1 == Some(true));
            if let Some(f) = out.as_mut() {
                let post = places[i].1.map_or(String::new(), |p| u8::from(p).to_string());
                writeln!(
                    f,
                    "{r},{},{},{},{},{},{},{},{},{},{post}",
                    c.league,
                    c.division.as_deref().unwrap_or(""),
                    c.id,
                    st.won,
                    st.lost,
                    st.tied,
                    st.runs_for,
                    st.runs_against,
                    u8::from(places[i].0)
                )
                .expect("standings");
            }
        }
    }
    if let Some(mut f) = out {
        f.flush().expect("standings");
    }
    let n = o.repeat as f64;
    println!(
        "{year}: the season played {} times with the same rosters and schedule ({} games each, leagues {}) in {:.1} s",
        o.repeat,
        base.fixtures.len(),
        leagues.join(", "),
        t0.elapsed().as_secs_f64()
    );
    for league in &leagues {
        let mut idx: Vec<usize> = (0..clubs.len()).filter(|&i| &clubs[i].league == league).collect();
        if idx.is_empty() {
            continue;
        }
        let wins = |i: usize| records[i].iter().map(|s| s.won as f64).collect::<Vec<_>>();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
        let sd = |v: &[f64]| {
            let m = mean(v);
            (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() as f64 - 1.0).max(1.0)).sqrt()
        };
        idx.sort_by(|&a, &b| mean(&wins(b)).partial_cmp(&mean(&wins(a))).unwrap_or(std::cmp::Ordering::Equal));
        let has_post = FIRST_TEAMS.contains(&league.as_str());
        println!("\n  {league}: club, mean W-L, SD of wins, 5-95 % of wins, first in division or league %{}", if has_post { ", postseason %" } else { "" });
        for &i in &idx {
            let mut w = wins(i);
            w.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let q = |p: f64| w[((p * (w.len() - 1) as f64).round() as usize).min(w.len() - 1)];
            let lost = mean(&records[i].iter().map(|s| s.lost as f64).collect::<Vec<_>>());
            println!(
                "    {:<28} {:5.1}-{:<5.1} {:4.1}  {:3.0}-{:<3.0}  {:5.1}{}",
                clubs[i].name.en,
                mean(&w),
                lost,
                sd(&w),
                q(0.05),
                q(0.95),
                100.0 * firsts[i] as f64 / n,
                if has_post { format!("  {:5.1}", 100.0 * posts[i] as f64 / n) } else { String::new() }
            );
        }
        // Luck against talent: the spread of a club's wins over the repeats, and the spread
        // of the clubs' mean wins. A coin-flip season of G decided games has SD sqrt(G)/2.
        let luck = (idx.iter().map(|&i| sd(&wins(i)).powi(2)).sum::<f64>() / idx.len() as f64).sqrt();
        let means: Vec<f64> = idx.iter().map(|&i| mean(&wins(i))).collect();
        let decided = mean(&idx.iter().map(|&i| mean(&records[i].iter().map(|s| (s.won + s.lost) as f64).collect::<Vec<_>>())).collect::<Vec<_>>());
        println!(
            "    wins: SD over the repeats {:.2} (coin flips over {:.0} decided games: {:.2}), SD of the clubs' means {:.2}",
            luck,
            decided,
            decided.sqrt() / 2.0,
            sd(&means)
        );
    }
}

/// Fielding positions in the order of the engine's `Team::defense`.
const POSITIONS: [&str; 9] = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

/// A field of a CSV row, quoted when it holds a comma, a quote or a line break.
fn csv(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The box score of every game as CSV (`bm season --game-logs DIR`): `batting.csv` has a row
/// for every player who came to the plate, scored or ran in a game (a defensive substitute
/// with nothing to count has none), `pitching.csv` one for every pitcher who pitched. Both start with the season, date, fixture index, league, club, opponent, whether
/// the club was at home (1) and the final score from the club's side, then the player's id and
/// names. A batter's `order` is his place in the starting batting order (0 for a substitute)
/// and `position` the position he started at (DH for the designated hitter, empty for a
/// substitute, whose positions the box score does not keep).
struct GameLogs {
    batting: std::io::BufWriter<std::fs::File>,
    pitching: std::io::BufWriter<std::fs::File>,
}

impl GameLogs {
    fn create(dir: &std::path::Path) -> Self {
        use std::io::Write;
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let open = |name: &str, header: &str| {
            let path = dir.join(name);
            let mut f = std::io::BufWriter::new(std::fs::File::create(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())));
            writeln!(f, "{header}").unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            f
        };
        const GAME: &str = "season,date,game,league,club,opponent,home,runs,opp_runs,player,name,name_ko";
        GameLogs {
            batting: open("batting.csv", &format!("{GAME},order,position,pa,ab,r,h,2b,3b,hr,rbi,bb,ibb,hbp,so,sf,sh,gidp,sb,cs,roe")),
            pitching: open("pitching.csv", &format!("{GAME},gs,decision,outs,ip,bf,h,r,er,bb,ibb,so,hr,hbp,pitches,strikes,wp")),
        }
    }

    fn write(&mut self, year: i32, pop: &bm_world::Population, g: &bm_world::PlayedGame) {
        use std::io::Write;
        let (f, res) = (g.fixture, g.result);
        for side in 0..2 {
            let (club, opp) = if side == 0 { (&f.away, &f.home) } else { (&f.home, &f.away) };
            let head = format!("{year},{},{},{},{club},{opp},{side},{},{}", f.date, g.index, f.league, res.score[side], res.score[1 - side]);
            let ids = g.players[side];
            let who = |k: usize| {
                let (en, ko) = pop.people.get(&ids[k]).map_or((String::new(), String::new()), |p| (csv(&p.name.en), csv(&p.name.ko)));
                format!("{},{en},{ko}", ids[k])
            };
            let bx = &res.boxes[side];
            // Pinch runners score and steal without a plate appearance.
            for (k, l) in bx.batting.iter().enumerate().filter(|(_, l)| l.pa + l.r + l.sb + l.cs > 0) {
                let order = g.lineups[side].iter().position(|&i| i == k).map_or(0, |o| o + 1);
                let position = g.defense[side].iter().position(|&i| i == k).map_or(if order > 0 { "DH" } else { "" }, |p| POSITIONS[p]);
                writeln!(
                    self.batting,
                    "{head},{},{order},{position},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    who(k),
                    l.pa,
                    l.ab,
                    l.r,
                    l.h,
                    l.doubles,
                    l.triples,
                    l.hr,
                    l.rbi,
                    l.bb,
                    l.ibb,
                    l.hbp,
                    l.so,
                    l.sf,
                    l.sh,
                    l.gidp,
                    l.sb,
                    l.cs,
                    l.roe
                )
                .expect("batting.csv");
            }
            for (k, l) in bx.pitching.iter().enumerate().filter(|(_, l)| l.appeared) {
                let decision = [(res.win, "W"), (res.loss, "L"), (res.save, "S")].iter().find(|(d, _)| *d == Some((side, k))).map_or("", |(_, s)| s);
                writeln!(
                    self.pitching,
                    "{head},{},{},{decision},{},{}.{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    who(k),
                    u8::from(l.started),
                    l.outs,
                    l.outs / 3,
                    l.outs % 3,
                    l.bf,
                    l.h,
                    l.r,
                    l.er,
                    l.bb,
                    l.ibb,
                    l.so,
                    l.hr,
                    l.hbp,
                    l.pitches,
                    l.strikes,
                    l.wp
                )
                .expect("pitching.csv");
            }
        }
    }

    fn finish(mut self) {
        use std::io::Write;
        self.batting.flush().expect("batting.csv");
        self.pitching.flush().expect("pitching.csv");
    }
}

/// Every rostered player's line of `season` as CSV rows (`bm season --out`).
fn write_players(path: &std::path::Path, pop: &bm_world::Population, season: &Season) {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())));
    writeln!(f, "id,level,slot,pitcher,age,value,g,pa,ab,h,doubles,triples,hr,bb,hbp,so,sf,r,gp,gs,outs,bf,ph,pr,er,pbb,pso,phr,club,batting")
        .expect("write --out");
    let jun30 = Date::new(season.year, 6, 30);
    for r in &pop.rosters {
        let s = &r.squad;
        let groups: [(&str, Vec<&bm_world::Player>); 5] = [
            ("lineup", s.hitters.iter().map(|(p, _)| p).collect()),
            ("bench", s.bench.iter().map(|(p, _)| p).collect()),
            ("reserve", r.reserves.iter().map(|(p, _)| p).collect()),
            ("starter", s.starters.iter().collect()),
            ("reliever", s.relievers.iter().collect()),
        ];
        for (slot, players) in groups {
            for p in players {
                let age = pop.people.get(&p.id).map_or(f64::NAN, |x| jun30.days_since(x.born) as f64 / 365.2425);
                let b = season.batting.get(&p.id).cloned().unwrap_or_default();
                let q = season.pitching.get(&p.id).cloned().unwrap_or_default();
                writeln!(
                    f,
                    "{},{},{slot},{},{age:.2},{:.2},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2}",
                    p.id,
                    r.level,
                    u8::from(p.is_pitcher()),
                    bm_world::ability::value(p),
                    b.g,
                    b.pa,
                    b.ab,
                    b.h,
                    b.doubles,
                    b.triples,
                    b.hr,
                    b.bb,
                    b.hbp,
                    b.so,
                    b.sf,
                    b.r,
                    q.g,
                    q.gs,
                    q.outs,
                    q.bf,
                    q.h,
                    q.r,
                    q.er,
                    q.bb,
                    q.so,
                    q.hr,
                    r.club,
                    if p.is_pitcher() { 0.0 } else { bm_world::ability::batting(p) }
                )
                .expect("write --out");
            }
        }
    }
}

/// A level's ages as Baseball-Reference reports them (docs/research/world/levels.md 4):
/// whole years of age on 30 June, batters weighted by AB + G, pitchers by 3 GS + G
/// (Baseball-Reference adds saves, which the season does not keep), foreign players
/// included. Whole years read half a year below exact ages: the MLB Stats API's 2025-2026
/// lines give Double-A batters 24.18 exact against 23.74 on Baseball-Reference, Single-A
/// 21.28 against 20.79.
struct LevelAges {
    /// Mean and standard deviation.
    batters: (f64, f64),
    pitchers: (f64, f64),
    /// Shares of the playing time at 23 or younger and at 34 or older.
    batter_tails: (f64, f64),
    pitcher_tails: (f64, f64),
    /// Mean weight of a player in each role of the roster he started the season on: lineup,
    /// bench, reserve position player, starter, reliever, reserve pitcher.
    roles: [f64; 6],
}

fn level_ages(pop: &bm_world::Population, season: &Season) -> BTreeMap<String, LevelAges> {
    let mid = Date::new(season.year, 6, 30);
    let age = |id: &u32| pop.people.get(id).map(|p| (mid.days_since(p.born) as f64 / 365.2425).floor());
    let mut place: BTreeMap<u32, (&str, usize)> = BTreeMap::new();
    for r in &pop.rosters {
        let l = r.level.as_str();
        for (p, _) in &r.squad.hitters {
            place.insert(p.id, (l, 0));
        }
        for (p, _) in &r.squad.bench {
            place.insert(p.id, (l, 1));
        }
        for (p, _) in &r.reserves {
            place.insert(p.id, (l, if p.is_pitcher() { 5 } else { 2 }));
        }
        for p in &r.squad.starters {
            place.insert(p.id, (l, 3));
        }
        for p in &r.squad.relievers {
            place.insert(p.id, (l, 4));
        }
    }
    // Weighted ages (age, weight) of batters and pitchers, and the weight and count by role.
    #[derive(Default)]
    struct Acc {
        b: Vec<(f64, f64)>,
        p: Vec<(f64, f64)>,
        roles: [(f64, f64); 6],
    }
    let mut acc: BTreeMap<&str, Acc> = BTreeMap::new();
    for (id, b) in &season.batting {
        if let (Some(&(l, role)), Some(a)) = (place.get(id), age(id)) {
            let w = (b.ab + b.g) as f64;
            let e = acc.entry(l).or_default();
            e.b.push((a, w));
            if role < 3 {
                e.roles[role].0 += w;
            }
        }
    }
    for (id, p) in &season.pitching {
        if let (Some(&(l, role)), Some(a)) = (place.get(id), age(id)) {
            let w = (3 * p.gs + p.g) as f64;
            let e = acc.entry(l).or_default();
            e.p.push((a, w));
            if role >= 3 {
                e.roles[role].0 += w;
            }
        }
    }
    for &(l, role) in place.values() {
        acc.entry(l).or_default().roles[role].1 += 1.0;
    }
    let moments = |v: &[(f64, f64)]| {
        let w: f64 = v.iter().map(|x| x.1).sum();
        let m = v.iter().map(|x| x.0 * x.1).sum::<f64>() / w;
        (m, (v.iter().map(|x| x.1 * (x.0 - m).powi(2)).sum::<f64>() / w).sqrt())
    };
    let tails = |v: &[(f64, f64)]| {
        let w: f64 = v.iter().map(|x| x.1).sum();
        let share = |keep: &dyn Fn(f64) -> bool| v.iter().filter(|x| keep(x.0)).map(|x| x.1).sum::<f64>() / w;
        (share(&|a| a <= 23.0), share(&|a| a >= 34.0))
    };
    acc.into_iter()
        .filter(|(_, a)| a.b.iter().any(|x| x.1 > 0.0) && a.p.iter().any(|x| x.1 > 0.0))
        .map(|(l, a)| {
            let ages = LevelAges {
                batters: moments(&a.b),
                pitchers: moments(&a.p),
                batter_tails: tails(&a.b),
                pitcher_tails: tails(&a.p),
                roles: a.roles.map(|(w, n)| w / n.max(1.0)),
            };
            (l.to_string(), ages)
        })
        .collect()
}

/// Each level's batting totals and team games in a season (all its leagues together).
fn level_totals(world: &World, pop: &bm_world::Population, season: &Season) -> BTreeMap<String, (Batting, u32)> {
    let mut level_of_player: BTreeMap<u32, &str> = BTreeMap::new();
    for r in &pop.rosters {
        for p in r.players() {
            level_of_player.insert(p.id, r.level.as_str());
        }
    }
    let mut out: BTreeMap<String, (Batting, u32)> = BTreeMap::new();
    for (id, b) in &season.batting {
        if let Some(l) = level_of_player.get(id) {
            let t = &mut out.entry(l.to_string()).or_default().0;
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
    for (club, s) in &season.standings {
        if let Some(c) = world.club(club) {
            out.entry(bm_world::populate::level_key(world, c).to_string()).or_default().1 += s.won + s.lost + s.tied;
        }
    }
    out
}

pub fn run(tuning: &Tuning, seed: u64, threads: usize, o: &Options) {
    if o.repeat > 0 && (o.seasons > 1 || o.until.is_some() || o.game_logs.is_some() || o.json.is_some() || o.out.is_some()) {
        eprintln!("--repeat plays the first season again; it does not go with --seasons, --until, --game-logs, --json or --out");
        std::process::exit(2);
    }
    let read = |p: &std::path::Path| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let world = World::bundled();
    let levels = o.levels_file.map_or_else(LevelTable::bundled, |p| LevelTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let aging = o
        .aging_file
        .map_or_else(bm_world::AgingTable::bundled, |p| bm_world::AgingTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let careers = o
        .careers_file
        .map_or_else(bm_world::CareerTable::bundled, |p| bm_world::CareerTable::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let people = bm_world::PeopleMaker::new();
    let contracts = o
        .contracts_file
        .map_or_else(bm_world::ContractRules::bundled, |p| bm_world::ContractRules::from_json(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display())));
    let ctx = bm_world::Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people, contracts: &contracts };
    let t0 = Instant::now();
    let history = o.history.unwrap_or(careers.history);
    let mut pop = bm_world::new_world(&ctx, Date::new(2027, 2, 1), history, seed);
    let t_pop = t0.elapsed().as_secs_f64();
    // Offseasons without games before the first season played: the world settles into the
    // steady state of the career model.
    let first = 2027 + o.warmup as i32;
    for y in 2027..first {
        bm_world::offseason(&ctx, &mut pop, y, None, seed);
    }
    if o.warmup > 0 {
        println!("{} offseasons without games: {} players, {} people so far", o.warmup, pop.players().count(), pop.people.len());
    }
    let play = Play { params: tuning.params.clone(), baseline: tuning.baseline.clone(), style: tuning.style, threads, ..Play::default() }.with_balls(&levels);
    if o.repeat > 0 {
        repeat(&world, &pop, &play, first, seed, o);
        return;
    }
    let (until, seasons) = (o.until, o.seasons);
    let seasons = seasons.max(1);
    let mut logs = o.game_logs.map(GameLogs::create);
    for k in 0..seasons {
        let year = first + k as i32;
        let mut season = Season::new(&world, year, seed.wrapping_add(k as u64));
        let t1 = Instant::now();
        {
            let mut on_game = |g: &bm_world::PlayedGame| {
                if let Some(l) = logs.as_mut() {
                    l.write(year, &pop, g);
                }
            };
            match until.filter(|_| seasons == 1) {
                Some(d) => season.play_until_with(&world, &pop, &play, d, &mut on_game),
                None => season.play_out_with(&world, &pop, &play, &mut on_game),
            }
        }
        let secs = t1.elapsed().as_secs_f64();
        println!(
            "\n{year}: {} players{}; {} of {} games played in {:.1} s on {threads} threads ({:.0} games/s)",
            pop.players().count(),
            if k == 0 { format!(" generated in {t_pop:.2} s ({history} offseasons of past)") } else { String::new() },
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
            if let Some(path) = o.json {
                let ages = level_ages(&pop, &season);
                let levels_json: serde_json::Map<String, serde_json::Value> = level_totals(&world, &pop, &season)
                    .into_iter()
                    .filter(|(_, (t, g))| t.pa > 0 && *g > 0)
                    .map(|(k, (t, g))| {
                        let mut v = serde_json::json!({"games": g, "pa": t.pa, "rates": rates(&t, g)});
                        if let Some(a) = ages.get(&k) {
                            v["age_b"] = a.batters.0.into();
                            v["age_b_sd"] = a.batters.1.into();
                            v["age_p"] = a.pitchers.0.into();
                            v["age_p_sd"] = a.pitchers.1.into();
                            v["age_b_young"] = a.batter_tails.0.into();
                            v["age_b_old"] = a.batter_tails.1.into();
                            v["age_p_young"] = a.pitcher_tails.0.into();
                            v["age_p_old"] = a.pitcher_tails.1.into();
                            v["roles"] = serde_json::json!(a.roles);
                        }
                        (k, v)
                    })
                    .collect();
                std::fs::write(path, serde_json::to_string_pretty(&serde_json::json!({"year": year, "levels": levels_json})).unwrap())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
            if let Some(path) = o.out {
                write_players(path, &pop, &season);
            }
            standings(&world, &season);
            if let Some(l) = logs.take() {
                l.finish();
                println!("game logs written to {}", o.game_logs.map_or(String::new(), |d| d.display().to_string()));
            }
        } else {
            let r = bm_world::offseason(&ctx, &mut pop, year, Some(&season.standings), seed);
            println!(
                "  offseason: retired {}, released {}, minor-league free agents {}, drafted {}, signed {}, foreign {} in / {} out, military service {} in ({} with Sangmu) / {} back, {} exempt",
                r.retired, r.released, r.minor_free_agents, r.drafted, r.signed, r.foreign_signed, r.foreign_replaced, r.enlisted, r.sangmu, r.discharged, r.exempt
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postseason_takes_twelve_major_league_clubs_five_kbo_and_three_per_npb_league() {
        let world = World::bundled();
        let clubs: Vec<&bm_world::league::Club> = world.clubs().filter(|c| FIRST_TEAMS.contains(&c.league.as_str())).collect();
        // Records falling with the club's place in the list, a tie on winning percentage broken
        // by run difference.
        let standings: BTreeMap<String, bm_world::Standing> = clubs
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let won = 100 - (i as u32 % 40);
                (c.id.clone(), bm_world::Standing { won, lost: 162 - won, tied: 0, runs_for: 700 + i as u32, runs_against: 700 })
            })
            .collect();
        let places = postseason(&world, &clubs, &standings);
        type Place = (bool, Option<bool>);
        let count = |league: &str, f: &dyn Fn(&Place) -> bool| clubs.iter().zip(&places).filter(|(c, p)| c.league == league && f(p)).count();
        assert_eq!(count("us-major", &|p| p.1 == Some(true)), 12);
        assert_eq!(count("us-major", &|p| p.0), 6);
        assert!(clubs.iter().zip(&places).filter(|(c, _)| c.league == "us-major").all(|(_, p)| !p.0 || p.1 == Some(true)), "every division winner plays on");
        assert_eq!(count("kr-top", &|p| p.1 == Some(true)), 5);
        assert_eq!(count("kr-top", &|p| p.0), 1);
        assert_eq!(count("jp-top", &|p| p.1 == Some(true)), 6);
        assert_eq!(count("jp-top", &|p| p.0), 2);
        // Six per MLB league.
        let al = clubs
            .iter()
            .zip(&places)
            .filter(|(c, p)| c.league == "us-major" && c.division.as_deref().is_some_and(|d| d.contains("american")) && p.1 == Some(true))
            .count();
        assert_eq!(al, 6);
    }
}
