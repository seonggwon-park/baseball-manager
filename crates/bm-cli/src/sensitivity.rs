//! `bm attributes`: what one standard deviation of each attribute is worth.
//!
//! A "test" club whose players all have one attribute raised (or lowered) by one
//! standard deviation plays ordinary clubs. The change in the runs it scores and
//! allows, against the same schedule played by unchanged clubs, is the value of
//! the attribute. Both directions are simulated, so the table also shows whether
//! the effect has the right sign on both sides of the average.

use bm_engine::{play_game_styled, Club, GameEvent, ManagerStyle, MatchEnv, Observer, PaEvent, PaOutcome, PlayKind, Rng};
use bm_world::{Attr, Baseline, Player, Squad, Talent};

/// Batting line of one side.
#[derive(Clone, Copy, Default)]
pub struct Line {
    pub pa: f64,
    pub so: f64,
    pub bb: f64,
    pub hbp: f64,
    pub singles: f64,
    pub doubles: f64,
    pub triples: f64,
    pub hr: f64,
    pub roe: f64,
    pub sf: f64,
    pub sac: f64,
    pub sb: f64,
    pub cs: f64,
    pub runs: f64,
    pub games: f64,
}

impl Line {
    fn add(&mut self, o: &Line) {
        self.pa += o.pa;
        self.so += o.so;
        self.bb += o.bb;
        self.hbp += o.hbp;
        self.singles += o.singles;
        self.doubles += o.doubles;
        self.triples += o.triples;
        self.hr += o.hr;
        self.roe += o.roe;
        self.sf += o.sf;
        self.sac += o.sac;
        self.sb += o.sb;
        self.cs += o.cs;
        self.runs += o.runs;
        self.games += o.games;
    }
    pub fn hits(&self) -> f64 {
        self.singles + self.doubles + self.triples + self.hr
    }
    pub fn ab(&self) -> f64 {
        self.pa - self.bb - self.hbp - self.sf - self.sac
    }
    pub fn avg(&self) -> f64 {
        self.hits() / self.ab()
    }
    pub fn obp(&self) -> f64 {
        (self.hits() + self.bb + self.hbp) / (self.ab() + self.bb + self.hbp + self.sf)
    }
    pub fn slg(&self) -> f64 {
        (self.singles + 2.0 * self.doubles + 3.0 * self.triples + 4.0 * self.hr) / self.ab()
    }
    /// Weighted on-base average with the 2025 linear weights (FanGraphs "Guts!").
    pub fn woba(&self) -> f64 {
        (0.691 * self.bb + 0.722 * self.hbp + 0.882 * self.singles + 1.252 * self.doubles + 1.584 * self.triples + 2.037 * self.hr) / (self.pa - self.sac)
    }
    pub fn babip(&self) -> f64 {
        (self.hits() - self.hr) / (self.ab() - self.so - self.hr + self.sf)
    }
    pub fn rate(&self, x: f64) -> f64 {
        100.0 * x / self.pa
    }
    pub fn runs_per_game(&self) -> f64 {
        self.runs / self.games
    }
}

/// Collects the batting lines of the two sides of a game.
#[derive(Default)]
struct Split {
    side: [Line; 2],
    batting: usize,
}

impl Observer for Split {
    fn plate_appearance(&mut self, e: &PaEvent) {
        let l = &mut self.side[if e.top { 0 } else { 1 }];
        match e.outcome {
            PaOutcome::Interrupted => return,
            PaOutcome::Strikeout => l.so += 1.0,
            PaOutcome::Walk => l.bb += 1.0,
            PaOutcome::HitByPitch => l.hbp += 1.0,
            PaOutcome::Play(k) => match k {
                PlayKind::Single => l.singles += 1.0,
                PlayKind::Double | PlayKind::GroundRuleDouble => l.doubles += 1.0,
                PlayKind::Triple => l.triples += 1.0,
                PlayKind::HomeRun => l.hr += 1.0,
                PlayKind::Error => l.roe += 1.0,
                PlayKind::SacFly => l.sf += 1.0,
                PlayKind::SacBunt => l.sac += 1.0,
                _ => {}
            },
        }
        l.pa += 1.0;
    }
    fn steal(&mut self, success: bool) {
        let l = &mut self.side[self.batting];
        if success {
            l.sb += 1.0;
        } else {
            l.cs += 1.0;
        }
    }
    fn event(&mut self, e: &GameEvent) {
        if let GameEvent::HalfInning { top, .. } = e {
            self.batting = if *top { 0 } else { 1 };
        }
    }
}

