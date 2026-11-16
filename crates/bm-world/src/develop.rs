//! Development and aging: how a player's attributes change from one season to the next.
//!
//! Every attribute belongs to a group with its own aging drift by age (for example, a
//! fastball loses about 3.75 mph between 26 and 38, sprint speed 0.14 ft/s a year from
//! 23). Before his peak a player also closes a share of the gap to his potential each
//! year. Each year adds noise of the group's size. The curves are in the editable file
//! `data/people/aging.json`, each with the evidence behind it
//! (`docs/research/world/aging.md`).

use crate::attr::{Attr, Attributes, N_ATTR};
use crate::generate::Talent;
use crate::player::Player;
use bm_engine::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const AGING: &str = include_str!("../../../data/people/aging.json");

#[derive(Clone, Debug, Deserialize)]
struct GroupFile {
    attrs: Vec<String>,
    drift: Vec<f32>,
    noise: f32,
    /// Years by which the group's growth towards the ceiling comes later than the others'.
    #[serde(default)]
    growth_delay: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct AgingFile {
    from_age: u32,
    growth: Vec<f32>,
    remaining_growth: Vec<f32>,
    groups: BTreeMap<String, GroupFile>,
}

/// The aging curves of every attribute.
#[derive(Clone, Debug)]
pub struct AgingTable {
    from_age: u32,
    growth: Vec<f32>,
    remaining: Vec<f32>,
    /// Per attribute: drift by age and yearly noise.
    drift: Vec<Vec<f32>>,
    noise: [f32; N_ATTR],
    /// Group of each attribute, by name (for pitch grades: the `stuff` group).
    group_of: [String; N_ATTR],
    /// Per attribute: years by which its growth comes later.
    delay: [f32; N_ATTR],
    stuff_drift: Vec<f32>,
    stuff_noise: f32,
    stuff_delay: f32,
}

fn at(v: &[f32], from: u32, age: f64) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let i = (age.floor() as i64 - from as i64).clamp(0, v.len() as i64 - 1) as usize;
    v[i]
}

impl AgingTable {
    pub fn bundled() -> AgingTable {
        AgingTable::from_json(AGING).unwrap_or_else(|e| panic!("data/people/aging.json: {e}"))
    }

    pub fn from_json(text: &str) -> Result<AgingTable, String> {
        let f: AgingFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let mut drift = vec![Vec::new(); N_ATTR];
        let mut noise = [0.0f32; N_ATTR];
        let mut group_of: [String; N_ATTR] = std::array::from_fn(|_| String::new());
        let mut delay = [0.0f32; N_ATTR];
        for (name, g) in &f.groups {
            for key in &g.attrs {
                let a = Attr::ALL.iter().position(|a| a.key() == key).ok_or_else(|| format!("group {name}: unknown attribute '{key}'"))?;
                if !group_of[a].is_empty() {
                    return Err(format!("attribute '{key}' is in two groups"));
                }
                drift[a] = g.drift.clone();
                noise[a] = g.noise;
                delay[a] = g.growth_delay;
                group_of[a] = name.clone();
            }
        }
        if let Some(a) = group_of.iter().position(String::is_empty) {
            return Err(format!("attribute '{}' has no aging group", Attr::ALL[a].key()));
        }
        let stuff = f.groups.get("stuff").ok_or("no 'stuff' group (pitch grades follow it)")?;
        Ok(AgingTable {
            from_age: f.from_age,
            growth: f.growth,
            remaining: f.remaining_growth,
            drift,
            noise,
            group_of,
            delay,
            stuff_drift: stuff.drift.clone(),
            stuff_noise: stuff.noise,
            stuff_delay: stuff.growth_delay,
        })
    }

    /// Drift of `attr` in the year after a birthday at `age`, standard deviations.
    pub fn drift(&self, attr: Attr, age: f64) -> f32 {
        at(&self.drift[attr as usize], self.from_age, age)
    }

    pub fn noise(&self, attr: Attr) -> f32 {
        self.noise[attr as usize]
    }

    pub fn group(&self, attr: Attr) -> &str {
        &self.group_of[attr as usize]
    }

    /// Share of the gap to the potential closed in the year after a birthday at `age`.
    pub fn growth(&self, age: f64) -> f32 {
        at(&self.growth, self.from_age, age)
    }

