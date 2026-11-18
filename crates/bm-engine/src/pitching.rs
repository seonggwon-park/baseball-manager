//! The pitcher's side of a pitch: what to throw, where to aim, and how the
//! pitch actually comes out of the hand.

use crate::params::ModelParams;
use crate::profile::{Hand, PitchDef, PitchGroup, PitchKind, PitcherProfile};
use crate::rng::Rng;
use crate::rules::{Rules, StrikeZone};
use bm_physics::units::{FT, G};
use bm_physics::{AeroModel, AimModel, Atmosphere, BallSpec, FlightModel, Park, PitchFlight, PitchSpec, Vec3, Wind, RUBBER_Y};

/// Integration step for pitches, s. With RK4 the plate location is converged to a
/// fraction of a millimetre at this step size.
pub const PITCH_DT: f64 = 0.008;
/// Maximum number of pitch types per pitcher.
pub const MAX_PITCHES: usize = 8;

/// Everything about the venue and rules that stays fixed during a game.
#[derive(Clone, Debug)]
pub struct MatchEnv {
    pub ball: BallSpec,
    pub pitch_flight: FlightModel,
    pub batted_flight: FlightModel,
    pub park: Park,
    pub rules: Rules,
    pub params: ModelParams,
    /// Flight time of the fastball the league's batters are used to, s (the league-average
    /// four-seamer unless the world layer sets the league's own).
    pub typical_fastball_time: f64,
    /// Spin-induced vertical movement of the league-average pitch of each type, m, indexed
    /// by `PitchKind::index`: what a batter who has recognised the pitch expects it to do.
    pub typical_rise: [f64; 8],
    /// Speed at the plate of the league-average pitch of each type, m/s.
    pub typical_plate_speed: [f64; 8],
}

impl MatchEnv {
    pub fn new(atmosphere: &Atmosphere, wind: Wind, park: Park, rules: Rules, params: ModelParams) -> Self {
        let ball = BallSpec::mlb();
        let mut env = MatchEnv {
            ball,
            pitch_flight: FlightModel::new(&ball, AeroModel::pitched_2025(), atmosphere, wind),
            batted_flight: FlightModel::new(&ball, AeroModel::batted_2025(), atmosphere, wind),
            park,
            rules,
            params,
            typical_fastball_time: 0.0,
            typical_rise: [0.0; 8],
            typical_plate_speed: [0.0; 8],
        };
        // The league-average pitch of every type, thrown from the average release point.
        let mut typical = PitcherProfile::league_average(Hand::Right);
        typical.pitches = PitchKind::ALL
            .iter()
            .map(|&kind| PitchDef { kind, shape: kind.league_average(), usage_same: 1.0, usage_opp: 1.0, command: 0.17, disguise: 0.0, aim_height: 0.0 })
            .collect();
        let state = PitcherState::new(&typical, &env);
        for (kind, mean) in PitchKind::ALL.iter().zip(&state.means) {
            env.typical_rise[kind.index()] = mean.aim.move_z;
            env.typical_plate_speed[kind.index()] = mean.aim.plate_speed;
        }
        env.typical_fastball_time = state.means[PitchKind::FourSeam.index()].aim.time;
        env
    }

    /// Neutral MLB-like conditions: about 24 C, 55 % humidity, 180 m elevation
    /// (air density 1.155 kg/m^3, the average game-time value), no wind, average park.
    pub fn neutral_mlb() -> Self {
        MatchEnv::new(&Atmosphere::new(24.0, 760.0, 55.0, 180.0), Wind::CALM, Park::mlb_average(), Rules::mlb_2025(), ModelParams::default())
    }
}

/// Mean behaviour of one pitch of one pitcher in the current environment.
#[derive(Clone, Copy, Debug)]
pub struct PitchMean {
    pub spec: PitchSpec,
    pub aim: AimModel,
}

/// Per-game state of a pitcher.
#[derive(Clone, Debug)]
pub struct PitcherState {
    pub means: Vec<PitchMean>,
    pub pitch_count: u32,
}

impl PitcherState {
    pub fn new(profile: &PitcherProfile, env: &MatchEnv) -> Self {
        let means = profile
            .pitches
            .iter()
            .map(|d| {
                let spec = mean_spec(profile, d);
                PitchMean { spec, aim: AimModel::build(&spec, &env.pitch_flight, 0.0, 0.77, PITCH_DT) }
            })
            .collect();
        PitcherState { means, pitch_count: 0 }
    }
}

