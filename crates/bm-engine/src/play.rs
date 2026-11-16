//! Resolution of a batted ball: catch, fair/foul, fielding, throws, and
//! base-running. Every out is the result of a race between the ball (flight time,
//! fielder arrival, transfer, throw) and the runners.

use crate::batting::BattedInfo;
use crate::fielding::{arrival_time, best_catch, best_pickup, move_time, throw_time, B1, B2, C, LF, P, SS};
use crate::params::ModelParams;
use crate::pitching::MatchEnv;
use crate::profile::FielderProfile;
use crate::rng::{sigmoid, Rng};
use bm_physics::{base_position, is_fair, BattedPath, PathEnd, Vec3, BASE_PATH};

/// A runner on base (or the batter-runner).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Runner {
    /// Index of the player in the batting team.
    pub player: usize,
    pub sprint: f64,
    pub accel: f64,
    /// Base-running aggressiveness, 0..=1.
    pub aggr: f64,
    /// Multiplier on the error of his estimate of when the throw will arrive (1 = average).
    pub judge: f64,
    /// Index (in the fielding team) of the pitcher charged with this runner.
    pub pitcher: usize,
    /// False if the runner reached base through an error (run is unearned).
    pub earned: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayKind {
    /// Foul ball, not caught: the plate appearance continues.
    Foul,
    Single,
    Double,
    Triple,
    HomeRun,
    GroundRuleDouble,
    GroundOut,
    FlyOut,
    LineOut,
    PopOut,
    FoulOut,
    DoublePlay,
    FieldersChoice,
    Error,
    SacFly,
    /// Sacrifice bunt: the batter is out and a runner moved up.
    SacBunt,
}

impl PlayKind {
    pub fn is_hit(self) -> bool {
        matches!(self, PlayKind::Single | PlayKind::Double | PlayKind::Triple | PlayKind::HomeRun | PlayKind::GroundRuleDouble)
    }
}

/// A throw made during a play (or a fielder carrying the ball to a base himself).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThrowLeg {
    /// Position index of the thrower.
    pub by: usize,
    pub from: Vec3,
    /// Base thrown to (1..=3, 4 = home).
    pub to_base: u8,
    pub t_release: f64,
    pub t_arrive: f64,
    /// The fielder ran the ball to the base instead of throwing it.
    pub carried: bool,
}

/// Movement of the batter or a runner during a play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunLeg {
    /// Index of the player in the batting team.
    pub player: usize,
    /// Base he started from (0 = the batter's box).
    pub from: u8,
    /// Base he reached or was put out at (4 = home).
    pub to: u8,
    pub t_start: f64,
    pub t_arrive: f64,
    pub out: bool,
}

/// Timeline of a play, for the match view. Times are seconds after contact.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayTrace {
    /// Time at which the ball was caught or picked up (0 if it left the field).
    pub fielded_t: f64,
    /// The ball was caught in the air.
    pub caught: bool,
    pub throws: [Option<ThrowLeg>; 3],
    pub runs: [Option<RunLeg>; 4],
}

impl PlayTrace {
    fn throw(&mut self, by: usize, from: Vec3, to_base: usize, t_release: f64, t_arrive: f64, carried: bool) {
        if let Some(slot) = self.throws.iter_mut().find(|t| t.is_none()) {
            *slot = Some(ThrowLeg { by, from, to_base: to_base as u8, t_release, t_arrive: t_arrive.max(t_release), carried });
        }
    }
    fn run(&mut self, r: &Runner, from: usize, to: usize, t_start: f64, t_arrive: f64, out: bool) {
        if let Some(slot) = self.runs.iter_mut().find(|t| t.is_none()) {
            *slot = Some(RunLeg { player: r.player, from: from as u8, to: to as u8, t_start, t_arrive: t_arrive.max(t_start), out });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayResult {
    pub kind: PlayKind,
    /// Outs recorded on the play.
    pub outs: u8,
    /// Runners on first, second, third after the play.
    pub bases: [Option<Runner>; 3],
    pub scored: [Option<Runner>; 4],
    pub n_scored: u8,
    pub rbi: u8,
    /// Position index of the fielder who made the first play on the ball.
    pub fielder: usize,
    /// Where the ball was first touched by a fielder (or where it left the field), m.
    pub field_pos: Vec3,
    /// The batter grounded into a double play.
    pub gidp: bool,
    /// A runner was thrown out trying to advance on a hit or a fly ball.
    pub runner_thrown_out: bool,
    pub trace: PlayTrace,
}

impl PlayResult {
    fn new(kind: PlayKind, bases: [Option<Runner>; 3]) -> Self {
        PlayResult {
            kind,
            outs: 0,
            bases,
            scored: [None; 4],
            n_scored: 0,
            rbi: 0,
            fielder: 0,
            field_pos: Vec3::ZERO,
            gidp: false,
            runner_thrown_out: false,
            trace: PlayTrace::default(),
        }
    }
    fn score(&mut self, r: Runner, rbi: bool) {
        self.scored[self.n_scored as usize] = Some(r);
        self.n_scored += 1;
        if rbi {
            self.rbi += 1;
        }
    }
}

pub struct PlayInput<'a> {
    pub env: &'a MatchEnv,
    pub path: &'a BattedPath,
    pub info: &'a BattedInfo,
    pub fielders: &'a [FielderProfile; 9],
    pub positions: &'a [Vec3; 9],
    pub bases: [Option<Runner>; 3],
    pub batter: Runner,
    /// Time before the batter leaves the box, s.
    pub box_delay: f64,
    pub outs: u8,
    /// Seconds each runner (on first, second, third) had already been running at
    /// contact because he left with the pitch; 0 = he was not going.
    pub running: [f64; 3],
    /// The infield is drawn in to cut the run off at the plate.
    pub infield_in: bool,
}

#[inline]
fn gdist(a: Vec3, b: Vec3) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Running times of the batter and the runners for one play.
struct Clock<'a> {
    p: &'a ModelParams,
    /// Time at which the runners commit to advancing (they go on contact on a ground
    /// ball or with two outs, and wait to see whether a ball in the air will drop).
    go: f64,
    /// Head start of runners who left with the pitch (see `PlayInput::running`).
    running: [f64; 3],
}

impl Clock<'_> {
    /// Time at which a runner starting on `base` (1..=3) reaches the base `n` further on.
    fn runner(&self, r: &Runner, base: usize, n: usize) -> f64 {
        let lead = self.p.lead[base - 1] + 1.2;
        let turns = (n as f64 - 1.0) * self.p.turn_length;
        let finish = if base + n >= 4 { 0.0 } else { self.p.slide_time };
        let v = r.sprint * 0.97;
        let head = self.running[base - 1];
        if head > 0.0 {
            // He left with the pitch and never stopped.
            return -head + move_time(n as f64 * BASE_PATH - lead + turns, r.accel, v) + finish;
        }
        // While reading the ball the runner edges off the base.
        let drift = (lead + 0.4 * v * (self.go - 0.15).max(0.0)).min(0.42 * BASE_PATH);
        self.go + move_time(n as f64 * BASE_PATH - drift + turns, r.accel, v) + finish
    }

