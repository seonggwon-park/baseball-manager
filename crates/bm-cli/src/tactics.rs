//! `bm tactics`: the manager's decision domain of the engine, from the games `bm validate`
//! plays (the same clubs, parks and AI managers).
//!
//! * Run expectancy by base-out state (RE24): runs from the start of a plate appearance to
//!   the end of the half-inning, and the share of plate appearances after which at least one
//!   run scored (R1). Innings 1-8 only, as the published tables (Tango, from Retrosheet) and
//!   `tools/tactics/re24_statcast.py` do, so that no half-inning is cut short by a walk-off.
//! * What the AI managers' tactics returned: plate appearances with a bunt attempt (a bunt
//!   offered at a pitch: missed, fouled or put in play, as Statcast can see it) and intentional
//!   walks, by base-out state, with the runs that followed and the state the plate appearance
//!   left; stolen-base attempts and successes by base.
//! * Plate appearances that start with a runner on third and fewer than two outs (innings
//!   1-8), by how they ended and whether a run scored during them (Statcast's events can be
//!   counted the same way: tools/tactics/third_base.py).
//! * Batting average on balls in play by batter hand, with the bases empty and overall, and
//!   how often the infield stood in a full shift (three infielders on the pull side): the
//!   measures of the 2023 shift restriction (`--shift-rule off` plays the earlier rules).
//!
//! The break-even rates and run values that follow from these tables are worked out and
//! compared with measured tables by `tools/tactics/compare.py`.

use bm_engine::fielding::{B2, SS};
use bm_engine::{play_game_styled, Club, GameEvent, Hand, ManagerStyle, MatchEnv, Observer, PaEvent, PaOutcome, PitchEvent, PlayKind, Rng};

/// Plate appearances, runs to the end of the half-inning, their squares, and plate
/// appearances after which a run scored.
#[derive(Clone, Copy, Default)]
struct Cell {
    n: f64,
    runs: f64,
    runs2: f64,
    scored: f64,
}

impl Cell {
    fn add(&mut self, runs: f64) {
        self.n += 1.0;
        self.runs += runs;
        self.runs2 += runs * runs;
        if runs > 0.0 {
            self.scored += 1.0;
        }
    }

    fn merge(&mut self, o: &Cell) {
        self.n += o.n;
        self.runs += o.runs;
        self.runs2 += o.runs2;
        self.scored += o.scored;
    }

    fn json(&self) -> serde_json::Value {
        let n = self.n.max(1.0);
        let mean = self.runs / n;
        let sd = (self.runs2 / n - mean * mean).max(0.0).sqrt();
        serde_json::json!({ "n": self.n, "re": mean, "se": sd / n.sqrt(), "r1": self.scored / n })
    }
}

/// A plate appearance of the half-inning under way: base state, outs, runs scored during it,
/// and what the manager did (0 = nothing recorded, 1 = bunt attempt, 2 = intentional walk),
/// with the state it left.
#[derive(Clone, Copy)]
struct Pa {
    bases: u8,
    outs: u8,
    runs: u8,
    tag: u8,
    bases_after: u8,
    outs_after: u8,
}

#[derive(Clone, Default)]
pub struct Tactics {
    re: [[Cell; 3]; 8],
    sac: [[Cell; 3]; 8],
    ibb: [[Cell; 3]; 8],
    /// Sacrifice-bunt attempts by the state before ([bases][outs]) and the state after
    /// ([bases][outs], outs 3 = inning over).
    sac_after: [[[[f64; 4]; 8]; 3]; 8],
    /// Stolen-base attempts and successes, to second and to third.
    steals: [[f64; 2]; 2],
    /// [left, right][bases empty, all]: balls in play (not home runs or sacrifice bunts), hits
    /// on them, plate appearances, plate appearances against a full shift.
    babip: [[[f64; 4]; 2]; 2],
    /// Batter hand and full shift at the first pitch of the plate appearance under way.
    hand_now: Option<(usize, bool)>,
    /// Runner on third, fewer than two outs, innings 1-8: by outcome (`third_label`), plate
    /// appearances and those during which a run scored.
    third: std::collections::BTreeMap<&'static str, [f64; 2]>,
    /// Ground-ball outs (with fielder's choices and double plays) in those plate appearances,
    /// [not forced, forced (bases loaded)][fielder P..SS]: count and those during which a run
    /// scored (tools/tactics/third_base.py counts Statcast's the same way).
    third_ground: [[[f64; 2]; 6]; 2],
    /// Fielder and kind of the last ball in play of the plate appearance under way.
    play_now: Option<(usize, PlayKind)>,
    half: Vec<Pa>,
    /// The half-inning under way: inning and top.
    key: (u8, bool),
    sac_now: bool,
    pub games: f64,
    pub runs: f64,
}

