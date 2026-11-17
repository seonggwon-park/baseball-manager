//! Saved games in an SQLite file.
//!
//! The tables are meant to be readable with any SQLite tool:
//!
//! * `meta (key, value)`: format version, game date, world seed.
//! * `people (id, name_ko, name_en, born, birthplace)`: everyone in the world.
//! * `players (id, data)`: each player's attributes and traits as JSON (`bm_world::Player`),
//!   so that the attribute set can grow without a schema change.
//! * `rosters (club, level, slot, ord, player, position)`: who is where. `slot` is
//!   `lineup`, `bench`, `rotation`, `bullpen` or `reserve`; `ord` is the order inside it.
//! * `potentials (id, data)`: each player's ceiling (`bm_world::Potential`) as JSON.
//! * `foreign (id, signed_from)`: foreign players of KBO and NPB clubs and the birthplace
//!   group they were signed from.
//! * `careers (id, data)`: how each player came into the world, his seasons and how he
//!   left (`bm_world::Career`) as JSON, for everyone who has played, retired players
//!   included.
//! * `away (id, org)`: Korean players on active military duty, off every roster, and the
//!   organisation that holds their rights; their attributes are in `players`.
//!
//! A save is written in one transaction into a fresh file, so an interrupted save never
//! leaves a half-written game behind.

use bm_world::{Away, Date, Person, Player, Population, Position, Roster, Squad};
use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// Version of the table layout. Older files are refused rather than misread.
pub const FORMAT: i64 = 4;