    /// Time at which the batter reaches the `n`-th base.
    fn batter(&self, b: &Runner, delay: f64, n: usize) -> f64 {
        let turns = (n as f64 - 1.0) * self.p.turn_length;
        let finish = if n == 1 || n == 4 { 0.0 } else { self.p.slide_time };
        delay + move_time(n as f64 * BASE_PATH + turns, b.accel, b.sprint * 0.97) + finish
    }

    /// Tag up at `catch_t` and run one base.
    fn tag_up(&self, r: &Runner, catch_t: f64, to_home: bool) -> f64 {
        catch_t + 0.10 + move_time(BASE_PATH, r.accel, r.sprint * 0.97) + if to_home { 0.0 } else { self.p.slide_time }
    }
}

/// Throw from an outfielder, relayed when it is too long for one throw. `slop` is
/// the average time lost to throws that are off line, cut off or have to be handled
/// on a hop before a tag can be made.
fn long_throw(d: f64, arm: f64, drag: f64, slop: f64) -> f64 {
    slop + if d > 62.0 {
        throw_time(d * 0.58, arm, drag) + 0.85 + throw_time(d * 0.42, 36.5, drag)
    } else {
        throw_time(d, arm, drag) + if d > 45.0 { 0.08 } else { 0.0 }
    }
}

/// Is the ball fair, given where it was (or would be) first touched on the ground?
fn fair_ball(path: &BattedPath, pickup_t: f64) -> bool {
    let land = match path.landing {
        Some(l) => l,
        None => return false,
    };
    if land.pos.horiz() >= BASE_PATH {
        return is_fair(land.pos);
    }
    for pt in &path.points {
        if pt.t < land.t {
            continue;
        }
        if pt.pos.horiz() >= BASE_PATH || pt.t >= pickup_t {
            return is_fair(pt.pos);
        }
    }
    is_fair(path.last().pos)
}

/// Resolve a batted ball.
pub fn resolve(inp: &PlayInput, rng: &mut Rng) -> PlayResult {
    let p = &inp.env.params;
    let path = inp.path;
    let home = base_position(0);

    // ---- over the fence ----
    if path.end == PathEnd::OverFence {
        let clock = Clock { p, go: 0.15, running: [0.0; 3] };
        if path.landing.is_none() {
            let mut res = PlayResult::new(PlayKind::HomeRun, [None; 3]);
            res.field_pos = path.last().pos;
            for (i, r) in inp.bases.iter().enumerate().rev() {
                if let Some(r) = r {
                    res.score(*r, true);
                    res.trace.run(r, i + 1, 4, clock.go, clock.runner(r, i + 1, 3 - i), false);
                }
            }
            res.score(inp.batter, true);
            res.trace.run(&inp.batter, 0, 4, inp.box_delay, clock.batter(&inp.batter, inp.box_delay, 4), false);
            return res;
        }
        let mut res = PlayResult::new(PlayKind::GroundRuleDouble, [None, Some(inp.batter), inp.bases[0]]);
        res.field_pos = path.last().pos;
        for (i, r) in [(2usize, inp.bases[2]), (1, inp.bases[1])] {
            if let Some(r) = r {
                res.score(r, true);
                res.trace.run(&r, i + 1, 4, clock.go, clock.runner(&r, i + 1, 3 - i), false);
            }
        }
        if let Some(r) = inp.bases[0] {
            res.trace.run(&r, 1, 3, clock.go, clock.runner(&r, 1, 2), false);
        }
        res.trace.run(&inp.batter, 0, 2, inp.box_delay, clock.batter(&inp.batter, inp.box_delay, 2), false);
        return res;
    }

    // ---- ball in the air: can it be caught? ----
    let end = path.last();
    let t_land = path.landing.map_or(end.t, |l| l.t);
    let t_air_end = match path.wall {
        Some(w) if w.t < t_land => w.t,
        _ => t_land,
    };
    let chance = best_catch(path, t_air_end, inp.fielders, inp.positions, p);
    if let Some(c) = chance {
        let reach_prob = sigmoid((c.margin - p.catch_margin_bias) / p.catch_margin_scale);
        if rng.chance(reach_prob) {
            let f = &inp.fielders[c.fielder];
            let fair = is_fair(c.pos);
            if rng.chance((p.error_fly * f.hands).min(0.5)) && fair {
                return dropped_fly(inp, c.fielder, c.t, c.pos);
            }
            return caught(inp, c.fielder, c.t, c.pos, fair, rng);
        }
    }

    // ---- not caught: fair or foul? ----
    let landing_t = match path.landing {
        Some(l) => l.t,
        None => return PlayResult::new(PlayKind::Foul, inp.bases),
    };
    let pickup = best_pickup(path, landing_t, inp.fielders, inp.positions, p);
    if !fair_ball(path, pickup.t) {
        return PlayResult::new(PlayKind::Foul, inp.bases);
    }

    let grounder = path.landing.is_some_and(|l| l.t < 0.9 && l.pos.horiz() < 45.0);
    // Runners go on contact on the ground or with two outs. On a ball in the air they
    // read it: if no fielder has a play they are off early, otherwise they wait for it to drop.
    let go = if grounder || inp.outs >= 2 || landing_t <= 0.9 {
        0.15
    } else if chance.is_none_or(|c| c.margin < p.read_margin) {
        (p.read_time + p.read_hang_share * landing_t).min(landing_t + 0.12)
    } else {
        landing_t + 0.12
    };
    let clock = Clock { p, go, running: inp.running };
    let mut res = if grounder && pickup.fielder <= SS && pickup.pos.horiz() < 55.0 {
        infield_play(inp, &clock, &pickup, rng)
    } else {
        outfield_play(inp, &clock, &pickup, home, rng)
    };
    res.field_pos = pickup.pos;
    // A bunt that retires only the batter and moves a runner up is a sacrifice.
    if inp.info.bunt && res.kind == PlayKind::GroundOut && inp.outs < 2 && res.outs == 1 {
        let moved = (0..3).any(|i| inp.bases[i].is_some_and(|r| res.bases[i].map(|x| x.player) != Some(r.player)));
        if moved {
            res.kind = PlayKind::SacBunt;
        }
    }
    res
}

