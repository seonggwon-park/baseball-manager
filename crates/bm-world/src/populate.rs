//! The world's players at the start of a game: every club's roster at the talent of its
//! level, with names, dates and places of birth, foreign players where a league limits
//! them, and the two development squads (rookie complex and Dominican academy) of every
//! major-league organisation, which play no official schedule.
//!
//! What each level looks like comes from the editable file `data/people/levels.json`:
//! talent offsets fitted with `bm level` (docs/data-files.md), ages from Baseball-Reference
//! league averages, roster sizes from the 2026 rules (`docs/research/world/`).

use crate::calendar::Date;
use crate::career::{record_season, Career, CareerTable};
use crate::develop::{AgingTable, Potential};
use crate::generate::{pitcher, position_player, Squad, Talent};
use crate::league::{Club, Level, World};
use crate::people::{AgeSpread, PeopleMaker, Person};
use crate::player::{Player, Position, Role};
use bm_engine::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const LEVELS: &str = include_str!("../../../data/people/levels.json");

/// Talent offsets as written in the data file: hitting, pitching, fielding, physical,
/// power, command and, if given, velocity (see [`Talent`]).
pub(crate) fn talent(v: &[f32]) -> Talent {
    let at = |i: usize| v.get(i).copied().unwrap_or(0.0);
    Talent {
        hitting: at(0),
        pitching: at(1),
        fielding: at(2),
        physical: at(3),
        power: at(4),
        command: at(5),
        velocity: at(6),
        discipline: at(7),
        spread: 1.0,
        tradeoff: 0.0,
    }
}

/// Foreign players of a club in a league that limits them.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ForeignSlots {
    pub talent: Vec<f32>,
    /// Birthplace group of the regular foreign players and, for the KBO's Asia quota, of
    /// the quota player.
    pub birthplaces: String,
    #[serde(default)]
    pub asia_quota: Option<String>,
    /// Starting pitchers, relievers and lineup slots (0 = catcher ... 8 = designated hitter)
    /// they fill; the Asia-quota player is one more reliever.
    pub starters: usize,
    pub relievers: usize,
    pub hitters: Vec<usize>,
    /// Foreign players among the reserves (farm and development players of NPB clubs).
    #[serde(default)]
    pub reserves: usize,
}

/// One level of play as the data file describes it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LevelProfile {
    pub talent: Vec<f32>,
    pub batters: AgeSpread,
    pub pitchers: AgeSpread,
    /// Birthplace group of the club's own players (`data/people/birthplaces.json`).
    pub birthplaces: String,
    /// Players on the roster; those beyond the 26 of a game-day squad are reserves.
    pub roster: usize,
    #[serde(default)]
    pub foreign: Option<ForeignSlots>,
    /// Factor on the drag of a batted ball in the level's own games: Triple-A and the
    /// majors use the major-league ball, the lower minors a ball of their own.
    #[serde(default = "one")]
    pub ball_drag: f64,
}

fn one() -> f64 {
    1.0
}

impl LevelProfile {
    pub fn talent(&self) -> Talent {
        talent(&self.talent)
    }
}

#[derive(Deserialize)]
struct LevelFile {
    levels: BTreeMap<String, LevelProfile>,
}

/// Every level of the data file, by key: `mlb`, `aaa`, `aa`, `high_a`, `single_a`,
/// `complex`, `academy`, `kbo`, `kbo2`, `npb`, `npb2`.
pub struct LevelTable {
    pub levels: BTreeMap<String, LevelProfile>,
}

impl LevelTable {
    pub fn bundled() -> LevelTable {
        LevelTable::from_json(LEVELS).unwrap_or_else(|e| panic!("data/people/levels.json: {e}"))
    }

    pub fn from_json(text: &str) -> Result<LevelTable, serde_json::Error> {
        let f: LevelFile = serde_json::from_str(text)?;
        Ok(LevelTable { levels: f.levels })
    }

    pub fn get(&self, key: &str) -> &LevelProfile {
        self.levels.get(key).unwrap_or_else(|| panic!("no level '{key}' in data/people/levels.json"))
    }
}

/// The first-team level whose balance of skills the players of `level` grow into.
pub fn peak_level(level: &str) -> &'static str {
    match level {
        "kbo" | "kbo2" => "kbo",
        "npb" | "npb2" => "npb",
        _ => "mlb",
    }
}