/// The pitch as intended by an average execution (no noise).
pub fn mean_spec(profile: &PitcherProfile, def: &PitchDef) -> PitchSpec {
    let arm = profile.hand.arm_sign();
    PitchSpec {
        release_pos: Vec3::new(arm * profile.release_side, RUBBER_Y - profile.extension, profile.release_height),
        speed: def.shape.speed,
        spin_rate: def.shape.spin_rate,
        spin_tilt: def.shape.tilt * arm,
        spin_active: def.shape.active,
        gyro_sign: match profile.hand {
            Hand::Right => 1.0,
            Hand::Left => -1.0,
        },
    }
}

/// What the batter has learned to expect from this pitcher in this count: the
/// usage-weighted mean flight time and total deviation from a straight line.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Expectation {
    pub time: f64,
    pub move_x: f64,
    /// Vertical deviation from the straight release line: spin-induced movement minus gravity drop.
    pub drop_z: f64,
    /// The expected pitch is a fastball.
    pub fastball: bool,
    /// Deviation from the straight line of the usage-weighted mean of the pitcher's pitches
    /// in this count, horizontal and vertical (gravity included), m: the batter's prior for
    /// where a pitch ends up before he has seen its late break (`ModelParams::prior_pull`).
    pub prior_x: f64,
    pub prior_z: f64,
}

/// Selection weights for each pitch of the arsenal in a given count.
pub fn selection_weights(
    profile: &PitcherProfile,
    batter_hand: Hand,
    balls: usize,
    strikes: usize,
    params: &ModelParams,
    out: &mut [f64; MAX_PITCHES],
) -> usize {
    let same = profile.hand == batter_hand;
    let fb = params.fastball_bias[balls.min(3)][strikes.min(2)];
    // The count's mix of pitch types first (`count_kind`), then any bias of the fastballs as a
    // group.
    let n = profile.pitches.len().min(MAX_PITCHES);
    let mut fast_total = 0.0;
    let mut other_total = 0.0;
    let (b, s) = (balls.min(3), strikes.min(2));
    for (i, d) in profile.pitches.iter().take(n).enumerate() {
        let u = (if same { d.usage_same } else { d.usage_opp }) * params.count_kind[d.kind.index()][b][s];
        out[i] = u;
        if d.kind.group() == PitchGroup::Fastball {
            fast_total += u;
        } else {
            other_total += u;
        }
    }
    // Scale the fastball share by the count bias while keeping the weights normalised.
    if fast_total > 0.0 && other_total > 0.0 {
        let total = fast_total + other_total;
        let share = (fast_total / total * fb).clamp(0.03, 0.97);
        let f_scale = share * total / fast_total;
        let o_scale = (1.0 - share) * total / other_total;
        for (i, d) in profile.pitches.iter().take(n).enumerate() {
            out[i] *= if d.kind.group() == PitchGroup::Fastball { f_scale } else { o_scale };
        }
    }
    n
}