/// A catchable fly ball was dropped: everyone moves up a base, the batter reaches.
fn dropped_fly(inp: &PlayInput, fielder: usize, t: f64, pos: Vec3) -> PlayResult {
    let mut res = PlayResult::new(PlayKind::Error, [None; 3]);
    res.fielder = fielder;
    res.field_pos = pos;
    res.trace.fielded_t = t;
    let mut batter = inp.batter;
    batter.earned = false;
    if let Some(r) = inp.bases[2] {
        res.score(r, false);
    }
    res.bases = [Some(batter), inp.bases[0], inp.bases[1]];
    // The runners were holding until the ball came down.
    let clock = Clock { p: &inp.env.params, go: t, running: [0.0; 3] };
    for (i, r) in inp.bases.iter().enumerate() {
        if let Some(r) = r {
            res.trace.run(r, i + 1, i + 2, t, clock.runner(r, i + 1, 1), false);
        }
    }
    res.trace.run(&inp.batter, 0, 1, inp.box_delay, clock.batter(&inp.batter, inp.box_delay, 1), false);
    res
}

/// The ball was caught in the air: the batter is out and runners may tag up.
fn caught(inp: &PlayInput, fielder: usize, t: f64, pos: Vec3, fair: bool, rng: &mut Rng) -> PlayResult {
    let p = &inp.env.params;
    let la = inp.info.launch_angle.to_degrees();
    let kind = if !fair {
        PlayKind::FoulOut
    } else if la > 50.0 {
        PlayKind::PopOut
    } else if la > 25.0 {
        PlayKind::FlyOut
    } else {
        PlayKind::LineOut
    };
    let mut res = PlayResult::new(kind, inp.bases);
    res.fielder = fielder;
    res.field_pos = pos;
    res.outs = 1;
    res.trace.fielded_t = t;
    res.trace.caught = true;
    // Runners who left with the pitch have to get back before the ball does. On a
    // ball that hangs in the air they see the catch coming and retreat in time.
    if t < 2.0 {
        for i in 0..3 {
            let head = inp.running[i];
            let Some(r) = inp.bases[i] else { continue };
            if head <= 0.0 || inp.outs + res.outs >= 3 {
                continue;
            }
            let v = r.sprint * 0.97;
            let (run_t, t_acc) = (head + t, v / r.accel);
            let ran = if run_t < t_acc { 0.5 * r.accel * run_t * run_t } else { 0.5 * r.accel * t_acc * t_acc + v * (run_t - t_acc) };
            let off = (p.lead[i] + 1.2 + ran).min(BASE_PATH - 1.0);
            let back = t + 0.35 + move_time(off, r.accel, v);
            let f = &inp.fielders[fielder];
            let base = base_position(i + 1);
            let release = t + f.transfer * 0.9;
            let arrive = release + throw_time(gdist(pos, base), f.arm, p.throw_drag);
            if arrive + rng.gauss(0.0, p.play_noise) < back {
                res.outs += 1;
                res.bases[i] = None;
                res.runner_thrown_out = true;
                res.trace.throw(fielder, pos, i + 1, release, arrive, false);
                res.trace.run(&r, i + 1, i + 1, t, back, true);
            }
        }
    }
    if inp.outs + res.outs >= 3 || fielder < LF {
        return res;
    }

    // Tag-ups on a catch by an outfielder.
    let f = &inp.fielders[fielder];
    let clock = Clock { p, go: 0.15, running: [0.0; 3] };
    let release = t + f.transfer * 0.9;
    let throw_to = |target: Vec3| release + long_throw(gdist(pos, target), f.arm, p.throw_drag, p.throw_slop);
    let mut bases = res.bases;
    // Runner on third tries to score.
    if let Some(r) = bases[2] {
        let eta = clock.tag_up(&r, t, true);
        let arrive = throw_to(base_position(0));
        let margin = p.advance_margin * (1.2 - 0.6 * r.aggr);
        if eta + margin < arrive + rng.gauss(0.0, 0.30 * r.judge) {
            bases[2] = None;
            let safe = eta + rng.gauss(0.0, 0.05) < arrive + p.tag_time + rng.gauss(0.0, p.play_noise);
            if safe {
                res.score(r, true);
                res.kind = PlayKind::SacFly;
            } else {
                res.outs += 1;
                res.runner_thrown_out = true;
            }
            res.trace.run(&r, 3, 4, t + 0.10, eta, !safe);
            res.trace.throw(fielder, pos, 4, release, arrive, false);
        }
    }
    // Runner on second tags to third on a deep fly (only if third is open).
    if res.outs + inp.outs < 3 && bases[2].is_none() {
        if let Some(r) = bases[1] {
            let eta = clock.tag_up(&r, t, false);
            let arrive = throw_to(base_position(3));
            let margin = p.advance_margin * (1.3 - 0.6 * r.aggr);
            if eta + margin < arrive + rng.gauss(0.0, 0.30 * r.judge) {
                bases[1] = None;
                let safe = eta + rng.gauss(0.0, 0.05) < arrive + p.tag_time + rng.gauss(0.0, p.play_noise);
                if safe {
                    bases[2] = Some(r);
                } else {
                    res.outs += 1;
                    res.runner_thrown_out = true;
                }
                res.trace.run(&r, 2, 3, t + 0.10, eta, !safe);
                // One ball: the throw is shown going to third only when nobody went home.
                if res.trace.throws[0].is_none() {
                    res.trace.throw(fielder, pos, 3, release, arrive, false);
                }
            }
        }
    }
    res.bases = bases;
    res
}

