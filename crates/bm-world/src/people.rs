//! Generated people: a name in both scripts of the interface, a date and a place of birth.
//! The birthplace comes from the shares of the player's group (`birthplace.rs`) and picks
//! the name book; the date of birth comes from the age spread of the group.

use crate::birthplace::Birthplaces;
use crate::calendar::Date;
use crate::league::Text;
use crate::names::Names;
use bm_engine::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Person {
    pub id: u32,
    pub name: Text,
    pub born: Date,
    /// ISO 3166-1 alpha-2 code of the country of birth.
    pub birthplace: String,
}

/// Ages of a group of players on one date, years: a normal spread cut at `min` and `max`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct AgeSpread {
    pub mean: f64,
    pub sd: f64,
    pub min: f64,
    pub max: f64,
}

/// Makes people from the bundled birthplace shares and name books.
pub struct PeopleMaker {
    births: Birthplaces,
    books: BTreeMap<String, Names>,
}

impl Default for PeopleMaker {
    fn default() -> Self {
        PeopleMaker::new()
    }
}

impl PeopleMaker {
    pub fn new() -> PeopleMaker {
        let books = ["en", "es", "ja", "ko", "zh"].into_iter().map(|l| (l.to_string(), Names::bundled(l).expect("bundled name book"))).collect();
        PeopleMaker { births: Birthplaces::bundled(), books }
    }

    pub fn birthplaces(&self) -> &Birthplaces {
        &self.births
    }

    /// A person of birthplace group `group` (`mlb`, `aaa`, `kbo_foreign`, ...) whose age on
    /// `on` follows `age`.
    pub fn make(&self, id: u32, group: &str, age: &AgeSpread, on: Date, rng: &mut Rng) -> Person {
        let birthplace = self.births.draw(group, rng).to_string();
        self.make_from(id, &birthplace, age, on, rng)
    }

    /// A person born in `birthplace`.
    pub fn make_from(&self, id: u32, birthplace: &str, age: &AgeSpread, on: Date, rng: &mut Rng) -> Person {
        let book = &self.books[self.births.locale(birthplace)];
        let name = book.draw(rng);
        let years = rng.gauss_clamped(age.mean, age.sd, age.min, age.max);
        let born = on.add_days(-(years * 365.2425).round() as i32);
        Person { id, name, born, birthplace: birthplace.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_follow_their_group() {
        let maker = PeopleMaker::new();
        let on = Date::new(2027, 2, 1);
        let age = AgeSpread { mean: 28.0, sd: 3.7, min: 20.0, max: 42.0 };
        let mut rng = Rng::new(5);
        let n = 4000;
        let people: Vec<Person> = (0..n).map(|i| maker.make(i, "mlb", &age, on, &mut rng)).collect();
        let mean_age = people.iter().map(|p| on.days_since(p.born) as f64 / 365.2425).sum::<f64>() / n as f64;
        assert!((mean_age - 28.0).abs() < 0.2, "{mean_age}");
        assert!(people.iter().all(|p| (20..=42).contains(&p.born.age_on(on))));
        let us = people.iter().filter(|p| p.birthplace == "US").count() as f64 / n as f64;
        assert!((us - maker.birthplaces().share("mlb", "US")).abs() < 0.025, "{us}");
        // A Dominican gets a Spanish name, a Japanese player a Japanese one, written in both scripts.
        let dr = people.iter().find(|p| p.birthplace == "DO").unwrap();
        assert!(!dr.name.ko.is_empty() && dr.name.en.is_ascii(), "{:?}", dr.name);
        let kr = maker.make(9999, "kbo_domestic", &age, on, &mut rng);
        assert_eq!(kr.birthplace, "KR");
        assert_eq!(kr.name.ko.chars().count(), 3, "{:?}", kr.name);
        let jp = maker.make_from(10000, "JP", &age, on, &mut rng);
        assert!(jp.name.ko.contains(' ') && jp.name.en.contains(' '));
    }
}
