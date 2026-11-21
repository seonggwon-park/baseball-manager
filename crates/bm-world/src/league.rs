//! The fictional league world read from the editable files `data/leagues/*.json`:
//! countries, leagues and their divisions, clubs with their parent clubs, and ballparks.
//!
//! Cities, ballpark dimensions and the league structure follow the real 2026 leagues
//! (`docs/research/world`); the names of clubs, leagues and ballparks are invented.

use bm_engine::{MatchEnv, ModelParams, Rules};
use bm_physics::{Atmosphere, BallSpec, FencePoint, Park, Wind};
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

const US: &str = include_str!("../../../data/leagues/us.json");
const KR: &str = include_str!("../../../data/leagues/kr.json");
const JP: &str = include_str!("../../../data/leagues/jp.json");

/// Text in the languages of the interface.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Text {
    pub ko: String,
    pub en: String,
}

/// Rung of the professional ladder a league sits on.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// MLB, KBO and NPB first teams.
    Top,
    Aaa,
    Aa,
    HighA,
    SingleA,
    /// Korean and Japanese second teams.
    Second,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Roof {
    Open,
    Retractable,
    #[serde(alias = "fixed")]
    Dome,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Grass,
    Turf,
}

/// A sub-league (MLB's two leagues, NPB's two leagues).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Group {
    pub id: String,
    pub name: Text,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Division {
    pub id: String,
    pub name: Text,
    /// Sub-league the division belongs to, if the league has them.
    pub group: Option<String>,
}

