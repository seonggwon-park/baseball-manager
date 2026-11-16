//! A season of every league: the year's fixtures from the schedule generators, then play
//! day by day with standings and every player's season line.
//!
//! Every game is simulated pitch by pitch in the home club's park under its league's
//! rules. Each fixture has its own random stream, so the results do not depend on how
//! many threads play them.
//!
//! Dates: the leagues' `season_2027` entries in `data/leagues/*.json` where they exist;
//! otherwise the defaults below, which are assumptions (no 2027 calendar was published
//! for the KBO, the NPB and their second leagues when the data were collected). The
//! second leagues of Korea and Japan use the minor-league generator (six-game series,
//! Mondays off) until they get schedules of their own.

use crate::calendar::{Date, Weekday};
use crate::convert::Baseline;
use crate::league::{Club as WorldClub, League, Level, World};
use crate::populate::{level_key, Population};
use crate::schedule;
use bm_engine::{play_game_styled, Club, ManagerStyle, MatchEnv, ModelParams, NoObserver, Rng, Rules};
use bm_physics::{Atmosphere, Wind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One game of the season.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fixture {
    pub date: Date,
    pub league: String,
    pub home: String,
    pub away: String,
}

/// Major-league interleague "prime" rivals: every club of the first conference with its
/// rival in the second, by city (MLB's natural rivals of 2023-2027).
const RIVALS: [(&str, &str); 15] = [
    ("baltimore-schooners", "washington-statesmen"),
    ("boston-lanterns", "atlanta-firebirds"),
    ("new-york-empires", "new-york-ferrymen"),
    ("st-petersburg-herons", "miami-flamingos"),
    ("toronto-voyageurs", "philadelphia-liberties"),
    ("chicago-sentinels", "chicago-gales"),
    ("cleveland-forgers", "cincinnati-steamers"),
    ("detroit-motormen", "pittsburgh-ironclads"),
    ("kansas-city-plainsmen", "st-louis-archers"),
    ("minneapolis-lumberjacks", "milwaukee-harvesters"),
    ("sacramento-prospectors", "san-francisco-fogcutters"),
    ("houston-wildcatters", "denver-summits"),
    ("anaheim-surfriders", "los-angeles-palms"),
    ("seattle-evergreens", "san-diego-navigators"),
    ("arlington-wranglers", "phoenix-scorpions"),
];

fn parse(s: &Option<String>) -> Option<Date> {
    s.as_deref().and_then(Date::parse)
}

/// The first `weekday` on or after `d`.
fn on_or_after(d: Date, weekday: Weekday) -> Date {
    let mut x = d;
    while x.weekday() != weekday {
        x = x.add_days(1);
    }
    x
}

/// Clubs of `league` in the order of its divisions (data order inside a division).
fn clubs_by_division<'a>(world: &'a World, league: &League) -> Vec<&'a WorldClub> {
    let mut clubs: Vec<&WorldClub> = world.clubs().filter(|c| c.league == league.id).collect();
    let rank = |c: &WorldClub| league.divisions.iter().position(|d| Some(&d.id) == c.division.as_ref()).unwrap_or(usize::MAX);
    clubs.sort_by_key(|c| rank(c));
    clubs
}

/// Games per club the schedule gives `league`. The minor-league generator plays an opening
/// three-game series, six-game series and one more three-game series, so the second
/// leagues of Korea (121 games) and Japan (140) get the nearest smaller count it can make
/// (120 and 138) until they have schedules of their own.
pub fn generated_games(league: &League) -> u32 {
    match league.level {
        Level::Second => 6 + 6 * ((league.games.max(6) - 6) / 6),
        _ => league.games,
    }
}

