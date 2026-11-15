//! Random players and clubs with MLB-like spreads of physical ability.
//!
//! This is the population used to calibrate and validate the engine. The game
//! world proper generates players from its own attribute system and converts
//! them to the same profiles.

use crate::fielding::{B1, B2, B3, C, CF, LF, P, RF, SS};
use crate::game::Team;
use crate::profile::{BatterProfile, CatcherProfile, FielderProfile, Hand, PitchDef, PitchKind, PitchShape, PitcherProfile, Player, NO_POSITION};
use crate::rng::Rng;
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::BatSpec;

/// League-wide averages and spreads from which a population is drawn. The
/// default is the calibrated MLB level; other leagues shift these values.
#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    /// Added to every pitcher's fastball velocity, m/s.
    pub velocity: f64,
    /// Added to every batter's bat speed, m/s.
    pub bat_speed: f64,
    /// Multiplier on the batters' timing and barrel errors (1 = MLB).
    pub precision: f64,
    /// Multiplier on the pitchers' command error (1 = MLB).
    pub command: f64,
    /// League-average timing SD of a batter, s.
    pub timing_sd: f64,
    /// League-average barrel SD, m.
    pub barrel_sd: f64,
    /// League-average barrel aim below the centre of the ball, m.
    pub barrel_aim: f64,
    /// Mean, between-batter spread and swing-to-swing spread of the attack angle, rad.
    pub attack_mean: f64,
    pub attack_spread: f64,
    pub attack_sd: f64,
    /// League-average pitch recognition.
    pub eye: f64,
    /// League-average location perception noise, m.
    pub perception_sd: f64,
    /// SD of the contact point along the bat, m.
    pub reach_sd: f64,
    /// Swing-to-swing bat-speed spread, m/s.
    pub bat_speed_sd: f64,
    /// League-average timing bias, s.
    pub timing_bias: f64,
    pub bat: BatSpec,
}

impl Level {
    pub fn mlb() -> Self {
        let avg = BatterProfile::league_average(Hand::Right);
        Level {
            velocity: 0.0,
            bat_speed: 0.0,
            precision: 1.0,
            command: 1.0,
            timing_sd: avg.timing_sd,
            barrel_sd: avg.barrel_sd,
            barrel_aim: avg.barrel_aim,
            attack_mean: avg.attack_angle,
            attack_spread: 3.0f64.to_radians(),
            attack_sd: avg.attack_sd,
            eye: avg.eye,
            perception_sd: avg.perception_sd,
            reach_sd: avg.reach_sd,
            bat_speed_sd: avg.bat_speed_sd,
            timing_bias: avg.timing_bias,
            bat: avg.bat,
        }
    }
}

