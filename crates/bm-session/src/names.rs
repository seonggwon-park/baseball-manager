//! Generated names for players and the fictional clubs, from the editable data
//! files under `data/`.

use bm_engine::Rng;
use serde::{Deserialize, Serialize};

const KO_NAMES: &str = include_str!("../../../data/names/ko.json");
const KR_LEAGUE: &str = include_str!("../../../data/leagues/kr.json");

#[derive(Deserialize)]
struct NamePart {
    n: String,
    en: String,
    #[serde(default = "one")]
    w: f64,
}

fn one() -> f64 {
    1.0
}

#[derive(Deserialize)]
struct KoreanNames {
    surnames: Vec<NamePart>,
    given_first: Vec<NamePart>,
    given_second: Vec<NamePart>,
}

/// Text in the languages the interface knows about.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Localized {
    pub ko: String,
    pub en: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ClubInfo {
    pub id: String,
    pub city: Localized,
    pub name: Localized,
    pub abbr: String,
    /// Primary and secondary colour, `#rrggbb`.
    pub colors: [String; 2],
}

#[derive(Deserialize)]
struct LeagueFile {
    league: Localized,
    clubs: Vec<ClubInfo>,
}

/// Source of player names.
pub struct NameGenerator {
    names: KoreanNames,
    weights: Vec<f64>,
}

impl NameGenerator {
    pub fn korean() -> Self {
        let names: KoreanNames = serde_json::from_str(KO_NAMES).expect("data/names/ko.json is valid");
        let weights = names.surnames.iter().map(|s| s.w).collect();
        NameGenerator { names, weights }
    }

    /// A full name in Korean and in Latin script.
    pub fn next(&self, rng: &mut Rng) -> Localized {
        let s = &self.names.surnames[rng.pick_weighted(&self.weights)];
        let a = &self.names.given_first[rng.below(self.names.given_first.len())];
        let mut b = &self.names.given_second[rng.below(self.names.given_second.len())];
        // No doubled syllables.
        if b.n == a.n {
            b = &self.names.given_second[(rng.below(self.names.given_second.len() - 1) + 1) % self.names.given_second.len()];
        }
        let mut given_en = format!("{}-{}", a.en, b.en);
        if let Some(first) = given_en.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        Localized { ko: format!("{}{}{}", s.n, a.n, b.n), en: format!("{} {}", given_en, s.en) }
    }
}

/// The fictional clubs of the Korean league and the league's name.
pub fn korean_league() -> (Localized, Vec<ClubInfo>) {
    let file: LeagueFile = serde_json::from_str(KR_LEAGUE).expect("data/leagues/kr.json is valid");
    (file.league, file.clubs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_files_load_and_names_look_right() {
        let (league, clubs) = korean_league();
        assert!(!league.ko.is_empty() && clubs.len() >= 2);
        let mut ids: Vec<&str> = clubs.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), clubs.len(), "club ids are unique");
        for c in &clubs {
            assert!(c.colors.iter().all(|x| x.len() == 7 && x.starts_with('#')));
        }
        let names = NameGenerator::korean();
        let mut rng = Rng::new(3);
        let mut kims = 0;
        for _ in 0..2000 {
            let n = names.next(&mut rng);
            assert_eq!(n.ko.chars().count(), 3);
            assert!(n.en.contains(' ') && n.en.contains('-'));
            if n.ko.starts_with('김') {
                kims += 1;
            }
        }
        // About one Korean in five is a Kim.
        assert!((300..560).contains(&kims), "{kims}");
    }
}