/// Every fixture of `year` in every league of the world, sorted by date.
pub fn fixtures(world: &World, year: i32, seed: u64) -> Vec<Fixture> {
    let mut out = Vec::new();
    let mut rng = Rng::new(seed ^ 0x5EA5_0000 ^ year as u64);
    let d = |m: u32, day: u32| Date::new(year, m, day);
    for league in world.leagues() {
        let clubs = clubs_by_division(world, league);
        if clubs.len() < 2 {
            continue;
        }
        let dates = league.season_2027.clone().unwrap_or_default();
        let country = clubs[0].country.as_str();
        let games: Vec<schedule::Game> = match (league.level, country) {
            (Level::Top, "US") => {
                let opening = on_or_after(parse(&dates.opening).unwrap_or(d(3, 25)), Weekday::Thursday);
                let all_star_monday = parse(&dates.all_star).unwrap_or(d(7, 13)).add_days(-1);
                let index = |id: &str| clubs.iter().position(|c| c.id == id);
                let mut rival: [usize; 15] = std::array::from_fn(|i| i);
                for (a, b) in RIVALS {
                    if let (Some(i), Some(j)) = (index(a), index(b)) {
                        if i < 15 && j >= 15 {
                            rival[i] = j - 15;
                        }
                    }
                }
                schedule::mlb(opening, all_star_monday, &rival, year % 2 == 1, &mut rng)
            }
            (Level::Top, "KR") => {
                let opening = on_or_after(parse(&dates.opening).unwrap_or(d(3, 26)), Weekday::Friday);
                // Five clubs host 73 games and five 71, alternating by year.
                let more_home: Vec<bool> = (0..clubs.len()).map(|i| (i + year as usize) % 2 == 0).collect();
                schedule::kbo(clubs.len(), opening, (d(7, 9), d(7, 14)), &more_home, &mut rng)
            }
            (Level::Top, _) => {
                let opening = parse(&dates.opening).unwrap_or(d(3, 26));
                let inter = on_or_after(d(6, 1), Weekday::Tuesday);
                let more_home: [bool; 12] = std::array::from_fn(|i| (i + year as usize) % 2 == 0);
                schedule::npb(opening, inter, (d(7, 19), d(7, 22)), &more_home, year % 2 == 1, &mut rng)
            }
            _ => {
                let opening = on_or_after(parse(&dates.opening).unwrap_or(d(4, 2)), Weekday::Friday);
                let division: Vec<usize> =
                    clubs.iter().map(|c| league.divisions.iter().position(|x| Some(&x.id) == c.division.as_ref()).unwrap_or(0)).collect();
                schedule::milb(&division, opening, generated_games(league), (d(7, 12), d(7, 15)), 0.6, &mut rng)
            }
        };
        out.extend(games.into_iter().map(|g| Fixture {
            date: g.date,
            league: league.id.clone(),
            home: clubs[g.home].id.clone(),
            away: clubs[g.away].id.clone(),
        }));
    }
    out.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.league.cmp(&b.league)).then_with(|| a.home.cmp(&b.home)));
    out
}

/// Won-lost record.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Standing {
    pub won: u32,
    pub lost: u32,
    pub tied: u32,
    pub runs_for: u32,
    pub runs_against: u32,
}

impl Standing {
    pub fn pct(&self) -> f64 {
        let n = self.won + self.lost;
        if n == 0 {
            0.0
        } else {
            self.won as f64 / n as f64
        }
    }
}

/// A batter's season.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Batting {
    pub g: u32,
    pub pa: u32,
    pub ab: u32,
    pub h: u32,
    pub doubles: u32,
    pub triples: u32,
    pub hr: u32,
    pub bb: u32,
    pub ibb: u32,
    pub hbp: u32,
    pub so: u32,
    pub sf: u32,
    pub sh: u32,
    pub r: u32,
    pub rbi: u32,
    pub sb: u32,
    pub cs: u32,
}

impl Batting {
    fn add(&mut self, l: &bm_engine::BatLine) {
        self.g += 1;
        self.pa += l.pa as u32;
        self.ab += l.ab as u32;
        self.h += l.h as u32;
        self.doubles += l.doubles as u32;
        self.triples += l.triples as u32;
        self.hr += l.hr as u32;
        self.bb += l.bb as u32;
        self.ibb += l.ibb as u32;
        self.hbp += l.hbp as u32;
        self.so += l.so as u32;
        self.sf += l.sf as u32;
        self.sh += l.sh as u32;
        self.r += l.r as u32;
        self.rbi += l.rbi as u32;
        self.sb += l.sb as u32;
        self.cs += l.cs as u32;
    }

    pub fn avg(&self) -> f64 {
        self.h as f64 / (self.ab as f64).max(1.0)
    }

    pub fn obp(&self) -> f64 {
        (self.h + self.bb + self.hbp) as f64 / ((self.ab + self.bb + self.hbp + self.sf) as f64).max(1.0)
    }

    pub fn slg(&self) -> f64 {
        (self.h + self.doubles + 2 * self.triples + 3 * self.hr) as f64 / (self.ab as f64).max(1.0)
    }
}

/// A pitcher's season.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Pitching {
    pub g: u32,
    pub gs: u32,
    pub outs: u32,
    pub bf: u32,
    pub h: u32,
    pub r: u32,
    pub er: u32,
    pub bb: u32,
    pub so: u32,
    pub hr: u32,
    pub pitches: u32,
}