/// Batting lines of the test clubs and of their opponents.
#[derive(Clone, Copy, Default)]
pub struct Outcome {
    pub test: Line,
    pub opp: Line,
}

/// Play `games` games between a test club and an ordinary club, home and away in turn.
pub fn play(env: &MatchEnv, test: &[Club], base: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize) -> Outcome {
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let mut total = Outcome::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .map(|mut rng| {
                scope.spawn(move || {
                    let mut out = Outcome::default();
                    for g in 0..per {
                        let i = rng.below(test.len());
                        let mut j = rng.below(base.len() - 1);
                        if j >= i {
                            j += 1;
                        }
                        let ours = test[i].team_for_game(rng.below(5), env.rules.dh);
                        let theirs = base[j].team_for_game(rng.below(5), env.rules.dh);
                        let home = g % 2 == 0;
                        let mut split = Split::default();
                        let res = if home {
                            play_game_styled(env, &theirs, &ours, [style; 2], &mut rng, &mut split)
                        } else {
                            play_game_styled(env, &ours, &theirs, [style; 2], &mut rng, &mut split)
                        };
                        let (t, o) = if home { (1, 0) } else { (0, 1) };
                        split.side[t].runs = res.score[t] as f64;
                        split.side[o].runs = res.score[o] as f64;
                        split.side[t].games = 1.0;
                        split.side[o].games = 1.0;
                        out.test.add(&split.side[t]);
                        out.opp.add(&split.side[o]);
                    }
                    out
                })
            })
            .collect();
        for h in handles {
            let o = h.join().expect("simulation thread panicked");
            total.test.add(&o.test);
            total.opp.add(&o.opp);
        }
    });
    total
}

/// Who an experiment changes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Who {
    Hitters,
    Pitchers,
    /// The grades of every pitch other than the main fastball.
    Pitches,
}