/// What the batter is set up for. Hitters time the pitcher's main fastball and
/// adjust to everything else, leaning toward the count's pitch mix by `mix`
/// (0 = pure fastball timing, 1 = the usage-weighted average pitch).
pub fn expectation(profile: &PitcherProfile, state: &PitcherState, weights: &[f64], mix: f64) -> Expectation {
    let of = |m: &PitchMean| {
        let drop_z = m.aim.move_z - 0.5 * G * m.aim.time * m.aim.time;
        Expectation { time: m.aim.time, move_x: m.aim.move_x, drop_z, fastball: false, prior_x: m.aim.move_x, prior_z: drop_z }
    };
    // Main fastball: the most used fastball in this situation, else the fastest pitch.
    let mut best: Option<usize> = None;
    for (i, d) in profile.pitches.iter().enumerate().take(weights.len()) {
        if d.kind.group() == PitchGroup::Fastball && best.is_none_or(|b| weights[i] > weights[b]) {
            best = Some(i);
        }
    }
    let fb = best.unwrap_or_else(|| {
        let mut k = 0;
        for (i, d) in profile.pitches.iter().enumerate() {
            if d.shape.speed > profile.pitches[k].shape.speed {
                k = i;
            }
        }
        k
    });
    let base = Expectation { fastball: profile.pitches[fb].kind.group() == PitchGroup::Fastball, ..of(&state.means[fb]) };
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return base;
    }
    let mut avg = Expectation::default();
    for (m, &w) in state.means.iter().zip(weights) {
        let e = of(m);
        let f = w / total;
        avg.time += f * e.time;
        avg.move_x += f * e.move_x;
        avg.drop_z += f * e.drop_z;
    }
    let mix = mix.max(0.0);
    Expectation {
        time: base.time + mix * (avg.time - base.time),
        move_x: base.move_x + mix * (avg.move_x - base.move_x),
        drop_z: base.drop_z + mix * (avg.drop_z - base.drop_z),
        fastball: base.fastball,
        prior_x: avg.move_x,
        prior_z: avg.drop_z,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchChoice {
    pub idx: usize,
    pub target_x: f64,
    pub target_z: f64,
}

/// Direction in which each pitch is "wasted" to induce a chase, per unit of chase intent,
/// against a batter on the pitcher's own side or the other side: (toward the outside in m,
/// in zone heights). The sideways part is how far the mean location moves in 0-2 and 1-2
/// counts in the 2025 Statcast sample, by the batter's side: sliders further away from
/// same-side batters and further in on opposite-side ones, changeups further away from
/// opposite-side batters.
fn chase_direction(kind: PitchKind, same_side: bool) -> (f64, f64) {
    let (same, opposite, height) = match kind {
        PitchKind::FourSeam => (0.084, 0.119, 0.32),
        PitchKind::Sinker => (0.162, -0.239, -0.22),
        PitchKind::Cutter => (0.276, 0.067, 0.02),
        PitchKind::Slider => (0.404, -0.225, -0.28),
        PitchKind::Sweeper => (0.500, -0.273, -0.28),
        PitchKind::Curve => (0.266, -0.063, -0.38),
        PitchKind::Changeup => (0.028, 0.272, -0.33),
        PitchKind::Splitter => (0.165, 0.093, -0.33),
    };
    let x = if same_side { same } else { opposite };
    (x * FT, height)
}

/// Choose a pitch and an intended location.
#[allow(clippy::too_many_arguments)]
pub fn choose_pitch(
    profile: &PitcherProfile,
    batter_hand: Hand,
    zone: &StrikeZone,
    balls: usize,
    strikes: usize,
    weights: &[f64],
    params: &ModelParams,
    rng: &mut Rng,
) -> PitchChoice {
    let idx = rng.pick_weighted(weights);
    let kind = profile.pitches[idx].kind;
    let aim_height = profile.pitches[idx].aim_height;
    let same_side = profile.hand == batter_hand;
    let (x_out, zf) = kind.typical_target(same_side);
    let intent = params.zone_intent[balls.min(3)][strikes.min(2)];
    let (x, z_frac) = if intent >= 0.0 {
        (x_out * (1.0 - 0.8 * intent), 0.5 + (zf - 0.5) * (1.0 - 0.8 * intent))
    } else {
        let (cx, cz) = chase_direction(kind, same_side);
        (x_out + cx * -intent, zf + cz * -intent)
    };
    // The less precisely he throws the pitch, the closer to the middle he aims it.
    let typical = params.aim_reference
        * match kind.group() {
            PitchGroup::Fastball => 1.0,
            PitchGroup::Breaking => 1.12,
            PitchGroup::Offspeed => 1.10,
        };
    let margin = if params.aim_adapt == 0.0 { 1.0 } else { (typical / profile.pitches[idx].command).powf(params.aim_adapt).clamp(0.5, 1.4) };
    let (x, z_frac) = (x * margin, 0.5 + (z_frac - 0.5) * margin);
    let out = batter_hand.outside_sign();
    // When he has to throw a strike the pitcher also aims at a smaller area.
    let spread = 1.0 - 0.1 * intent.max(0.0);
    PitchChoice {
        idx,
        target_x: out * (x + rng.gauss(0.0, params.target_sd_x * spread)),
        target_z: zone.bottom + zone.height() * z_frac + aim_height + rng.gauss(0.0, params.target_sd_z * spread),
    }
}

/// A pitch as it was actually thrown.
#[derive(Clone, Copy, Debug)]
pub struct Thrown {
    pub idx: usize,
    pub kind: PitchKind,
    pub spec: PitchSpec,
    pub flight: PitchFlight,
    /// The pitch's disguise (see [`PitchDef::disguise`]).
    pub disguise: f64,
}

/// Execute a pitch: add release scatter, command error and fatigue, then fly it.
///
/// `command_scale` multiplies the location error (1 in ordinary circumstances).
pub fn throw_pitch(profile: &PitcherProfile, state: &mut PitcherState, choice: &PitchChoice, command_scale: f64, env: &MatchEnv, rng: &mut Rng) -> Thrown {
    let p = &env.params;
    let def = &profile.pitches[choice.idx];
    let mean = &state.means[choice.idx];
    let n = state.pitch_count as f64;
    let over = (n - profile.stamina).max(0.0);
    let speed_factor = 1.0 - 0.00008 * n - p.fatigue_speed * over;
    let command_factor = 1.0 + p.fatigue_command * over;
    let arm = profile.hand.arm_sign();

    let mut spec = mean.spec;
    spec.speed = (def.shape.speed * speed_factor + rng.gauss(0.0, profile.speed_sd)).max(15.0);
    spec.spin_rate = (def.shape.spin_rate + rng.gauss(0.0, profile.spin_sd)).max(0.0);
    spec.spin_tilt = (def.shape.tilt + rng.gauss(0.0, profile.tilt_sd)) * arm;
    spec.spin_active = (def.shape.active + rng.gauss(0.0, 0.025)).clamp(0.0, 1.0);
    spec.release_pos.x += rng.gauss(0.0, p.release_sd_side);
    spec.release_pos.z += rng.gauss(0.0, p.release_sd_height);

    // The location error is an ellipse whose long axis follows the arm's swing: a
    // release that is early sails high to the arm side, a late one is pulled low to the
    // glove side (Shinya et al. 2017; 2025 Statcast: 25.9 cm along the axis against
    // 19.1 cm across it, the axis at 34 deg for sidearmers and 72 deg over the top).
    // The pitches that get away altogether are errors of release timing as well.
    let cmd = def.command * command_factor * command_scale;
    let r2 = profile.error_ratio * profile.error_ratio;
    let mut along = cmd * (2.0 * r2 / (1.0 + r2)).sqrt();
    let across = cmd * (2.0 / (1.0 + r2)).sqrt();
    if rng.chance((p.wild_fraction * profile.wildness).min(0.9)) {
        along *= p.wild_factor;
    }
    let (sa, ca) = profile.error_axis().sin_cos();
    let (u, v) = (rng.gauss(0.0, along), rng.gauss(0.0, across));
    let (ex, ez) = (arm * ca * u - sa * v, sa * u + arm * ca * v);
    let mut angles = mean.aim.angles_for(choice.target_x, choice.target_z);
    angles.horiz += ex / mean.aim.dx_dh;
    angles.vert += ez / mean.aim.dz_dv;

    let flight = spec.fly(&env.pitch_flight, angles, PITCH_DT);
    state.pitch_count += 1;
    Thrown { idx: choice.idx, kind: def.kind, spec, flight, disguise: def.disguise }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bm_physics::units::{INCH, MPH};

    #[test]
    fn average_pitcher_locates_around_the_target() {
        let env = MatchEnv::neutral_mlb();
        let prof = PitcherProfile::league_average(Hand::Right);
        let mut st = PitcherState::new(&prof, &env);
        let mut rng = Rng::new(3);
        let choice = PitchChoice { idx: 0, target_x: 0.10, target_z: 0.85 };
        let n = 4000;
        let (mut sx, mut sz, mut sxx, mut szz, mut sxz, mut sv) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..n {
            st.pitch_count = 20;
            let t = throw_pitch(&prof, &mut st, &choice, 1.0, &env, &mut rng);
            sx += t.flight.plate_pos.x;
            sz += t.flight.plate_pos.z;
            sxx += t.flight.plate_pos.x.powi(2);
            szz += t.flight.plate_pos.z.powi(2);
            sxz += t.flight.plate_pos.x * t.flight.plate_pos.z;
            sv += t.spec.speed;
        }
        let nf = n as f64;
        let (mx, mz) = (sx / nf, sz / nf);
        let (sdx, sdz) = ((sxx / nf - mx * mx).sqrt(), (szz / nf - mz * mz).sqrt());
        assert!((mx - 0.10).abs() < 0.02 && (mz - 0.85).abs() < 0.02, "mean location ({mx}, {mz})");
        // Command of 6.6 in (root mean square of the two axes) plus the pitches that get
        // away and the release scatter.
        assert!(sdx > 5.5 * INCH && sdx < 8.5 * INCH, "sd x {} in", sdx / INCH);
        assert!(sdz > 6.5 * INCH && sdz < 10.0 * INCH, "sd z {} in", sdz / INCH);
        // The error is an ellipse along the arm's swing: a right-hander misses high toward
        // the third-base side (-x) and low toward the first-base side.
        let corr = (sxz / nf - mx * mz) / (sdx * sdz);
        assert!(sdz > sdx && corr < -0.15 && corr > -0.55, "sd x {} in, sd z {} in, corr {corr}", sdx / INCH, sdz / INCH);
        // A left-hander's ellipse is the mirror image.
        let left = PitcherProfile::league_average(Hand::Left);
        let mut st = PitcherState::new(&left, &env);
        let (mut lx, mut lz, mut lxz) = (0.0, 0.0, 0.0);
        let pts: Vec<(f64, f64)> = (0..n)
            .map(|_| {
                st.pitch_count = 20;
                let t = throw_pitch(&left, &mut st, &choice, 1.0, &env, &mut rng);
                (t.flight.plate_pos.x, t.flight.plate_pos.z)
            })
            .collect();
        for &(x, z) in &pts {
            lx += x;
            lz += z;
        }
        for &(x, z) in &pts {
            lxz += (x - lx / nf) * (z - lz / nf);
        }
        assert!(lxz > 0.0, "left-hander's errors should run from low on the first-base side... to high on it");
        // Doubling the command scale doubles the scatter.
        let mut st = PitcherState::new(&prof, &env);
        let wide: f64 = (0..n)
            .map(|_| {
                st.pitch_count = 20;
                (throw_pitch(&prof, &mut st, &choice, 2.0, &env, &mut rng).flight.plate_pos.z - 0.85).powi(2)
            })
            .sum::<f64>()
            / nf;
        assert!(wide.sqrt() > 1.6 * sdz, "scaled sd z {} in against {} in", wide.sqrt() / INCH, sdz / INCH);
        assert!((sv / nf / MPH - 94.5).abs() < 0.4);
    }

    #[test]
    fn count_changes_the_pitch_mix_and_the_expectation() {
        let env = MatchEnv::neutral_mlb();
        let prof = PitcherProfile::league_average(Hand::Right);
        let st = PitcherState::new(&prof, &env);
        let mut w = [0.0; MAX_PITCHES];
        let share = |w: &[f64]| w[0] / w.iter().sum::<f64>();
        let n = selection_weights(&prof, Hand::Right, 3, 0, &env.params, &mut w);
        let behind = share(&w[..n]);
        let e_behind = expectation(&prof, &st, &w[..n], 1.0);
        let n = selection_weights(&prof, Hand::Right, 0, 2, &env.params, &mut w);
        let ahead = share(&w[..n]);
        let e_ahead = expectation(&prof, &st, &w[..n], 1.0);
        assert!(behind > 0.85 && ahead < 0.45, "fastball share 3-0 {behind}, 0-2 {ahead}");
        // Expecting more breaking balls means expecting a slower, more dropping pitch.
        assert!(e_ahead.time > e_behind.time && e_ahead.drop_z < e_behind.drop_z);
    }

    #[test]
    fn fatigue_costs_velocity() {
        let env = MatchEnv::neutral_mlb();
        let prof = PitcherProfile::league_average(Hand::Left);
        let mut st = PitcherState::new(&prof, &env);
        let mut rng = Rng::new(9);
        let choice = PitchChoice { idx: 0, target_x: 0.0, target_z: 0.8 };
        let avg = |st: &mut PitcherState, rng: &mut Rng, count: u32| {
            (0..500)
                .map(|_| {
                    st.pitch_count = count;
                    throw_pitch(&prof, st, &choice, 1.0, &env, rng).spec.speed
                })
                .sum::<f64>()
                / 500.0
        };
        let fresh = avg(&mut st, &mut rng, 0);
        let tired = avg(&mut st, &mut rng, 125);
        assert!(fresh - tired > 1.0 * MPH && fresh - tired < 3.0 * MPH, "velocity loss {} mph", (fresh - tired) / MPH);
        // A left-hander releases from the first-base side.
        assert!(st.means[0].spec.release_pos.x > 0.3);
    }
}