impl Pitching {
    fn add(&mut self, l: &bm_engine::PitchLine) {
        self.g += 1;
        self.gs += l.started as u32;
        self.outs += l.outs as u32;
        self.bf += l.bf as u32;
        self.h += l.h as u32;
        self.r += l.r as u32;
        self.er += l.er as u32;
        self.bb += l.bb as u32;
        self.so += l.so as u32;
        self.hr += l.hr as u32;
        self.pitches += l.pitches as u32;
    }

    pub fn era(&self) -> f64 {
        27.0 * self.er as f64 / (self.outs as f64).max(1.0)
    }
}

/// How the season's games are played: model constants, the attribute baseline and the
/// managers' style.
#[derive(Clone, Debug)]
pub struct Play {
    pub params: ModelParams,
    pub baseline: Baseline,
    pub style: ManagerStyle,
    pub threads: usize,
}

impl Default for Play {
    fn default() -> Self {
        Play {
            params: ModelParams::default(),
            baseline: Baseline::default(),
            style: ManagerStyle::mlb(),
            threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
        }
    }
}

/// A season in progress.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Season {
    pub year: i32,
    pub seed: u64,
    pub fixtures: Vec<Fixture>,
    /// Fixtures already played (a prefix of `fixtures`).
    pub played: usize,
    pub standings: BTreeMap<String, Standing>,
    pub batting: BTreeMap<u32, Batting>,
    pub pitching: BTreeMap<u32, Pitching>,
    /// Games each club has started, to turn its five-man rotation.
    pub starts: BTreeMap<String, usize>,
}

/// The rules each league plays under.
fn rules_for(level: Level, country: &str) -> Rules {
    match (level, country) {
        (_, "KR") => Rules::kbo_2025(),
        // Both Japanese first-team leagues use the designated hitter from 2027.
        (_, "JP") => Rules::npb_2025(true),
        _ => Rules::mlb_2025(),
    }
}

impl Season {
    pub fn new(world: &World, year: i32, seed: u64) -> Season {
        Season {
            year,
            seed,
            fixtures: fixtures(world, year, seed),
            played: 0,
            standings: BTreeMap::new(),
            batting: BTreeMap::new(),
            pitching: BTreeMap::new(),
            starts: BTreeMap::new(),
        }
    }

    pub fn finished(&self) -> bool {
        self.played >= self.fixtures.len()
    }

    /// Date of the next game to be played.
    pub fn next_date(&self) -> Option<Date> {
        self.fixtures.get(self.played).map(|f| f.date)
    }

