//! What a player is worth: runs per 162 games against an average major-leaguer in the
//! same role, from the match engine's own value of each attribute.
//!
//! The weights are the "per SD" column of `bm attributes --games 8000`
//! (`docs/attributes.md` 5): how many runs per game a club gains when every one of its
//! batters, fielders or pitchers is one standard deviation better at one attribute.
//! Estimates below zero are all within about one standard error (0.025 runs per game)
//! of zero and are counted as zero, so that being better at something never makes a
//! player worth less. The value is linear in the attributes; interactions between them
//! are left out.

use crate::attr::Attr;
use crate::convert;
use crate::player::{Player, Position, Role, Side};
use bm_engine::PitchKind;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const PITCH_VALUE: &str = include_str!("../../../data/people/pitch_value.json");

/// Runs per game per standard deviation when all nine batters change.
pub const BATTING: [(Attr, f64); 16] = [
    (Attr::Contact, 0.264),
    (Attr::Timing, 0.102),
    (Attr::Power, 0.757),
    (Attr::Discipline, 0.146),
    (Attr::Recognition, 0.138),
    (Attr::BatControl, 0.249),
    (Attr::Bunt, 0.0),
    (Attr::Baserunning, 0.053),
    (Attr::Stealing, 0.005),
    (Attr::Speed, 0.415),
    (Attr::Acceleration, 0.239),
    (Attr::Aggression, 0.0),
    (Attr::Anticipation, 0.0),
    (Attr::Concentration, 0.019),
    (Attr::Composure, 0.043),
    (Attr::Strength, 0.289),
];

/// Runs per game per standard deviation when all eight fielders change. Fielding is
/// valued position by position ([`BY_POSITION`]); these split each position's glove
/// between its five attributes.
pub const FIELDING: [(Attr, f64); 7] = [
    (Attr::Hands, 0.018),
    (Attr::Range, 0.169),
    (Attr::Reads, 0.142),
    (Attr::ArmStrength, 0.226),
    (Attr::ArmAccuracy, 0.078),
    (Attr::Footwork, 0.164),
    (Attr::Agility, 0.113),
];

/// Runs per game per standard deviation of the catcher alone (the experiment changed
/// every fielder, but only the catcher frames, blocks and throws out runners).
pub const CATCHING: [(Attr, f64); 3] = [(Attr::Framing, 0.067), (Attr::Blocking, 0.0), (Attr::ThrowingOut, 0.015)];

/// Share of a club's innings thrown by one starter and by one reliever: MLB starters
/// threw 57 % of the innings in 2026 (Verducci, Sports Illustrated, 2026-08-10), spread
/// here over a five-man rotation, and relievers the other 43 % over eight.
pub const STARTER_SHARE: f64 = 0.57 / 5.0;
pub const RELIEVER_SHARE: f64 = 0.43 / 8.0;
/// Share of a team's innings one pitcher of a 13-man staff throws on average: the unit that
/// ranks starters and relievers on one scale. The roles differ in innings, not in how well
/// a pitcher gets batters out, and runs counted over a role's innings would rank a reliever
/// ahead of a better starter whenever both are below the major-league average (the
/// starter's deficit counts over twice the innings), which empties low-level staffs of
/// starters.
pub const STAFF_SHARE: f64 = 1.0 / 13.0;

/// Runs per game per standard deviation of the regular at one position: his glove (range,
/// reads, footwork, agility and hands moved together), his arm strength and his top speed,
/// in the field only (`bm attributes --by-position --games 8000`, steps of 2 SD; standard
/// error about 0.012; estimates below zero counted as zero). Summed over the eight
/// positions they come close to the values measured for all fielders at once (glove 0.642
/// against 0.679, arm 0.243 against 0.195), and speed is worth 0.201 runs per game in the
/// field on top of its 0.418 at the plate. The engine values the glove at shortstop as at
/// third base and in centre field below the corners, where real fielders' positional
/// adjustment puts shortstop and centre field highest (`docs/attributes.md` 6).
pub const BY_POSITION: [(Position, f64, f64, f64); 8] = [
    (Position::Catcher, 0.0, 0.059, 0.0),
    (Position::First, 0.047, 0.012, 0.0),
    (Position::Second, 0.095, 0.017, 0.011),
    (Position::Third, 0.102, 0.034, 0.013),
    (Position::Short, 0.102, 0.029, 0.002),
    (Position::Left, 0.097, 0.012, 0.070),
    (Position::Center, 0.085, 0.022, 0.046),
    (Position::Right, 0.114, 0.058, 0.059),
];

/// The glove attributes and their shares of the glove (from the all-fielder values).
const GLOVE: [Attr; 5] = [Attr::Range, Attr::Reads, Attr::Footwork, Attr::Agility, Attr::Hands];

