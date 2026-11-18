//! The batter's side of a pitch: perception, the swing/take decision and the
//! swing itself. The outcome of a swing is not drawn from a table: the model
//! produces a bat position, orientation and velocity, and the bat-ball collision
//! in `bm-physics` decides whether and how the ball is struck.

use crate::params::ModelParams;
use crate::pitching::{Expectation, MatchEnv, Thrown};
use crate::profile::{BatterProfile, PitchGroup, PitchKind};
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
    /// Backspin, sidespin and gyrospin (about the direction of flight), rad/s.
    pub backspin: f64,
    pub sidespin: f64,
    pub gyrospin: f64,
    /// Bat speed at the sweet spot, m/s.
    pub bat_speed: f64,
    /// Attack angle of the swing, rad.
    pub attack_angle: f64,
    /// The ball was bunted.
    pub bunt: bool,
    /// Where and how the barrel met the ball.
    pub shape: SwingShape,
}

/// Geometry of a swing where the barrel crossed the path of the ball.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwingShape {
    /// Bat speed at the sweet spot, m/s.
    pub bat_speed: f64,
    /// Contact point ahead (+) or behind (-) the batter's usual one, m.
    pub depth: f64,
    /// Horizontal direction of the barrel's travel, rad (+ = toward the pull side).
    pub direction: f64,
    /// Speed of the ball off the bat, m/s, and its elevation above the horizontal, rad,
    /// whichever way it went; zero when the bat missed.
    pub exit_speed: f64,
    pub elevation: f64,
    /// Where the ball met the bat, m from the end of the barrel; zero when the bat missed.
    pub along: f64,
    /// Impact parameter of the collision, m: how far the ball's centre passed above (+)
    /// or below (-) the bat's axis, also for swings that missed.
    pub offset: f64,
    /// Sideways tilt of the swing plane above the horizontal, rad.
    pub tilt: f64,
    /// Direction of the bat from knob to tip at impact.
    pub axis: Vec3,
    /// Horizontal direction of the sweet spot's travel at impact, rad (+ = toward the pull
    /// side), as bat tracking measures it. `direction` is the same turn measured in the
    /// tilted swing plane.
    pub azimuth: f64,
    /// Climb of the sweet spot's travel at impact (bat tracking's attack angle), rad.
    /// `azimuth`, `attack`, `tilt` and `offset` are zero for swings timed too far off to
    /// come near the ball (`tilt` is never zero otherwise); `axis` is also zero when the
    /// ball passed beyond either end of the bat.
    pub attack: f64,
}

/// What the batter has been told to do with the pitch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Approach {
    /// Hit away.
    #[default]
    Normal,
    /// Bunt to move the runners up.
    SacrificeBunt,
    /// Bunt for a base hit.
    BuntForHit,
    /// Swing at anything hittable, with a short swing, to protect the runner who is going.
    HitAndRun,
    /// Do not swing.
    Take,
}