pub fn random_batter(rng: &mut Rng, id: u32, level: &Level) -> Player {
    // Traits added after the engine was calibrated come from a generator of their own,
    // so that the calibrated population stays exactly what it was.
    let mut extra = Rng::new(0xB0A7_5EED ^ ((id as u64) << 8));
    let hand = if rng.chance(0.56) { Hand::Right } else { Hand::Left };
    let sprint_ft = rng.gauss_clamped(27.0, 1.4, 23.0, 30.5);
    let batter = BatterProfile {
        hand,
        height: rng.gauss_clamped(1.855, 0.055, 1.68, 2.03),
        bat_speed: rng.gauss_clamped(71.9, 2.6, 63.0, 81.0) * MPH + level.bat_speed,
        bat_speed_sd: level.bat_speed_sd,
        attack_angle: rng.gauss_clamped(level.attack_mean, level.attack_spread, level.attack_mean - 0.2, level.attack_mean + 0.2),
        attack_sd: level.attack_sd,
        timing_sd: level.timing_sd * rng.gauss_clamped(1.0, 0.10, 0.75, 1.35) * level.precision,
        barrel_sd: level.barrel_sd * rng.gauss_clamped(1.0, 0.12, 0.70, 1.40) * level.precision,
        barrel_aim: level.barrel_aim + rng.gauss(0.0, 0.003),
        reach_sd: level.reach_sd,
        eye: rng.gauss_clamped(level.eye, 0.07, 0.3, 0.97),
        perception_sd: level.perception_sd * rng.gauss_clamped(1.0, 0.10, 0.7, 1.3),
        chase: rng.gauss(0.0, 0.012),
        aggression: rng.gauss_clamped(1.0, 0.06, 0.8, 1.2),
        timing_bias: rng.gauss(level.timing_bias, 0.0015),
        sprint: sprint_ft * FT,
        accel: (4.9 + 0.25 * (sprint_ft - 27.0) + rng.gauss(0.0, 0.25)).clamp(3.8, 6.2),
        baserunning: rng.gauss_clamped(0.5, 0.15, 0.0, 1.0),
        steal: ((sprint_ft - 25.5) / 4.0 + rng.gauss(0.0, 0.15)).clamp(0.0, 1.0),
        bunt: extra.gauss_clamped(0.5, 0.18, 0.05, 1.0),
        bat: level.bat,
    };
    let fielder = FielderProfile {
        reaction: rng.gauss_clamped(0.30, 0.03, 0.22, 0.40),
        accel: rng.gauss_clamped(4.6, 0.3, 3.8, 5.4),
        speed: (sprint_ft + 0.5) * FT,
        read_delay: 0.0,
        fly_accel: 4.6,
        route: rng.gauss_clamped(0.95, 0.015, 0.90, 0.99),
        hands: rng.gauss_clamped(1.0, 0.2, 0.5, 1.6),
        arm: rng.gauss_clamped(84.0, 3.0, 74.0, 96.0) * MPH,
        transfer: rng.gauss_clamped(0.80, 0.06, 0.65, 1.0),
        arm_accuracy: rng.gauss_clamped(1.0, 0.2, 0.5, 1.6),
    };
    Player {
        id,
        position: NO_POSITION,
        batter,
        fielder,
        pitcher: None,
        catcher: CatcherProfile { pop_time: rng.gauss_clamped(1.97, 0.06, 1.85, 2.12), blocking: rng.gauss_clamped(1.0, 0.15, 0.6, 1.5), framing: 0.0 },
    }
}

/// Adjust a position player's tools to what the position demands.
pub fn fit_to_position(p: &mut Player, pos: usize, rng: &mut Rng) {
    p.position = pos;
    let (speed_ft, arm_mph) = match pos {
        C => (-1.6, 81.0),
        B1 => (-1.0, 80.0),
        B2 => (0.2, 82.0),
        B3 => (-0.2, 86.0),
        SS => (0.5, 86.5),
        LF => (0.0, 87.0),
        CF => (1.1, 89.0),
        RF => (0.3, 91.0),
        _ => (0.0, 84.0),
    };
    p.batter.sprint += speed_ft * FT;
    p.fielder.speed += speed_ft * FT;
    p.fielder.arm = rng.gauss_clamped(arm_mph, 2.5, 72.0, 99.0) * MPH;
    if pos >= LF {
        // Outfielders build speed more slowly while tracking a ball in the air. With these
        // values an average outfielder reaches a ball 46 ft away in 3 s, 75 ft in 4 s and
        // 100 ft in 5 s about half the time, in line with Statcast's catch-probability model.
        p.fielder.transfer += 0.05;
        p.fielder.read_delay = rng.gauss_clamped(0.02, 0.03, 0.0, 0.12);
        p.fielder.fly_accel = p.fielder.accel - 0.6;
    } else if pos >= B1 {
        // Infielders read the ball off the bat and move with a quick first step.
        p.fielder.reaction -= 0.08;
        p.fielder.accel += 0.8;
        if pos == B2 || pos == SS {
            // The middle infielders are the rangiest defenders on the field.
            p.fielder.reaction -= 0.03;
            p.fielder.accel += 0.5;
        }
        p.fielder.fly_accel = p.fielder.accel;
    }
}