fn shifted(squads: &[Squad], base: &Baseline, who: Who, attr: Attr, dz: f32) -> Vec<Club> {
    squads
        .iter()
        .map(|s| {
            let mut s = s.clone();
            let bump = |p: &mut Player| p.attrs[attr] += dz;
            match who {
                Who::Hitters => s.hitters.iter_mut().chain(s.bench.iter_mut()).for_each(|(p, _)| bump(p)),
                Who::Pitchers => s.starters.iter_mut().chain(s.relievers.iter_mut()).for_each(bump),
                Who::Pitches => s.starters.iter_mut().chain(s.relievers.iter_mut()).for_each(|p| {
                    if let Some(pit) = p.pitching.as_mut() {
                        pit.pitches.iter_mut().for_each(|k| k.grade += dz);
                    }
                }),
            }
            s.to_engine_with(base)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn run(env: &MatchEnv, baseline: &Baseline, style: ManagerStyle, games: usize, seed: u64, threads: usize, n_clubs: usize, only: Option<&str>) {
    use Attr::*;
    let mut rng = Rng::new(seed ^ 0xA77);
    let squads: Vec<Squad> = (0..n_clubs).map(|i| Squad::random(&mut rng, 1000 * (i as u32 + 1), &Talent::MAJOR)).collect();
    let base: Vec<Club> = squads.iter().map(|s| s.to_engine_with(baseline)).collect();

    // The reference: ordinary clubs against ordinary clubs, four times the games of one experiment.
    let reference = play(env, &base, &base, style, 4 * games, seed, threads);
    let (r0, a0) = (reference.test, reference.opp);
    println!("Each row: every player of the kind named gets the attribute moved by one standard deviation; {games} games per direction.");
    println!(
        "Reference ({} games): {:.3} runs per game, {:.3}/{:.3}/{:.3}, wOBA {:.4}, K {:.2} %, BB {:.2} %, HR {:.2} %",
        4 * games,
        r0.runs_per_game(),
        r0.avg(),
        r0.obp(),
        r0.slg(),
        r0.woba(),
        r0.rate(r0.so),
        r0.rate(r0.bb),
        r0.rate(r0.hr)
    );
    let noise = 3.15 / (games as f64).sqrt() / 2f64.sqrt();
    println!("Standard error of a run difference per standard deviation (half the +1/-1 gap): about {noise:.3} runs per game.\n");
    println!(
        "{:<14} {:<9} {:>8} {:>8} {:>8} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>6} {:>6}",
        "attribute", "who", "+1 runs", "-1 runs", "per SD", "wOBA", "K %", "BB %", "HR %", "BABIP", "SB/G", "ROE/G", "season"
    );

    let hitters = [
        Contact,
        Timing,
        Power,
        Discipline,
        Recognition,
        BatControl,
        Bunt,
        Baserunning,
        Stealing,
        Speed,
        Acceleration,
        Aggression,
        Anticipation,
        Concentration,
        Composure,
        Strength,
    ];
    let fielders = [Hands, Range, Reads, ArmStrength, ArmAccuracy, Footwork, Agility, Framing, Blocking, ThrowingOut];
    let pitchers = [Velocity, Stuff, Control, Command, Deception, HoldRunners, Stamina, Concentration, Composure];
    let mut rows: Vec<(String, Who, Attr, bool)> = Vec::new();
    rows.extend(hitters.iter().map(|&a| (a.key().to_string(), Who::Hitters, a, true)));
    rows.extend(fielders.iter().map(|&a| (a.key().to_string(), Who::Hitters, a, false)));
    rows.extend(pitchers.iter().map(|&a| (a.key().to_string(), Who::Pitchers, a, false)));
    rows.push(("pitch grades".to_string(), Who::Pitches, Stuff, false));

    for (k, (name, who, attr, offence)) in rows.into_iter().enumerate() {
        if only.is_some_and(|o| !name.contains(o)) {
            continue;
        }
        let up = play(env, &shifted(&squads, baseline, who, attr, 1.0), &base, style, games, seed + 1 + 2 * k as u64, threads);
        let down = play(env, &shifted(&squads, baseline, who, attr, -1.0), &base, style, games, seed + 2 + 2 * k as u64, threads);
        // Offence is measured on the runs the test clubs score, everything else on the runs they allow.
        let (plus, minus, reference_line, sign) = if offence { (up.test, down.test, r0, 1.0) } else { (up.opp, down.opp, a0, -1.0) };
        let d_up = sign * (plus.runs_per_game() - reference_line.runs_per_game());
        let d_down = sign * (minus.runs_per_game() - reference_line.runs_per_game());
        let per_sd = (d_up - d_down) / 2.0;
        let half = |f: &dyn Fn(&Line) -> f64| (f(&plus) - f(&minus)) / 2.0;
        // Runs per season: nine lineup spots share the offence; the staff or the eight fielders share the defence.
        let season = per_sd * 162.0 / if offence { 9.0 } else { 1.0 };
        println!(
            "{:<14} {:<9} {:>+8.3} {:>+8.3} {:>+8.3} | {:>+7.4} {:>+7.2} {:>+7.2} {:>+7.2} {:>+7.4} {:>+7.3} | {:>+6.3} {:>+6.1}",
            name,
            match who {
                Who::Hitters if offence => "batting",
                Who::Hitters => "fielding",
                _ => "pitching",
            },
            d_up,
            d_down,
            per_sd,
            half(&|l| l.woba()),
            half(&|l| l.rate(l.so)),
            half(&|l| l.rate(l.bb)),
            half(&|l| l.rate(l.hr)),
            half(&|l| l.babip()),
            half(&|l| (l.sb + l.cs) / l.games),
            half(&|l| l.roe / l.games),
            season,
        );
    }
    println!("\n\"+1 runs\" and \"-1 runs\": change in runs per game in the club's favour (scored for batting rows, prevented for the others).");
    println!("The rate columns are half the difference between the +1 and -1 clubs: their own batting for batting rows, their opponents' batting otherwise.");
    println!("\"season\": runs per 162 games per standard deviation, for one lineup spot (batting) or for the whole staff or defence (other rows).");
}
