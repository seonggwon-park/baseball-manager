//! Win expectancy: the probability that the home team goes on to win from the start of a
//! plate appearance, by inning, half, outs, base state and score.
//!
//! The table is counted from games between the engine's AI managers (`bm we`), so it is the
//! engine's own win expectancy, not a published one. The environment uses it twice: as a
//! feature of the observation, and as the potential of the shaped reward (the change of win
//! expectancy over a step), which leaves the best policy unchanged whatever the table's
//! errors (Ng, Harada & Russell 1999); a better table only makes learning faster.

use crate::league::League;
use bm_engine::{ManagerStyle, NoObserver, Rng, Situation};
use serde::{Deserialize, Serialize};

/// Innings 1-9, then every extra inning together.
pub const INNINGS: usize = 10;
/// The home team's lead is clipped to `-LEAD..=LEAD`.
pub const LEAD: i32 = 10;
const LEADS: usize = 2 * LEAD as usize + 1;
/// Cells of the table: inning × half × outs × base state × lead.
pub const CELLS: usize = INNINGS * 2 * 3 * 8 * LEADS;
/// Plate appearances a cell needs before its own count outweighs the pooled estimate.
const SHRINK: f64 = 20.0;

const BUNDLED: &str = include_str!("../../../data/engine/win_expectancy.json");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinExpectancy {
    /// How the table was made.
    pub source: String,
    pub games: u64,
    /// Plate appearances that began in each cell, and how many of them the home team won.
    pub n: Vec<u32>,
    pub wins: Vec<u32>,
    #[serde(skip)]
    p: Vec<f32>,
}

/// Cell of a state. `bases`: bit 0 = runner on first, bit 1 = second, bit 2 = third.
pub fn cell(inning: u8, top: bool, outs: u8, bases: u8, home_lead: i32) -> usize {
    let i = (inning.clamp(1, INNINGS as u8) - 1) as usize;
    let h = usize::from(!top);
    let o = outs.min(2) as usize;
    let b = (bases & 7) as usize;
    let l = (home_lead.clamp(-LEAD, LEAD) + LEAD) as usize;
    (((i * 2 + h) * 3 + o) * 8 + b) * LEADS + l
}

/// Base state of a situation as bits (first = 1, second = 2, third = 4).
pub fn bases(s: &Situation) -> u8 {
    s.bases.iter().enumerate().fold(0, |acc, (i, r)| if r.is_some() { acc | (1 << i) } else { acc })
}

impl WinExpectancy {
    /// The table bundled with the program (`data/engine/win_expectancy.json`).
    pub fn bundled() -> Self {
        let mut t: WinExpectancy = serde_json::from_str(BUNDLED).expect("data/engine/win_expectancy.json");
        t.smooth();
        t
    }

    pub fn from_counts(source: String, games: u64, n: Vec<u32>, wins: Vec<u32>) -> Self {
        assert_eq!(n.len(), CELLS);
        assert_eq!(wins.len(), CELLS);
        let mut t = WinExpectancy { source, games, n, wins, p: Vec::new() };
        t.smooth();
        t
    }

    /// Probabilities from the counts. A cell with few plate appearances leans on the same
    /// inning, half and lead with all outs and base states pooled, and that in turn on the
    /// half and lead with every inning pooled (each with weight `SHRINK`).
    fn smooth(&mut self) {
        let mut coarse = vec![(0.0, 0.0); 2 * LEADS];
        let mut mid = vec![(0.0, 0.0); INNINGS * 2 * LEADS];
        for c in 0..CELLS {
            let l = c % LEADS;
            let ih = c / (LEADS * 24);
            let h = ih % 2;
            let (n, w) = (self.n[c] as f64, self.wins[c] as f64);
            coarse[h * LEADS + l].0 += n;
            coarse[h * LEADS + l].1 += w;
            mid[ih * LEADS + l].0 += n;
            mid[ih * LEADS + l].1 += w;
        }
        self.p = (0..CELLS)
            .map(|c| {
                let l = c % LEADS;
                let ih = c / (LEADS * 24);
                let h = ih % 2;
                let (cn, cw) = coarse[h * LEADS + l];
                let p0 = (cw + 1.0) / (cn + 2.0);
                let (mn, mw) = mid[ih * LEADS + l];
                let p1 = (mw + SHRINK * p0) / (mn + SHRINK);
                ((self.wins[c] as f64 + SHRINK * p1) / (self.n[c] as f64 + SHRINK)) as f32
            })
            .collect();
    }

