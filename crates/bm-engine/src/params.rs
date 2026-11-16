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
    /// Fraction of pitches that get away from the pitcher, and the multiplier on the
    /// error along the arm's swing for those.
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
    /// not compensate (0..1), for contact out in front and for contact deep in the zone,
    /// and the distance from the point the bat sweeps about to the sweet spot, m.
    pub off_end_gain: f64,
    pub off_end_late: f64,
    pub pivot_radius: f64,
    /// Further shift toward the end of the bat per radian that the barrel has come round, m.
    pub off_end_linear: f64,
    /// The shift toward the end of the bat levels off at this distance, m.
    pub off_end_max: f64,
    /// The way the bat faces: its turn per radian of the direction its sweet spot is
    /// heading (Statcast: the spray of balls in play moves 1.7 deg per inch of contact
    /// depth, the direction of the barrel's travel 1.26-1.44), how much further round
    /// it faces at the usual contact point, rad, and the swing-to-swing scatter of that.
    pub face_gain: f64,
    pub face_lead: f64,
    pub face_sd: f64,
    /// Limit of the angle the bat can face away from square at contact, rad.
    pub face_max: f64,
    /// Rolling over: the tilt of the bat falls by the fraction `roll_flatten` as the
    /// direction of the barrel's travel goes from `roll_from` to `roll_from + roll_span`, rad.
    pub roll_from: f64,
    pub roll_span: f64,
    pub roll_flatten: f64,
    /// Distance the barrel travels forward while it turns by one radian, m. Statcast's
    /// attack direction changes by 1.26 deg per inch of contact depth.
    pub swing_arc_radius: f64,
    /// How far the barrel rides up over the ball when contact is made away from where
    /// the swing is on the plane of the pitch, as a fraction of the rise of a point
    /// swept round a fixed centre in the tilted swing plane: for contact deeper than
    /// `arc_centre` (m in front of the usual contact point) and for contact further
    /// out in front than `arc_early_from`. Fitted to the launch angle by depth of
    /// contact (Statcast, depth relative to the batter's usual one: 17-20 deg from 3 in
    /// deep to 17 in out in front, -18 deg at 15-25 in deep, -3 deg beyond 17 in out
    /// in front).
    pub arc_scale: f64,
    pub arc_scale_early: f64,
    pub arc_centre: f64,
    pub arc_early_from: f64,
    /// Contact further than this from the intended depth is a clean miss, m.
    pub max_depth: f64,
    /// Depth out in front beyond which the barrel gains no more speed, m.
    pub bat_speed_depth_cap: f64,
    /// Multiplier on the timing and barrel errors of a partial swing.
    pub partial_precision: f64,
    /// Getting a piece of it: share of the swings passing over or under the ball by less
    /// than `piece_reach` (m, surface to surface) that graze it instead, and the depth
    /// of that grazing contact, m.
    pub piece_rate: f64,
    pub piece_reach: f64,
    pub piece_band: f64,
    /// Mistiming, as depth of contact, at which that share has fallen to 61 %, m.
    pub piece_depth: f64,
    /// Depth of contact at which batters miss least, m in front of the usual one.
    pub piece_centre: f64,
    /// Multiplier on that share with two strikes, when batters protect the plate
    /// (Statcast: 22.3 % of two-strike swings miss against 23.8 % with no strikes,
    /// although far more of them are at pitches outside the zone).
    pub piece_two_strikes: f64,
    /// Swing-to-swing scatter of the barrel's direction that the depth of contact does
    /// not explain, rad (Statcast: 5.6-6.0 deg), and of the tilt of the swing plane
    /// at a given pitch height, rad (Statcast: 5.6 deg).
    pub direction_sd: f64,
    pub bat_tilt_sd: f64,
    /// A swung-on ball driven into the ground within this distance of the plate is
    /// often foul (off the plate, the dirt of the box or the batter): distance, m, and probability.
    pub chop_foul_dist: f64,
    pub chop_foul_prob: f64,
    /// Where along the bat the batter means to meet the ball, m toward the hands from the sweet spot.
    pub contact_along: f64,
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
    /// Catcher's pop time to third base relative to second (the throw is shorter).
    pub pop_third_factor: f64,
    /// Attempts to steal third relative to second for the same margin.
    pub steal_third_rate: f64,
    /// Extra margin a runner wants before he tries for third, s.
    pub steal_third_margin: f64,

    // ---- bunting ----
    /// Forward speed of the bat on a sacrifice bunt: mean and SD, m/s.
    pub bunt_push: f64,
    pub bunt_push_sd: f64,
    /// Extra forward speed when bunting for a hit, m/s.
    pub bunt_hit_push: f64,
    /// Where the bunter puts the bat relative to the centre of the ball
    /// (negative = above it, to bunt the ball down), m.
    pub bunt_aim: f64,
    /// SD of the vertical bat-ball offset for an average bunter, m.
    pub bunt_offset_sd: f64,
    /// Angle at which the bunter sets the bat away from square, and its SD for an
    /// average bunter, rad. The ball keeps much of its speed along the bat, so it
    /// comes off at roughly three times this angle from the middle of the field.
    pub bunt_direction: f64,
    pub bunt_direction_sd: f64,
    /// Contact point of a bunt, m from the tip of the bat, and its SD.
    pub bunt_along: f64,
    pub bunt_along_sd: f64,
    /// A bunter offers at pitches he sees within this distance of the zone, m.
    pub bunt_offer_margin: f64,
    /// Probability that an average bunter misses a pitch of average speed outright
    /// (pulls the bat back late, loses the ball).
    pub bunt_miss: f64,
    /// Time gained out of the box by a batter who bunts for a hit, s.
    pub bunt_hit_jump: f64,
    /// Extra margin the defence wants before it goes for the lead runner on a bunt
    /// instead of taking the out at first, s.
    pub bunt_lead_margin: f64,

    // ---- hit and run ----
    /// Extra reaction time of a runner going on a hit-and-run compared with a straight steal, s.
    pub hit_and_run_jump: f64,
    /// On a hit-and-run the batter swings at any pitch he sees within this distance of the zone, m.
    pub hit_and_run_reach: f64,
}

