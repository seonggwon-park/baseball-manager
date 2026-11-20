//! Run expectancy by base state, outs and count: the runs the batting side goes on to score in
//! the rest of the half inning from the moment before a pitch.
//!
//! It is counted from games between the engine's AI managers (`bm re`), so it is the engine's
//! own table. The pitching environment pays each pitch its run value, the change of this
//! expectancy plus the runs that scored on the pitch, which is how Statcast values a pitch
//! (the run impact of an event from the runners on base, outs, balls and strikes). As a
//! potential-based reward it leaves the best policy for runs allowed unchanged whatever the
//! table's errors (Ng, Harada & Russell 1999).

use crate::league::League;
use crate::we::bases;
use bm_engine::{ManagerStyle, NoObserver, Rng, Step};
use serde::{Deserialize, Serialize};

/// Cells: base state (8) × outs (3) × balls (4) × strikes (3).
pub const CELLS: usize = 8 * 3 * 4 * 3;

const BUNDLED: &str = include_str!("../../../data/engine/run_expectancy.json");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunExpectancy {
    /// How the table was made.
    pub source: String,
    pub games: u64,
    /// Pitches thrown from each cell, and the runs scored from there to the end of the half.
    pub n: Vec<u32>,
    pub runs: Vec<u64>,
}

/// Cell of a state. `bases`: bit 0 = runner on first, bit 1 = second, bit 2 = third.
pub fn cell(bases: u8, outs: u8, balls: u8, strikes: u8) -> usize {
    (((bases & 7) as usize * 3 + outs.min(2) as usize) * 4 + balls.min(3) as usize) * 3 + strikes.min(2) as usize
}

impl RunExpectancy {
    /// The table bundled with the program (`data/engine/run_expectancy.json`).
    pub fn bundled() -> Self {
        serde_json::from_str(BUNDLED).expect("data/engine/run_expectancy.json")
    }

    /// Expected runs to the end of the half inning; with three outs, none.
    pub fn value(&self, bases: u8, outs: u8, balls: u8, strikes: u8) -> f64 {
        if outs >= 3 {
            return 0.0;
        }
        let c = cell(bases, outs, balls, strikes);
        if self.n[c] == 0 {
            // A count never reached from this base-out state (cannot happen with enough games):
            // fall back to the same base-out state at 0-0.
            let c0 = cell(bases, outs, 0, 0);
            return if self.n[c0] == 0 { 0.0 } else { self.runs[c0] as f64 / self.n[c0] as f64 };
        }
        self.runs[c] as f64 / self.n[c] as f64
    }

    /// Count the table from `games` games between AI managers of `style`. Halves that can end
    /// early (the bottom of the ninth and later, where the home side wins as soon as it leads)
    /// and extra innings (a runner starts on second) are left out.
    pub fn build(league: &League, style: ManagerStyle, games: usize, seed: u64, threads: usize) -> Self {
        let threads = threads.max(1);
        let per = games.div_ceil(threads);
        let mut master = Rng::new(seed);
        let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
        let mut n = vec![0u32; CELLS];
        let mut runs = vec![0u64; CELLS];
        let mut played = 0u64;
        std::thread::scope(|scope| {
            let handles: Vec<_> = seeds
                .into_iter()
                .enumerate()
                .map(|(i, mut rng)| {
                    let count = per.min(games.saturating_sub(i * per));
                    scope.spawn(move || {
                        let mut n = vec![0u32; CELLS];
                        let mut runs = vec![0u64; CELLS];
                        // Pitches of the current half: (cell, the batting side's score before the pitch).
                        let mut half: Vec<(usize, u32)> = Vec::with_capacity(64);
                        for _ in 0..count {
                            let f = league.fixture(&mut rng);
                            let mut sim = league.start(&f);
                            sim.set_style(0, style);
                            sim.set_style(1, style);
                            half.clear();
                            let mut key = None;
                            while !sim.is_over() {
                                let s = sim.situation();
                                let here = (s.inning, s.top);
                                if key != Some(here) {
                                    half.clear();
                                    key = Some(here);
                                }
                                let counted = s.inning <= 8 || (s.inning == 9 && s.top);
                                let c = cell(bases(&s), s.outs, s.balls, s.strikes);
                                let before = s.score[s.batting];
                                let batting = s.batting;
                                let step = sim.step(&mut rng, &mut NoObserver);
                                if step == Step::Pitch && counted {
                                    half.push((c, before));
                                }
                                let after = sim.situation();
                                if after.over || (after.inning, after.top) != here {
                                    // The half is over: everything its pitches led to is in.
                                    let end = after.score[batting];
                                    for &(c, b) in &half {
                                        n[c] += 1;
                                        runs[c] += u64::from(end - b);
                                    }
                                    half.clear();
                                    key = None;
                                }
                            }
                        }
                        (n, runs, count as u64)
                    })
                })
                .collect();
            for h in handles {
                let (tn, tr, g) = h.join().expect("thread");
                for c in 0..CELLS {
                    n[c] += tn[c];
                    runs[c] += tr[c];
                }
                played += g;
            }
        });
        let source =
            format!("bm re: {played} games between AI managers in the 30 major-league parks, clubs of seed {seed}; innings 1-8 and the top of the ninth");
        RunExpectancy { source, games: played, n, runs }
    }

    /// Pitches behind the table.
    pub fn pitches(&self) -> u64 {
        self.n.iter().map(|&x| x as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_are_distinct_and_in_range() {
        let mut seen = vec![false; CELLS];
        for b in 0..8 {
            for o in 0..3 {
                for balls in 0..4 {
                    for s in 0..3 {
                        let c = cell(b, o, balls, s);
                        assert!(!seen[c]);
                        seen[c] = true;
                    }
                }
            }
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn bundled_table_makes_sense() {
        let t = RunExpectancy::bundled();
        assert!(t.pitches() > 1_000_000);
        // More runners, fewer outs, more balls: more runs; more strikes: fewer.
        let v = |b, o, ba, s| t.value(b, o, ba, s);
        assert!(v(7, 0, 0, 0) > v(1, 0, 0, 0) && v(1, 0, 0, 0) > v(0, 0, 0, 0));
        assert!(v(0, 0, 0, 0) > v(0, 1, 0, 0) && v(0, 1, 0, 0) > v(0, 2, 0, 0));
        assert!(v(0, 0, 3, 0) > v(0, 0, 0, 0) && v(0, 0, 0, 0) > v(0, 0, 0, 2));
        // Bases empty, nobody out, 0-0: about half a run (Tango 2010-2015: 0.481).
        assert!(v(0, 0, 0, 0) > 0.40 && v(0, 0, 0, 0) < 0.56, "{}", v(0, 0, 0, 0));
        assert_eq!(t.value(7, 3, 0, 0), 0.0);
    }
}
