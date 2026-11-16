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
const ZH: &str = include_str!("../../../data/names/zh.json");
const KO: &str = include_str!("../../../data/names/ko.json");

impl NameBook {
    /// The name book bundled with the program for `locale` (`en`, `es`, `ja`, `zh`).
    pub fn bundled(locale: &str) -> Option<NameBook> {
        let text = match locale {
            "en" => EN,
            "es" => ES,
            "ja" => JA,
            "zh" => ZH,
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

/// A syllable or surname of a Korean name: Hangul and its romanisation.
#[derive(Clone, Debug, Deserialize)]
struct KoreanPart {
    n: String,
    en: String,
    #[serde(default = "one")]
    w: f64,
}

/// Korean names: a surname and two given-name syllables (`data/names/ko.json`). The
/// Korean screen writes them together ("김민준"), the English screen given name first
/// with a hyphen ("Min-jun Kim").
#[derive(Clone, Debug, Deserialize)]
pub struct KoreanBook {
    surnames: Vec<KoreanPart>,
    given_first: Vec<KoreanPart>,
    given_second: Vec<KoreanPart>,
    #[serde(skip)]
    weights: Vec<f64>,
}

impl KoreanBook {
    pub fn bundled() -> KoreanBook {
        let mut b: KoreanBook = serde_json::from_str(KO).unwrap_or_else(|e| panic!("data/names/ko.json: {e}"));
        b.weights = b.surnames.iter().map(|p| p.w).collect();
        assert!(!b.surnames.is_empty() && !b.given_first.is_empty() && b.given_second.len() >= 2, "data/names/ko.json is incomplete");
        b
    }

    pub fn draw(&self, rng: &mut Rng) -> Text {
        let s = &self.surnames[rng.pick_weighted(&self.weights)];
        let a = &self.given_first[rng.below(self.given_first.len())];
        let mut b = &self.given_second[rng.below(self.given_second.len())];
        // No doubled syllables.
        if b.n == a.n {
            b = &self.given_second[(rng.below(self.given_second.len() - 1) + 1) % self.given_second.len()];
        }
        let mut given = format!("{}-{}", a.en, b.en.to_lowercase());
        if let Some(first) = given.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        Text { ko: format!("{}{}{}", s.n, a.n, b.n), en: format!("{given} {}", s.en) }
    }
}

/// Any of the bundled naming traditions.
#[derive(Clone, Debug)]
pub enum Names {
    Book(NameBook),
    Korean(KoreanBook),
}

impl Names {
    /// `en`, `es`, `ja`, `zh` or `ko`.
    pub fn bundled(locale: &str) -> Option<Names> {
        if locale == "ko" {
            return Some(Names::Korean(KoreanBook::bundled()));
        }
        NameBook::bundled(locale).map(Names::Book)
    }

    pub fn draw(&self, rng: &mut Rng) -> Text {
        match self {
            Names::Book(b) => b.draw(rng),
            Names::Korean(k) => k.draw(rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hangul(s: &str) -> bool {
        s.chars().all(|c| ('가'..='힣').contains(&c) || c == ' ')
    }

    #[test]
    fn taiwanese_and_korean_names_look_right() {
        let zh = NameBook::bundled("zh").unwrap();
        assert!(zh.surnames.len() >= 20 && zh.given.len() >= 20);
        for p in zh.surnames.iter().chain(&zh.given) {
            assert!(hangul(&p.ko) && !p.en.is_empty() && p.w > 0.0, "{p:?}");
        }
        let mut rng = Rng::new(3);
        let ko = Names::bundled("ko").unwrap();
        for _ in 0..500 {
            let n = ko.draw(&mut rng);
            let syllables: Vec<char> = n.ko.chars().collect();
            assert!(syllables.len() == 3 && hangul(&n.ko), "{}", n.ko);
            assert_ne!(syllables[1], syllables[2], "{}", n.ko);
            let (given, surname) = n.en.split_once(' ').unwrap();
            assert!(given.contains('-') && given.chars().next().unwrap().is_ascii_uppercase() && !surname.is_empty(), "{}", n.en);
        }
        let n = Names::bundled("zh").unwrap().draw(&mut rng);
        assert!(n.ko.contains(' ') && n.en.contains(' '), "{n:?}");
        assert!(Names::bundled("xx").is_none());
    }

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