/// Key of the level table that a club of the world plays at.
pub fn level_key(world: &World, club: &Club) -> &'static str {
    match (world.level_of(club), club.country.as_str()) {
        (Level::Top, "KR") => "kbo",
        (Level::Top, "JP") => "npb",
        (Level::Top, _) => "mlb",
        (Level::Aaa, _) => "aaa",
        (Level::Aa, _) => "aa",
        (Level::HighA, _) => "high_a",
        (Level::SingleA, _) => "single_a",
        (Level::Second, "KR") => "kbo2",
        (Level::Second, _) => "npb2",
    }
}

/// A club's players: the game-day squad and the reserves beyond it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Roster {
    /// Club id, or `<organisation id>/complex` and `<organisation id>/academy` for the
    /// development squads.
    pub club: String,
    pub level: String,
    pub squad: Squad,
    pub reserves: Vec<(Player, Option<Position>)>,
}

impl Roster {
    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.squad
            .hitters
            .iter()
            .chain(&self.squad.bench)
            .chain(&self.reserves)
            .map(|(p, _)| p)
            .chain(self.squad.starters.iter())
            .chain(self.squad.relievers.iter())
    }
}

/// Everyone in the world on the day the game starts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Population {
    pub date: Date,
    pub people: BTreeMap<u32, Person>,
    pub rosters: Vec<Roster>,
    /// Every player's ceiling (`develop.rs`).
    #[serde(default)]
    pub potentials: BTreeMap<u32, Potential>,
    /// Foreign players of KBO and NPB clubs: the birthplace group they were signed from
    /// (`kbo_foreign`, `kbo_asia`, `npb_foreign`).
    #[serde(default)]
    pub foreign: BTreeMap<u32, String>,
    /// Everyone who has played in the world, retired players included (`career.rs`).
    #[serde(default)]
    pub careers: BTreeMap<u32, Career>,
    /// Korean players on active military duty, off every roster, with the organisation that
    /// holds their rights (`career.rs`, military service).
    #[serde(default)]
    pub away: BTreeMap<u32, Away>,
    /// Every rostered player's contract, and those of players on military duty
    /// (`contract.rs`).
    #[serde(default)]
    pub contracts: BTreeMap<u32, crate::contract::Contract>,
    /// Compensation picks and forfeited picks of coming drafts (`contract::DraftChange`).
    #[serde(default)]
    pub draft_changes: Vec<crate::contract::DraftChange>,
}

/// A player away from the game for a while, and the organisation he comes back to.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Away {
    pub org: String,
    pub player: Player,
}

impl Population {
    pub fn roster(&self, club: &str) -> Option<&Roster> {
        self.rosters.iter().find(|r| r.club == club)
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.rosters.iter().flat_map(|r| r.players())
    }
}

/// Hands out player ids and makes the people behind them.
struct Maker<'a> {
    people: &'a PeopleMaker,
    date: Date,
    next: u32,
    out: BTreeMap<u32, Person>,
}

impl Maker<'_> {
    /// A block of ids for a squad (Squad::random uses offsets below 60).
    fn block(&mut self) -> u32 {
        let id = self.next;
        self.next += 100;
        id
    }

    fn person(&mut self, id: u32, is_pitcher: bool, birthplaces: &str, profile: &LevelProfile, rng: &mut Rng) {
        let age = if is_pitcher { &profile.pitchers } else { &profile.batters };
        let person = self.people.make(id, birthplaces, age, self.date, rng);
        self.out.insert(id, person);
    }
}

fn reserve_positions(n: usize) -> Vec<Option<Position>> {
    use Position::*;
    // Alternate pitchers and position players, the latter round the field.
    let fielders = [Catcher, Short, Center, Second, Third, Left, First, Right];
    (0..n).map(|i| if i % 2 == 0 { Some(Pitcher) } else { Some(fielders[(i / 2) % fielders.len()]) }).collect()
}