/// Announced dates of the 2027 season, `YYYY-MM-DD`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SeasonDates {
    pub opening: Option<String>,
    pub last_day: Option<String>,
    pub all_star: Option<String>,
    pub spring_games_from: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct League {
    pub id: String,
    pub level: Level,
    pub name: Text,
    /// Regular-season games per club.
    pub games: u32,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub divisions: Vec<Division>,
    #[serde(default)]
    pub season_2027: Option<SeasonDates>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Ballpark {
    pub name: Text,
    pub capacity: Option<u32>,
    /// Elevation above sea level, m (unknown for some second-team grounds).
    pub elevation_m: Option<f64>,
    pub roof: Roof,
    pub surface: Surface,
    /// Fence distance, m: left-field line, left-centre, centre, right-centre, right-field line.
    pub fence_m: [f64; 5],
    /// Wall height, m: left, centre, right.
    pub wall_m: [f64; 3],
    /// Values that were not published and were filled in.
    #[serde(default)]
    pub estimated: Vec<String>,
    /// Factor on the batted ball's drag in this park: how far balls carry there beyond what
    /// its elevation and the game's temperature explain (prevailing wind and the like),
    /// measured on the 2025 Statcast sample for the major-league parks
    /// (`tools/scenarios/park_carry.py`, `docs/physics-model.md` 7); 1 where it was not
    /// measured. It applies to batted balls only, not to pitches.
    #[serde(default = "unit")]
    pub batted_drag: f64,
}

fn unit() -> f64 {
    1.0
}

impl Ballpark {
    /// The playing field for the match engine: fence distances at five bearings from line
    /// to line, with the wall height of the nearest field (left, centre, right).
    pub fn field(&self) -> Park {
        let mut park = Park::from_fence(&self.fence_m, self.wall_m[1]);
        let heights = [self.wall_m[0], 0.5 * (self.wall_m[0] + self.wall_m[1]), self.wall_m[1], 0.5 * (self.wall_m[1] + self.wall_m[2]), self.wall_m[2]];
        park.fence = self
            .fence_m
            .iter()
            .zip(heights)
            .enumerate()
            .map(|(i, (&distance, height))| FencePoint { angle: -FRAC_PI_4 + FRAC_PI_2 * i as f64 / 4.0, distance, height })
            .collect();
        park.synthetic = self.surface == Surface::Turf;
        park
    }

    /// Elevation used for the air density; sea level where it is not known.
    pub fn elevation(&self) -> f64 {
        self.elevation_m.unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Club {
    pub id: String,
    pub league: String,
    pub division: Option<String>,
    /// Club whose farm or second team this is.
    pub parent: Option<String>,
    /// Second team of a Korean or Japanese club: it carries the parent's name.
    #[serde(default)]
    pub second_team: bool,
    /// The armed-forces team, where Korean players do their military service.
    #[serde(default)]
    pub military: bool,
    pub city: Text,
    pub region: Option<Text>,
    /// ISO 3166 country code of the club's city.
    pub country: String,
    pub name: Text,
    pub abbr: String,
    /// Primary and secondary colour, `#rrggbb`.
    pub colors: [String; 2],
    pub park: Ballpark,
}

/// One country's file.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Country {
    /// `US`, `KR` or `JP`.
    pub country: String,
    pub leagues: Vec<League>,
    pub clubs: Vec<Club>,
}

/// Every league and club of the world.
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    pub countries: Vec<Country>,
}

impl World {
    /// The world in the data files bundled with the program.
    pub fn bundled() -> Self {
        let read = |name: &str, text: &str| -> Country { serde_json::from_str(text).unwrap_or_else(|e| panic!("data/leagues/{name}.json: {e}")) };
        World { countries: vec![read("kr", KR), read("us", US), read("jp", JP)] }
    }

    pub fn leagues(&self) -> impl Iterator<Item = &League> {
        self.countries.iter().flat_map(|c| c.leagues.iter())
    }

    pub fn clubs(&self) -> impl Iterator<Item = &Club> {
        self.countries.iter().flat_map(|c| c.clubs.iter())
    }

    pub fn club(&self, id: &str) -> Option<&Club> {
        self.clubs().find(|c| c.id == id)
    }

    pub fn league(&self, id: &str) -> Option<&League> {
        self.leagues().find(|l| l.id == id)
    }

    pub fn level_of(&self, club: &Club) -> Level {
        self.league(&club.league).map(|l| l.level).unwrap_or(Level::Top)
    }

    /// The clubs whose farm or second teams are `id`'s.
    pub fn affiliates<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Club> + 'a {
        self.clubs().filter(move |c| c.parent.as_deref() == Some(id))
    }

    /// Top-level clubs a manager can take charge of.
    pub fn first_teams(&self) -> impl Iterator<Item = &Club> {
        self.clubs().filter(|c| c.parent.is_none() && self.level_of(c) == Level::Top)
    }

    /// Match conditions in the ballparks of the clubs of the given leagues, in data-file
    /// order: a mild day (24 °C, 760 mmHg, 55 % humidity), no wind, the park's fences and
    /// elevation, with the batted ball's drag multiplied by `drag` (1 = the major-league
    /// ball) and by the park's measured carry (`Ballpark::batted_drag`). Returns the
    /// environments and, for each, the index of its league in `ids`.
    pub fn park_envs(&self, ids: &[&str], rules: &Rules, params: &ModelParams, ball: BallSpec, drag: f64) -> (Vec<MatchEnv>, Vec<usize>) {
        let (mut envs, mut league) = (Vec::new(), Vec::new());
        for (li, id) in ids.iter().enumerate() {
            for c in self.clubs().filter(|c| c.league == *id) {
                let mut env = MatchEnv::new(&Atmosphere::new(24.0, 760.0, 55.0, c.park.elevation()), Wind::CALM, c.park.field(), rules.clone(), params.clone());
                env.ball = ball;
                env.batted_flight.aero.cd0 *= drag * c.park.batted_drag;
                envs.push(env);
                league.push(li);
            }
        }
        (envs, league)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_world_has_the_real_structure() {
        let w = World::bundled();
        let count = |level: Level| w.clubs().filter(|c| w.level_of(c) == level).count();
        assert_eq!(count(Level::Top), 30 + 10 + 12);
        for level in [Level::Aaa, Level::Aa, Level::HighA, Level::SingleA] {
            assert_eq!(count(level), 30, "{level:?}");
        }
        assert_eq!(count(Level::Second), 12 + 14);
        assert_eq!(w.first_teams().count(), 52);
        // Every MLB club has one affiliate at each of the four minor-league levels.
        for club in w.clubs().filter(|c| c.league == "us-major") {
            let mut levels: Vec<Level> = w.affiliates(&club.id).map(|a| w.level_of(a)).collect();
            levels.sort();
            assert_eq!(levels, [Level::Aaa, Level::Aa, Level::HighA, Level::SingleA], "{}", club.id);
        }
        // Korean and Japanese first teams have one second team each, except where the
        // second-team league has clubs of its own (the armed forces and the city club in
        // Korea, two farm-only clubs in Japan).
        for club in w.clubs().filter(|c| c.league == "kr-top" || c.league == "jp-top") {
            assert_eq!(w.affiliates(&club.id).count(), 1, "{}", club.id);
        }
        assert_eq!(w.clubs().filter(|c| c.military).count(), 1);
    }

    #[test]
    fn ballparks_become_engine_fields() {
        let w = World::bundled();
        // The highest park is the one in Denver, about 1,580 m up.
        let high = w.clubs().max_by(|a, b| a.park.elevation().total_cmp(&b.park.elevation())).unwrap();
        assert_eq!(high.city.en, "Denver");
        assert!((1500.0..1650.0).contains(&high.park.elevation()));
        for c in w.clubs() {
            let field = c.park.field();
            let (centre, wall) = field.fence_at(0.0);
            assert!((c.park.fence_m[2] - centre).abs() < 1e-9 && (c.park.wall_m[1] - wall).abs() < 1e-9, "{}", c.id);
            let (left, left_wall) = field.fence_at(-FRAC_PI_4);
            assert!((c.park.fence_m[0] - left).abs() < 1e-9 && (c.park.wall_m[0] - left_wall).abs() < 1e-9, "{}", c.id);
            assert_eq!(field.synthetic, c.park.surface == Surface::Turf);
        }
    }

    #[test]
    fn measured_carry_reaches_the_batted_ball_only() {
        let w = World::bundled();
        // The major-league parks carry their measured factor; together they leave the league's
        // carry as it was, and no park is more than a few per cent from the model's air.
        let major: Vec<&Club> = w.clubs().filter(|c| c.league == "us-major").collect();
        let measured: Vec<f64> = major.iter().map(|c| c.park.batted_drag).filter(|&d| d != 1.0).collect();
        assert!(measured.len() >= 25, "{} parks measured", measured.len());
        let mean = measured.iter().sum::<f64>() / measured.len() as f64;
        assert!((mean - 1.0).abs() < 0.005, "mean factor {mean}");
        assert!(measured.iter().all(|d| (0.93..1.07).contains(d)), "{measured:?}");
        // Denver's thin air carries less than the model's elevation alone says.
        let denver = major.iter().find(|c| c.city.en == "Denver").unwrap();
        assert!(denver.park.batted_drag > 1.02, "{}", denver.park.batted_drag);
        // Parks that were not measured keep the model's air.
        assert!(w.clubs().filter(|c| c.league != "us-major").all(|c| c.park.batted_drag == 1.0));
        // The factor reaches the batted ball's drag, not the pitch's.
        let (envs, _) = w.park_envs(&["us-major"], &Rules::mlb_2025(), &ModelParams::default(), BallSpec::mlb(), 1.0);
        let plain = MatchEnv::new(
            &Atmosphere::new(24.0, 760.0, 55.0, denver.park.elevation()),
            Wind::CALM,
            denver.park.field(),
            Rules::mlb_2025(),
            ModelParams::default(),
        );
        let i = major.iter().position(|c| c.city.en == "Denver").unwrap();
        assert!((envs[i].batted_flight.aero.cd0 - plain.batted_flight.aero.cd0 * denver.park.batted_drag).abs() < 1e-12);
        assert_eq!(envs[i].pitch_flight.aero.cd0, plain.pitch_flight.aero.cd0);
    }
}
