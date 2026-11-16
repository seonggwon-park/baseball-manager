//! `bm players`: season lines of individual players in a simulated league, as CSV.
//!
//! The league totals say whether the average player is right; this says whether
//! players differ from one another as much as real ones do. Clubs made of
//! attribute-generated players play a 162-game schedule and every hitter's and
//! pitcher's line is written out, with the plate-discipline and batted-ball rates
//! that Statcast publishes per player.

use crate::tally::is_barrel;
use bm_engine::{play_game_styled, Club, Hand, ManagerStyle, MatchEnv, Observer, PitchEvent, PitchKind, PitchResult, Rng};
use bm_physics::units::{FT, MPH};
use bm_world::{Attr, Baseline, Player, Squad, Talent};

/// Players of a club as the engine lists them for a game: nine hitters, the starter,
/// eight relievers, four reserves. The five starters get slots of their own.
const SLOTS: usize = 27;

fn slot(team_idx: usize, starter: usize) -> usize {
    if team_idx == 9 {
        22 + starter
    } else {
        team_idx
    }
}

/// What is counted for every player. The first block comes from the box score, the
/// second from the pitches he saw or threw, the third from the balls put in play.
#[derive(Clone, Copy)]
#[repr(usize)]
enum F {
    Pa,
    Ab,
    H,
    Doubles,
    Triples,
    Hr,
    Bb,
    So,
    Hbp,
    Sf,
    Sh,
    Sb,
    Cs,
    Outs,
    Runs,
    Earned,
    Games,
    Starts,
    Pitches,
    Zone,
    ZSwings,
    OSwings,
    Swings,
    Whiffs,
    ZContact,
    OContact,
    BatSpeed,
    BatN,
    Squared,
    Bip,
    Ev,
    La,
    Hard,
    Sweet,
    Barrel,
    Gb,
    Ld,
    Fb,
    Pu,
    Pull,
    FfSpeed,
    FfN,
    FfIvb,
}

const N: usize = 43;
const NAMES: [&str; N] = [
    "pa",
    "ab",
    "h",
    "doubles",
    "triples",
    "hr",
    "bb",
    "so",
    "hbp",
    "sf",
    "sh",
    "sb",
    "cs",
    "outs",
    "runs",
    "earned",
    "games",
    "starts",
    "pitches",
    "zone",
    "z_swings",
    "o_swings",
    "swings",
    "whiffs",
    "z_contact",
    "o_contact",
    "bat_speed",
    "bat_n",
    "squared",
    "bip",
    "ev",
    "la",
    "hard",
    "sweet",
    "barrel",
    "gb",
    "ld",
    "fb",
    "pu",
    "pull",
    "ff_speed",
    "ff_n",
    "ff_ivb",
];

#[derive(Clone, Copy)]
struct Line([f64; N]);

impl Default for Line {
    fn default() -> Self {
        Line([0.0; N])
    }
}

impl std::ops::Index<F> for Line {
    type Output = f64;
    fn index(&self, f: F) -> &f64 {
        &self.0[f as usize]
    }
}

impl std::ops::IndexMut<F> for Line {
    fn index_mut(&mut self, f: F) -> &mut f64 {
        &mut self.0[f as usize]
    }
}

impl Line {
    fn add(&mut self, o: &Line) {
        for (x, y) in self.0.iter_mut().zip(&o.0) {
            *x += *y;
        }
    }
}

/// Histogram of a hitter's bat speeds in 1 mph bins from 30 mph, for the average of
/// his fastest 90 % of swings (Statcast's "competitive swings").
type Swings = [u32; 70];

fn competitive_mean(h: &Swings) -> f64 {
    let total: u32 = h.iter().sum();
    let mut drop = (total as f64 * 0.1).round() as u32;
    let (mut sum, mut n) = (0.0, 0.0);
    for (i, &c) in h.iter().enumerate() {
        let skipped = c.min(drop);
        drop -= skipped;
        sum += (c - skipped) as f64 * (30.5 + i as f64);
        n += (c - skipped) as f64;
    }
    if n > 0.0 {
        sum / n
    } else {
        0.0
    }
}