    /// Probability that the home team wins.
    pub fn home(&self, inning: u8, top: bool, outs: u8, bases: u8, home_lead: i32) -> f64 {
        self.p[cell(inning, top, outs, bases, home_lead)] as f64
    }

    /// Probability that `team` (0 = away, 1 = home) wins from the situation.
    pub fn for_team(&self, s: &Situation, team: usize) -> f64 {
        let lead = s.score[1] as i32 - s.score[0] as i32;
        let p = self.home(s.inning, s.top, s.outs, bases(s), lead);
        if team == 1 {
            p
        } else {
            1.0 - p
        }
    }

    /// Count the table from `games` games between AI managers of the given style.
    pub fn build(league: &League, style: ManagerStyle, games: usize, seed: u64, threads: usize) -> Self {
        let threads = threads.max(1);
        let per = games.div_ceil(threads);
        let mut master = Rng::new(seed);
        let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
        let mut n = vec![0u32; CELLS];
        let mut wins = vec![0u32; CELLS];
        let mut played = 0u64;
        std::thread::scope(|scope| {
            let handles: Vec<_> = seeds
                .into_iter()
                .enumerate()
                .map(|(i, mut rng)| {
                    let count = per.min(games.saturating_sub(i * per));
                    scope.spawn(move || {
                        let mut n = vec![0u32; CELLS];
                        let mut wins = vec![0u32; CELLS];
                        let mut seen = Vec::with_capacity(96);
                        for _ in 0..count {
                            let f = league.fixture(&mut rng);
                            let mut sim = league.start(&f);
                            sim.set_style(0, style);
                            sim.set_style(1, style);
                            seen.clear();
                            let mut last = None;
                            while !sim.is_over() {
                                let s = sim.situation();
                                if !s.in_plate_appearance {
                                    // The start of a plate appearance; substitutions may come
                                    // first, so it is counted once.
                                    let mark = (s.inning, s.top, s.outs, bases(&s), s.score);
                                    if last != Some(mark) {
                                        last = Some(mark);
                                        seen.push(cell(s.inning, s.top, s.outs, bases(&s), s.score[1] as i32 - s.score[0] as i32));
                                    }
                                }
                                sim.step(&mut rng, &mut NoObserver);
                            }
                            let r = sim.result().expect("game over");
                            let home_won = r.score[1] > r.score[0];
                            for &c in &seen {
                                n[c] += 1;
                                wins[c] += u32::from(home_won);
                            }
                        }
                        (n, wins, count as u64)
                    })
                })
                .collect();
            for h in handles {
                let (tn, tw, g) = h.join().expect("thread");
                for c in 0..CELLS {
                    n[c] += tn[c];
                    wins[c] += tw[c];
                }
                played += g;
            }
        });
        let source = format!("bm we: {played} games between AI managers in the 30 major-league parks, clubs of seed {seed}");
        WinExpectancy::from_counts(source, played, n, wins)
    }

    /// Plate appearances behind the table.
    pub fn plate_appearances(&self) -> u64 {
        self.n.iter().map(|&x| x as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_are_distinct_and_in_range() {
        let mut seen = vec![false; CELLS];
        for inning in 1..=INNINGS as u8 {
            for top in [true, false] {
                for outs in 0..3 {
                    for b in 0..8 {
                        for lead in -LEAD..=LEAD {
                            let c = cell(inning, top, outs, b, lead);
                            assert!(!seen[c]);
                            seen[c] = true;
                        }
                    }
                }
            }
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(cell(14, false, 2, 7, 25), cell(10, false, 2, 7, LEAD));
    }

    #[test]
    fn bundled_table_makes_sense() {
        let t = WinExpectancy::bundled();
        assert!(t.games >= 100_000, "{} games", t.games);
        // Even game at the start: the home side is slightly favoured.
        let start = t.home(1, true, 0, 0, 0);
        assert!((0.50..0.58).contains(&start), "{start}");
        // A lead is worth more the later it is.
        assert!(t.home(9, true, 0, 0, 1) > t.home(3, true, 0, 0, 1));
        assert!(t.home(9, true, 0, 0, -1) < t.home(3, true, 0, 0, -1));
        // Runners help the batting side: bottom of the ninth, tied, bases loaded and nobody out.
        assert!(t.home(9, false, 0, 7, 0) > t.home(9, false, 0, 0, 0) + 0.15);
        // Outs hurt it.
        assert!(t.home(7, false, 2, 1, 0) < t.home(7, false, 0, 1, 0));
        for &p in &t.p {
            assert!((0.0..=1.0).contains(&p));
        }
    }
}