fn weight(attr: Attr) -> f64 {
    FIELDING.iter().find(|w| w.0 == attr).map_or(0.0, |w| w.1)
}

/// Runs per game per standard deviation of `attr` for the regular at `pos`.
pub fn fielding_weight(attr: Attr, pos: Position) -> f64 {
    let Some(&(_, glove, arm, speed)) = BY_POSITION.iter().find(|r| r.0 == pos) else { return 0.0 };
    match attr {
        Attr::ArmStrength => arm,
        Attr::Speed => speed,
        a if GLOVE.contains(&a) => glove * weight(a) / GLOVE.iter().map(|&g| weight(g)).sum::<f64>(),
        _ => 0.0,
    }
}

/// Fielding lost at a position a player does not know, in standard deviations of each
/// attribute at zero familiarity: the engine adds 0.10 s to his reaction (0.03 s per
/// standard deviation of range), 0.10 s to an outfielder's read and 0.04 to his route
/// (0.035 s and 0.016 per standard deviation of reads), 0.15 s to his transfer (0.06 s
/// per standard deviation of footwork) and doubles or more his errors
/// (`convert::fielder`).
fn unfamiliar(attr: Attr, pos: Position) -> f64 {
    match attr {
        Attr::Range => 0.10 / convert::spread::REACTION,
        Attr::Reads if pos.is_outfield() => 0.10 / convert::spread::READ_DELAY + 0.04 / convert::spread::ROUTE,
        Attr::Reads => 0.04 / convert::spread::ROUTE,
        Attr::Footwork => 0.15 / convert::spread::TRANSFER,
        Attr::Hands => 1.0,
        _ => 0.0,
    }
}

/// Runs per 162 games as a regular in the lineup, from batting and running.
pub fn batting(p: &Player) -> f64 {
    let per_game: f64 = BATTING.iter().map(|&(a, w)| w * p.attrs.z(a)).sum();
    per_game * 162.0 / 9.0
}

/// Runs per 162 games as the regular at `pos`, from fielding. Zero for a designated
/// hitter, who does not field.
pub fn fielding(p: &Player, pos: Option<Position>) -> f64 {
    let Some(pos) = pos else { return 0.0 };
    if pos == Position::Pitcher {
        return 0.0;
    }
    let lost = 1.0 - p.familiarity(pos);
    let per_game: f64 =
        GLOVE.iter().chain(&[Attr::ArmStrength, Attr::Speed]).map(|&a| fielding_weight(a, pos) * (p.attrs.z(a) - unfamiliar(a, pos) * lost)).sum();
    let mut runs = per_game * 162.0;
    if pos == Position::Catcher {
        // Catching skills, and a big penalty for anyone who is not a catcher.
        runs += CATCHING.iter().map(|&(a, w)| w * p.attrs.z(a)).sum::<f64>() * 162.0;
        runs -= 25.0 * lost;
    }
    runs
}

/// Runs per 162 games of a position player at `pos`: batting and fielding.
pub fn at_position(p: &Player, pos: Option<Position>) -> f64 {
    batting(p) + fielding(p, pos)
}

/// Runs per 162 games that playing a position is worth against an average position: the
/// positional adjustment of FanGraphs WAR (Sabermetrics Library, "Positional
/// Adjustment"). Fielding is measured against an average fielder at the same position, so
/// this is what puts a shortstop and a first baseman of the same bat on one scale.
pub fn positional(pos: Option<Position>) -> f64 {
    match pos {
        Some(Position::Catcher) => 12.5,
        Some(Position::Short) => 7.5,
        Some(Position::Second | Position::Third | Position::Center) => 2.5,
        Some(Position::Left | Position::Right) => -7.5,
        Some(Position::First) => -12.5,
        Some(Position::Pitcher) => 0.0,
        None => -17.5,
    }
}

/// The best place for a position player and what he is worth there, positional
/// adjustment included.
pub fn best_position(p: &Player) -> (Option<Position>, f64) {
    let mut best = (None, at_position(p, None) + positional(None));
    for pos in Position::ALL.into_iter().skip(1) {
        let v = at_position(p, Some(pos)) + positional(Some(pos));
        if v > best.1 {
            best = (Some(pos), v);
        }
    }
    best
}

/// The pitching value model (decision 2026-11-18: a linear model of the arsenal): runs allowed
/// per nine innings, linear in the pitcher's attributes, his delivery and, pitch type by pitch
/// type, how often he throws it and how often times its grade and its own shape, fitted to
/// the engine's results (`tools/value/fit_pitching.py`, `data/people/pitch_value.json`).
/// Each feature has the mean it is measured from and its weight in runs per nine per unit.
#[derive(Deserialize)]
struct PitchValueFile {
    features: BTreeMap<String, (f64, f64)>,
    /// Runs a game for a staff: the pitchers' unit of `per_sd`.
    per_sd: f64,
}