    /// Play every fixture up to and including `until`.
    pub fn play_until(&mut self, world: &World, population: &Population, play: &Play, until: Date) {
        let end = self.played + self.fixtures[self.played..].iter().take_while(|f| f.date <= until).count();
        if end == self.played {
            return;
        }
        // The engine's clubs and the venues, built once for this stretch.
        let clubs: BTreeMap<&str, Club> = population.rosters.iter().map(|r| (r.club.as_str(), r.squad.to_engine_with(&play.baseline))).collect();
        let mut envs: BTreeMap<&str, MatchEnv> = BTreeMap::new();
        for c in world.clubs() {
            let level = world.level_of(c);
            let atmosphere = Atmosphere::new(24.0, 760.0, 55.0, c.park.elevation());
            envs.insert(c.id.as_str(), MatchEnv::new(&atmosphere, Wind::CALM, c.park.field(), rules_for(level, &c.country), play.params.clone()));
        }
        let _ = level_key; // the level of each club is in the population's rosters
        let jobs: Vec<(usize, usize, usize)> = (self.played..end)
            .map(|i| {
                let f = &self.fixtures[i];
                let away = *self.starts.entry(f.away.clone()).and_modify(|n| *n += 1).or_insert(0);
                let home = *self.starts.entry(f.home.clone()).and_modify(|n| *n += 1).or_insert(0);
                (i, away, home)
            })
            .collect();
        let threads = play.threads.max(1);
        let chunk = jobs.len().div_ceil(threads);
        let fixtures = &self.fixtures;
        let seed = self.seed;
        let results: Vec<(usize, bm_engine::GameResult, Vec<u32>, Vec<u32>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = jobs
                .chunks(chunk.max(1))
                .map(|part| {
                    let (clubs, envs) = (&clubs, &envs);
                    scope.spawn(move || {
                        part.iter()
                            .filter_map(|&(i, away_start, home_start)| {
                                let f = &fixtures[i];
                                let (a, h) = (clubs.get(f.away.as_str())?, clubs.get(f.home.as_str())?);
                                let env = envs.get(f.home.as_str())?;
                                let dh = env.rules.dh;
                                let (away, home) = (a.team_for_game(away_start, dh), h.team_for_game(home_start, dh));
                                let mut rng = Rng::new(seed ^ 0xF1C7_0000_0000 ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
                                let res = play_game_styled(env, &away, &home, [play.style; 2], &mut rng, &mut NoObserver);
                                Some((i, res, away.players.iter().map(|p| p.id).collect(), home.players.iter().map(|p| p.id).collect()))
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles.into_iter().flat_map(|h| h.join().expect("season thread panicked")).collect()
        });
        for (i, res, away_ids, home_ids) in results {
            let f = &self.fixtures[i];
            let (ra, rh) = (res.score[0], res.score[1]);
            for (club, us, them) in [(&f.away, ra, rh), (&f.home, rh, ra)] {
                let s = self.standings.entry(club.clone()).or_default();
                s.runs_for += us;
                s.runs_against += them;
                match us.cmp(&them) {
                    std::cmp::Ordering::Greater => s.won += 1,
                    std::cmp::Ordering::Less => s.lost += 1,
                    std::cmp::Ordering::Equal => s.tied += 1,
                }
            }
            for (bx, ids) in [(&res.boxes[0], &away_ids), (&res.boxes[1], &home_ids)] {
                for (k, l) in bx.batting.iter().enumerate() {
                    if l.pa > 0 {
                        self.batting.entry(ids[k]).or_default().add(l);
                    }
                }
                for (k, l) in bx.pitching.iter().enumerate() {
                    if l.appeared {
                        self.pitching.entry(ids[k]).or_default().add(l);
                    }
                }
            }
        }
        self.played = end;
    }

    /// Play the rest of the season.
    pub fn play_out(&mut self, world: &World, population: &Population, play: &Play) {
        if let Some(last) = self.fixtures.last().map(|f| f.date) {
            self.play_until(world, population, play, last);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::populate::{populate, LevelTable};

    #[test]
    fn every_league_has_its_full_schedule() {
        let world = World::bundled();
        let fx = fixtures(&world, 2027, 3);
        let mut games: BTreeMap<&str, usize> = BTreeMap::new();
        for f in &fx {
            *games.entry(f.home.as_str()).or_default() += 1;
            *games.entry(f.away.as_str()).or_default() += 1;
        }
        for c in world.clubs() {
            let league = world.league(&c.league).unwrap();
            assert_eq!(games.get(c.id.as_str()).copied().unwrap_or(0), generated_games(league) as usize, "{} in {}", c.id, league.id);
        }
        // In date order, and every game inside one league.
        assert!(fx.windows(2).all(|w| w[0].date <= w[1].date));
        for f in &fx {
            assert_eq!(world.club(&f.home).unwrap().league, f.league);
            assert_eq!(world.club(&f.away).unwrap().league, f.league);
        }
        // The prime rivals meet six times.
        let nyc = fx
            .iter()
            .filter(|f| (f.home == "new-york-empires" && f.away == "new-york-ferrymen") || (f.away == "new-york-empires" && f.home == "new-york-ferrymen"))
            .count();
        assert_eq!(nyc, 6);
    }

    #[test]
    fn a_week_of_play_fills_standings_and_season_lines() {
        let world = World::bundled();
        let pop = populate(&world, &LevelTable::bundled(), Date::new(2027, 2, 1), 5);
        let mut season = Season::new(&world, 2027, 5);
        let first = season.next_date().unwrap();
        let play = Play { threads: 4, ..Play::default() };
        season.play_until(&world, &pop, &play, first.add_days(6));
        assert!(season.played > 0 && !season.finished());
        let games: u32 = season.standings.values().map(|s| s.won + s.lost + s.tied).sum();
        assert_eq!(games as usize, 2 * season.played);
        let runs: u32 = season.standings.values().map(|s| s.runs_for).sum();
        let ra: u32 = season.standings.values().map(|s| s.runs_against).sum();
        assert_eq!(runs, ra);
        // Every plate appearance belongs to a player of the world.
        assert!(season.batting.keys().all(|id| pop.people.contains_key(id)));
        assert!(season.pitching.values().any(|p| p.gs > 0));
        // The same seed plays the same week.
        let mut again = Season::new(&world, 2027, 5);
        again.play_until(&world, &pop, &Play { threads: 2, ..Play::default() }, first.add_days(6));
        assert_eq!(again.standings, season.standings);
        assert_eq!(again.batting, season.batting);
    }
}