/// (kind, probability of having it, usage weight when present, same-hand factor, opposite-hand factor)
const ARSENAL: [(PitchKind, f64, f64, f64, f64); 8] = [
    (PitchKind::FourSeam, 0.75, 38.0, 0.89, 1.09),
    (PitchKind::Sinker, 0.55, 28.0, 1.28, 0.70),
    (PitchKind::Cutter, 0.30, 22.0, 0.97, 1.08),
    (PitchKind::Slider, 0.55, 28.0, 1.34, 0.74),
    (PitchKind::Sweeper, 0.30, 29.0, 1.43, 0.65),
    (PitchKind::Curve, 0.40, 26.0, 0.69, 1.21),
    (PitchKind::Changeup, 0.55, 19.5, 0.48, 1.43),
    (PitchKind::Splitter, 0.14, 20.0, 0.76, 1.37),
];

pub fn random_pitcher(rng: &mut Rng, id: u32, starter: bool, level: &Level) -> Player {
    let hand = if rng.chance(0.73) { Hand::Right } else { Hand::Left };
    let fb_delta = rng.gauss_clamped(if starter { -0.6 } else { 0.9 }, 2.2, -7.0, 7.0) * MPH + level.velocity;
    let command_in = rng.gauss_clamped(6.6, 0.7, 5.0, 8.8) * level.command;
    let mut pitches: Vec<PitchDef> = Vec::with_capacity(6);
    for &(kind, prob, weight, same, opp) in &ARSENAL {
        if pitches.len() >= 6 || !rng.chance(prob) {
            continue;
        }
        pitches.push(make_pitch(rng, kind, fb_delta, command_in, weight, same, opp));
    }
    // Everybody has a fastball and at least two other offerings.
    if !pitches.iter().any(|d| matches!(d.kind, PitchKind::FourSeam | PitchKind::Sinker)) {
        let (kind, _, weight, same, opp) = ARSENAL[0];
        pitches.insert(0, make_pitch(rng, kind, fb_delta, command_in, weight, same, opp));
    }
    let mut guard = 0;
    while pitches.len() < 3 && guard < 50 {
        guard += 1;
        let (kind, _, weight, same, opp) = ARSENAL[2 + rng.below(6)];
        if !pitches.iter().any(|d| d.kind == kind) {
            pitches.push(make_pitch(rng, kind, fb_delta, command_in, weight, same, opp));
        }
    }
    let profile = PitcherProfile {
        hand,
        release_side: rng.gauss_clamped(1.9, 0.5, 0.4, 3.4) * FT,
        release_height: rng.gauss_clamped(5.8, 0.35, 4.6, 6.8) * FT,
        extension: rng.gauss_clamped(6.45, 0.3, 5.5, 7.4) * FT,
        pitches,
        speed_sd: rng.gauss_clamped(1.05, 0.12, 0.7, 1.5) * MPH,
        spin_sd: rng.gauss_clamped(85.0, 10.0, 55.0, 130.0) * RPM,
        tilt_sd: rng.gauss_clamped(4.5, 0.6, 2.5, 7.0).to_radians(),
        stamina: if starter { rng.gauss_clamped(92.0, 7.0, 70.0, 112.0) } else { rng.gauss_clamped(28.0, 5.0, 15.0, 45.0) },
        time_to_plate: rng.gauss_clamped(1.38, 0.08, 1.15, 1.65),
    };
    let mut p = random_batter(rng, id, level);
    p.position = P;
    p.batter.hand = hand;
    p.batter.bat_speed *= 0.93;
    p.batter.barrel_sd *= 1.45;
    p.batter.timing_sd *= 1.35;
    p.batter.eye = 0.55;
    p.pitcher = Some(profile);
    p
}