#[derive(Debug)]
pub enum SaveError {
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    /// The file is not a saved game of a format this program reads.
    Format(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Sql(e) => write!(f, "database: {e}"),
            SaveError::Json(e) => write!(f, "player data: {e}"),
            SaveError::Io(e) => write!(f, "file: {e}"),
            SaveError::Format(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<rusqlite::Error> for SaveError {
    fn from(e: rusqlite::Error) -> Self {
        SaveError::Sql(e)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(e: serde_json::Error) -> Self {
        SaveError::Json(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

/// What a saved game holds.
#[derive(Clone, Debug)]
pub struct SaveGame {
    /// Seed the world was generated from.
    pub seed: u64,
    pub population: Population,
}

const SCHEMA: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE people (id INTEGER PRIMARY KEY, name_ko TEXT NOT NULL, name_en TEXT NOT NULL, born TEXT NOT NULL, birthplace TEXT NOT NULL);
CREATE TABLE players (id INTEGER PRIMARY KEY, data TEXT NOT NULL);
CREATE TABLE rosters (club TEXT NOT NULL, level TEXT NOT NULL, slot TEXT NOT NULL, ord INTEGER NOT NULL, player INTEGER NOT NULL, position TEXT,
                      PRIMARY KEY (club, slot, ord));
CREATE INDEX rosters_player ON rosters (player);
CREATE TABLE potentials (id INTEGER PRIMARY KEY, data TEXT NOT NULL);
CREATE TABLE foreign_players (id INTEGER PRIMARY KEY, signed_from TEXT NOT NULL);
CREATE TABLE careers (id INTEGER PRIMARY KEY, data TEXT NOT NULL);
CREATE TABLE away (id INTEGER PRIMARY KEY, org TEXT NOT NULL);
";

fn position_code(p: Option<Position>) -> Option<String> {
    p.map(|p| serde_json::to_value(p).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default())
}

fn position_from(code: Option<String>) -> Result<Option<Position>, SaveError> {
    match code {
        None => Ok(None),
        Some(c) => Ok(Some(serde_json::from_value(serde_json::Value::String(c))?)),
    }
}

/// Write `game` to `path`, replacing any file there.
pub fn save(path: &Path, game: &SaveGame) -> Result<(), SaveError> {
    // Build the new file next to the old one and move it into place at the end.
    let tmp = path.with_extension("tmp");
    if tmp.exists() {
        std::fs::remove_file(&tmp)?;
    }
    {
        let mut db = Connection::open(&tmp)?;
        db.execute_batch(SCHEMA)?;
        let tx = db.transaction()?;
        {
            let mut meta = tx.prepare("INSERT INTO meta (key, value) VALUES (?1, ?2)")?;
            meta.execute(params!["format", FORMAT.to_string()])?;
            meta.execute(params!["date", game.population.date.to_string()])?;
            meta.execute(params!["seed", game.seed.to_string()])?;
            let mut people = tx.prepare("INSERT INTO people (id, name_ko, name_en, born, birthplace) VALUES (?1, ?2, ?3, ?4, ?5)")?;
            for p in game.population.people.values() {
                people.execute(params![p.id, p.name.ko, p.name.en, p.born.to_string(), p.birthplace])?;
            }
            let mut potentials = tx.prepare("INSERT INTO potentials (id, data) VALUES (?1, ?2)")?;
            for (id, p) in &game.population.potentials {
                potentials.execute(params![id, serde_json::to_string(p)?])?;
            }
            let mut foreign = tx.prepare("INSERT INTO foreign_players (id, signed_from) VALUES (?1, ?2)")?;
            for (id, group) in &game.population.foreign {
                foreign.execute(params![id, group])?;
            }
            let mut careers = tx.prepare("INSERT INTO careers (id, data) VALUES (?1, ?2)")?;
            for (id, c) in &game.population.careers {
                careers.execute(params![id, serde_json::to_string(c)?])?;
            }
            let mut players = tx.prepare("INSERT INTO players (id, data) VALUES (?1, ?2)")?;
            let mut slots = tx.prepare("INSERT INTO rosters (club, level, slot, ord, player, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
            for r in &game.population.rosters {
                let mut put = |slot: &str, ord: usize, p: &Player, pos: Option<Position>| -> Result<(), SaveError> {
                    players.execute(params![p.id, serde_json::to_string(p)?])?;
                    slots.execute(params![r.club, r.level, slot, ord as i64, p.id, position_code(pos)])?;
                    Ok(())
                };
                for (i, (p, pos)) in r.squad.hitters.iter().enumerate() {
                    put("lineup", i, p, *pos)?;
                }
                for (i, (p, pos)) in r.squad.bench.iter().enumerate() {
                    put("bench", i, p, *pos)?;
                }
                for (i, p) in r.squad.starters.iter().enumerate() {
                    put("rotation", i, p, Some(Position::Pitcher))?;
                }
                for (i, p) in r.squad.relievers.iter().enumerate() {
                    put("bullpen", i, p, Some(Position::Pitcher))?;
                }
                for (i, (p, pos)) in r.reserves.iter().enumerate() {
                    put("reserve", i, p, *pos)?;
                }
            }
            let mut away = tx.prepare("INSERT INTO away (id, org) VALUES (?1, ?2)")?;
            for (id, a) in &game.population.away {
                players.execute(params![id, serde_json::to_string(&a.player)?])?;
                away.execute(params![id, a.org])?;
            }
        }
        tx.commit()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read a game written by [`save`].
pub fn load(path: &Path) -> Result<SaveGame, SaveError> {
    let db = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let meta = |key: &str| -> Result<String, SaveError> {
        db.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get::<_, String>(0))
            .map_err(|_| SaveError::Format(format!("not a saved game: no '{key}' entry")))
    };
    let format: i64 = meta("format")?.parse().map_err(|_| SaveError::Format("bad format entry".into()))?;
    if format != FORMAT {
        return Err(SaveError::Format(format!("saved game format {format}, this program reads {FORMAT}")));
    }
    let date = Date::parse(&meta("date")?).ok_or_else(|| SaveError::Format("bad date entry".into()))?;
    let seed: u64 = meta("seed")?.parse().map_err(|_| SaveError::Format("bad seed entry".into()))?;

    let mut people = BTreeMap::new();
    let mut q = db.prepare("SELECT id, name_ko, name_en, born, birthplace FROM people")?;
    let rows =
        q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?)))?;
    for row in rows {
        let (id, ko, en, born, birthplace) = row?;
        let born = Date::parse(&born).ok_or_else(|| SaveError::Format(format!("bad date of birth for {id}")))?;
        people.insert(id, Person { id, name: bm_world::league::Text { ko, en }, born, birthplace });
    }

    let mut players: BTreeMap<u32, Player> = BTreeMap::new();
    let mut q = db.prepare("SELECT id, data FROM players")?;
    for row in q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))? {
        let (id, data) = row?;
        players.insert(id, serde_json::from_str(&data)?);
    }

    let mut potentials = BTreeMap::new();
    let mut q = db.prepare("SELECT id, data FROM potentials")?;
    for row in q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))? {
        let (id, data) = row?;
        potentials.insert(id, serde_json::from_str(&data)?);
    }

    let mut foreign = BTreeMap::new();
    let mut q = db.prepare("SELECT id, signed_from FROM foreign_players")?;
    for row in q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))? {
        let (id, group) = row?;
        foreign.insert(id, group);
    }