/// The same, laid out for the computation.
struct PitchValue {
    per_sd: f64,
    attrs: Vec<(Attr, f64, f64)>,
    left: (f64, f64),
    delivery: [(f64, f64); 5],
    n_pitches: (f64, f64),
    /// By pitch type: usage share, then usage times grade, speed, spin, active spin, tilt and
    /// height.
    kinds: [[(f64, f64); 7]; 8],
}

/// The pitch fields of the model, after the usage share.
pub const PITCH_FIELDS: [&str; 6] = ["grade", "speed", "spin", "active", "tilt", "height"];
/// The delivery fields of the model.
pub const DELIVERY: [&str; 5] = ["arm_angle", "extension", "speed_sd", "spin_sd", "tilt_sd"];

fn pitch_value() -> &'static PitchValue {
    static MODEL: OnceLock<PitchValue> = OnceLock::new();
    MODEL.get_or_init(|| {
        let f: PitchValueFile = serde_json::from_str(PITCH_VALUE).expect("bundled pitch_value.json");
        let get = |k: &str| f.features.get(k).copied().unwrap_or((0.0, 0.0));
        let attrs = Attr::ALL.iter().filter_map(|&a| f.features.get(a.key()).map(|&(m, w)| (a, m, w))).collect();
        let mut kinds = [[(0.0, 0.0); 7]; 8];
        for kind in PitchKind::ALL {
            let code = kind.code();
            kinds[kind.index()][0] = get(&format!("use_{code}"));
            for (i, field) in PITCH_FIELDS.iter().enumerate() {
                kinds[kind.index()][i + 1] = get(&format!("{code}_{field}"));
            }
        }
        PitchValue { per_sd: f.per_sd, attrs, left: get("left"), delivery: DELIVERY.map(get), n_pitches: get("n_pitches"), kinds }
    })
}

/// Runs per game a whole staff of pitchers like him would save, per inning share 1: the
/// pitching value model's runs allowed per nine against its mean, with the sign turned.
pub fn pitching_rate(p: &Player) -> f64 {
    let Some(pit) = &p.pitching else { return 0.0 };
    let m = pitch_value();
    let term = |x: f64, (mean, w): (f64, f64)| w * (x - mean);
    let mut ra9: f64 = m.attrs.iter().map(|&(a, mean, w)| w * (p.attrs.z(a) - mean)).sum();
    ra9 += term(f64::from(u8::from(p.throws == Side::Left)), m.left);
    let delivery = [pit.arm_angle, pit.extension, pit.speed_sd, pit.spin_sd, pit.tilt_sd];
    ra9 += delivery.iter().zip(m.delivery).map(|(&x, c)| term(f64::from(x), c)).sum::<f64>();
    ra9 += term(pit.pitches.len() as f64, m.n_pitches);
    let total: f64 = pit.pitches.iter().map(|k| f64::from(k.usage)).sum();
    for kind in PitchKind::ALL {
        let of_kind = || pit.pitches.iter().filter(|k| k.kind == kind);
        let usage = if total > 0.0 { of_kind().map(|k| f64::from(k.usage)).sum::<f64>() / total } else { 0.0 };
        let c = &m.kinds[kind.index()];
        ra9 += term(usage, c[0]);
        let fields = of_kind().next().map_or([0.0; 6], |k| [k.grade, k.speed, k.spin, k.active, k.tilt, k.height].map(f64::from));
        ra9 += fields.iter().zip(&c[1..]).map(|(&x, &cw)| term(usage * x, cw)).sum::<f64>();
    }
    -ra9
}

/// Runs per 162 games as a starter or a reliever.
pub fn pitching(p: &Player, role: Role) -> f64 {
    let share = match role {
        Role::Starter => STARTER_SHARE,
        Role::Reliever => RELIEVER_SHARE,
    };
    pitching_rate(p) * 162.0 * share
}

/// Can he start: enough stamina to go through a lineup more than twice. Generated
/// starters sit near +2 standard deviations, relievers near -2.
pub fn can_start(p: &Player) -> bool {
    p.attrs.z(Attr::Stamina) > 0.0
}

/// The unit that puts position players and pitchers on one scale, runs per 162 games
/// (`value`): for a position player what one standard deviation on every attribute that
/// counts is worth (about 72; fielding averaged over the positions); for a pitcher the
/// pitching value model's `per_sd` (`data/people/pitch_value.json`), the earlier model's
/// unit (one standard deviation on every pitching attribute and pitch grade, 0.551 runs a
/// game for a staff) widened as far as the new model spreads major-league starters wider.
pub fn per_sd(p: &Player) -> f64 {
    if p.is_pitcher() {
        pitch_value().per_sd * 162.0 * STAFF_SHARE
    } else {
        let field: f64 = BY_POSITION.iter().map(|r| r.1 + r.2 + r.3).sum::<f64>() / 8.0;
        BATTING.iter().map(|w| w.1).sum::<f64>() * 162.0 / 9.0 + field * 162.0
    }
}