impl Tactics {
    /// Close the half-inning: every plate appearance gets the runs from its start to the end.
    fn flush(&mut self) {
        let mut rest = 0.0;
        for pa in self.half.iter().rev() {
            rest += pa.runs as f64;
            if self.key.0 > 8 || pa.outs > 2 {
                continue;
            }
            let (b, o) = (pa.bases as usize, pa.outs as usize);
            self.re[b][o].add(rest);
            match pa.tag {
                1 => {
                    self.sac[b][o].add(rest);
                    self.sac_after[b][o][pa.bases_after as usize][pa.outs_after.min(3) as usize] += 1.0;
                }
                2 => self.ibb[b][o].add(rest),
                _ => {}
            }
        }
        self.half.clear();
    }

    fn merge(&mut self, o: &Tactics) {
        for (mine, theirs) in [(&mut self.re, &o.re), (&mut self.sac, &o.sac), (&mut self.ibb, &o.ibb)] {
            for (c, d) in mine.as_flattened_mut().iter_mut().zip(theirs.as_flattened()) {
                c.merge(d);
            }
        }
        let add = |dst: &mut [f64], src: &[f64]| dst.iter_mut().zip(src).for_each(|(x, y)| *x += y);
        add(self.sac_after.as_flattened_mut().as_flattened_mut().as_flattened_mut(), o.sac_after.as_flattened().as_flattened().as_flattened());
        add(self.steals.as_flattened_mut(), o.steals.as_flattened());
        add(self.babip.as_flattened_mut().as_flattened_mut(), o.babip.as_flattened().as_flattened());
        add(self.third_ground.as_flattened_mut().as_flattened_mut(), o.third_ground.as_flattened().as_flattened());
        for (k, v) in &o.third {
            let e = self.third.entry(k).or_default();
            e[0] += v[0];
            e[1] += v[1];
        }
        self.games += o.games;
        self.runs += o.runs;
    }

    fn table(cells: &[[Cell; 3]; 8]) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        for (b, row) in cells.iter().enumerate() {
            for (o, c) in row.iter().enumerate() {
                if c.n > 0.0 {
                    m.insert(format!("{b}-{o}"), c.json());
                }
            }
        }
        serde_json::Value::Object(m)
    }

    pub fn json(&self) -> serde_json::Value {
        let mut after = serde_json::Map::new();
        for (b, row) in self.sac_after.iter().enumerate() {
            for (o, by_state) in row.iter().enumerate() {
                let mut cells = serde_json::Map::new();
                for (a, by_outs) in by_state.iter().enumerate() {
                    for (m, &x) in by_outs.iter().enumerate() {
                        if x > 0.0 {
                            cells.insert(format!("{a}-{m}"), x.into());
                        }
                    }
                }
                if !cells.is_empty() {
                    after.insert(format!("{b}-{o}"), serde_json::Value::Object(cells));
                }
            }
        }
        let babip: serde_json::Map<_, _> = ["left", "right"]
            .iter()
            .enumerate()
            .map(|(h, name)| {
                let row = |k: usize| {
                    let c = &self.babip[h][k];
                    serde_json::json!({ "balls_in_play": c[0], "hits": c[1], "babip": c[1] / c[0].max(1.0), "plate_appearances": c[2], "full_shift": c[3] / c[2].max(1.0) })
                };
                ((*name).to_string(), serde_json::json!({ "bases_empty": row(0), "all": row(1) }))
            })
            .collect();
        serde_json::json!({
            "source": "bm tactics, innings 1-8",
            "games": self.games,
            "runs_per_team_game": self.runs / (2.0 * self.games.max(1.0)),
            "plate_appearances": self.re.iter().flatten().map(|c| c.n).sum::<f64>(),
            "cells": Self::table(&self.re),
            "sacrifice_bunt": Self::table(&self.sac),
            "sacrifice_bunt_after": serde_json::Value::Object(after),
            "intentional_walk": Self::table(&self.ibb),
            "steal_second": { "attempts": self.steals[0][0], "safe": self.steals[0][1] },
            "steal_third": { "attempts": self.steals[1][0], "safe": self.steals[1][1] },
            "babip": babip,
            "runner_on_third_ground_outs": (["not_forced", "forced"].iter().zip(&self.third_ground).map(|(k, row)| {
                let by: serde_json::Map<_, _> = ["P", "C", "1B", "2B", "3B", "SS"].iter().zip(row).map(|(f, c)| ((*f).to_string(), serde_json::json!({ "outs": c[0], "scored": c[1] }))).collect();
                ((*k).to_string(), serde_json::Value::Object(by))
            }).collect::<serde_json::Map<_, _>>()),
            "runner_on_third": self.third.iter().map(|(k, v)| ((*k).to_string(), serde_json::json!({ "plate_appearances": v[0], "scored": v[1] }))).collect::<serde_json::Map<_, _>>(),
        })
    }
}