    let mut careers = BTreeMap::new();
    let mut q = db.prepare("SELECT id, data FROM careers")?;
    for row in q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))? {
        let (id, data) = row?;
        careers.insert(id, serde_json::from_str(&data)?);
    }

    // Rosters in the order they were written: first appearance of each club.
    let mut rosters: Vec<Roster> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    let mut q = db.prepare("SELECT club, level, slot, player, position FROM rosters ORDER BY rowid")?;
    let rows =
        q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, u32>(3)?, r.get::<_, Option<String>>(4)?)))?;
    for row in rows {
        let (club, level, slot, id, pos) = row?;
        let i = *index.entry(club.clone()).or_insert_with(|| {
            rosters.push(Roster {
                club: club.clone(),
                level: level.clone(),
                squad: Squad { hitters: Vec::new(), bench: Vec::new(), starters: Vec::new(), relievers: Vec::new() },
                reserves: Vec::new(),
            });
            rosters.len() - 1
        });
        let player = players.get(&id).cloned().ok_or_else(|| SaveError::Format(format!("roster of {club} names unknown player {id}")))?;
        let pos = position_from(pos)?;
        let r = &mut rosters[i];
        match slot.as_str() {
            "lineup" => r.squad.hitters.push((player, pos)),
            "bench" => r.squad.bench.push((player, pos)),
            "rotation" => r.squad.starters.push(player),
            "bullpen" => r.squad.relievers.push(player),
            "reserve" => r.reserves.push((player, pos)),
            other => return Err(SaveError::Format(format!("unknown roster slot '{other}'"))),
        }
    }
    let mut away = BTreeMap::new();
    let mut q = db.prepare("SELECT id, org FROM away")?;
    for row in q.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))? {
        let (id, org) = row?;
        let player = players.get(&id).cloned().ok_or_else(|| SaveError::Format(format!("player {id} away from {org} is unknown")))?;
        away.insert(id, Away { org, player });
    }
    Ok(SaveGame { seed, population: Population { date, people, rosters, potentials, foreign, careers, away } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bm_world::{populate, LevelTable, World};

    #[test]
    fn a_world_survives_a_save_and_a_load() {
        let world = World::bundled();
        let levels = LevelTable::bundled();
        let mut pop = populate(&world, &levels, Date::new(2027, 2, 1), 11);
        // An offseason sends Korean players into military service, some off every roster.
        let (aging, careers, people) = (bm_world::AgingTable::bundled(), bm_world::CareerTable::bundled(), bm_world::PeopleMaker::new());
        let ctx = bm_world::Offseason { world: &world, levels: &levels, aging: &aging, careers: &careers, people: &people };
        bm_world::offseason(&ctx, &mut pop, 2027, None, 11);
        assert!(!pop.away.is_empty());
        let date = pop.date;
        let game = SaveGame { seed: 11, population: pop };
        let dir = std::env::temp_dir().join(format!("bm-save-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("world.bmsave");
        save(&path, &game).unwrap();
        let back = load(&path).unwrap();
        assert_eq!(back.seed, 11);
        assert_eq!(back.population.date, date);
        assert_eq!(back.population.people, game.population.people);
        // Rosters come back slot for slot, player for player, with every ceiling.
        let json = |p: &Population| serde_json::to_string(&p.rosters).unwrap();
        assert_eq!(json(&back.population), json(&game.population));
        assert_eq!(back.population.potentials, game.population.potentials);
        assert_eq!(back.population.foreign, game.population.foreign);
        assert_eq!(back.population.careers, game.population.careers);
        assert_eq!(back.population.away, game.population.away);
        assert!(!back.population.foreign.is_empty() && back.population.careers.len() == back.population.people.len());
        // Saving again over the file works and leaves no temporary file.
        save(&path, &back).unwrap();
        assert!(!path.with_extension("tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn other_files_are_refused() {
        let dir = std::env::temp_dir().join(format!("bm-save-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("other.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT); INSERT INTO meta VALUES ('format', '99');").unwrap();
        drop(db);
        assert!(matches!(load(&path), Err(SaveError::Format(_))));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
