//! The pitcher's side of a pitch: what to throw, where to aim, and how the
//! pitch actually comes out of the hand.

use crate::params::ModelParams;
use crate::profile::{Hand, PitchDef, PitchGroup, PitchKind, PitcherProfile};
use crate::rng::Rng;
use crate::rules::{Rules, StrikeZone};
use bm_physics::units::G;
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
}

impl MatchEnv {
    pub fn new(atmosphere: &Atmosphere, wind: Wind, park: Park, rules: Rules, params: ModelParams) -> Self {
        let ball = BallSpec::mlb();
        MatchEnv {
            ball,
            pitch_flight: FlightModel::new(&ball, AeroModel::pitched_2025(), atmosphere, wind),
            batted_flight: FlightModel::new(&ball, AeroModel::batted_2025(), atmosphere, wind),
            park,
            rules,
            params,
        }
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
    let n = profile.pitches.len().min(MAX_PITCHES);
    let mut fast_total = 0.0;
    let mut other_total = 0.0;
    for (i, d) in profile.pitches.iter().take(n).enumerate() {
        let u = if same { d.usage_same } else { d.usage_opp };
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
    let of = |m: &PitchMean| Expectation { time: m.aim.time, move_x: m.aim.move_x, drop_z: m.aim.move_z - 0.5 * G * m.aim.time * m.aim.time };
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
    let base = of(&state.means[fb]);
    let total: f64 = weights.iter().sum();
    if total <= 0.0 || mix <= 0.0 {
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
    Expectation {
        time: base.time + mix * (avg.time - base.time),
        move_x: base.move_x + mix * (avg.move_x - base.move_x),
        drop_z: base.drop_z + mix * (avg.drop_z - base.drop_z),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchChoice {
    pub idx: usize,
    pub target_x: f64,
    pub target_z: f64,
}

/// Direction in which each pitch is "wasted" to induce a chase: (toward the outside, in zone heights).
fn chase_direction(kind: PitchKind) -> (f64, f64) {
    match kind {
        PitchKind::FourSeam => (0.02, 0.32),
        PitchKind::Sinker => (-0.08, -0.22),
        PitchKind::Cutter => (0.12, 0.02),
        PitchKind::Slider | PitchKind::Sweeper => (0.16, -0.28),
        PitchKind::Curve => (0.05, -0.38),
        PitchKind::Changeup | PitchKind::Splitter => (0.10, -0.33),
    }
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
    let (x_out, zf) = kind.typical_target();
    let intent = params.zone_intent[balls.min(3)][strikes.min(2)];
    let (x, z_frac) = if intent >= 0.0 {
        (x_out * (1.0 - 0.8 * intent), 0.5 + (zf - 0.5) * (1.0 - 0.8 * intent))
    } else {
        let (cx, cz) = chase_direction(kind);
        (x_out + cx * -intent, zf + cz * -intent)
    };
    let out = batter_hand.outside_sign();
    // When he has to throw a strike the pitcher also aims at a smaller area.
    let spread = 1.0 - 0.1 * intent.max(0.0);
    PitchChoice {
        idx,
        target_x: out * (x + rng.gauss(0.0, params.target_sd_x * spread)),
        target_z: zone.bottom + zone.height() * z_frac + rng.gauss(0.0, params.target_sd_z * spread),
    }
}

/// A pitch as it was actually thrown.
#[derive(Clone, Copy, Debug)]
pub struct Thrown {
    pub idx: usize,
    pub kind: PitchKind,
    pub spec: PitchSpec,
    pub flight: PitchFlight,
}

/// Execute a pitch: add release scatter, command error and fatigue, then fly it.
pub fn throw_pitch(profile: &PitcherProfile, state: &mut PitcherState, choice: &PitchChoice, env: &MatchEnv, rng: &mut Rng) -> Thrown {
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

    let mut cmd = def.command * command_factor;
    if rng.chance(p.wild_fraction) {
        cmd *= p.wild_factor;
    }
    let mut angles = mean.aim.angles_for(choice.target_x, choice.target_z);
    angles.horiz += rng.gauss(0.0, cmd / mean.aim.dx_dh.abs());
    angles.vert += rng.gauss(0.0, cmd / mean.aim.dz_dv.abs());

    let flight = spec.fly(&env.pitch_flight, angles, PITCH_DT);
    state.pitch_count += 1;
    Thrown { idx: choice.idx, kind: def.kind, spec, flight }
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
        let (mut sx, mut sz, mut sxx, mut szz, mut sv) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..n {
            st.pitch_count = 20;
            let t = throw_pitch(&prof, &mut st, &choice, &env, &mut rng);
            sx += t.flight.plate_pos.x;
            sz += t.flight.plate_pos.z;
            sxx += t.flight.plate_pos.x.powi(2);
            szz += t.flight.plate_pos.z.powi(2);
            sv += t.spec.speed;
        }
        let nf = n as f64;
        let (mx, mz) = (sx / nf, sz / nf);
        let (sdx, sdz) = ((sxx / nf - mx * mx).sqrt(), (szz / nf - mz * mz).sqrt());
        assert!((mx - 0.10).abs() < 0.02 && (mz - 0.85).abs() < 0.02, "mean location ({mx}, {mz})");
        // Command of 6.6 in per axis plus a 6 % share of bad misses and release scatter.
        assert!(sdx > 6.0 * INCH && sdx < 9.0 * INCH, "sd x {} in", sdx / INCH);
        assert!(sdz > 6.0 * INCH && sdz < 9.5 * INCH, "sd z {} in", sdz / INCH);
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
                    throw_pitch(&prof, st, &choice, &env, rng).spec.speed
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