/// Ground ball handled by an infielder.
fn infield_play(inp: &PlayInput, clock: &Clock, pk: &crate::fielding::Pickup, rng: &mut Rng) -> PlayResult {
    let p = &inp.env.params;
    let f = &inp.fielders[pk.fielder];
    let [r1, r2, r3] = inp.bases;
    let first = base_position(1);
    let second = base_position(2);
    let batter_eta = clock.batter(&inp.batter, inp.box_delay, 1);
    let two_outs = inp.outs == 2;

    // Fielding error: everybody is safe and moves up one base.
    let difficulty = 1.0 + 0.5 * (pk.ball_speed / 30.0).powi(2) + pk.stretch;
    if rng.chance((p.error_ground * f.hands * difficulty).min(0.5)) {
        let mut res = PlayResult::new(PlayKind::Error, [None; 3]);
        res.fielder = pk.fielder;
        res.trace.fielded_t = pk.t;
        let mut batter = inp.batter;
        batter.earned = false;
        if let Some(r) = r3 {
            res.score(r, false);
        }
        res.bases = [Some(batter), r1, r2];
        for (i, r) in inp.bases.iter().enumerate() {
            if let Some(r) = r {
                res.trace.run(r, i + 1, i + 2, clock.go, clock.runner(r, i + 1, 1), false);
            }
        }
        res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, false);
        return res;
    }

    let release = pk.t + p.gather_time + f.transfer * (1.0 + 0.5 * pk.stretch);
    let arm = f.arm * (1.0 - 0.12 * pk.stretch);
    let throw_from = |target: Vec3| release + throw_time(gdist(pk.pos, target), arm, p.throw_drag);
    let wild = |rng: &mut Rng| rng.chance((p.error_throw * f.arm_accuracy * (1.0 + pk.stretch)).min(0.5));

    // ---- bunt with a runner on second: go for the lead runner at third when it is comfortable ----
    if inp.info.bunt && !two_outs && r3.is_none() {
        if let Some(lead2) = r2 {
            let third_base = base_position(3);
            let t3 = throw_from(third_base) + if r1.is_some() { 0.0 } else { p.tag_time };
            let eta3 = clock.runner(&lead2, 2, 1);
            if t3 + p.force_margin + p.bunt_lead_margin + rng.gauss(0.0, p.play_noise) < eta3 {
                let mut res = PlayResult::new(PlayKind::FieldersChoice, [Some(inp.batter), r1, None]);
                res.fielder = pk.fielder;
                res.outs = 1;
                res.trace.fielded_t = pk.t;
                res.trace.throw(pk.fielder, pk.pos, 3, release, t3, false);
                res.trace.run(&lead2, 2, 3, clock.go, eta3, true);
                if let Some(r) = r1 {
                    res.trace.run(&r, 1, 2, clock.go, clock.runner(&r, 1, 1), false);
                }
                res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, false);
                return res;
            }
        }
    }

    // ---- bases loaded with the infield in: force at the plate, then try for the batter ----
    if inp.infield_in && !two_outs {
        if let (Some(run3), Some(run2), Some(run1)) = (r3, r2, r1) {
            let home = base_position(0);
            let t_home = throw_from(home);
            let eta = clock.runner(&run3, 3, 1);
            if t_home + p.force_margin + rng.gauss(0.0, p.play_noise) < eta {
                let mut res = PlayResult::new(PlayKind::FieldersChoice, [None, Some(run1), Some(run2)]);
                res.fielder = pk.fielder;
                res.outs = 1;
                res.trace.fielded_t = pk.t;
                let t1 = t_home + p.pivot_time + throw_time(BASE_PATH, inp.fielders[C].arm * 0.95, p.throw_drag);
                let doubled = inp.outs + 1 < 3 && t1 + rng.gauss(0.0, p.play_noise) < batter_eta;
                if doubled {
                    res.outs = 2;
                    res.kind = PlayKind::DoublePlay;
                    res.gidp = true;
                } else {
                    res.bases[0] = Some(inp.batter);
                }
                res.trace.throw(pk.fielder, pk.pos, 4, release, t_home, false);
                res.trace.throw(C, home, 1, t_home + p.pivot_time, t1, false);
                res.trace.run(&run3, 3, 4, clock.go, eta, true);
                res.trace.run(&run2, 2, 3, clock.go, clock.runner(&run2, 2, 1), false);
                res.trace.run(&run1, 1, 2, clock.go, clock.runner(&run1, 1, 1), false);
                res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, doubled);
                return res;
            }
        }
    }
    // With the infield in, a runner on third who is not forced holds on a ball an infielder gets to.
    let third_holds = inp.infield_in && !two_outs && !(r1.is_some() && r2.is_some());

    // ---- force at second with fewer than two outs: try for two ----
    if let (Some(lead), false) = (r1, two_outs) {
        let near_bag = (pk.fielder == SS || pk.fielder == B2) && gdist(pk.pos, second) < 5.0;
        // The feed to second is a short, quick throw.
        let feed = pk.t + p.gather_time + f.transfer * p.feed_transfer * (1.0 + 0.5 * pk.stretch) + throw_time(gdist(pk.pos, second), arm * 0.9, p.throw_drag);
        let t2 = if near_bag { pk.t + p.gather_time + move_time(gdist(pk.pos, second), f.accel, f.speed) } else { feed };
        let lead_eta = clock.runner(&lead, 1, 1);
        // The defence only goes after the lead runner when the force is comfortable,
        // and on a bunt it wants to be surer still.
        let margin = p.force_margin + if inp.info.bunt { p.bunt_lead_margin } else { 0.0 };
        if t2 + margin + rng.gauss(0.0, p.play_noise) < lead_eta {
            let mut res = PlayResult::new(PlayKind::FieldersChoice, [None; 3]);
            res.fielder = pk.fielder;
            res.outs = 1;
            res.trace.fielded_t = pk.t;
            let pivot_pos = if pk.pos.x < 0.0 { B2 } else { SS };
            let pivot = &inp.fielders[pivot_pos];
            let turn = if near_bag { f.transfer * 0.8 } else { p.pivot_time };
            let t1 = t2 + turn + throw_time(BASE_PATH, pivot.arm * 0.95, p.throw_drag);
            let doubled = inp.outs + 1 < 3 && t1 + rng.gauss(0.0, p.play_noise) < batter_eta;
            if doubled {
                res.outs = 2;
                res.kind = PlayKind::DoublePlay;
                res.gidp = true;
            } else {
                res.bases[0] = Some(inp.batter);
            }
            let feed_release = if near_bag { pk.t + p.gather_time } else { feed - throw_time(gdist(pk.pos, second), arm * 0.9, p.throw_drag) };
            res.trace.throw(pk.fielder, pk.pos, 2, feed_release, t2, near_bag);
            if inp.outs + 1 < 3 {
                res.trace.throw(if near_bag { pk.fielder } else { pivot_pos }, second, 1, t2 + turn, t1, false);
            }
            res.trace.run(&lead, 1, 2, clock.go, lead_eta, true);
            res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, doubled);
            if inp.outs + res.outs < 3 {
                res.bases[2] = r2;
                if let Some(r) = r2 {
                    res.trace.run(&r, 2, 3, clock.go, clock.runner(&r, 2, 1), false);
                }
                if let Some(r) = r3 {
                    if third_holds && r2.is_none() {
                        res.bases[2] = Some(r);
                    } else {
                        res.score(r, !doubled);
                        res.trace.run(&r, 3, 4, clock.go, clock.runner(&r, 3, 1), false);
                    }
                }
            }
            return res;
        }
        // No play at second: take the out at first if there is one.
        let t1 = throw_from(first);
        let mut res = PlayResult::new(PlayKind::Single, [None; 3]);
        res.fielder = pk.fielder;
        res.trace.fielded_t = pk.t;
        let batter_out = t1 + rng.gauss(0.0, p.play_noise) < batter_eta;
        if batter_out {
            res.kind = PlayKind::GroundOut;
            res.outs = 1;
            res.bases = [None, Some(lead), r2];
        } else {
            res.bases = [Some(inp.batter), Some(lead), r2];
        }
        if let Some(r) = r3 {
            if third_holds && r2.is_none() {
                res.bases[2] = Some(r);
            } else {
                res.score(r, true);
                res.trace.run(&r, 3, 4, clock.go, clock.runner(&r, 3, 1), false);
            }
        }
        if let Some(r) = r2 {
            res.trace.run(&r, 2, 3, clock.go, clock.runner(&r, 2, 1), false);
        }
        res.trace.run(&lead, 1, 2, clock.go, lead_eta, false);
        res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, batter_out);
        res.trace.throw(pk.fielder, pk.pos, 1, release, t1, false);
        return res;
    }

    // ---- play at first ----
    let unassisted = pk.fielder == B1 && gdist(pk.pos, first) < 8.0;
    let t1 = if unassisted { pk.t + p.gather_time + move_time(gdist(pk.pos, first), f.accel, f.speed) } else { throw_from(first) };
    let in_time = t1 + rng.gauss(0.0, p.play_noise) < batter_eta;
    if in_time && !unassisted && wild(rng) {
        // The throw gets away: the batter takes second and every runner moves up two bases.
        let mut res = PlayResult::new(PlayKind::Error, [None; 3]);
        res.fielder = pk.fielder;
        res.trace.fielded_t = pk.t;
        let mut batter = inp.batter;
        batter.earned = false;
        for (i, r) in [(3usize, r3), (2, r2)] {
            if let Some(r) = r {
                res.score(r, false);
                res.trace.run(&r, i, 4, clock.go, clock.runner(&r, i, 4 - i), false);
            }
        }
        if let Some(r) = r1 {
            res.trace.run(&r, 1, 3, clock.go, clock.runner(&r, 1, 2), false);
        }
        res.trace.run(&inp.batter, 0, 2, inp.box_delay, clock.batter(&inp.batter, inp.box_delay, 2), false);
        res.trace.throw(pk.fielder, pk.pos, 1, release, t1, false);
        res.bases = [None, Some(batter), r1];
        return res;
    }
    let batter_out = in_time;
    let mut res = PlayResult::new(if batter_out { PlayKind::GroundOut } else { PlayKind::Single }, [None; 3]);
    res.fielder = pk.fielder;
    res.trace.fielded_t = pk.t;
    res.trace.run(&inp.batter, 0, 1, inp.box_delay, batter_eta, batter_out);
    res.trace.throw(pk.fielder, pk.pos, 1, if unassisted { pk.t + p.gather_time } else { release }, t1, unassisted);
    if batter_out {
        res.outs = 1;
        if inp.outs + 1 >= 3 {
            res.bases = inp.bases;
            return res;
        }
    } else {
        res.bases[0] = Some(inp.batter);
    }
    // Runners. With two outs they were running on contact; otherwise the runner on third
    // goes on a ball to the middle infield and the runner on second on a ball behind him.
    let forced_from_first = !batter_out && r1.is_some();
    let third_goes = two_outs || (forced_from_first && r2.is_some()) || !(third_holds || pk.fielder == P || pk.fielder == C || pk.pos.horiz() < 28.0);
    // A runner on second moves up on a ball hit behind him, and often on a ball to the
    // left side once the throw goes across the diamond.
    let second_goes = two_outs || forced_from_first || inp.info.bunt || pk.pos.x > 0.0 || rng.chance(p.advance_left_side);
    let mut third = None;
    if let Some(r) = r3 {
        if third_goes {
            res.score(r, true);
            res.trace.run(&r, 3, 4, clock.go, clock.runner(&r, 3, 1), false);
        } else {
            third = Some(r);
        }
    }
    let mut second = None;
    if let Some(r) = r2 {
        if second_goes && third.is_none() {
            third = Some(r);
            res.trace.run(&r, 2, 3, clock.go, clock.runner(&r, 2, 1), false);
        } else {
            second = Some(r);
        }
    }
    if let Some(r) = r1 {
        // Only reachable with two outs (otherwise the force play above handled it).
        if second.is_none() {
            second = Some(r);
            res.trace.run(&r, 1, 2, clock.go, clock.runner(&r, 1, 1), false);
        } else {
            res.bases[0] = Some(r);
        }
    }
    res.bases[1] = second;
    res.bases[2] = third;
    res
}