    /// The same for one attribute, whose growth may come later than the others'
    /// (`growth_delay` of its group: command, for example, keeps improving into the late
    /// twenties).
    pub fn growth_of(&self, attr: Attr, age: f64) -> f32 {
        at(&self.growth, self.from_age, age - self.delay[attr as usize] as f64)
    }

    /// Mean gap between a player's current attributes and his peak at `age`.
    pub fn remaining_growth(&self, age: f64) -> f32 {
        at(&self.remaining, self.from_age, age)
    }
}

/// A player's ceiling: the attribute values he would reach at his peak.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Potential {
    pub attrs: Attributes,
}

impl Potential {
    /// A ceiling for a player of `age`: the table's mean remaining growth at that age,
    /// scaled by a personal factor (`scale`, 1 = typical) and spread a little between
    /// attribute groups.
    pub fn draw(player: &Player, age: f64, scale: f32, table: &AgingTable, rng: &mut Rng) -> Potential {
        let mean = table.remaining_growth(age) * scale.max(0.0);
        let mut groups: BTreeMap<&str, f32> = BTreeMap::new();
        let mut attrs = player.attrs;
        for a in Attr::ALL {
            let g = *groups.entry(table.group(a)).or_insert_with(|| (1.0 + 0.25 * rng.gauss(0.0, 1.0) as f32).max(0.0));
            attrs[a] = player.attrs[a] + mean * g;
        }
        Potential { attrs }
    }

    /// Move the skill balance of the player's level (`from`) towards that of the level he
    /// is growing into (`to`): minor-leaguers have more raw power than bat-to-ball skill,
    /// weaker defence and bodies, far worse command and nearly major-league velocity (the
    /// `power`, `fielding`, `physical`, `command` and `velocity` parts of a level's talent),
    /// and a Japanese prospect grows into the contact and control of the NPB rather than
    /// into a major-leaguer. A share `closure` of each difference closes by the peak.
    pub fn balance(&mut self, player: &Player, from: &Talent, to: &Talent, closure: &Closure) {
        use Attr::*;
        let mut shift = |attrs: &[Attr], by: f32| {
            for &a in attrs {
                self.attrs[a] += by;
            }
        };
        const FIELD: [Attr; 11] = [Hands, Range, Reads, ArmAccuracy, Footwork, Baserunning, Stealing, Framing, Blocking, GameCalling, ThrowingOut];
        if player.is_pitcher() {
            let rel = |t: &Talent| (t.command, t.velocity, t.fielding - t.pitching, t.physical - t.pitching);
            let (a, b) = (rel(from), rel(to));
            shift(&[Control, Command], closure.command * (b.0 - a.0));
            shift(&[Velocity], closure.velocity * (b.1 - a.1));
            shift(&FIELD[..5], closure.fielding * (b.2 - a.2));
            shift(&[Strength, Durability, Agility], closure.physical * (b.3 - a.3));
        } else {
            let rel = |t: &Talent| (t.power, t.fielding - t.hitting, t.physical - t.hitting);
            let (a, b) = (rel(from), rel(to));
            shift(&[Power], closure.power * (b.0 - a.0));
            shift(&FIELD, closure.fielding * (b.1 - a.1));
            shift(&[Strength, Stamina, Durability, Agility, Speed, Acceleration, ArmStrength], closure.physical * (b.2 - a.2));
        }
    }
}

/// Shares of a level's skill gaps that close by a player's peak (`Potential::balance`).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Closure {
    pub power: f32,
    pub fielding: f32,
    pub physical: f32,
    pub command: f32,
    #[serde(default = "one")]
    pub velocity: f32,
}

fn one() -> f32 {
    1.0
}

impl Closure {
    pub const NONE: Closure = Closure { power: 0.0, fielding: 0.0, physical: 0.0, command: 0.0, velocity: 0.0 };
}

