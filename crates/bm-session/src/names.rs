//! Generated names for players and the fictional clubs, from the editable data
//! files under `data/`.

use bm_engine::Rng;
use serde::{Deserialize, Serialize};

const KR_LEAGUE: &str = include_str!("../../../data/leagues/kr.json");

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
    /// League the club plays in (`kr-top`, `kr-second`, ...).
    #[serde(default)]
    pub league: String,
}

#[derive(Deserialize)]
struct LeagueFile {
    league: Localized,
    clubs: Vec<ClubInfo>,
}

/// Source of player names.
pub struct NameGenerator {
    book: bm_world::names::KoreanBook,
}

impl NameGenerator {
    pub fn korean() -> Self {
        NameGenerator { book: bm_world::names::KoreanBook::bundled() }
    }

    /// A full name in Korean and in Latin script.
    pub fn next(&self, rng: &mut Rng) -> Localized {
        let t = self.book.draw(rng);
        Localized { ko: t.ko, en: t.en }
    }
}

/// The fictional first-team clubs of the Korean league and the league's name.
pub fn korean_league() -> (Localized, Vec<ClubInfo>) {
    let file: LeagueFile = serde_json::from_str(KR_LEAGUE).expect("data/leagues/kr.json is valid");
    (file.league, file.clubs.into_iter().filter(|c| c.league == "kr-top").collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_files_load_and_names_look_right() {
        let (league, clubs) = korean_league();
        assert!(!league.ko.is_empty() && clubs.len() == 10, "{} first-team clubs", clubs.len());
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

    /// The league files are edited by hand: every club must point at a league, a
    /// division and a parent club that exist, ids must be unique and parks complete.
    #[test]
    fn league_files_are_consistent() {
        let files =
            [("kr", KR_LEAGUE, 22), ("us", include_str!("../../../data/leagues/us.json"), 150), ("jp", include_str!("../../../data/leagues/jp.json"), 26)];
        for (name, text, expected) in files {
            let v: serde_json::Value = serde_json::from_str(text).unwrap_or_else(|e| panic!("data/leagues/{name}.json: {e}"));
            let leagues = v["leagues"].as_array().expect("leagues");
            let clubs = v["clubs"].as_array().expect("clubs");
            assert_eq!(clubs.len(), expected, "{name}: number of clubs");
            let league_ids: Vec<&str> = leagues.iter().map(|l| l["id"].as_str().unwrap()).collect();
            let division_ids: Vec<&str> = leagues.iter().flat_map(|l| l["divisions"].as_array().unwrap().iter().map(|d| d["id"].as_str().unwrap())).collect();
            let club_ids: Vec<&str> = clubs.iter().map(|c| c["id"].as_str().unwrap()).collect();
            let mut sorted = club_ids.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), club_ids.len(), "{name}: club ids are unique");
            for c in clubs {
                let id = c["id"].as_str().unwrap();
                assert!(league_ids.contains(&c["league"].as_str().unwrap()), "{name}/{id}: unknown league");
                if let Some(d) = c["division"].as_str() {
                    assert!(division_ids.contains(&d), "{name}/{id}: unknown division {d}");
                }
                if let Some(p) = c["parent"].as_str() {
                    assert!(club_ids.contains(&p), "{name}/{id}: unknown parent {p}");
                }
                assert!(!c["name"]["ko"].as_str().unwrap().is_empty() && !c["city"]["en"].as_str().unwrap().is_empty(), "{name}/{id}: names");
                let fence = c["park"]["fence_m"].as_array().expect("fence");
                assert_eq!(fence.len(), 5, "{name}/{id}: five fence distances");
                for f in fence {
                    let m = f.as_f64().unwrap();
                    assert!((85.0..135.0).contains(&m), "{name}/{id}: fence distance {m} m");
                }
                assert_eq!(c["park"]["wall_m"].as_array().map(|w| w.len()), Some(3), "{name}/{id}: wall heights");
            }
        }
    }
}