fn make_pitch(rng: &mut Rng, kind: PitchKind, fb_delta: f64, command_in: f64, weight: f64, same: f64, opp: f64) -> PitchDef {
    let base = kind.league_average();
    let shape = PitchShape {
        speed: base.speed + 0.9 * fb_delta + rng.gauss(0.0, 1.0 * MPH),
        spin_rate: (base.spin_rate + rng.gauss(0.0, 150.0 * RPM)).max(600.0 * RPM),
        active: (base.active + rng.gauss(0.0, 0.06)).clamp(0.03, 0.99),
        tilt: base.tilt + rng.gauss(0.0, 8.0f64.to_radians()),
    };
    let w = weight * rng.gauss(0.0, 0.30).exp();
    let cmd_factor = match kind.group() {
        crate::profile::PitchGroup::Fastball => 1.0,
        crate::profile::PitchGroup::Breaking => 1.12,
        crate::profile::PitchGroup::Offspeed => 1.10,
    };
    PitchDef { kind, shape, usage_same: w * same, usage_opp: w * opp, command: command_in * cmd_factor * INCH }
}

/// A club: enough players to field a team for any game.
#[derive(Clone, Debug)]
pub struct Club {
    /// Position players in defensive order C, 1B, 2B, 3B, SS, LF, CF, RF, then the DH.
    pub hitters: Vec<Player>,
    pub starters: Vec<Player>,
    pub relievers: Vec<Player>,
    /// Reserves: a catcher, a middle infielder, an outfielder and a corner infielder.
    pub bench: Vec<Player>,
}

impl Club {
    pub fn random(rng: &mut Rng, id_base: u32, level: &Level) -> Self {
        let mut hitters = Vec::with_capacity(9);
        for (i, pos) in [C, B1, B2, B3, SS, LF, CF, RF].into_iter().enumerate() {
            let mut p = random_batter(rng, id_base + i as u32, level);
            fit_to_position(&mut p, pos, rng);
            hitters.push(p);
        }
        hitters.push(random_batter(rng, id_base + 8, level));
        let starters = (0..5).map(|i| random_pitcher(rng, id_base + 20 + i, true, level)).collect();
        let relievers = (0..8).map(|i| random_pitcher(rng, id_base + 30 + i, false, level)).collect();
        // The bench is drawn from its own generator (see `random_batter`). Reserves are
        // a little short of the regulars with the bat.
        let mut side = Rng::new(0xBE7C_4000 ^ ((id_base as u64) << 12));
        let bench = [C, SS, CF, B3]
            .into_iter()
            .enumerate()
            .map(|(i, pos)| {
                let mut p = random_batter(&mut side, id_base + 50 + i as u32, level);
                fit_to_position(&mut p, pos, &mut side);
                p.batter.bat_speed -= 1.2 * MPH;
                p.batter.barrel_sd *= 1.06;
                p
            })
            .collect();
        Club { hitters, starters, relievers, bench }
    }

    /// Line-up for one game with the given starter (0-based rotation slot).
    pub fn team_for_game(&self, starter: usize, dh: bool) -> Team {
        let mut players: Vec<Player> = self.hitters.clone();
        let sp = players.len();
        players.push(self.starters[starter % self.starters.len()].clone());
        let pen_start = players.len();
        players.extend(self.relievers.iter().cloned());
        let bullpen: Vec<usize> = (pen_start..players.len()).collect();
        let bench_start = players.len();
        players.extend(self.bench.iter().cloned());
        let bench: Vec<usize> = (bench_start..players.len()).collect();

        // Defence: P, C, 1B, 2B, 3B, SS, LF, CF, RF.
        let mut defense = [0usize; 9];
        defense[P] = sp;
        for (slot, d) in defense.iter_mut().enumerate().skip(1) {
            *d = slot - 1;
        }
        // Batting order: best bats first (bat speed and barrel control), pitcher or weakest last.
        let mut order: Vec<usize> = if dh { (0..9).collect() } else { (0..8).collect() };
        let quality = |i: usize| {
            let b = &players[i].batter;
            b.bat_speed / (71.9 * MPH) - b.barrel_sd / 0.034 * 0.6 + b.eye * 0.5
        };
        order.sort_by(|&a, &b| quality(b).partial_cmp(&quality(a)).unwrap_or(std::cmp::Ordering::Equal));
        if !dh {
            order.push(sp);
        }
        let mut lineup = [0usize; 9];
        lineup.copy_from_slice(&order[..9]);
        Team { players, lineup, defense, bullpen, bench }
    }
}