/// One year of development for a player who was `age` at his last birthday. The ceiling
/// ages with the player and takes the same yearly noise, so the gap between them only
/// closes through growth: a player at his ceiling simply follows the aging curves.
pub fn develop(player: &mut Player, potential: &mut Potential, age: f64, table: &AgingTable, rng: &mut Rng) {
    let stuff_gap = (potential.attrs[Attr::Stuff] - player.attrs[Attr::Stuff]).max(0.0);
    for a in Attr::ALL {
        let gap = (potential.attrs[a] - player.attrs[a]).max(0.0);
        let shift = table.drift(a, age) + table.noise(a) * rng.gauss(0.0, 1.0) as f32;
        potential.attrs[a] = (potential.attrs[a] + shift).clamp(-6.0, 4.0);
        player.attrs[a] = (player.attrs[a] + table.growth_of(a, age) * gap + shift).clamp(-6.0, 4.0);
    }
    // Pitch grades age and grow with stuff.
    let growth = at(&table.growth, table.from_age, age - table.stuff_delay as f64);
    if let Some(p) = player.pitching.as_mut() {
        for pitch in &mut p.pitches {
            let change = growth * stuff_gap + at(&table.stuff_drift, table.from_age, age) + table.stuff_noise * rng.gauss(0.0, 1.0) as f32;
            pitch.grade = (pitch.grade + change).clamp(-6.0, 4.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::spread;
    use crate::generate::{pitcher, position_player, Talent};
    use crate::player::{Position, Role};

    #[test]
    fn every_attribute_ages() {
        let t = AgingTable::bundled();
        for a in Attr::ALL {
            assert!(!t.group(a).is_empty(), "{}", a.key());
        }
        // Nothing declines before the mid-20s except speed.
        assert_eq!(t.drift(Attr::Velocity, 24.0), 0.0);
        assert!(t.drift(Attr::Speed, 24.0) < 0.0);
        assert!(t.drift(Attr::Velocity, 35.0) < t.drift(Attr::Velocity, 27.0));
        assert!(AgingTable::from_json(r#"{"from_age": 16, "growth": [], "remaining_growth": [], "groups": {}}"#).is_err());
    }

    /// Mean change of an attribute between two ages, in its physical unit, for players
    /// already at their ceiling.
    fn mean_change(attr: Attr, from: u32, to: u32, unit: f64, pitchers: bool) -> f64 {
        let t = AgingTable::bundled();
        let mut rng = Rng::new(3);
        let n = 3000;
        let mut total = 0.0;
        for i in 0..n {
            let mut p = if pitchers {
                pitcher(&mut rng, i, Role::Starter, &Talent::MAJOR)
            } else {
                position_player(&mut rng, i, Some(Position::Center), &Talent::MAJOR)
            };
            let start = p.attrs[attr];
            let mut ceiling = Potential { attrs: p.attrs };
            for age in from..to {
                develop(&mut p, &mut ceiling, age as f64, &t, &mut rng);
            }
            total += (p.attrs[attr] - start) as f64;
        }
        total / n as f64 * unit
    }

    #[test]
    fn decline_follows_the_published_curves() {
        // Fastballs: about -3.75 mph from 26 to 38 (Petti & Zimmerman 2012).
        let v = mean_change(Attr::Velocity, 26, 38, spread::VELOCITY_MPH, true);
        assert!((-4.2..-3.3).contains(&v), "velocity {v:.2} mph");
        // Sprint speed: about -0.14 ft/s a year from 23 on.
        let s = mean_change(Attr::Speed, 25, 35, spread::SPRINT_FT, false) / 10.0;
        assert!((-0.17..-0.11).contains(&s), "sprint {s:.3} ft/s a year");
        // Bat speed: flat to 28, then about -0.31 mph a year.
        let early = mean_change(Attr::Power, 24, 28, spread::BAT_SPEED_MPH, false);
        let late = mean_change(Attr::Power, 32, 36, spread::BAT_SPEED_MPH, false) / 4.0;
        assert!(early.abs() < 0.25, "bat speed before 28: {early:.2} mph");
        assert!((-0.4..-0.22).contains(&late), "bat speed after 31: {late:.2} mph a year");
    }

    #[test]
    fn young_players_grow_toward_their_potential() {
        let t = AgingTable::bundled();
        let mut rng = Rng::new(8);
        let (mut gained, n) = (0.0, 2000);
        for i in 0..n {
            let mut p = position_player(&mut rng, i, Some(Position::Short), &Talent { hitting: -2.0, ..Talent::MAJOR });
            let start = p.attrs[Attr::Contact];
            let mut ceiling = Potential::draw(&p, 20.0, 1.0, &t, &mut rng);
            for age in 20..27 {
                develop(&mut p, &mut ceiling, age as f64, &t, &mut rng);
            }
            gained += (p.attrs[Attr::Contact] - start) as f64;
        }
        let mean = gained / n as f64;
        // Most of the 2.2 standard deviations of remaining growth at 20 arrive by 27.
        assert!((1.6..2.4).contains(&mean), "contact gained {mean:.2} SD from 20 to 27");
    }
}