/// Ball that got through or dropped in: retrieved and thrown back in while the
/// runners decide how far to go.
fn outfield_play(inp: &PlayInput, clock: &Clock, pk: &crate::fielding::Pickup, home: Vec3, rng: &mut Rng) -> PlayResult {
    let p = &inp.env.params;
    let f = &inp.fielders[pk.fielder];
    // A ball that reaches the wall has to be played off the carom.
    let (fence, _) = inp.env.park.boundary(pk.pos.x.atan2(pk.pos.y));
    let carom = if pk.pos.horiz() > fence - 4.0 { p.wall_play_delay } else { 0.0 };
    let release = pk.t + f.transfer + 0.15 + carom;
    let drag = p.throw_drag;
    // Arrival of a throw at second, third, home (index = base number).
    let slop = p.throw_slop;
    let arrive = [
        0.0,
        0.0,
        release + long_throw(gdist(pk.pos, base_position(2)), f.arm, drag, slop),
        release + long_throw(gdist(pk.pos, base_position(3)), f.arm, drag, slop),
        release + long_throw(gdist(pk.pos, home), f.arm, drag, slop),
    ];
    let two_outs = inp.outs == 2;
    let bobble = rng.chance((p.error_fly * f.hands).min(0.5));
    let extra = if bobble { 1.4 } else { 0.0 };

    // Each mover: (runner, starting base 0..=3, final base 1..=4, attempted an extra base)
    #[derive(Clone, Copy)]
    struct Mover {
        r: Runner,
        from: usize,
        to: usize,
        eta: f64,
        pushed: bool,
    }
    let mut movers: [Option<Mover>; 4] = [None; 4];
    let mut limit = 5usize; // nearest occupied base ahead (exclusive upper bound for the next runner)
    let starters = [(inp.bases[2], 3usize), (inp.bases[1], 2), (inp.bases[0], 1), (Some(inp.batter), 0)];
    for (slot, (maybe, from)) in starters.into_iter().enumerate() {
        let Some(r) = maybe else { continue };
        let eta_to = |n: usize| if from == 0 { clock.batter(&r, inp.box_delay, n) } else { clock.runner(&r, from, n) };
        let mut to = (from + 1).min(limit - 1).max(from);
        let mut eta = if to > from { eta_to(to - from) } else { 0.0 };
        let mut pushed = false;
        let margin = p.advance_margin * (1.3 - 0.6 * r.aggr) * if two_outs { 0.5 } else { 1.0 };
        let mut target = to + 1;
        while target <= 4 && target < limit {
            let e = eta_to(target - from);
            let throw_est = arrive[target] + extra + rng.gauss(0.0, 0.30 * r.judge);
            if e + margin < throw_est {
                to = target;
                eta = e;
                pushed = true;
                target += 1;
            } else {
                break;
            }
        }
        movers[slot] = Some(Mover { r, from, to, eta, pushed });
        limit = if to >= 4 { 5 } else { to };
    }

    // The defence throws to the closest play among the runners who took an extra base.
    let mut play: Option<usize> = None;
    let mut best_slack = 0.9;
    for (i, m) in movers.iter().enumerate() {
        if let Some(m) = m {
            if m.pushed && m.to >= 2 {
                let slack = arrive[m.to] + extra - m.eta;
                if slack < best_slack {
                    best_slack = slack;
                    play = Some(i);
                }
            }
        }
    }

    let batter_reached = movers[3].map_or(1, |m| m.to);
    let mut res = PlayResult::new(
        match batter_reached {
            1 => PlayKind::Single,
            2 => PlayKind::Double,
            3 => PlayKind::Triple,
            _ => PlayKind::HomeRun,
        },
        [None; 3],
    );
    res.fielder = pk.fielder;

    let mut out_slot = None;
    if let Some(i) = play {
        let m = movers[i].unwrap();
        if arrive[m.to] + extra + p.tag_time + rng.gauss(0.0, p.play_noise) < m.eta + rng.gauss(0.0, 0.05) {
            out_slot = Some(i);
            res.outs = 1;
            res.runner_thrown_out = true;
            if i == 3 {
                // Batter thrown out stretching: credited with the base before.
                res.kind = match m.to {
                    2 => PlayKind::Single,
                    3 => PlayKind::Double,
                    _ => PlayKind::Triple,
                };
            }
        } else if m.to == 4 || m.to == 3 {
            // The throw went through: a trailing batter takes second.
            let second_open = movers[2].is_none_or(|m1| m1.to >= 3);
            if let Some(b) = movers[3].as_mut() {
                if b.to == 1 && second_open && clock.batter(&b.r, inp.box_delay, 2) < arrive[m.to] + 1.0 {
                    b.to = 2;
                }
            }
        }
    }

    res.trace.fielded_t = pk.t;
    match play {
        Some(i) => {
            let to = movers[i].unwrap().to;
            res.trace.throw(pk.fielder, pk.pos, to, release + extra, arrive[to] + extra, false);
        }
        // Nobody to throw out: the ball comes back in to second base.
        None => res.trace.throw(pk.fielder, pk.pos, 2, release + extra, arrive[2] + extra, false),
    }
    for (i, m) in movers.iter().enumerate() {
        let Some(m) = m else { continue };
        if m.to > m.from {
            let (start, eta) =
                if m.from == 0 { (inp.box_delay, clock.batter(&m.r, inp.box_delay, m.to)) } else { (clock.go, clock.runner(&m.r, m.from, m.to - m.from)) };
            res.trace.run(&m.r, m.from, m.to, start, eta, out_slot == Some(i));
        }
    }

    let third_out_made = inp.outs + res.outs >= 3;
    for (i, m) in movers.iter().enumerate() {
        let Some(m) = m else { continue };
        if out_slot == Some(i) {
            continue;
        }
        if m.to >= 4 {
            // A run counts unless the third out was made first on this play.
            if !third_out_made || out_slot.is_some_and(|o| movers[o].unwrap().eta > m.eta) {
                res.score(m.r, true);
            }
        } else if m.to >= 1 {
            res.bases[m.to - 1] = Some(m.r);
        }
    }
    res
}

