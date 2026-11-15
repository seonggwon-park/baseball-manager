//! Behavioural constants of the match engine.
//!
//! The flight and collision physics live in `bm-physics` and use published
//! constants. What is collected here is the *human* part of the model (pitch
//! selection, perception, swing decisions, motor precision, fielding and
//! base-running judgement). These constants are not taken from the literature:
//! they are calibrated so that a league of average players reproduces the 2025
//! MLB aggregates measured from Statcast and the MLB Stats API (see
//! `docs/physics-model.md` for the targets and the calibration report).

use bm_physics::units::{INCH, MPH};

#[derive(Clone, Debug, PartialEq)]
pub struct ModelParams {
    // ---- pitch selection and location ----
    /// SD of the intended target around the pitch type's typical location, m (x, z).
    pub target_sd_x: f64,
    pub target_sd_z: f64,
    /// Fraction of pitches that are badly missed, and the error multiplier for those.
    pub wild_fraction: f64,
    pub wild_factor: f64,
    /// Release-point scatter, m (side, height).
    pub release_sd_side: f64,
    pub release_sd_height: f64,
    /// How strongly the target moves toward the middle (+) or the chase zone (-) by count,
    /// indexed `[balls][strikes]`, in -1..=1.
    pub zone_intent: [[f64; 3]; 4],
    /// Multiplier on fastball usage by count, indexed `[balls][strikes]`.
    pub fastball_bias: [[f64; 3]; 4],
    /// Velocity lost per pitch thrown beyond the pitcher's stamina (fraction of speed).
    pub fatigue_speed: f64,
    /// Command lost per pitch beyond stamina (fraction of the location SD).
    pub fatigue_command: f64,

    // ---- umpire (human) ----
    /// Called-strike probability is `sigmoid(-(d + bias) / scale)` in the signed
    /// distance `d` outside the rulebook zone (m).
    pub ump_bias: f64,
    pub ump_scale: f64,

    // ---- swing decision ----
    /// Probability of swinging at a pitch perceived in the heart of the zone,
    /// indexed `[balls][strikes]`.
    pub swing_base: [[f64; 3]; 4],
    /// Location (signed distance outside the zone, m) of the swing/take boundary.
    pub swing_d0: f64,
    /// Extra expansion of the boundary with one strike and with two strikes, m.
    pub swing_d0_one_strike: f64,
    pub swing_d0_two_strikes: f64,
    /// Width of the swing/take transition, m.
    pub swing_width: f64,
    /// Spread of the recognition of a pitch that was picked up.
    pub recognition_sd: f64,
    /// Upper bound of the recognition of a pitch that was not picked up.
    pub fooled_recognition: f64,
    /// Share of the perceptual location noise that survives to contact (0..1).
    pub noise_residual: f64,
    /// Movement difference at which the misjudgement of an unrecognised pitch is halved, m.
    pub recognition_scale: f64,
    /// Flight-time difference at which the timing misjudgement is halved, s.
    pub recognition_time_scale: f64,
    /// Batters adapt only partly to a fastball's "ride": share of the difference between
    /// its induced vertical movement and the generic fastball that ends up as barrel error.
    pub ride_bias: f64,
    /// Induced vertical movement (whole flight) of the generic fastball, m.
    pub ride_reference: f64,
    /// How far the batter's expectation leans from the main fastball toward the
    /// count's average pitch (0..1).
    pub expect_mix: f64,
    /// Share of pitches whose location is badly misread, and the noise multiplier then.
    pub misread_rate: f64,
    pub misread_factor: f64,