struct Watch<'a> {
    /// Lines of the two clubs in this game: `[away, home]`.
    bat: [&'a mut [Line; SLOTS]; 2],
    pit: [&'a mut [Line; SLOTS]; 2],
    swings: [&'a mut [Swings; SLOTS]; 2],
    starter: [usize; 2],
}

impl Observer for Watch<'_> {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.approach.is_bunt() {
            return;
        }
        let (bt, ft) = if e.top { (0, 1) } else { (1, 0) };
        let in_zone = e.zone_distance <= 0.0;
        let contact = matches!(e.result, PitchResult::Foul | PitchResult::InPlay);
        let whiff = matches!(e.result, PitchResult::SwingingStrike | PitchResult::FoulTip);
        let f = &e.thrown.flight;
        let mut bip = Line::default();
        if let (PitchResult::InPlay, Some(info)) = (e.result, e.batted) {
            let (ev, la) = (info.exit_speed / MPH, info.launch_angle.to_degrees());
            bip[F::Bip] = 1.0;
            bip[F::Ev] = ev;
            bip[F::La] = la;
            bip[F::Hard] = (ev >= 95.0) as u8 as f64;
            bip[F::Sweet] = (8.0..=32.0).contains(&la) as u8 as f64;
            bip[F::Barrel] = is_barrel(ev, la) as u8 as f64;
            bip[F::Gb] = (la < 10.0) as u8 as f64;
            bip[F::Ld] = (10.0..25.0).contains(&la) as u8 as f64;
            bip[F::Fb] = (25.0..50.0).contains(&la) as u8 as f64;
            bip[F::Pu] = (la >= 50.0) as u8 as f64;
            bip[F::Pull] = (info.spray * e.batter_hand.pull_sign() > 15f64.to_radians()) as u8 as f64;
        }
        let both = |l: &mut Line| {
            l[F::Pitches] += 1.0;
            if in_zone {
                l[F::Zone] += 1.0;
            }
            if e.swung {
                l[F::Swings] += 1.0;
                if in_zone {
                    l[F::ZSwings] += 1.0;
                    l[F::ZContact] += contact as u8 as f64;
                } else {
                    l[F::OSwings] += 1.0;
                    l[F::OContact] += contact as u8 as f64;
                }
                l[F::Whiffs] += whiff as u8 as f64;
            }
            l.add(&bip);
        };
        let who = slot(e.batter, self.starter[bt]);
        let b = &mut self.bat[bt][who];
        both(b);
        if let Some(shape) = e.swing {
            let bin = ((shape.bat_speed / MPH - 30.0).max(0.0) as usize).min(69);
            self.swings[bt][who][bin] += 1;
            b[F::BatN] += 1.0;
            // Squared up: at least 80 % of the exit speed the bat and pitch speeds allow.
            if contact && shape.exit_speed >= 0.8 * (1.23 * shape.bat_speed + 0.23 * f.plate_vel.norm()) {
                b[F::Squared] += 1.0;
            }
        }
        let p = &mut self.pit[ft][slot(e.pitcher, self.starter[ft])];
        both(p);
        if e.thrown.kind == PitchKind::FourSeam {
            p[F::FfSpeed] += e.thrown.spec.speed / MPH;
            p[F::FfIvb] += f.move_z / FT * 12.0;
            p[F::FfN] += 1.0;
        }
    }
}

fn world_player(s: &Squad, slot: usize) -> &Player {
    match slot {
        0..=8 => &s.hitters[slot].0,
        10..=17 => &s.relievers[slot - 10],
        18..=21 => &s.bench[slot - 18].0,
        _ => &s.starters[slot - 22],
    }
}

