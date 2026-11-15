//! The batter's side of a pitch: perception, the swing/take decision and the
//! swing itself. The outcome of a swing is not drawn from a table: the model
//! produces a bat position, orientation and velocity, and the bat-ball collision
//! in `bm-physics` decides whether and how the ball is struck.

use crate::params::ModelParams;
use crate::pitching::{Expectation, MatchEnv, Thrown};
use crate::profile::{BatterProfile, PitchKind};
use crate::rng::{sigmoid, Rng};
use crate::rules::StrikeZone;
use bm_physics::units::G;
use bm_physics::{collide, launch_angle, spin_components, spray_angle, BallState, Contact, Vec3, PLATE_FRONT_Y};

/// Launch conditions of a batted ball.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BattedInfo {
    /// Exit speed, m/s.
    pub exit_speed: f64,
    /// Vertical launch angle, rad.
    pub launch_angle: f64,
    /// Horizontal spray angle, rad (+ toward first base / right field).
    pub spray: f64,
    /// Backspin and sidespin, rad/s.
    pub backspin: f64,
    pub sidespin: f64,
    /// Bat speed at the sweet spot, m/s.
    pub bat_speed: f64,
    /// Attack angle of the swing, rad.
    pub attack_angle: f64,
}

/// Why a swing missed (for commentary and diagnostics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Miss {
    /// Far too early: the barrel had already left the hitting zone.
    Early,
    /// Far too late.
    Late,
    /// The ball passed beyond the end of the bat.
    OffTheEnd,
    /// The ball passed inside the barrel, at the hands.
    Jammed,
    /// The bat passed over the ball.
    Over,
    /// The bat passed under the ball.
    Under,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Swing {
    /// The batter did not offer.
    Take,
    /// Swing and miss.
    Whiff { bat_speed: f64, miss: Miss },
    /// Tipped straight back; a strike if the catcher holds it.
    FoulTip { caught: bool, bat_speed: f64 },
    /// Fouled backward out of play.
    FoulBack { bat_speed: f64 },
    /// Ball put in the air or on the ground in front of the plate (fair or foul to be decided).
    Batted { state: BallState, info: BattedInfo },
}

/// Diagnostic values of the perception step, exposed for tests and calibration.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Perception {
    pub recognition: f64,
    pub perceived_x: f64,
    pub perceived_z: f64,
    /// Part of the vertical misperception that comes from misjudging the pitch type, m.
    pub bias_z: f64,
    pub swing_prob: f64,
}

/// Probability that the batter swings, and what he perceived.
#[allow(clippy::too_many_arguments)]
pub fn perceive(
    params: &ModelParams,
    batter: &BatterProfile,
    zone: &StrikeZone,
    balls: usize,
    strikes: usize,
    thrown: &Thrown,
    expected: &Expectation,
    ball_radius: f64,
    rng: &mut Rng,
) -> Perception {
    let f = &thrown.flight;
    // Recognition is close to all-or-nothing: the batter either picks the pitch up out
    // of the hand or commits to the wrong one.
    let rho = if rng.chance(batter.eye) { 1.0 - rng.gauss(0.0, params.recognition_sd).abs().min(0.5) } else { rng.range(0.0, params.fooled_recognition) };
    let drop_act = f.move_z - 0.5 * G * f.time * f.time;
    // A pitch that differs a lot from the fastball is easier to identify, so the
    // misjudgement grows less than proportionally with the difference in movement.
    let (dmx, dmz) = {
        let (x, z) = (expected.move_x - f.move_x, expected.drop_z - drop_act);
        let sat = 1.0 / (1.0 + (x * x + z * z).sqrt() / params.recognition_scale);
        (x * sat, z * sat)
    };
    // Mostly small errors, now and then a pitch that is badly misread.
    let sd = batter.perception_sd * if rng.chance(params.misread_rate) { params.misread_factor } else { 1.0 };
    let px = f.plate_pos.x + (1.0 - rho) * dmx + rng.gauss(0.0, sd);
    let pz = f.plate_pos.z + (1.0 - rho) * dmz + rng.gauss(0.0, sd);
    let d_hat = zone.signed_distance(px, pz, ball_radius);
    let d0 = params.swing_d0
        + batter.chase
        + match strikes {
            0 => 0.0,
            1 => params.swing_d0_one_strike,
            _ => params.swing_d0_two_strikes,
        };
    let base = (params.swing_base[balls.min(3)][strikes.min(2)] * batter.aggression).min(0.992);
    Perception { recognition: rho, perceived_x: px, perceived_z: pz, bias_z: (1.0 - rho) * dmz, swing_prob: base * sigmoid(-(d_hat - d0) / params.swing_width) }
}