    // ---- swing execution ----
    /// Reference plate speed for the speed dependence of timing/barrel precision, m/s.
    pub ref_plate_speed: f64,
    /// Exponent of the speed dependence of the timing error.
    pub timing_speed_exp: f64,
    /// Exponent of the speed dependence of the barrel error.
    pub barrel_speed_exp: f64,
    /// Share of a pitch-type misjudgement (wrong expected drop) that survives to contact (0..1).
    pub tracking_residual: f64,
    /// Growth of the barrel error for pitches outside the zone, per metre outside.
    pub barrel_outside_gain: f64,
    /// Quadratic term of the same growth, per metre squared.
    pub barrel_outside_gain2: f64,
    /// Offset added to the distance outside the zone before the gain applies, m.
    pub barrel_outside_offset: f64,
    /// Distance outside the plate centre beyond which the barrel cannot reach, m.
    pub reach_limit: f64,
    /// Vertical barrel bias per metre of pitch height above the middle of the zone
    /// (batters get under high pitches and on top of low ones).
    pub height_bias: f64,
    /// Bat tilt (barrel below the hands) at the middle of the zone, rad, and its
    /// change per metre of pitch height.
    pub bat_tilt_mid: f64,
    pub bat_tilt_slope: f64,
    /// Planned horizontal bat angle per metre of inside location, rad/m.
    pub plan_angle_per_m: f64,
    /// Distance from the bat's instantaneous rotation centre to the sweet spot, m.
    pub swing_radius: f64,
    /// Relative gain of bat speed per metre of contact depth out in front.
    pub bat_speed_depth_gain: f64,
    /// Contact depth of a square swing, m in front of the back tip of the plate.
    pub contact_depth: f64,
    /// Horizontal bat angle beyond which the barrel is out of the hitting zone, rad.
    pub max_bat_angle: f64,
    /// Bat-speed loss when badly fooled on timing (fraction at a full mistime).
    pub fooled_speed_loss: f64,
    /// Timing error at which the swing counts as fully mistimed, s.
    pub fooled_time: f64,
    /// Share of swings that are partial (checked or defensive), its multiplier with two
    /// strikes, the extra share when the batter is fooled on timing, and the lowest
    /// bat-speed factor of such a swing.
    pub partial_swing_rate: f64,
    pub partial_two_strike_gain: f64,
    pub partial_fooled_rate: f64,
    pub partial_speed_min: f64,
    /// Share of the geometric off-the-end shift of a mistimed swing that the hands do
    /// not compensate (0..1).
    pub off_end_gain: f64,
    /// Jam / reach: shift of the contact point along the bat per metre of location, m/m.
    pub reach_per_m: f64,
    /// Fraction of a ball tipped straight back that the catcher holds.
    pub foul_tip_catch: f64,
    /// Deflection below which a ball going backward is a tip, rad.
    pub foul_tip_angle: f64,
    /// Two-strike swing: bat-speed factor and precision gain.
    pub two_strike_speed: f64,
    pub two_strike_precision: f64,

    // ---- hit by pitch / wild pitch ----
    /// Lateral distance from the plate centre to the batter's body, m.
    pub batter_body_x: f64,
    pub hbp_width: f64,
    /// Probability that the batter fails to get out of the way.
    pub hbp_no_evade: f64,
    /// Probability that a pitch in the dirt or far off the plate gets by the catcher.
    pub wild_pitch_prob: f64,

    // ---- fielding ----
    /// Height below which a ball in the air can be caught, m.
    pub catch_height: f64,
    /// Scale of the catch probability in the time margin, s.
    pub catch_margin_scale: f64,
    /// Margin offset so that an even race is caught at about 50 %, s.
    pub catch_margin_bias: f64,
    /// Time lost redirecting to a ball that dropped in after a flight, s.
    pub redirect_delay: f64,
    /// Height below which a bouncing ball can be gloved, m.
    pub field_height: f64,
    /// Time to secure a ground ball before the transfer starts, s.
    pub gather_time: f64,
    /// Margin the defence wants before going for the force at second instead of the out at first, s.
    pub force_margin: f64,
    /// Chance that a runner on second advances on a ground out to the left side.
    pub advance_left_side: f64,
    /// Transfer time of a short feed to second base, as a fraction of a normal transfer.
    pub feed_transfer: f64,
    /// Extra time for the pivot on a double play, s.
    pub pivot_time: f64,
    /// Base error probabilities (ground ball, fly ball, throw).
    pub error_ground: f64,
    pub error_fly: f64,
    pub error_throw: f64,
    /// Extra time to play a ball off the outfield wall, s.
    pub wall_play_delay: f64,
    /// Time for a tag after the ball arrives, s.
    pub tag_time: f64,
    /// SD of the timing noise on close plays, s.
    pub play_noise: f64,
    /// SD of the drag coefficient from one batted ball to the next. In a neutral
    /// environment this also stands in for wind and weather, which the flight model
    /// otherwise treats as fixed.
    pub drag_jitter: f64,
    /// Average time lost on throws from the outfield (off line, cut off, short hop), s.
    pub throw_slop: f64,
    /// Drag constant for throws, 1/m (K * Cd of a thrown ball).
    pub throw_drag: f64,
    /// Infielders' lateral reach when diving / lunging, m.
    pub infield_reach: f64,