impl Observer for Tactics {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.approach.is_bunt() && e.swung {
            self.sac_now = true;
        }
        if let Some(play) = e.play {
            self.play_now = Some((play.fielder, play.kind));
        }
        if self.hand_now.is_none() {
            // Full shift: the shortstop right of second against a left-handed batter, the
            // second baseman left of it against a right-handed one.
            let (h, full) = match e.batter_hand {
                Hand::Left => (0, e.positions[SS].x > 0.0),
                Hand::Right => (1, e.positions[B2].x < 0.0),
            };
            self.hand_now = Some((h, full));
        }
    }

    fn plate_appearance(&mut self, e: &PaEvent) {
        if (e.inning, e.top) != self.key {
            self.flush();
            self.key = (e.inning, e.top);
        }
        let tag = if self.sac_now {
            1
        } else if e.intentional {
            2
        } else {
            0
        };
        self.sac_now = false;
        if let Some((h, full)) = self.hand_now.take() {
            let (bip, hit) = match e.outcome {
                PaOutcome::Play(PlayKind::HomeRun | PlayKind::SacBunt | PlayKind::Foul) => (false, false),
                PaOutcome::Play(k) => (true, k.is_hit()),
                _ => (false, false),
            };
            for k in [0, 1] {
                if k == 0 && e.bases_before != 0 {
                    continue;
                }
                let c = &mut self.babip[h][k];
                c[0] += bip as u8 as f64;
                c[1] += hit as u8 as f64;
                c[2] += 1.0;
                c[3] += full as u8 as f64;
            }
        }
        let play = self.play_now.take();
        if e.inning <= 8 && e.bases_before & 4 != 0 && e.outs_before < 2 {
            let c = self.third.entry(third_label(e)).or_default();
            c[0] += 1.0;
            if e.runs > 0 {
                c[1] += 1.0;
            }
            if let (Some((fielder, kind)), true) = (play, e.ground_ball) {
                if matches!(kind, PlayKind::GroundOut | PlayKind::FieldersChoice | PlayKind::DoublePlay) && fielder < 6 {
                    let c = &mut self.third_ground[(e.bases_before == 7) as usize][fielder];
                    c[0] += 1.0;
                    if e.runs > 0 {
                        c[1] += 1.0;
                    }
                }
            }
        }
        self.half.push(Pa {
            bases: e.bases_before,
            outs: e.outs_before,
            runs: e.runs,
            tag,
            bases_after: e.bases_after,
            outs_after: e.outs_before + e.outs_made,
        });
    }

    fn event(&mut self, e: &GameEvent) {
        match *e {
            GameEvent::HalfInning { inning, top } => {
                self.flush();
                self.key = (inning, top);
            }
            GameEvent::Steal { to_base, safe, .. } => {
                let t = if to_base == 3 { 1 } else { 0 };
                self.steals[t][0] += 1.0;
                if safe {
                    self.steals[t][1] += 1.0;
                }
            }
            _ => {}
        }
    }
}

/// How a plate appearance ended, in the groups Statcast's events can be put into
/// (tools/tactics/third_base.py): ground-ball outs of every kind (with fielder's choices and
/// double plays) together, fly-ball outs apart from sacrifice flies, walks with hit batters.
fn third_label(e: &PaEvent) -> &'static str {
    match e.outcome {
        PaOutcome::Strikeout => "strikeout",
        PaOutcome::Walk if e.intentional => "intentional walk",
        PaOutcome::Walk | PaOutcome::HitByPitch => "walk or hit by pitch",
        PaOutcome::Interrupted => "inning ended on the bases",
        PaOutcome::Play(k) => match k {
            PlayKind::Single => "single",
            PlayKind::Double | PlayKind::GroundRuleDouble => "double",
            PlayKind::Triple => "triple",
            PlayKind::HomeRun => "home run",
            PlayKind::SacFly => "sacrifice fly",
            PlayKind::GroundOut | PlayKind::FieldersChoice | PlayKind::DoublePlay => "ground-ball out, fielder's choice or double play",
            PlayKind::FlyOut => "fly out",
            PlayKind::LineOut => "line out",
            PlayKind::PopOut | PlayKind::FoulOut => "pop or foul out",
            PlayKind::Error => "error",
            PlayKind::SacBunt => "sacrifice bunt",
            PlayKind::Foul => "other",
        },
    }
}