/// Probability-free helper used by the game loop: time for a fielder to reach a base.
pub fn cover_time(f: &FielderProfile, from: Vec3, base: usize) -> f64 {
    arrival_time(f, gdist(from, base_position(base)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fielding::standard_alignment;
    use bm_physics::units::{MPH, RPM};
    use bm_physics::{spin_from_components, velocity_from_angles, BallState, BattedSim};

    fn runner(player: usize, ft_per_s: f64) -> Runner {
        Runner { player, sprint: ft_per_s * 0.3048, accel: 4.3, aggr: 0.5, judge: 1.0, pitcher: 0, earned: true }
    }

    struct Rig {
        env: MatchEnv,
        fielders: [FielderProfile; 9],
        positions: [Vec3; 9],
    }

    impl Rig {
        fn new() -> Self {
            Rig { env: MatchEnv::neutral_mlb(), fielders: [FielderProfile::league_average(); 9], positions: standard_alignment() }
        }

        fn play(&self, ev: f64, la: f64, spray: f64, bases: [Option<Runner>; 3], outs: u8, seed: u64) -> PlayResult {
            self.play_with(ev, la, spray, bases, outs, [0.0; 3], false, seed)
        }

        #[allow(clippy::too_many_arguments)]
        fn play_with(&self, ev: f64, la: f64, spray: f64, bases: [Option<Runner>; 3], outs: u8, running: [f64; 3], infield_in: bool, seed: u64) -> PlayResult {
            let th = la.to_radians();
            let ph = spray.to_radians();
            let backspin = (-763.0 + 120.0 * la).clamp(-3000.0, 5000.0);
            let state =
                BallState::new(Vec3::new(0.0, 0.9, 0.85), velocity_from_angles(ev * MPH, th, ph), spin_from_components(th, ph, backspin * RPM, 0.0, 0.0));
            let mut path = BattedPath::default();
            BattedSim::new(&self.env.batted_flight, &self.env.ball, &self.env.park).run(state, &mut path);
            let info = BattedInfo {
                exit_speed: ev * MPH,
                launch_angle: th,
                spray: ph,
                backspin: 0.0,
                sidespin: 0.0,
                gyrospin: 0.0,
                bat_speed: 32.0,
                attack_angle: 0.17,
                bunt: false,
                shape: Default::default(),
            };
            let inp = PlayInput {
                env: &self.env,
                path: &path,
                info: &info,
                fielders: &self.fielders,
                positions: &self.positions,
                bases,
                batter: runner(9, 27.0),
                box_delay: 0.25,
                outs,
                running,
                infield_in,
            };
            resolve(&inp, &mut Rng::new(seed))
        }

        fn rate(&self, ev: f64, la: f64, spray: f64, bases: [Option<Runner>; 3], outs: u8, pred: impl Fn(&PlayResult) -> bool) -> f64 {
            let n = 300;
            (0..n).filter(|&s| pred(&self.play(ev, la, spray, bases, outs, s as u64 + 1))).count() as f64 / n as f64
        }
    }

    #[test]
    fn routine_plays() {
        let rig = Rig::new();
        let empty = [None; 3];
        // Routine fly ball to centre: an out essentially every time.
        assert!(rig.rate(90.0, 35.0, 0.0, empty, 0, |r| r.kind == PlayKind::FlyOut) > 0.97);
        // Routine grounder at the shortstop: an out almost every time.
        assert!(rig.rate(88.0, -6.0, -13.0, empty, 0, |r| r.kind == PlayKind::GroundOut) > 0.90);
        // No-doubter.
        assert!(rig.rate(110.0, 28.0, -10.0, empty, 0, |r| r.kind == PlayKind::HomeRun && r.rbi == 1) > 0.99);
        // Line drive into the right-centre gap falls for a hit; the same ball hit at the
        // right fielder is usually caught.
        assert!(rig.rate(100.0, 12.0, 13.0, empty, 0, |r| r.kind.is_hit()) > 0.85);
        assert!(rig.rate(100.0, 15.0, 26.0, empty, 0, |r| r.kind == PlayKind::LineOut) > 0.7);
        // Hard grounder past the pitcher and between the middle infielders gets through.
        assert!(rig.rate(102.0, -3.0, 5.0, empty, 0, |r| r.kind == PlayKind::Single) > 0.6);
        // Pop-up on the infield.
        assert!(rig.rate(80.0, 65.0, 5.0, empty, 1, |r| r.kind == PlayKind::PopOut) > 0.97);
        // Foul territory, into the seats.
        assert!(rig.rate(85.0, 35.0, 70.0, empty, 0, |r| r.kind == PlayKind::Foul) > 0.99);
    }

    #[test]
    fn double_plays_sac_flies_and_grand_slams() {
        let rig = Rig::new();
        let first = [Some(runner(1, 27.0)), None, None];
        let dp = rig.rate(92.0, -7.0, -13.0, first, 0, |r| r.kind == PlayKind::DoublePlay && r.outs == 2 && r.gidp);
        assert!(dp > 0.5, "double-play rate on a routine grounder to short: {dp}");
        // With two outs the defence just takes the out at first.
        assert!(rig.rate(92.0, -7.0, -13.0, first, 2, |r| r.kind == PlayKind::GroundOut && r.outs == 1) > 0.9);

        let third = [None, None, Some(runner(3, 27.5))];
        let sf = rig.rate(95.0, 32.0, 5.0, third, 0, |r| r.kind == PlayKind::SacFly && r.n_scored == 1 && r.rbi == 1);
        assert!(sf > 0.8, "sac fly rate on a deep fly: {sf}");
        // A shallow pop does not score him.
        assert!(rig.rate(75.0, 60.0, 0.0, third, 0, |r| r.n_scored == 0) > 0.95);

        let loaded = [Some(runner(1, 27.0)), Some(runner(2, 27.0)), Some(runner(3, 27.0))];
        assert!(rig.rate(110.0, 28.0, 10.0, loaded, 1, |r| r.kind == PlayKind::HomeRun && r.n_scored == 4 && r.rbi == 4) > 0.99);
    }

    #[test]
    fn runner_in_motion_avoids_the_double_play_but_can_be_doubled_off() {
        let rig = Rig::new();
        let first = [Some(runner(1, 27.0)), None, None];
        let n = 300;
        let dp = |running: [f64; 3]| {
            (0..n).filter(|&s| rig.play_with(92.0, -7.0, -13.0, first, 0, running, false, s as u64 + 1).kind == PlayKind::DoublePlay).count() as f64 / n as f64
        };
        let (standing, going) = (dp([0.0; 3]), dp([1.1, 0.0, 0.0]));
        assert!(standing > 0.5 && going < 0.1, "double plays: standing {standing}, in motion {going}");
        // A line drive at the shortstop with the runner going: he cannot get back.
        let doubled = (0..n)
            .filter(|&s| {
                let r = rig.play_with(95.0, 9.0, -13.0, first, 0, [1.1, 0.0, 0.0], false, s as u64 + 1);
                r.kind == PlayKind::LineOut && r.outs == 2
            })
            .count() as f64
            / n as f64;
        let caught = rig.rate(95.0, 9.0, -13.0, first, 0, |r| r.kind == PlayKind::LineOut);
        assert!(caught > 0.3 && doubled > 0.8 * caught, "caught {caught}, doubled off {doubled}");
        assert!(rig.rate(95.0, 9.0, -13.0, first, 0, |r| r.outs == 2) < 0.01);
    }

    #[test]
    fn drawn_in_infield_holds_the_runner_at_third() {
        use crate::fielding::{alignment, Alignment, InfieldDepth};
        let mut rig = Rig::new();
        let third = [None, None, Some(runner(3, 27.0))];
        // Routine grounder to short with one out: the run scores against a normal infield.
        let back = rig.rate(88.0, -6.0, -13.0, third, 1, |r| r.n_scored == 1);
        rig.positions = alignment(&Alignment { infield: InfieldDepth::In, ..Default::default() }, -1.0, false, false, true);
        let n = 300;
        let scored_in = (0..n).filter(|&s| rig.play_with(88.0, -6.0, -13.0, third, 1, [0.0; 3], true, s as u64 + 1).n_scored == 1).count() as f64 / n as f64;
        assert!(back > 0.9 && scored_in < 0.15, "run scores: infield back {back}, infield in {scored_in}");
        // The price: a harder grounder gets through the drawn-in infield more often.
        let through = |rig: &Rig, inn: bool| {
            (0..n).filter(|&s| rig.play_with(97.0, -3.0, -7.0, third, 1, [0.0; 3], inn, s as u64 + 1).kind.is_hit()).count() as f64 / n as f64
        };
        let hits_in = through(&rig, true);
        rig.positions = standard_alignment();
        let hits_back = through(&rig, false);
        assert!(hits_in > hits_back + 0.1, "hits: infield in {hits_in}, back {hits_back}");
    }

    #[test]
    fn runners_score_from_second_on_most_singles_and_speed_matters() {
        let rig = Rig::new();
        let second_avg = [None, Some(runner(2, 27.0)), None];
        let second_fast = [None, Some(runner(2, 30.0)), None];
        let second_slow = [None, Some(runner(2, 24.0)), None];
        // Ground-ball single through the right side, fielded by the right fielder.
        let score = |b| rig.rate(93.0, 2.0, 22.0, b, 1, |r| r.kind == PlayKind::Single && r.n_scored == 1);
        let (avg, fast, slow) = (score(second_avg), score(second_fast), score(second_slow));
        assert!(fast >= avg && avg >= slow, "fast {fast} avg {avg} slow {slow}");
        assert!(fast > slow + 0.15, "speed should matter: fast {fast} slow {slow}");
        assert!(fast > 0.6, "a fast runner usually scores from second on a single: {fast}");
        // Outs on the bases never exceed three in total and bases never hold duplicates.
        for seed in 0..200 {
            let r = rig.play(98.0, 12.0, -20.0, [Some(runner(1, 28.0)), Some(runner(2, 26.0)), None], 2, seed);
            assert!(r.outs <= 1);
            let ids: Vec<usize> = r.bases.iter().flatten().map(|x| x.player).collect();
            let mut d = ids.clone();
            d.dedup();
            assert_eq!(ids.len(), d.len());
        }
    }
}