impl Default for ModelParams {
    fn default() -> Self {
        ModelParams {
            target_sd_x: 0.13,
            target_sd_z: 0.155,
            wild_fraction: 0.037,
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
            swing_d0: 0.039,
            swing_d0_one_strike: 0.020,
            swing_d0_two_strikes: 0.075,
            swing_width: 0.045,
            recognition_sd: 0.08,
            fooled_recognition: 0.5,
            noise_residual: 0.05,
            recognition_scale: 0.8,
            recognition_time_scale: 0.181,
            ride_bias: 0.098,
            ride_reference: 0.36,
            expect_mix: 0.0,
            misread_rate: 0.12,
            misread_factor: 2.5,

            ref_plate_speed: 86.0 * MPH,
            timing_speed_exp: 3.2,
            barrel_speed_exp: 2.8,
            tracking_residual: 0.122,
            barrel_outside_gain: 6.95,
            barrel_outside_gain2: 70.0,
            barrel_outside_offset: 0.03,
            reach_limit: 0.40,
            height_bias: 0.0542,
            bat_tilt_mid: 32.0f64.to_radians(),
            bat_tilt_slope: 0.31,
            plan_angle_per_m: 0.2,
            swing_radius: 0.63,
            bat_speed_depth_gain: 0.115,
            contact_depth: 0.90,
            max_bat_angle: 1.05,
            fooled_speed_loss: 0.25,
            fooled_time: 0.045,
            partial_swing_rate: 0.05,
            partial_two_strike_gain: 1.62,
            partial_fooled_rate: 0.12,
            partial_speed_min: 0.55,
            off_end_gain: 1.0,
            off_end_late: 0.1,
            pivot_radius: 2.25,
            off_end_linear: 0.012,
            off_end_max: 0.16,
            face_gain: 1.35,
            face_lead: 0.118,
            face_sd: 0.123,
            face_max: 0.424,
            roll_from: 0.184,
            roll_span: 0.261,
            roll_flatten: 0.837,
            swing_arc_radius: 1.0,
            arc_scale: 0.048,
            arc_scale_early: 0.6,
            arc_centre: 0.11,
            arc_early_from: 0.283,
            max_depth: 0.90,
            bat_speed_depth_cap: 0.15,
            partial_precision: 1.5,
            piece_rate: 0.69,
            piece_reach: 0.0456,
            piece_band: 0.022,
            piece_depth: 0.37,
            piece_centre: 0.11,
            piece_two_strikes: 1.33,
            direction_sd: 0.115,
            bat_tilt_sd: 0.098,
            chop_foul_dist: 3.0,
            chop_foul_prob: 0.509,
            contact_along: 0.025,
            reach_per_m: 0.15,
            foul_tip_catch: 0.55,
            foul_tip_angle: 0.3,
            two_strike_speed: 0.98,
            two_strike_precision: 0.82,

            batter_body_x: 0.63,
            hbp_width: 0.063,
            hbp_no_evade: 0.80,
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
            pivot_time: 0.64,
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
            steal_margin: 0.04,
            steal_rate: 0.58,
            pop_third_factor: 0.80,
            steal_third_rate: 0.45,
            steal_third_margin: 0.10,

            bunt_push: 4.6,
            bunt_push_sd: 2.5,
            bunt_hit_push: 1.5,
            bunt_aim: -0.016,
            bunt_offset_sd: 0.026,
            bunt_direction: 0.24,
            bunt_direction_sd: 0.20,
            bunt_along: 0.11,
            bunt_along_sd: 0.03,
            bunt_offer_margin: 0.06,
            bunt_miss: 0.075,
            bunt_hit_jump: 0.08,
            bunt_lead_margin: 0.35,

            hit_and_run_jump: 0.06,
            hit_and_run_reach: 0.22,
        }
    }
}