/// A player's value, runs per 162 games: a position player at his best position, a
/// pitcher over an average pitcher's share of the innings whatever his role (`STAFF_SHARE`).
/// The yardstick for ranking players of the same kind (pitchers against pitchers, position
/// players against position players); `per_sd` puts the two kinds on one scale.
pub fn value(p: &Player) -> f64 {
    if p.is_pitcher() {
        pitching_rate(p) * 162.0 * STAFF_SHARE
    } else {
        best_position(p).1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::{pitcher, position_player, Talent};
    use bm_engine::Rng;

    #[test]
    fn an_average_player_is_worth_nothing_and_better_is_worth_more() {
        let mut rng = Rng::new(4);
        let mut p = position_player(&mut rng, 1, Some(Position::Short), &Talent::MAJOR);
        p.attrs = Default::default();
        assert!(batting(&p).abs() < 1e-9);
        assert!(fielding(&p, Some(Position::Short)).abs() < 1e-9);
        // A standard deviation of power is worth 13.6 runs a season in one lineup spot.
        p.attrs[Attr::Power] = 1.0;
        assert!((batting(&p) - 13.626).abs() < 0.01, "{}", batting(&p));
        // Out of position he loses a lot; a non-catcher cannot catch.
        assert!(fielding(&p, Some(Position::Center)) < fielding(&p, Some(Position::Short)) - 5.0);
        assert!(fielding(&p, Some(Position::Catcher)) < -20.0);
        assert_eq!(best_position(&p).0, Some(Position::Short));
    }

    #[test]
    fn the_pitching_value_model_centres_on_the_majors_and_rewards_skill() {
        let mut rng = Rng::new(21);
        let starters: Vec<Player> = (0..600).map(|i| pitcher(&mut rng, i, Role::Starter, &Talent::MAJOR)).collect();
        let rates: Vec<f64> = starters.iter().map(pitching_rate).collect();
        let mean = rates.iter().sum::<f64>() / rates.len() as f64;
        let sd = (rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rates.len() as f64).sqrt();
        // Fitted on generated major-league starters: their mean is the model's zero, and they
        // spread about as widely as the fit's predictions (0.43 runs a game).
        assert!(mean.abs() < 0.08, "mean {mean:.3}");
        assert!(sd > 0.25 && sd < 0.65, "sd {sd:.3}");
        // More of any pitching skill saves runs.
        for a in [Attr::Velocity, Attr::Stuff, Attr::Control, Attr::Command, Attr::Deception] {
            let mut p = starters[0].clone();
            let before = pitching_rate(&p);
            p.attrs[a] += 1.0;
            assert!(pitching_rate(&p) > before, "{a:?}");
        }
        // The pitchers' unit is wider than one runs a game for a staff and the spread fits in it.
        assert!(pitch_value().per_sd > sd);
    }

    #[test]
    fn minor_leaguers_are_worth_less_than_major_leaguers() {
        let mut rng = Rng::new(9);
        let aaa = Talent {
            hitting: -0.75,
            pitching: -0.66,
            fielding: -1.1,
            physical: -0.2,
            power: 0.37,
            command: -1.9,
            velocity: 0.0,
            discipline: 0.0,
            spread: 1.0,
            tradeoff: 0.0,
        };
        let mean = |t: &Talent, rng: &mut Rng, pitchers: bool| -> f64 {
            (0..400)
                .map(|i| if pitchers { value(&pitcher(rng, i, Role::Starter, t)) } else { value(&position_player(rng, i, Some(Position::Second), t)) })
                .sum::<f64>()
                / 400.0
        };
        let (mlb_h, aaa_h) = (mean(&Talent::MAJOR, &mut rng, false), mean(&aaa, &mut rng, false));
        let (mlb_p, aaa_p) = (mean(&Talent::MAJOR, &mut rng, true), mean(&aaa, &mut rng, true));
        assert!(mlb_h > aaa_h + 10.0, "hitters {mlb_h:.1} vs {aaa_h:.1}");
        assert!(mlb_p > aaa_p + 5.0, "starters {mlb_p:.1} vs {aaa_p:.1}");
        // An average second baseman is worth the position's adjustment, a little more for
        // the ones who would be better elsewhere.
        assert!((mlb_h - 2.5).abs() < 6.0 && mlb_p.abs() < 3.0, "major-leaguers near zero: {mlb_h:.1} {mlb_p:.1}");
    }
}