fn roster(club: String, key: &str, profile: &LevelProfile, maker: &mut Maker, rng: &mut Rng) -> (Roster, BTreeMap<u32, String>) {
    let t = profile.talent();
    let base = maker.block();
    let mut squad = Squad::random(rng, base, &t);
    let extra = profile.roster.saturating_sub(26);
    let mut reserves = Vec::with_capacity(extra);
    for (i, pos) in reserve_positions(extra).into_iter().enumerate() {
        let id = maker.next + i as u32;
        reserves.push(match pos {
            Some(Position::Pitcher) => (pitcher(rng, id, Role::Reliever, &t), Some(Position::Pitcher)),
            p => (position_player(rng, id, p, &t), p),
        });
    }
    maker.next += extra as u32 + 1;
    // Who is foreign: (player id, birthplace group).
    let mut foreign_ids: Vec<(u32, String)> = Vec::new();
    if let Some(f) = &profile.foreign {
        let fbase = maker.block();
        let pool = Squad::random(rng, fbase, &talent(&f.talent));
        squad.import_from(&pool, f.starters, f.relievers, &f.hitters);
        foreign_ids.extend(pool.starters.iter().take(f.starters).map(|p| (p.id, f.birthplaces.clone())));
        foreign_ids.extend(pool.relievers.iter().take(f.relievers).map(|p| (p.id, f.birthplaces.clone())));
        foreign_ids.extend(f.hitters.iter().filter_map(|&i| pool.hitters.get(i)).map(|(p, _)| (p.id, f.birthplaces.clone())));
        if let Some(asia) = &f.asia_quota {
            // The quota player takes the next relief role.
            let k = f.relievers.min(squad.relievers.len().saturating_sub(1));
            if let Some(p) = pool.relievers.get(k) {
                squad.relievers[k] = p.clone();
                foreign_ids.push((p.id, asia.clone()));
            }
        }
        for (i, r) in reserves.iter_mut().take(f.reserves).enumerate() {
            let src = if r.1 == Some(Position::Pitcher) {
                pool.relievers.get(4 + i).cloned()
            } else {
                pool.bench.get(i % pool.bench.len().max(1)).map(|(p, _)| p.clone())
            };
            if let Some(mut p) = src {
                // Keep the reserve's slot and the foreign player's ability.
                p.id = r.0.id;
                if r.1 != Some(Position::Pitcher) {
                    p.position = r.1.unwrap_or(p.position);
                }
                r.0 = p;
                foreign_ids.push((r.0.id, f.birthplaces.clone()));
            }
        }
    }
    let r = Roster { club, level: key.to_string(), squad, reserves };
    let foreign: BTreeMap<u32, String> = foreign_ids.into_iter().collect();
    let ids: Vec<(u32, bool)> = r.players().map(|p| (p.id, p.is_pitcher())).collect();
    for (id, is_pitcher) in ids {
        let group = foreign.get(&id).map_or(profile.birthplaces.as_str(), String::as_str);
        maker.person(id, is_pitcher, group, profile, rng);
    }
    (r, foreign)
}

/// Fill every club of `world` and the development squads of the major-league organisations.
pub fn populate(world: &World, levels: &LevelTable, date: Date, seed: u64) -> Population {
    populate_with(world, levels, &AgingTable::bundled(), &CareerTable::bundled(), date, seed)
}