/// Full response of the batter to a pitch.
#[allow(clippy::too_many_arguments)]
pub fn respond(
    env: &MatchEnv,
    batter: &BatterProfile,
    zone: &StrikeZone,
    balls: usize,
    strikes: usize,
    thrown: &Thrown,
    expected: &Expectation,
    rng: &mut Rng,
) -> Swing {
    let p = &env.params;
    let r_ball = env.ball.radius;
    let per = perceive(p, batter, zone, balls, strikes, thrown, expected, r_ball, rng);
    if !rng.chance(per.swing_prob) {
        return Swing::Take;
    }
    swing(env, batter, zone, strikes, thrown, expected, &per, rng)
}

/// Execute a swing at the pitch.
#[allow(clippy::too_many_arguments)]
pub fn swing(
    env: &MatchEnv,
    batter: &BatterProfile,
    zone: &StrikeZone,
    strikes: usize,
    thrown: &Thrown,
    expected: &Expectation,
    per: &Perception,
    rng: &mut Rng,
) -> Swing {
    let p = &env.params;
    let f = &thrown.flight;
    let bat = &batter.bat;
    let out = batter.hand.outside_sign();
    let two = strikes >= 2;
    let prec = if two { p.two_strike_precision } else { 1.0 };
    let v_plate = f.plate_vel.norm();
    let speed_ratio = v_plate / p.ref_plate_speed;

    // ---- timing ----
    let dt_raw = expected.time - f.time;
    let dt_bias = (1.0 - per.recognition) * dt_raw / (1.0 + dt_raw.abs() / p.recognition_time_scale);
    let dt = batter.timing_bias + dt_bias + rng.gauss(0.0, batter.timing_sd * speed_ratio.powf(p.timing_speed_exp) * prec);
    let fooled = (dt_bias.abs() / p.fooled_time).min(1.0);
    let mut bat_speed = rng.gauss(batter.bat_speed, batter.bat_speed_sd) * (1.0 - p.fooled_speed_loss * fooled * fooled);
    if two {
        bat_speed *= p.two_strike_speed;
    }
    // Partial swings: check swings, defensive two-strike hacks, swings the batter gives up on.
    if rng.chance((p.partial_swing_rate * if two { p.partial_two_strike_gain } else { 1.0 } + p.partial_fooled_rate * fooled).min(0.9)) {
        bat_speed *= rng.range(p.partial_speed_min, 0.92);
    }
    bat_speed = bat_speed.max(8.0);

    // Horizontal bat angle at contact (positive = barrel ahead of the hands = pull side).
    let inside = -out * f.plate_pos.x;
    let omega = bat_speed / p.swing_radius;
    let theta = p.plan_angle_per_m * inside - v_plate / (v_plate + bat_speed) * omega * dt;
    if theta.abs() > p.max_bat_angle {
        return Swing::Whiff { bat_speed, miss: if theta > 0.0 { Miss::Early } else { Miss::Late } };
    }
    let dy = p.swing_radius * theta.tan();
    // The barrel is still accelerating through the hitting zone: contact out in front
    // is made with more bat speed than contact deep in the zone.
    let bat_speed = bat_speed * (1.0 + p.bat_speed_depth_gain * dy.clamp(-0.6, 0.6));
    let y_c = p.contact_depth + dy;

    // Ball at the contact depth (straight-line extrapolation over well under a metre).
    let vel = f.plate_vel;
    let tau = (y_c - PLATE_FRONT_Y) / vel.y;
    let ball_pos = f.plate_pos + vel * tau;
    let descent = (-vel.z / vel.y.abs()).atan();

    // ---- swing plane and vertical offset between ball and barrel ----
    // The barrel moves in a plane that is tilted sideways (barrel below the hands) and
    // pitched upward toward the pitcher by the batter's attack angle. A pitch travelling
    // in that plane can be met at any depth; the mismatch between the pitch's descent
    // and the plane's pitch turns a timing error into a vertical miss.
    let plane_pitch = rng.gauss(batter.attack_angle, batter.attack_sd);
    let d_act = zone.signed_distance(f.plate_pos.x, f.plate_pos.z, env.ball.radius);
    let d_out = (d_act + p.barrel_outside_offset).max(0.0);
    let loc_gain = 1.0 + p.barrel_outside_gain * d_out + p.barrel_outside_gain2 * d_out * d_out;
    let offset = rng.gauss(batter.barrel_aim, batter.barrel_sd * speed_ratio.powf(p.barrel_speed_exp) * loc_gain * prec)
        - p.tracking_residual * per.bias_z
        - p.noise_residual * (per.perceived_z - f.plate_pos.z - per.bias_z)
        + p.height_bias * (f.plate_pos.z - zone.mid())
        + if matches!(thrown.kind, PitchKind::FourSeam | PitchKind::Sinker) { p.ride_bias * (f.move_z - p.ride_reference) } else { 0.0 }
        + (descent.tan() - plane_pitch.tan()) * dy;

    // ---- where along the bat ----
    let inside_perceived = -out * per.perceived_x;
    // Beyond his reach the batter can no longer bring the barrel to the ball.
    let beyond = (-inside - p.reach_limit).max(0.0);
    // A straight pitch crosses a bat that is turned by `theta` further from the hands
    // (toward the end of the bat) than it crosses a square bat; the hands make up for
    // part of that.
    let off_end = p.off_end_gain * p.swing_radius * (1.0 / theta.cos().max(0.3) - 1.0);
    let along = bat.sweet_spot_from_tip + p.reach_per_m * inside - beyond - off_end
        + p.tracking_residual * (inside - inside_perceived)
        + rng.gauss(0.0, batter.reach_sd);
    if !(-0.01..=0.60).contains(&along) {
        return Swing::Whiff { bat_speed, miss: if along < 0.0 { Miss::OffTheEnd } else { Miss::Jammed } };
    }
    let along = along.max(0.005);

    // ---- bat geometry ----
    let tilt = (p.bat_tilt_mid + p.bat_tilt_slope * (zone.mid() - f.plate_pos.z)).clamp(0.12, 0.95);
    let e1 = Vec3::new(0.0, plane_pitch.cos(), plane_pitch.sin());
    let lateral = Vec3::new(out * tilt.cos(), 0.0, -tilt.sin());
    let e2 = (lateral - e1 * lateral.dot(e1)).normalized();
    let (sth, cth) = theta.sin_cos();
    let axis = e2 * cth + e1 * sth;
    let forward = e1 * cth - e2 * sth;
    let attack = forward.z.clamp(-1.0, 1.0).asin();
    let point_speed = bat_speed * ((p.swing_radius + (bat.sweet_spot_from_tip - along)) / p.swing_radius).max(0.2);
    let bat_vel = forward * point_speed;

    let contact = Contact { ball_vel: vel, ball_spin: f.spin, bat_axis: axis, bat_vel, offset, dist_from_tip: along };
    match collide(&env.ball, bat, &contact) {
        Err(_) => Swing::Whiff { bat_speed, miss: if offset < 0.0 { Miss::Over } else { Miss::Under } },
        Ok(res) => {
            if res.vel.y <= 0.5 {
                // Deflected backward: a tip if it barely changed direction.
                let cos_defl = res.vel.normalized().dot(vel.normalized());
                if cos_defl > p.foul_tip_angle.cos() {
                    Swing::FoulTip { caught: rng.chance(p.foul_tip_catch), bat_speed }
                } else {
                    Swing::FoulBack { bat_speed }
                }
            } else {
                let (wb, ws, _) = spin_components(res.vel, res.spin);
                let info = BattedInfo {
                    exit_speed: res.vel.norm(),
                    launch_angle: launch_angle(res.vel),
                    spray: spray_angle(res.vel),
                    backspin: wb,
                    sidespin: ws,
                    bat_speed,
                    attack_angle: attack,
                };
                Swing::Batted { state: BallState::new(ball_pos, res.vel, res.spin), info }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitching::{choose_pitch, expectation, selection_weights, throw_pitch, PitcherState, MAX_PITCHES};
    use crate::profile::{Hand, PitcherProfile};
    use bm_physics::units::MPH;

    struct Tally {
        pitches: usize,
        swings: usize,
        whiffs: usize,
        batted: usize,
        ev_sum: f64,
        la_sum: f64,
        pull: usize,
    }

    fn run(batter: &BatterProfile, pitcher: &PitcherProfile, n: usize, seed: u64) -> Tally {
        let env = MatchEnv::neutral_mlb();
        let mut st = PitcherState::new(pitcher, &env);
        let zone = StrikeZone::for_batter(batter.height, &env.rules);
        let mut rng = Rng::new(seed);
        let mut w = [0.0; MAX_PITCHES];
        let mut t = Tally { pitches: 0, swings: 0, whiffs: 0, batted: 0, ev_sum: 0.0, la_sum: 0.0, pull: 0 };
        for i in 0..n {
            let (balls, strikes) = (i % 3, i % 2);
            let k = selection_weights(pitcher, batter.hand, balls, strikes, &env.params, &mut w);
            let exp = expectation(pitcher, &st, &w[..k], env.params.expect_mix);
            let choice = choose_pitch(pitcher, batter.hand, &zone, balls, strikes, &w[..k], &env.params, &mut rng);
            st.pitch_count = 30;
            let thrown = throw_pitch(pitcher, &mut st, &choice, &env, &mut rng);
            t.pitches += 1;
            match respond(&env, batter, &zone, balls, strikes, &thrown, &exp, &mut rng) {
                Swing::Take => {}
                Swing::Whiff { .. } => {
                    t.swings += 1;
                    t.whiffs += 1;
                }
                Swing::FoulTip { .. } | Swing::FoulBack { .. } => t.swings += 1,
                Swing::Batted { info, .. } => {
                    t.swings += 1;
                    t.batted += 1;
                    t.ev_sum += info.exit_speed;
                    t.la_sum += info.launch_angle;
                    if info.spray * batter.hand.pull_sign() > 0.0 {
                        t.pull += 1;
                    }
                }
            }
        }
        t
    }

    #[test]
    fn average_matchup_is_in_a_realistic_range() {
        let b = BatterProfile::league_average(Hand::Right);
        let p = PitcherProfile::league_average(Hand::Right);
        let t = run(&b, &p, 30_000, 11);
        let swing = t.swings as f64 / t.pitches as f64;
        let whiff = t.whiffs as f64 / t.swings as f64;
        let ev = t.ev_sum / t.batted as f64 / MPH;
        let la = (t.la_sum / t.batted as f64).to_degrees();
        assert!(swing > 0.38 && swing < 0.58, "swing rate {swing}");
        assert!(whiff > 0.15 && whiff < 0.36, "whiff per swing {whiff}");
        assert!(ev > 78.0 && ev < 96.0, "mean exit velocity {ev}");
        assert!(la > 0.0 && la < 30.0, "mean launch angle {la}");
    }

    #[test]
    fn skills_move_outcomes_in_the_right_direction() {
        let p = PitcherProfile::league_average(Hand::Right);
        let avg = BatterProfile::league_average(Hand::Left);
        let base = run(&avg, &p, 30_000, 5);

        // Better barrel control -> fewer whiffs.
        let mut contact = avg.clone();
        contact.barrel_sd *= 0.7;
        contact.timing_sd *= 0.8;
        let c = run(&contact, &p, 30_000, 5);
        assert!((c.whiffs as f64 / c.swings as f64) < (base.whiffs as f64 / base.swings as f64) - 0.03);

        // More bat speed -> harder contact.
        let mut power = avg.clone();
        power.bat_speed += 4.0 * MPH;
        let pw = run(&power, &p, 30_000, 5);
        let gain = (pw.ev_sum / pw.batted as f64 - base.ev_sum / base.batted as f64) / MPH;
        assert!(gain > 2.5 && gain < 7.0, "exit velocity gain for +4 mph bat speed: {gain}");

        // Harder thrower -> more whiffs.
        let mut gas = p.clone();
        for d in &mut gas.pitches {
            d.shape.speed += 4.0 * MPH;
        }
        let g = run(&avg, &gas, 30_000, 5);
        assert!((g.whiffs as f64 / g.swings as f64) > (base.whiffs as f64 / base.swings as f64) + 0.01);

        // An early swinger pulls the ball more.
        let mut early = avg.clone();
        early.timing_bias -= 0.006;
        let e = run(&early, &p, 30_000, 5);
        assert!((e.pull as f64 / e.batted as f64) > (base.pull as f64 / base.batted as f64) + 0.05);
    }
}