pub fn run(env: &MatchEnv, baseline: &Baseline, style: ManagerStyle, seed: u64, threads: usize, n_clubs: usize, games_each: usize) {
    let n_clubs = n_clubs.max(2) & !1;
    let mut rng = Rng::new(seed ^ 0x9E37);
    let squads: Vec<Squad> = (0..n_clubs).map(|i| Squad::random(&mut rng, 1000 * (i as u32 + 1), &Talent::MAJOR)).collect();
    let clubs: Vec<Club> = squads.iter().map(|s| s.to_engine_with(baseline)).collect();

    // Every day the clubs are paired off at random; the rotation turns over every five days.
    let mut days: Vec<Vec<(usize, usize)>> = Vec::with_capacity(games_each);
    for _ in 0..games_each {
        let mut order: Vec<usize> = (0..n_clubs).collect();
        for i in (1..order.len()).rev() {
            order.swap(i, rng.below(i + 1));
        }
        days.push(order.chunks(2).map(|c| (c[0], c[1])).collect());
    }
    type Book = Vec<[Line; SLOTS]>;
    let blank = || -> Book { vec![[Line::default(); SLOTS]; n_clubs] };
    let no_swings = || -> Vec<[Swings; SLOTS]> { vec![[[0; 70]; SLOTS]; n_clubs] };
    let chunk = days.len().div_ceil(threads);
    let seeds: Vec<Rng> = (0..threads).map(|_| rng.fork()).collect();
    let (mut bat, mut pit, mut swings) = (blank(), blank(), no_swings());
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .enumerate()
            .map(|(t, mut rng)| {
                let (days, clubs) = (&days, &clubs);
                scope.spawn(move || {
                    let (mut bat, mut pit, mut swings) = (blank(), blank(), no_swings());
                    for (d, day) in days.iter().enumerate().skip(t * chunk).take(chunk) {
                        for &(a, h) in day {
                            let starter = d % 5;
                            let away = clubs[a].team_for_game(starter, env.rules.dh);
                            let home = clubs[h].team_for_game(starter, env.rules.dh);
                            // The two clubs are different, so their books can be borrowed side by side.
                            let (ba, bh) = pair(&mut bat, a, h);
                            let (pa, ph) = pair(&mut pit, a, h);
                            let (sa, sh) = pair(&mut swings, a, h);
                            let mut watch = Watch { bat: [ba, bh], pit: [pa, ph], swings: [sa, sh], starter: [starter; 2] };
                            let res = play_game_styled(env, &away, &home, [style; 2], &mut rng, &mut watch);
                            for (side, club) in [(0, a), (1, h)] {
                                for (i, l) in res.boxes[side].batting.iter().enumerate().filter(|(_, l)| l.pa > 0) {
                                    let b = &mut bat[club][slot(i, starter)];
                                    b[F::Pa] += l.pa as f64;
                                    b[F::Ab] += l.ab as f64;
                                    b[F::H] += l.h as f64;
                                    b[F::Doubles] += l.doubles as f64;
                                    b[F::Triples] += l.triples as f64;
                                    b[F::Hr] += l.hr as f64;
                                    b[F::Bb] += l.bb as f64;
                                    b[F::So] += l.so as f64;
                                    b[F::Hbp] += l.hbp as f64;
                                    b[F::Sf] += l.sf as f64;
                                    b[F::Sh] += l.sh as f64;
                                    b[F::Sb] += l.sb as f64;
                                    b[F::Cs] += l.cs as f64;
                                    b[F::Games] += 1.0;
                                }
                                for (i, l) in res.boxes[side].pitching.iter().enumerate().filter(|(_, l)| l.appeared) {
                                    let p = &mut pit[club][slot(i, starter)];
                                    p[F::Pa] += l.bf as f64;
                                    p[F::Outs] += l.outs as f64;
                                    p[F::H] += l.h as f64;
                                    p[F::Hr] += l.hr as f64;
                                    p[F::Bb] += l.bb as f64;
                                    p[F::So] += l.so as f64;
                                    p[F::Hbp] += l.hbp as f64;
                                    p[F::Runs] += l.r as f64;
                                    p[F::Earned] += l.er as f64;
                                    p[F::Games] += 1.0;
                                    p[F::Starts] += l.started as u8 as f64;
                                }
                            }
                        }
                    }
                    (bat, pit, swings)
                })
            })
            .collect();
        for h in handles {
            let (b, p, w) = h.join().expect("simulation thread panicked");
            for c in 0..n_clubs {
                for s in 0..SLOTS {
                    bat[c][s].add(&b[c][s]);
                    pit[c][s].add(&p[c][s]);
                    for (x, y) in swings[c][s].iter_mut().zip(&w[c][s]) {
                        *x += *y;
                    }
                }
            }
        }
    });

    for c in 0..n_clubs {
        for s in 0..SLOTS {
            bat[c][s][F::BatSpeed] = competitive_mean(&swings[c][s]) * bat[c][s][F::BatN];
        }
    }
    let z = |p: &Player, a: Attr| p.attrs[a];
    println!("kind,club,slot,{},bats,sprint,attack_angle,arm_angle,a1,a2,a3,a4,a5,a6,a7,a8", NAMES.join(","));
    let row = |kind: &str, c: usize, s: usize, l: &Line, tail: String| {
        let nums: Vec<String> = l.0.iter().map(|x| format!("{:.3}", x)).collect();
        println!("{kind},{c},{s},{},{tail}", nums.join(","));
    };
    for (c, squad) in squads.iter().enumerate() {
        // Slot 9 is the day's starter in the engine's list; the starters are kept in slots of their own.
        for s in (0..SLOTS).filter(|&s| s != 9) {
            let p = world_player(squad, s);
            if bat[c][s][F::Pa] > 0.0 && p.pitching.is_none() {
                let prof = bm_world::convert::batter_with(p, baseline);
                let side = match prof.hand {
                    _ if prof.switch => "S",
                    Hand::Right => "R",
                    Hand::Left => "L",
                };
                let tail = format!(
                    "{side},{:.2},{:.2},0,{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
                    prof.sprint / FT,
                    prof.attack_angle.to_degrees(),
                    z(p, Attr::Power),
                    z(p, Attr::Contact),
                    z(p, Attr::Timing),
                    z(p, Attr::Discipline),
                    z(p, Attr::Recognition),
                    z(p, Attr::BatControl),
                    z(p, Attr::Aggression),
                    z(p, Attr::Speed)
                );
                row("B", c, s, &bat[c][s], tail);
            }
            if pit[c][s][F::Pa] > 0.0 {
                if let Some(pit_style) = p.pitching.as_ref() {
                    let tail = format!(
                        "-,0,0,{:.1},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
                        pit_style.arm_angle.to_degrees(),
                        z(p, Attr::Velocity),
                        z(p, Attr::Stuff),
                        z(p, Attr::Control),
                        z(p, Attr::Command),
                        z(p, Attr::Deception),
                        z(p, Attr::Stamina),
                        z(p, Attr::Concentration),
                        pit_style.pitches.len() as f32
                    );
                    row("P", c, s, &pit[c][s], tail);
                }
            }
        }
    }
}

/// Mutable references to two different elements of a slice.
fn pair<T>(v: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert_ne!(a, b);
    if a < b {
        let (lo, hi) = v.split_at_mut(b);
        (&mut lo[a], &mut hi[0])
    } else {
        let (lo, hi) = v.split_at_mut(a);
        (&mut hi[0], &mut lo[b])
    }
}