impl Approach {
    pub fn is_bunt(self) -> bool {
        matches!(self, Approach::SacrificeBunt | Approach::BuntForHit)
    }
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
    Whiff { shape: SwingShape, miss: Miss },
    /// Tipped straight back; a strike if the catcher holds it.
    FoulTip { caught: bool, shape: SwingShape },
    /// Fouled backward out of play.
    FoulBack { shape: SwingShape },
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
    let recognized = rng.chance((batter.eye - thrown.disguise + params.kind_recognition[thrown.kind.index()]).clamp(0.02, 0.99));
    let rho = if recognized {
        1.0 - rng.gauss(0.0, params.recognition_sd).abs().min(0.5)
    } else {
        rng.range(0.0, (params.fooled_recognition * batter.adjust).min(0.95))
    };
    // A pitch picked up for what it is may be let go on purpose (`take_kind`).
    let take = if recognized { params.take_kind[thrown.kind.index()][strikes.min(2)].clamp(0.0, 1.0) } else { 0.0 };
    let drop_act = f.move_z - 0.5 * G * f.time * f.time;
    // A pitch that differs a lot from the fastball is easier to identify, so the
    // misjudgement grows less than proportionally with the difference in movement.
    // One fastball taken for another is tracked like any fastball.
    let same_group = if thrown.kind.group() == PitchGroup::Fastball && expected.fastball { params.fastball_misread } else { 1.0 };
    let (dmx, dmz) = {
        let (x, z) = (expected.move_x - f.move_x, expected.drop_z - drop_act);
        let sat = same_group / (1.0 + (x * x + z * z).sqrt() / params.recognition_scale);
        (x * sat, z * sat)
    };
    // Mostly small errors, now and then a pitch that is badly misread.
    let sd = batter.perception_sd * if rng.chance((params.misread_rate * batter.lapse).min(0.9)) { params.misread_factor } else { 1.0 };
    let miss = 1.0 - params.location_adjust * rho;
    let px = f.plate_pos.x + miss * dmx + rng.gauss(0.0, sd);
    let pz = f.plate_pos.z + miss * dmz + rng.gauss(0.0, sd);
    // The decision comes before the late break: the location is judged partly as if the
    // pitch moved like the pitcher's average pitch in this count (`prior_pull`).
    let g = usize::from(strikes >= 2);
    let dec_x = px + params.prior_pull[0][g] * (expected.prior_x - f.move_x);
    let dec_z = pz + params.prior_pull[1][g] * (expected.prior_z - drop_act);
    let d_hat = zone.signed_distance(dec_x, dec_z, ball_radius);
    let d0 = params.swing_d0
        + batter.chase
        + match strikes {
            0 => 0.0,
            1 => params.swing_d0_one_strike,
            _ => params.swing_d0_two_strikes,
        };
    let base = (params.swing_base[balls.min(3)][strikes.min(2)] * batter.aggression).min(0.992);
    let swing_prob = base * sigmoid(-(d_hat - d0) / params.swing_width) * (1.0 - take);
    Perception { recognition: rho, perceived_x: px, perceived_z: pz, bias_z: miss * dmz, swing_prob }
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

/// Response of the batter to a pitch under a given approach. `Approach::Normal` is
/// exactly [`respond`].
#[allow(clippy::too_many_arguments)]
pub fn respond_with(
    env: &MatchEnv,
    batter: &BatterProfile,
    zone: &StrikeZone,
    balls: usize,
    strikes: usize,
    thrown: &Thrown,
    expected: &Expectation,
    approach: Approach,
    bunt_toward_third: bool,
    rng: &mut Rng,
) -> Swing {
    match approach {
        Approach::Normal => respond(env, batter, zone, balls, strikes, thrown, expected, rng),
        Approach::Take => Swing::Take,
        Approach::HitAndRun => {
            let p = &env.params;
            let per = perceive(p, batter, zone, balls, strikes, thrown, expected, env.ball.radius, rng);
            if zone.signed_distance(per.perceived_x, per.perceived_z, env.ball.radius) > p.hit_and_run_reach {
                return Swing::Take;
            }
            // The short, contact-first swing of a two-strike count.
            swing(env, batter, zone, strikes.max(2), thrown, expected, &per, rng)
        }
        Approach::SacrificeBunt | Approach::BuntForHit => bunt(env, batter, zone, thrown, approach == Approach::BuntForHit, bunt_toward_third, rng),
    }
}

/// Bunt attempt. The bat is held across the plate and pushed gently at the ball;
/// what comes off it is decided by the same collision model as a swing.
pub fn bunt(env: &MatchEnv, batter: &BatterProfile, zone: &StrikeZone, thrown: &Thrown, for_hit: bool, toward_third: bool, rng: &mut Rng) -> Swing {
    let p = &env.params;
    let f = &thrown.flight;
    let out = batter.hand.outside_sign();
    // Error multiplier: 1.5 for a hopeless bunter, 1.0 for an average one, 0.5 for an expert.
    let err = 1.5 - batter.bunt.clamp(0.0, 1.0);
    // The bunter follows the pitch all the way in, so he sees its location better than on a swing.
    let px = f.plate_pos.x + rng.gauss(0.0, 0.5 * batter.perception_sd);
    let pz = f.plate_pos.z + rng.gauss(0.0, 0.5 * batter.perception_sd);
    if zone.signed_distance(px, pz, env.ball.radius) > p.bunt_offer_margin {
        return Swing::Take; // pulls the bat back
    }
    let speed_ratio = f.plate_vel.norm() / p.ref_plate_speed;
    let push = (rng.gauss(p.bunt_push + if for_hit { p.bunt_hit_push } else { 0.0 }, p.bunt_push_sd)).max(0.0);
    let shape = SwingShape { bat_speed: push, ..Default::default() };
    if rng.chance((p.bunt_miss * err * speed_ratio).min(0.9)) {
        return Swing::Whiff { shape, miss: Miss::Under };
    }
    let offset = rng.gauss(p.bunt_aim, p.bunt_offset_sd * err * speed_ratio);
    let along = rng.gauss(p.bunt_along, p.bunt_along_sd).clamp(0.01, 0.45);
    // Intended direction of the bunt (+ toward first base) with the bunter's error.
    let dir = if toward_third { -p.bunt_direction } else { p.bunt_direction } + rng.gauss(0.0, p.bunt_direction_sd * err);
    let normal = Vec3::new(dir.sin(), dir.cos(), 0.0);
    // The bat lies across the plate, barrel on the far side, perpendicular to that direction.
    let axis = Vec3::new(out * dir.cos(), -out * dir.sin(), 0.0);

    let vel = f.plate_vel;
    let tau = (p.contact_depth - PLATE_FRONT_Y) / vel.y;
    let ball_pos = f.plate_pos + vel * tau;
    let contact = Contact { ball_vel: vel, ball_spin: f.spin, bat_axis: axis, bat_vel: normal * push, offset, dist_from_tip: along };
    match collide(&env.ball, &batter.bat, &contact) {
        Err(_) => Swing::Whiff { shape, miss: if offset < 0.0 { Miss::Over } else { Miss::Under } },
        Ok(res) => {
            if res.vel.y <= 0.5 {
                let cos_defl = res.vel.normalized().dot(vel.normalized());
                if cos_defl > p.foul_tip_angle.cos() {
                    Swing::FoulTip { caught: rng.chance(p.foul_tip_catch), shape }
                } else {
                    Swing::FoulBack { shape }
                }
            } else {
                let (wb, ws, wg) = spin_components(res.vel, res.spin);
                let info = BattedInfo {
                    exit_speed: res.vel.norm(),
                    launch_angle: launch_angle(res.vel),
                    spray: spray_angle(res.vel),
                    backspin: wb,
                    sidespin: ws,
                    gyrospin: wg,
                    bat_speed: push,
                    attack_angle: 0.0,
                    bunt: true,
                    shape,
                };
                Swing::Batted { state: BallState::new(ball_pos, res.vel, res.spin), info }
            }
        }
    }
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
    // Speed against what the batter is used to from this kind of pitch: a slider is not
    // easier to time than a fastball for being slower, but a hard slider is harder than a soft one.
    let speed_ratio = v_plate / env.typical_plate_speed[thrown.kind.index()];
    // Committing earlier against a harder thrower (`commit_exp`).
    let commit = (env.typical_fastball_time / expected.time).powf(p.commit_exp);

    // ---- timing ----
    let dt_raw = expected.time - f.time;
    let dt_bias = (1.0 - p.timing_adjust * per.recognition) * dt_raw / (1.0 + dt_raw.abs() / p.recognition_time_scale);
    let fooled = (dt_bias.abs() / p.fooled_time).min(1.0);
    let mut bat_speed = rng.gauss(batter.bat_speed, batter.bat_speed_sd) * (1.0 - (p.fooled_speed_loss * batter.speed_loss).min(0.9) * fooled * fooled);
    if two {
        bat_speed *= p.two_strike_speed;
    }
    // Partial swings: check swings, defensive two-strike hacks, swings the batter gives
    // up on. The barrel is not delivered properly, so they rarely square the ball up.
    let partial = rng.chance((p.partial_swing_rate * if two { p.partial_two_strike_gain } else { 1.0 } + p.partial_fooled_rate * fooled).min(0.9));
    if partial {
        bat_speed *= 1.0 - ((1.0 - rng.range(p.partial_speed_min, 0.92)) * batter.speed_loss).min(0.9);
    }
    let prec = prec * if partial { p.partial_precision } else { 1.0 };
    bat_speed = bat_speed.max(8.0);
    // Late against a pitcher who throws harder than the fastballs the batter sees every day.
    let lag = p.velocity_lag * (env.typical_fastball_time - expected.time);
    let dt = batter.timing_bias + dt_bias + lag + rng.gauss(0.0, batter.timing_sd * speed_ratio.powf(p.timing_speed_exp) * commit * prec);

    // ---- depth of contact and direction of the barrel ----
    // A swing that is early meets the ball further out in front: ball and barrel
    // close on each other at v_plate + bat_speed.
    let inside = -out * f.plate_pos.x;
    let dy = -(v_plate * bat_speed / (v_plate + bat_speed)) * dt;
    // The barrel turns toward the pull side as it travels forward, by one radian per
    // `swing_arc_radius` of depth (Statcast: 1.26 deg per inch), and a little with the
    // pitch's location.
    let theta = p.plan_angle_per_m * inside + (dy / p.swing_arc_radius).atan() + rng.gauss(0.0, p.direction_sd);
    if dy.abs() > p.max_depth || theta.abs() > p.max_bat_angle {
        let shape = SwingShape { bat_speed, depth: dy.clamp(-1.5, 1.5), direction: theta, ..Default::default() };
        return Swing::Whiff { shape, miss: if dy > 0.0 { Miss::Early } else { Miss::Late } };
    }
    // The barrel is still accelerating through the hitting zone: contact out in front
    // is made with more bat speed than contact deep in the zone, up to a point.
    let bat_speed = bat_speed * (1.0 + p.bat_speed_depth_gain * dy.clamp(-0.6, p.bat_speed_depth_cap));
    let mut shape = SwingShape { bat_speed, depth: dy, direction: theta, ..Default::default() };
    let y_c = p.contact_depth + dy;

    // Ball at the contact depth (straight-line extrapolation over well under a metre).
    let vel = f.plate_vel;
    let tau = (y_c - PLATE_FRONT_Y) / vel.y;
    let ball_pos = f.plate_pos + vel * tau;
    let descent = (-vel.z / vel.y.abs()).atan();

    // ---- swing plane and vertical offset between ball and barrel ----
    // The barrel moves in a plane that is tilted sideways (barrel below the hands) and
    // pitched upward toward the pitcher: the batter's attack angle, steeper against high
    // pitches and further out in front, flatter against breaking and offspeed pitches
    // (2025 bat tracking). Had barrel and ball kept to their paths, contact `dy` away from
    // the planned point would miss by (tan descent - tan plane) dy. The batter brings the
    // barrel to the ball's height instead, but the correction is noisy in proportion to
    // its size: on balls in play the impact parameter does not follow the sign of that
    // miss at any depth, while swings and misses climb with its size. The arc of the swing
    // still lifts the barrel over the ball when contact is far from the planned point.
    let k = thrown.kind.index();
    let plane_pitch = rng.gauss(
        batter.attack_angle
            + p.attack_shift
            + p.attack_height * (f.plate_pos.z - zone.mid())
            + p.attack_depth * dy
            + p.attack_inside * inside
            + p.attack_kind[k],
        batter.attack_sd,
    );
    let tilt = (p.bat_tilt_mid + p.bat_tilt_slope * (zone.mid() - f.plate_pos.z) + rng.gauss(0.0, p.bat_tilt_sd)).clamp(0.12, 0.95);
    let (behind, ahead) = ((p.arc_centre - dy).max(0.0), (dy - p.arc_early_from).max(0.0));
    let arc_rise = 0.5 * tilt.sin() / p.swing_arc_radius * (p.arc_scale * behind * behind + p.arc_scale_early * ahead * ahead);
    let d_act = zone.signed_distance(f.plate_pos.x, f.plate_pos.z, env.ball.radius);
    let d_out = (d_act + p.barrel_outside_offset).max(0.0);
    let loc_gain = 1.0 + p.barrel_outside_gain * d_out + p.barrel_outside_gain2 * d_out * d_out;
    // Batters adapt only in part to how much a pitch rises or sinks compared with what
    // they expect of it: the league's typical pitch of its type, except that four-seamers
    // and sinkers are partly taken for one generic fastball.
    let typical_rise = env.typical_rise[thrown.kind.index()];
    let confusion = if matches!(thrown.kind, PitchKind::FourSeam | PitchKind::Sinker) { p.ride_confusion } else { 0.0 };
    let ride_expected = typical_rise + confusion * (p.ride_reference - typical_rise);
    let offset = rng
        .gauss(batter.barrel_aim + p.aim_kind[k], batter.barrel_sd * p.barrel_kind[k] * speed_ratio.powf(p.barrel_speed_exp) * commit * loc_gain * prec)
        - p.tracking_residual * per.bias_z
        - p.noise_residual * (per.perceived_z - f.plate_pos.z - per.bias_z)
        + p.height_bias * (f.plate_pos.z - zone.mid())
        + p.ride_bias * (f.move_z - ride_expected)
        + p.plane_mismatch * (descent.tan() - plane_pitch.tan()) * dy
        + rng.gauss(0.0, p.mismatch_noise * ((descent.tan() - plane_pitch.tan()) * dy).abs())
        - arc_rise;

    // ---- path of the sweet spot ----
    let e1 = Vec3::new(0.0, plane_pitch.cos(), plane_pitch.sin());
    let across = |tilt: f64| {
        let lateral = Vec3::new(out * tilt.cos(), 0.0, -tilt.sin());
        (lateral - e1 * lateral.dot(e1)).normalized()
    };
    // The sweet spot travels in the tilted swing plane.
    let (sth, cth) = theta.sin_cos();
    let forward = e1 * cth - across(tilt) * sth;
    let attack = forward.z.clamp(-1.0, 1.0).asin();
    shape.offset = offset;
    shape.tilt = (1.0 - e1.cross(across(tilt)).normalized().z.powi(2)).max(0.0).sqrt().asin();
    shape.azimuth = (-out * forward.x).atan2(forward.y);
    shape.attack = attack;

    // ---- where along the bat ----
    let inside_perceived = -out * per.perceived_x;
    // Beyond his reach the batter can no longer bring the barrel to the ball.
    let beyond = (-inside - p.reach_limit).max(0.0);
    // The bat sweeps about a point near the body, so a ball met early is met further
    // out along the bat, toward its end, and loses much of its speed there (Statcast:
    // exit velocity 85 mph at 10-17 in out in front and 72 mph beyond that, against 91
    // at the usual depth). Met late, the hands are still ahead of the barrel and the
    // ball stays near the sweet spot (88 mph at 15-25 in deep).
    let off_end =
        if theta > 0.0 { p.off_end_gain } else { p.off_end_late } * p.pivot_radius * (1.0 / theta.cos().max(0.3) - 1.0) + p.off_end_linear * theta.max(0.0);
    // The batter still gets the last few centimetres of the barrel to the ball.
    let off_end = p.off_end_max * (1.0 - (-off_end / p.off_end_max).exp());
    let along = bat.sweet_spot_from_tip + p.contact_along + p.reach_per_m * inside - beyond - off_end
        + p.tracking_residual * (inside - inside_perceived)
        + rng.gauss(0.0, batter.reach_sd);
    if along > 0.60 {
        return Swing::Whiff { shape, miss: Miss::Jammed };
    }
    if along < -0.5 * env.ball.radius {
        return Swing::Whiff { shape, miss: Miss::OffTheEnd };
    }
    if along < 0.0 {
        // The ball catches the very end of the bat and squirts away foul.
        shape.exit_speed = 0.6 * v_plate;
        shape.elevation = rng.range(-0.6, 0.6);
        return Swing::FoulBack { shape };
    }
    let along = along.max(0.005);

    // ---- the face of the bat ----
    // The barrel does not travel exactly square to the bat: the hands are still moving
    // forward, so the bat faces further round than its sweet spot is heading, and it
    // turns faster than the direction of its travel does. The wrists roll over before
    // the bat can face much further round than `face_max`.
    let face = p.face_max * ((p.face_gain * theta + rng.gauss(p.face_lead, p.face_sd)) / p.face_max).tanh();
    let (sf, cf) = face.sin_cos();
    // As they roll over, the bat itself levels out: balls met far out in front no
    // longer go the way a tilted bat sends them (Statcast: at the usual depth topped
    // balls are pulled 30 deg and balls hit under go 20 deg the other way; 10-25 in out
    // in front both stay within 4-30 deg of the pull side).
    let rolled = ((theta - p.roll_from) / p.roll_span).clamp(0.0, 1.0);
    let axis = across(tilt * (1.0 - p.roll_flatten * rolled)) * cf + e1 * sf;
    let point_speed = bat_speed * ((p.swing_radius + (bat.sweet_spot_from_tip - along)) / p.swing_radius).max(0.2);
    let bat_vel = forward * point_speed;
    shape.axis = axis;
    let mut contact = Contact { ball_vel: vel, ball_spin: f.spin, bat_axis: axis, bat_vel, offset, dist_from_tip: along };
    let mut result = collide(&env.ball, bat, &contact);
    if result.is_err() {
        // Getting a piece of it. With well-timed swings, batters foul the ball off far
        // more often and miss far less often than any smooth spread of the barrel's
        // height allows: at every distance, near misses are about a third as common as
        // the spread of contact heights implies (Statcast bat tracking, 2025). A share
        // of the swings that would pass over or under the ball therefore graze it.
        let limit = env.ball.radius + bat.radius_at(along);
        let on_time = (-0.5 * ((dy - p.piece_centre) / p.piece_depth).powi(2)).exp();
        let protect = if two { p.piece_two_strikes } else { 1.0 };
        if offset.abs() - limit < p.piece_reach && rng.chance((p.piece_rate * protect * on_time * batter.foul_skill).min(0.95)) {
            // Mostly the thinnest of contact.
            let u = rng.f64();
            contact.offset = offset.signum() * (limit - 0.0002 - p.piece_band * u * u);
            result = collide(&env.ball, bat, &contact);
        }
    }
    match result {
        Err(_) => Swing::Whiff { shape, miss: if offset < 0.0 { Miss::Over } else { Miss::Under } },
        Ok(mut res) => {
            if p.spray_turn != 0.0 {
                let (st, ct) = (p.spray_turn * out).sin_cos();
                let turn = |v: Vec3| Vec3::new(v.x * ct - v.y * st, v.x * st + v.y * ct, v.z);
                res.vel = turn(res.vel);
                res.spin = turn(res.spin);
            }
            shape.along = along;
            shape.offset = contact.offset;
            shape.exit_speed = res.vel.norm();
            shape.elevation = (res.vel.z / shape.exit_speed.max(1e-9)).clamp(-1.0, 1.0).asin();
            if res.vel.y <= 0.5 {
                // Deflected backward: a tip if it barely changed direction.
                let cos_defl = res.vel.normalized().dot(vel.normalized());
                if cos_defl > p.foul_tip_angle.cos() {
                    Swing::FoulTip { caught: rng.chance(p.foul_tip_catch), shape }
                } else {
                    Swing::FoulBack { shape }
                }
            } else {
                let (wb, ws, wg) = spin_components(res.vel, res.spin);
                let info = BattedInfo {
                    exit_speed: res.vel.norm(),
                    launch_angle: launch_angle(res.vel),
                    spray: spray_angle(res.vel),
                    backspin: wb,
                    sidespin: ws,
                    gyrospin: wg,
                    bat_speed,
                    attack_angle: attack,
                    bunt: false,
                    shape,
                };
                Swing::Batted { state: BallState::new(ball_pos, res.vel, res.spin), info }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitching::{choose_pitch, expectation, selection_weights, throw_pitch, PitchChoice, PitcherState, MAX_PITCHES};
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
            let thrown = throw_pitch(pitcher, &mut st, &choice, 1.0, &env, &mut rng);
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

    /// Swings of the average matchup: geometry, outcome (0 miss, 1 foul tip or foul
    /// back, 2 put in front of the plate) and pull-side spray of the batted ones, rad.
    fn shapes(env: &MatchEnv, n: usize, seed: u64) -> Vec<(SwingShape, u8, f64)> {
        shapes_against(env, &PitcherProfile::league_average(Hand::Right), n, seed)
    }

    /// Swings of the average right-handed batter against `pitcher`: the shape of each
    /// swing, what came of it (0 = miss, 1 = foul tip or foul back, 2 = batted) and the
    /// pull-side spray of batted balls.
    fn shapes_against(env: &MatchEnv, pitcher: &PitcherProfile, n: usize, seed: u64) -> Vec<(SwingShape, u8, f64)> {
        let batter = BatterProfile::league_average(Hand::Right);
        let pitcher = pitcher.clone();
        let mut st = PitcherState::new(&pitcher, env);
        let zone = StrikeZone::for_batter(batter.height, &env.rules);
        let mut rng = Rng::new(seed);
        let mut w = [0.0; MAX_PITCHES];
        let mut out = Vec::new();
        for i in 0..n {
            let (balls, strikes) = (i % 3, i % 2);
            let k = selection_weights(&pitcher, batter.hand, balls, strikes, &env.params, &mut w);
            let exp = expectation(&pitcher, &st, &w[..k], env.params.expect_mix);
            let choice = choose_pitch(&pitcher, batter.hand, &zone, balls, strikes, &w[..k], &env.params, &mut rng);
            st.pitch_count = 30;
            let thrown = throw_pitch(&pitcher, &mut st, &choice, 1.0, env, &mut rng);
            match respond(env, &batter, &zone, balls, strikes, &thrown, &exp, &mut rng) {
                Swing::Take => {}
                Swing::Whiff { shape, .. } => out.push((shape, 0, 0.0)),
                Swing::FoulTip { shape, .. } | Swing::FoulBack { shape } => out.push((shape, 1, 0.0)),
                Swing::Batted { info, .. } => out.push((info.shape, 2, info.spray * batter.hand.pull_sign())),
            }
        }
        out
    }

    #[test]
    fn the_barrel_turns_as_contact_moves_forward() {
        // Statcast: 1.26 deg of attack direction per inch of contact depth across
        // batters, 1.44 within a batter.
        let env = MatchEnv::neutral_mlb();
        let batted: Vec<_> = shapes(&env, 40_000, 5).into_iter().filter(|s| s.1 == 2).collect();
        let n = batted.len() as f64;
        let (mx, my) = (batted.iter().map(|s| s.0.depth).sum::<f64>() / n, batted.iter().map(|s| s.0.direction).sum::<f64>() / n);
        let sxy: f64 = batted.iter().map(|s| (s.0.depth - mx) * (s.0.direction - my)).sum();
        let sxx: f64 = batted.iter().map(|s| (s.0.depth - mx).powi(2)).sum();
        let deg_per_inch = (sxy / sxx).to_degrees() * 0.0254;
        assert!((1.0..1.7).contains(&deg_per_inch), "direction per inch of depth {deg_per_inch}");
        // Balls met out in front are pulled, balls met deep go the other way.
        let mean = |f: &dyn Fn(f64) -> bool| {
            let v: Vec<f64> = batted.iter().filter(|s| f(s.0.depth)).map(|s| s.2).collect();
            (v.iter().sum::<f64>() / v.len() as f64).to_degrees()
        };
        let (front, deep) = (mean(&|d| d > 0.15), mean(&|d| d < -0.15));
        assert!(front > deep + 15.0, "spray out in front {front}, deep {deep}");
        // The ball is met a little under its centre and somewhere on the barrel.
        assert!(batted.iter().all(|s| s.0.along > 0.0 && s.0.exit_speed > 0.0));
    }

    fn mean_of(v: &[(SwingShape, u8, f64)], f: impl Fn(&SwingShape) -> f64) -> f64 {
        let batted: Vec<f64> = v.iter().filter(|x| x.1 == 2).map(|x| f(&x.0)).collect();
        batted.iter().sum::<f64>() / batted.len() as f64
    }

    /// Against a pitcher who throws harder than the league's usual fastball the batter
    /// is a little late (Statcast: 0.26-0.32 in deeper per mph of the pitcher's average).
    #[test]
    fn batters_are_late_on_hard_throwers() {
        let env = MatchEnv::neutral_mlb();
        let usual = PitcherProfile::league_average(Hand::Right);
        let mut hard = usual.clone();
        for d in hard.pitches.iter_mut() {
            d.shape.speed += 5.0 * MPH;
        }
        let depth = |env: &MatchEnv, p: &PitcherProfile| mean_of(&shapes_against(env, p, 60000, 21), |s| s.depth);
        let gap = depth(&env, &usual) - depth(&env, &hard);
        // 5 mph at 0.3 in per mph is 1.5 in, or 0.038 m.
        assert!(gap > 0.015 && gap < 0.075, "contact against the hard thrower is {gap} m deeper");
        let mut flat = MatchEnv::neutral_mlb();
        flat.params.velocity_lag = 0.0;
        let none = depth(&flat, &usual) - depth(&flat, &hard);
        assert!(none < gap - 0.012, "without the lag the gap should shrink: {none} m against {gap} m");
    }

    /// A fastball with more ride than batters expect is hit in the air, one that sinks
    /// more is beaten into the ground; the same goes for a breaking ball against the
    /// usual one of its kind.
    #[test]
    fn ride_lifts_the_ball_and_sink_buries_it() {
        let env = MatchEnv::neutral_mlb();
        let only = |kind: PitchKind, rpm: f64| {
            let mut p = PitcherProfile::league_average(Hand::Right);
            p.pitches.retain(|d| d.kind == PitchKind::FourSeam);
            p.pitches[0].kind = kind;
            p.pitches[0].shape = kind.league_average();
            p.pitches[0].shape.spin_rate += rpm * bm_physics::units::RPM;
            p
        };
        let launch = |p: &PitcherProfile| mean_of(&shapes_against(&env, p, 50000, 33), |s| s.elevation).to_degrees();
        // Induced vertical movement of the pitch, in.
        let ride = |p: &PitcherProfile| {
            let mut st = PitcherState::new(p, &env);
            let mut rng = Rng::new(5);
            let choice = PitchChoice { idx: 0, target_x: 0.0, target_z: 0.75 };
            (0..400).map(|_| throw_pitch(p, &mut st, &choice, 1.0, &env, &mut rng).flight.move_z).sum::<f64>() / 400.0 / bm_physics::units::INCH
        };
        let (up, down) = (only(PitchKind::FourSeam, 300.0), only(PitchKind::FourSeam, -300.0));
        let (lively, flat) = (launch(&up), launch(&down));
        let per_inch = (lively - flat) / (ride(&up) - ride(&down));
        // 2025 Statcast, four-seamers compared between pitchers: 0.78 deg of launch angle
        // per inch of ride.
        assert!((0.5..1.2).contains(&per_inch), "four-seamer with more ride {lively} deg, with less {flat} deg: {per_inch} deg per inch");
        // More spin on a curve is more drop.
        let (sharp, lazy) = (launch(&only(PitchKind::Curve, 400.0)), launch(&only(PitchKind::Curve, -400.0)));
        assert!(sharp < lazy - 1.0, "curve with more drop {sharp} deg, with less {lazy} deg");
    }

    #[test]
    fn batters_get_a_piece_of_near_misses() {
        // Statcast: of the swings that are on time, 7 % miss and 35 % are fouled off.
        let on_time = |env: &MatchEnv| {
            let all: Vec<_> = shapes(env, 40_000, 9).into_iter().filter(|s| s.0.depth.abs() < 0.10 && s.0.direction.abs() < 0.09).collect();
            let n = all.len() as f64;
            (all.iter().filter(|s| s.1 == 0).count() as f64 / n, all.iter().filter(|s| s.1 == 2).count() as f64 / n)
        };
        let env = MatchEnv::neutral_mlb();
        let (miss, front) = on_time(&env);
        assert!((0.04..0.16).contains(&miss), "share of well-timed swings that miss {miss}");
        assert!((0.55..0.85).contains(&front), "share of well-timed swings put in front of the plate {front}");
        let mut rigid = MatchEnv::neutral_mlb();
        rigid.params.piece_rate = 0.0;
        let (miss_rigid, front_rigid) = on_time(&rigid);
        assert!(miss_rigid > miss + 0.05, "without the rule {miss_rigid}, with it {miss}");
        assert!((front_rigid - front).abs() < 0.03, "the rule turns misses into fouls, not into fair balls");
    }

    #[test]
    fn bunts_are_soft_and_mostly_on_the_ground() {
        let env = MatchEnv::neutral_mlb();
        let batter = BatterProfile::league_average(Hand::Right);
        let pitcher = PitcherProfile::league_average(Hand::Right);
        let mut st = PitcherState::new(&pitcher, &env);
        let zone = StrikeZone::for_batter(batter.height, &env.rules);
        let mut rng = Rng::new(21);
        let mut w = [0.0; MAX_PITCHES];
        let (mut offers, mut missed, mut back, mut fair_side, mut ev, mut down, mut third_side) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut good = batter.clone();
        good.bunt = 1.0;
        let mut good_missed = 0.0;
        for i in 0..20_000 {
            let k = selection_weights(&pitcher, batter.hand, 0, 0, &env.params, &mut w);
            let choice = choose_pitch(&pitcher, batter.hand, &zone, 0, 0, &w[..k], &env.params, &mut rng);
            st.pitch_count = 30;
            let thrown = throw_pitch(&pitcher, &mut st, &choice, 1.0, &env, &mut rng);
            if i % 2 == 1 {
                if let Swing::Whiff { .. } = bunt(&env, &good, &zone, &thrown, false, true, &mut rng) {
                    good_missed += 1.0;
                }
                continue;
            }
            match bunt(&env, &batter, &zone, &thrown, false, true, &mut rng) {
                Swing::Take => {}
                Swing::Whiff { .. } => {
                    offers += 1.0;
                    missed += 1.0;
                }
                Swing::FoulTip { .. } | Swing::FoulBack { .. } => {
                    offers += 1.0;
                    back += 1.0;
                }
                Swing::Batted { info, .. } => {
                    offers += 1.0;
                    assert!(info.bunt);
                    if info.spray.abs() < std::f64::consts::FRAC_PI_4 {
                        fair_side += 1.0;
                        ev += info.exit_speed / MPH;
                        if info.launch_angle < 0.0 {
                            down += 1.0;
                        }
                        if info.spray < 0.0 {
                            third_side += 1.0;
                        }
                    }
                }
            }
        }
        assert!(offers > 4000.0);
        let miss_rate = missed / offers;
        assert!((0.04..0.14).contains(&miss_rate), "missed {miss_rate}");
        assert!(good_missed / offers < 0.6 * miss_rate, "a good bunter misses less: {} vs {miss_rate}", good_missed / offers);
        assert!(back / offers < 0.35, "fouled straight back {}", back / offers);
        let fair = fair_side / offers;
        assert!((0.35..0.70).contains(&fair), "between the lines {fair}");
        let mean_ev = ev / fair_side;
        assert!((25.0..45.0).contains(&mean_ev), "bunt exit velocity {mean_ev} mph");
        assert!(down / fair_side > 0.6, "share bunted down {}", down / fair_side);
        assert!(third_side / fair_side > 0.65, "share toward third {}", third_side / fair_side);
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