/// [`populate`] with other aging curves and career rules than the bundled ones.
pub fn populate_with(world: &World, levels: &LevelTable, aging: &AgingTable, careers: &CareerTable, date: Date, seed: u64) -> Population {
    let people = PeopleMaker::new();
    let mut maker = Maker { people: &people, date, next: 1, out: BTreeMap::new() };
    let mut rng = Rng::new(seed ^ 0x9E0_91E);
    let mut rosters = Vec::new();
    let mut foreign = BTreeMap::new();
    for club in world.clubs() {
        let key = level_key(world, club);
        let (r, f) = roster(club.id.clone(), key, levels.get(key), &mut maker, &mut rng);
        rosters.push(r);
        foreign.extend(f);
        if key == "mlb" {
            for dev in ["complex", "academy"] {
                rosters.push(roster(format!("{}/{dev}", club.id), dev, levels.get(dev), &mut maker, &mut rng).0);
            }
        }
    }
    // Ceilings: the mean remaining growth at the player's age, times a personal factor
    // (log-normal, median 1) that makes some players prospects and others organisation
    // depth, with the skill gaps of the level closing by the peak. A separate stream, so
    // that the rosters stay what they were. The growth ahead is the one the career rules
    // give a new player of the same kind (`careers.json`: the class potential of his
    // origin, times the pitchers' factor): with a factor of 1 the first world's young
    // players were promised up to four times the growth of those who come after them, and
    // that generation took over the majors and rose a standard deviation above them.
    let closure = careers.balance;
    let class_potential = |name: &str| careers.classes.get(name).map_or(1.0, |c| c.potential);
    let mut prng = Rng::new(seed ^ 0x9E0_7E17);
    let mut potentials = BTreeMap::new();
    for r in &rosters {
        let profile = levels.get(&r.level);
        for p in r.players() {
            let person = &maker.out[&p.id];
            let age = date.days_since(person.born) as f64 / 365.2425;
            let origin = match peak_level(&r.level) {
                "kbo" => class_potential("kbo_high_school"),
                "npb" => class_potential("npb_high_school"),
                _ if ["US", "CA", "PR"].contains(&person.birthplace.as_str()) => class_potential("us_college"),
                _ => class_potential("international"),
            };
            let factor = origin * if p.is_pitcher() { careers.pitcher_potential } else { 1.0 };
            let scale = (careers.prospect_spread * prng.gauss(0.0, 1.0)).exp() as f32 * factor;
            let mut pot = Potential::draw(p, age, scale, aging, &mut prng);
            // A player grows into the balance of his league's first team; foreign players
            // keep theirs.
            let is_foreign = foreign.contains_key(&p.id);
            let (t, peak) = match (&profile.foreign, is_foreign) {
                (Some(f), true) => (talent(&f.talent), talent(&f.talent)),
                _ => (profile.talent(), levels.get(peak_level(&r.level)).talent()),
            };
            // The league styles shape new amateurs (`Style::shape`): the first world's
            // players already have their league's balance.
            pot.balance(p, &t, &peak, &closure);
            potentials.insert(p.id, pot);
        }
    }
    let mut pop = Population {
        date,
        people: maker.out,
        rosters,
        potentials,
        foreign,
        careers: BTreeMap::new(),
        away: BTreeMap::new(),
        contracts: BTreeMap::new(),
        draft_changes: Vec::new(),
    };
    record_season(world, &mut pop, date.year());
    crate::contract::initial(world, &mut pop, &crate::contract::ContractRules::bundled(), seed);
    pop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_world_is_filled_level_by_level() {
        let world = World::bundled();
        let levels = LevelTable::bundled();
        let date = Date::new(2027, 2, 1);
        let pop = populate(&world, &levels, date, 7);
        // Every club and two development squads per major-league organisation.
        assert_eq!(pop.rosters.len(), world.clubs().count() + 2 * 30);
        // Unique ids, one person each.
        let ids: Vec<u32> = pop.players().map(|p| p.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "duplicate player ids");
        assert_eq!(pop.people.len(), ids.len());
        let total = ids.len();
        assert!((7000..9500).contains(&total), "{total} players");
        // Roster sizes follow the level table.
        for r in &pop.rosters {
            assert_eq!(r.players().count(), levels.get(&r.level).roster.max(26), "{}", r.club);
        }
        // Korean first teams: three foreign players and one Asia-quota player each.
        let kbo = pop.roster(world.clubs().find(|c| c.league == "kr-top").unwrap().id.as_str()).unwrap();
        let foreign = kbo.players().filter(|p| pop.people[&p.id].birthplace != "KR").count();
        assert_eq!(foreign, 4, "foreign players on a KBO club");
        // The academies are Dominican and Venezuelan; the majors are mostly American.
        let share = |level: &str, country: &str| {
            let (mut n, mut k) = (0.0f64, 0.0f64);
            for r in pop.rosters.iter().filter(|r| r.level == level) {
                for p in r.players() {
                    n += 1.0;
                    if pop.people[&p.id].birthplace == country {
                        k += 1.0;
                    }
                }
            }
            k / n
        };
        assert!(share("academy", "DO") + share("academy", "VE") > 0.75);
        assert!((share("mlb", "US") - 0.737).abs() < 0.05, "{}", share("mlb", "US"));
        // Single-A players are younger than major-leaguers.
        let mean_age = |level: &str| {
            let ages: Vec<f64> = pop
                .rosters
                .iter()
                .filter(|r| r.level == level)
                .flat_map(|r| r.players())
                .map(|p| date.days_since(pop.people[&p.id].born) as f64 / 365.2425)
                .collect();
            ages.iter().sum::<f64>() / ages.len() as f64
        };
        assert!(mean_age("single_a") + 5.0 < mean_age("mlb"), "{} vs {}", mean_age("single_a"), mean_age("mlb"));
        // The same seed gives the same world.
        let again = populate(&world, &levels, date, 7);
        assert_eq!(again.people.len(), pop.people.len());
        assert_eq!(again.people.values().next(), pop.people.values().next());
    }
}
