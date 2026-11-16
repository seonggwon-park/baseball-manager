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
use crate::player::{Player, Position, Role};

/// Runs per game per standard deviation when all nine batters change.
pub const BATTING: [(Attr, f64); 16] = [
    (Attr::Contact, 0.295),
    (Attr::Timing, 0.102),
    (Attr::Power, 0.835),
    (Attr::Discipline, 0.206),
    (Attr::Recognition, 0.102),
    (Attr::BatControl, 0.225),
    (Attr::Bunt, 0.0),
    (Attr::Baserunning, 0.033),
    (Attr::Stealing, 0.0),
    (Attr::Speed, 0.410),
    (Attr::Acceleration, 0.150),
    (Attr::Aggression, 0.022),
    (Attr::Anticipation, 0.086),
    (Attr::Concentration, 0.023),
    (Attr::Composure, 0.010),
    (Attr::Strength, 0.322),
];

/// Runs per game per standard deviation when all eight fielders change.
pub const FIELDING: [(Attr, f64); 7] = [
    (Attr::Hands, 0.020),
    (Attr::Range, 0.184),
    (Attr::Reads, 0.140),
    (Attr::ArmStrength, 0.257),
    (Attr::ArmAccuracy, 0.0),
    (Attr::Footwork, 0.132),
    (Attr::Agility, 0.138),
];

/// Runs per game per standard deviation of the catcher alone (the experiment changed
/// every fielder, but only the catcher frames, blocks and throws out runners).
pub const CATCHING: [(Attr, f64); 3] = [(Attr::Framing, 0.006), (Attr::Blocking, 0.010), (Attr::ThrowingOut, 0.015)];

/// Runs per game per standard deviation when the whole staff changes.
pub const PITCHING: [(Attr, f64); 9] = [
    (Attr::Velocity, 0.075),
    (Attr::Stuff, 0.095),
    (Attr::Control, 0.084),
    (Attr::Command, 0.088),
    (Attr::Deception, 0.049),
    (Attr::HoldRunners, 0.040),
    (Attr::Stamina, 0.034),
    (Attr::Concentration, 0.022),
    (Attr::Composure, 0.060),
];

/// Runs per game when the grade of every pitch other than the main fastball moves by
/// one standard deviation.
pub const PITCH_GRADES: f64 = 0.203;

/// Share of a club's innings thrown by one starter and by one reliever: MLB starters
/// threw 57 % of the innings in 2026 (Verducci, Sports Illustrated, 2026-08-10), spread
/// here over a five-man rotation, and relievers the other 43 % over eight.
pub const STARTER_SHARE: f64 = 0.57 / 5.0;
pub const RELIEVER_SHARE: f64 = 0.43 / 8.0;

/// How much each position's fielding counts against the average of the eight. Until
/// they are measured position by position with the engine, every position counts the
/// same.
pub fn position_weight(pos: Position) -> f64 {
    match pos {
        Position::Pitcher => 0.0,
        _ => 1.0,
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
    let per_game: f64 = FIELDING.iter().map(|&(a, w)| w * (p.attrs.z(a) - unfamiliar(a, pos) * lost)).sum::<f64>() * position_weight(pos);
    let mut runs = per_game * 162.0 / 8.0;
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

/// Runs per game a whole staff of pitchers like him would save, per inning share 1.
pub fn pitching_rate(p: &Player) -> f64 {
    let mut per_game: f64 = PITCHING.iter().map(|&(a, w)| w * p.attrs.z(a)).sum();
    if let Some(pit) = &p.pitching {
        let main = convert::main_fastball(&pit.pitches);
        let (mut sum, mut weight) = (0.0, 0.0);
        for (i, k) in pit.pitches.iter().enumerate() {
            if Some(i) != main {
                sum += (k.usage * k.grade) as f64;
                weight += k.usage as f64;
            }
        }
        if weight > 0.0 {
            per_game += PITCH_GRADES * sum / weight;
        }
    }
    per_game
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

/// Runs per 162 games that one standard deviation on every attribute that counts is
/// worth to a player in his best role: the unit that puts position players, starters and
/// relievers on one scale (about 68, 14 and 6.5).
pub fn per_sd(p: &Player) -> f64 {
    if p.is_pitcher() {
        let rate: f64 = PITCHING.iter().map(|w| w.1).sum::<f64>() + PITCH_GRADES;
        rate * 162.0 * if can_start(p) { STARTER_SHARE } else { RELIEVER_SHARE }
    } else {
        BATTING.iter().map(|w| w.1).sum::<f64>() * 162.0 / 9.0 + FIELDING.iter().map(|w| w.1).sum::<f64>() * 162.0 / 8.0
    }
}

/// A player's value in his best role, runs per 162 games: the yardstick for ranking
/// players of the same kind (pitchers against pitchers, position players against
/// position players).
pub fn value(p: &Player) -> f64 {
    if p.is_pitcher() {
        pitching(p, if can_start(p) { Role::Starter } else { Role::Reliever })
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
        // A standard deviation of power is worth 15 runs a season in one lineup spot.
        p.attrs[Attr::Power] = 1.0;
        assert!((batting(&p) - 15.03).abs() < 0.01, "{}", batting(&p));
        // Out of position he loses a lot; a non-catcher cannot catch.
        assert!(fielding(&p, Some(Position::Center)) < fielding(&p, Some(Position::Short)) - 5.0);
        assert!(fielding(&p, Some(Position::Catcher)) < -20.0);
        assert_eq!(best_position(&p).0, Some(Position::Short));
    }

    #[test]
    fn minor_leaguers_are_worth_less_than_major_leaguers() {
        let mut rng = Rng::new(9);
        let aaa = Talent { hitting: -0.75, pitching: -0.66, fielding: -1.1, physical: -0.2, power: 0.37, command: -1.9 };
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
