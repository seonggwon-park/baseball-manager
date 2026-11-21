//! Where generated players were born and which name book names them, from the editable
//! file `data/people/birthplaces.json` (shares by level from MLB's Opening Day releases,
//! minor-league rosters and the KBO and NPB foreign players: `docs/research/world/nationality.md`).

use crate::league::Text;
use bm_engine::Rng;
use serde::Deserialize;
use std::collections::BTreeMap;

const BIRTHPLACES: &str = include_str!("../../../data/people/birthplaces.json");

#[derive(Deserialize)]
struct Group {
    weights: BTreeMap<String, f64>,
}

#[derive(Deserialize)]
struct File {
    groups: BTreeMap<String, Group>,
    locales: BTreeMap<String, String>,
    countries: BTreeMap<String, Text>,
}

/// Birthplace shares of every group of players (`mlb`, `aaa`, ..., `kbo_foreign`).
pub struct Birthplaces {
    groups: BTreeMap<String, (Vec<String>, Vec<f64>)>,
    locales: BTreeMap<String, String>,
    countries: BTreeMap<String, Text>,
}

impl Birthplaces {
    pub fn bundled() -> Birthplaces {
        Birthplaces::from_json(BIRTHPLACES).unwrap_or_else(|e| panic!("data/people/birthplaces.json: {e}"))
    }

    pub fn from_json(text: &str) -> Result<Birthplaces, serde_json::Error> {
        let f: File = serde_json::from_str(text)?;
        let groups = f.groups.into_iter().map(|(name, g)| (name, g.weights.into_iter().filter(|(_, w)| *w > 0.0).unzip())).collect();
        Ok(Birthplaces { groups, locales: f.locales, countries: f.countries })
    }

    /// Country code (ISO 3166-1 alpha-2) of a player of `group`.
    pub fn draw(&self, group: &str, rng: &mut Rng) -> &str {
        let (codes, weights) = self.groups.get(group).unwrap_or_else(|| panic!("no birthplace group '{group}'"));
        &codes[rng.pick_weighted(weights)]
    }

    /// Share of `country` in `group`.
    pub fn share(&self, group: &str, country: &str) -> f64 {
        let Some((codes, weights)) = self.groups.get(group) else { return 0.0 };
        let total: f64 = weights.iter().sum();
        codes.iter().zip(weights).filter(|(c, _)| *c == country).map(|(_, w)| w).sum::<f64>() / total
    }

    /// The name book (`en`, `es`, `ja`, `ko`, `zh`) for players born in `country`.
    pub fn locale(&self, country: &str) -> &str {
        self.locales.get(country).map_or("en", String::as_str)
    }

    /// The country's name on the Korean and the English screen.
    pub fn country_name(&self, country: &str) -> Option<&Text> {
        self.countries.get(country)
    }

    pub fn groups(&self) -> impl Iterator<Item = &str> {
        self.groups.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::Names;

    #[test]
    fn every_birthplace_has_a_name_book_and_a_name() {
        let b = Birthplaces::bundled();
        for g in b.groups() {
            let (codes, _) = &b.groups[g];
            assert!(!codes.is_empty(), "{g}");
            for c in codes {
                assert!(b.locales.contains_key(c), "{g}: no name book for {c}");
                assert!(Names::bundled(b.locale(c)).is_some(), "{g}: name book '{}' for {c}", b.locale(c));
                let n = b.country_name(c).unwrap_or_else(|| panic!("{g}: no name for {c}"));
                assert!(!n.ko.is_empty() && !n.en.is_empty());
            }
        }
    }

    #[test]
    fn shares_follow_the_rosters() {
        let b = Birthplaces::bundled();
        // 2026 Opening Day: 699 of 948 born in the 50 states.
        assert!((b.share("mlb", "US") - 699.0 / 948.0).abs() < 1e-9);
        // The Dominican academies are almost all Dominican and Venezuelan.
        assert!(b.share("academy", "DO") + b.share("academy", "VE") > 0.8);
        assert_eq!(b.share("kbo_domestic", "KR"), 1.0);
        let mut rng = Rng::new(9);
        let n = 20_000;
        let us = (0..n).filter(|_| b.draw("single_a", &mut rng) == "US").count() as f64 / n as f64;
        assert!((us - b.share("single_a", "US")).abs() < 0.015, "{us}");
    }
}