    // ---- running ----
    /// Delay between contact and the batter leaving the box, s (right-, left-handed).
    pub box_delay_right: f64,
    pub box_delay_left: f64,
    /// Lead-off distances, m (first, second, third).
    pub lead: [f64; 3],
    /// Extra path length per base rounded, m.
    pub turn_length: f64,
    /// Time lost stopping / sliding into a base, s.
    pub slide_time: f64,
    /// Reading a ball in the air: runners break early when the best fielder's margin is
    /// below `read_margin` (s); they then commit at `read_time + read_hang_share * hang time`.
    pub read_margin: f64,
    pub read_time: f64,
    pub read_hang_share: f64,
    /// Safety margin a runner wants before taking an extra base, s.
    pub advance_margin: f64,
    /// Steal attempts: threshold on the estimated margin and base rate.
    pub steal_margin: f64,
    pub steal_rate: f64,
}

impl Default for ModelParams {
    fn default() -> Self {
        ModelParams {
            target_sd_x: 0.13,
            target_sd_z: 0.14,
            wild_fraction: 0.06,
            wild_factor: 2.0,
            release_sd_side: 2.1 * INCH,
            release_sd_height: 1.3 * INCH,
            zone_intent: [[0.20, -0.30, -0.95], [0.25, -0.05, -0.70], [0.50, 0.40, -0.15], [0.82, 0.90, 0.58]],
            fastball_bias: [[1.08, 0.95, 0.75], [1.15, 1.00, 0.75], [1.45, 1.15, 0.85], [2.40, 1.60, 1.15]],
            fatigue_speed: 0.0004,
            fatigue_command: 0.004,

            ump_bias: 0.3 * INCH,
            ump_scale: 0.93 * INCH,

            swing_base: [[0.57, 0.92, 0.985], [0.74, 0.93, 0.985], [0.70, 0.93, 0.985], [0.14, 0.88, 0.985]],
            swing_d0: 0.035,
            swing_d0_one_strike: 0.020,
            swing_d0_two_strikes: 0.075,
            swing_width: 0.045,
            recognition_sd: 0.08,
            fooled_recognition: 0.5,
            noise_residual: 0.05,
            recognition_scale: 0.8,
            recognition_time_scale: 0.04,
            ride_bias: 0.068,
            ride_reference: 0.36,
            expect_mix: 0.0,
            misread_rate: 0.12,
            misread_factor: 2.5,

            ref_plate_speed: 86.0 * MPH,
            timing_speed_exp: 1.0,
            barrel_speed_exp: 1.0,
            tracking_residual: 0.30,
            barrel_outside_gain: 14.0,
            barrel_outside_gain2: 60.0,
            barrel_outside_offset: 0.03,
            reach_limit: 0.40,
            height_bias: 0.035,
            bat_tilt_mid: 32.0f64.to_radians(),
            bat_tilt_slope: 0.50,
            plan_angle_per_m: 0.9,
            swing_radius: 0.63,
            bat_speed_depth_gain: 0.15,
            contact_depth: 0.90,
            max_bat_angle: 1.05,
            fooled_speed_loss: 0.35,
            fooled_time: 0.045,
            partial_swing_rate: 0.08,
            partial_two_strike_gain: 2.0,
            partial_fooled_rate: 0.35,
            partial_speed_min: 0.55,
            off_end_gain: 0.10,
            reach_per_m: 0.15,
            foul_tip_catch: 0.55,
            foul_tip_angle: 0.22,
            two_strike_speed: 0.97,
            two_strike_precision: 0.88,

            batter_body_x: 0.63,
            hbp_width: 0.045,
            hbp_no_evade: 0.60,
            wild_pitch_prob: 0.13,

            catch_height: 2.6,
            catch_margin_scale: 0.18,
            catch_margin_bias: 0.0,
            redirect_delay: 0.35,
            field_height: 1.5,
            gather_time: 0.10,
            force_margin: 0.40,
            advance_left_side: 0.42,
            feed_transfer: 0.80,
            pivot_time: 0.60,
            error_ground: 0.009,
            error_fly: 0.004,
            error_throw: 0.012,
            wall_play_delay: 1.0,
            tag_time: 0.12,
            play_noise: 0.12,
            drag_jitter: 0.04,
            throw_slop: 0.50,
            throw_drag: 0.0061,
            infield_reach: 1.6,

            box_delay_right: 0.25,
            box_delay_left: 0.15,
            lead: [3.4, 4.2, 3.0],
            turn_length: 2.2,
            slide_time: 0.12,
            read_margin: -0.20,
            read_time: 0.30,
            read_hang_share: 0.18,
            advance_margin: 0.05,
            steal_margin: 0.0,
            steal_rate: 0.36,
        }
    }
}