/// Play `games` games as `bm validate` does (home club `h` in `envs[h % envs.len()]`) and
/// collect the tables.
pub fn run(envs: &[MatchEnv], clubs: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize) -> Tactics {
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let mut total = Tactics::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .enumerate()
            .map(|(i, mut rng)| {
                let n = per.min(games.saturating_sub(i * per));
                scope.spawn(move || {
                    let mut t = Tactics::default();
                    for _ in 0..n {
                        let (a, h) = crate::pick_pair(&mut rng, clubs.len());
                        let env = &envs[h % envs.len()];
                        let dh = env.rules.dh;
                        let away = clubs[a].team_for_game(rng.below(5), dh);
                        let home = clubs[h].team_for_game(rng.below(5), dh);
                        let res = play_game_styled(env, &away, &home, [style; 2], &mut rng, &mut t);
                        t.flush();
                        t.key = (0, false);
                        t.sac_now = false;
                        t.hand_now = None;
                        t.games += 1.0;
                        t.runs += (res.score[0] + res.score[1]) as f64;
                    }
                    t
                })
            })
            .collect();
        for h in handles {
            total.merge(&h.join().expect("thread"));
        }
    });
    total
}

/// Print the RE24 and R1 tables.
pub fn print(t: &Tactics) {
    println!("{:.0} games, {:.3} runs per team-game; innings 1-8", t.games, t.runs / (2.0 * t.games.max(1.0)));
    println!("run expectancy (R1: share of plate appearances after which a run scored)");
    println!("bases   0 outs          1 out           2 outs");
    for (b, row) in t.re.iter().enumerate() {
        let name: String = ['1', '2', '3'].iter().enumerate().map(|(i, c)| if b >> i & 1 == 1 { *c } else { '_' }).collect();
        print!("{name}  ");
        for c in row {
            print!("  {:.3} ({:.3})", c.runs / c.n.max(1.0), c.scored / c.n.max(1.0));
        }
        println!();
    }
    for (t_, name) in t.steals.iter().zip(["second", "third"]) {
        println!("steals of {name}: {:.0} attempts, {:.1}% safe", t_[0], 100.0 * t_[1] / t_[0].max(1.0));
    }
    let n3: f64 = t.third.values().map(|v| v[0]).sum();
    println!(
        "runner on third, fewer than two outs, innings 1-8: {n3:.0} plate appearances; a run scored during {:.3}",
        t.third.values().map(|v| v[1]).sum::<f64>() / n3.max(1.0)
    );
    for (k, v) in &t.third {
        println!("  {k:50} {:.3} of them, a run scored in {:.2}, contributes {:.3}", v[0] / n3.max(1.0), v[1] / v[0].max(1.0), v[1] / n3.max(1.0));
    }
    for (label, row) in ["not forced", "forced (bases loaded)"].iter().zip(&t.third_ground) {
        let cells: Vec<String> = ["P", "C", "1B", "2B", "3B", "SS"]
            .iter()
            .zip(row)
            .filter(|(_, c)| c[0] > 0.0)
            .map(|(f, c)| format!("{f} {:.2} ({:.0})", c[1] / c[0], c[0]))
            .collect();
        println!("  ground-ball outs, {label}: a run scored in {}", cells.join(", "));
    }
    let s = &t.sac[1][0];
    println!("bunt attempts with a runner on first and no outs: {:.0}, {:.3} runs to the end of the inning", s.n, s.runs / s.n.max(1.0));
    for ([e, a], name) in t.babip.iter().zip(["left", "right"]) {
        println!(
            "{name}-handed batters: BABIP {:.4} with the bases empty ({:.0} balls in play), {:.4} overall; full shift in {:.1}% of plate appearances ({:.1}% with the bases empty)",
            e[1] / e[0].max(1.0),
            e[0],
            a[1] / a[0].max(1.0),
            100.0 * a[3] / a[2].max(1.0),
            100.0 * e[3] / e[2].max(1.0)
        );
    }
}
