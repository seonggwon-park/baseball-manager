//! Generated player names. Every name exists in two forms: the one the Korean screen
//! shows (Hangul) and the one the English screen shows (Latin script).
//!
//! Name parts live in the editable files `data/names/*.json`. Korean names are built
//! from a surname and two given-name syllables; other name books hold whole surnames
//! and given names with their Hangul spelling.

use crate::league::Text;
use bm_engine::Rng;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Part {
    /// Hangul spelling.
    pub ko: String,
    /// Latin-script spelling.
    pub en: String,
    /// Relative frequency.
    #[serde(default = "one")]
    pub w: f64,
}

fn one() -> f64 {
    1.0
}

/// Which part comes first when the name is written out.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    GivenFirst,
    SurnameFirst,
}

/// Name parts of one naming tradition.
#[derive(Clone, Debug, Deserialize)]
pub struct NameBook {
    /// `en`, `es`, `ja`, ...
    pub locale: String,
    /// Order on the Korean and on the English screen (a Japanese name is written
    /// surname first in Korean and given name first in English).
    pub order_ko: Order,
    pub order_en: Order,
    pub surnames: Vec<Part>,
    pub given: Vec<Part>,
    #[serde(skip)]
    weights: (Vec<f64>, Vec<f64>),
}

const EN: &str = include_str!("../../../data/names/en.json");
const ES: &str = include_str!("../../../data/names/es.json");
const JA: &str = include_str!("../../../data/names/ja.json");

impl NameBook {
    /// The name book bundled with the program for `locale` (`en`, `es`, `ja`).
    pub fn bundled(locale: &str) -> Option<NameBook> {
        let text = match locale {
            "en" => EN,
            "es" => ES,
            "ja" => JA,
            _ => return None,
        };
        Some(NameBook::from_json(text).unwrap_or_else(|e| panic!("data/names/{locale}.json: {e}")))
    }

    pub fn from_json(text: &str) -> Result<NameBook, serde_json::Error> {
        let mut b: NameBook = serde_json::from_str(text)?;
        b.weights = (b.surnames.iter().map(|p| p.w).collect(), b.given.iter().map(|p| p.w).collect());
        assert!(!b.surnames.is_empty() && !b.given.is_empty(), "name book '{}' is empty", b.locale);
        Ok(b)
    }

    pub fn draw(&self, rng: &mut Rng) -> Text {
        let s = &self.surnames[rng.pick_weighted(&self.weights.0)];
        let g = &self.given[rng.pick_weighted(&self.weights.1)];
        let join = |order: Order, s: &str, g: &str| match order {
            Order::GivenFirst => format!("{g} {s}"),
            Order::SurnameFirst => format!("{s} {g}"),
        };
        Text { ko: join(self.order_ko, &s.ko, &g.ko), en: join(self.order_en, &s.en, &g.en) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_books_load() {
        for locale in ["en", "es", "ja"] {
            let book = NameBook::bundled(locale).unwrap();
            assert!(book.surnames.len() >= 50 && book.given.len() >= 50, "{locale}");
            for p in book.surnames.iter().chain(&book.given) {
                assert!(!p.ko.is_empty() && !p.en.is_empty() && p.w > 0.0, "{locale}: {p:?}");
                assert!(p.ko.chars().all(|c| ('가'..='힣').contains(&c) || c == ' '), "{locale}: Hangul spelling of {}", p.en);
            }
        }
        assert!(NameBook::bundled("xx").is_none());
    }

    #[test]
    fn names_are_written_in_the_order_of_each_screen() {
        let book = NameBook::from_json(
            r#"{"locale": "ja", "order_ko": "surname_first", "order_en": "given_first",
                "surnames": [{"ko": "다나카", "en": "Tanaka", "w": 3}, {"ko": "사토", "en": "Sato"}],
                "given": [{"ko": "쇼타", "en": "Shota"}]}"#,
        )
        .unwrap();
        let mut rng = Rng::new(1);
        let mut tanaka = 0;
        for _ in 0..4000 {
            let n = book.draw(&mut rng);
            if n.ko == "다나카 쇼타" {
                assert_eq!(n.en, "Shota Tanaka");
                tanaka += 1;
            } else {
                assert_eq!((n.ko.as_str(), n.en.as_str()), ("사토 쇼타", "Shota Sato"));
            }
        }
        // Weights 3 : 1.
        assert!((2800..3200).contains(&tanaka), "{tanaka}");
    }
}
