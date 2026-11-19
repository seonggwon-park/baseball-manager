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
    /// A pitcher who cannot hit his spots gives himself more margin. The intended
    /// location is pulled toward the middle of the zone (or pushed toward the edges) by
    /// the ratio of the typical location error (`aim_reference`, m, for a fastball) to
    /// the pitch's own, raised to `aim_adapt` (0 = everybody aims at the same spots).
    pub aim_adapt: f64,
    pub aim_reference: f64,
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
    /// How much more (or less) often each pitch type is thrown in each count than overall,
    /// [type][balls][strikes]: its share of the count's pitches over its share of all pitches
    /// (2025 Statcast sample, 123,000 pitches of the eight types). A pitcher's usage of a type
    /// is scaled by this in each count, so an average arsenal throws each type as often by
    /// count as the majors do (splitters 0.50 at 0-0 and 1.63 at 0-2, sinkers 0.42 at 0-2,
    /// curves 1.16 at 0-0 and 0.03 at 3-0). It replaces the count bias of the fastballs as a
    /// group (`fastball_bias`, now 1).
    pub count_kind: [[[f64; 3]; 4]; 8],
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
    /// Share of the difference in flight time from the expected fastball that a batter
    /// who has picked the pitch up corrects in his swing. Statcast 2025 bat tracking:
    /// against pitches 8-15 mph slower than the four-seamer, batters meet the ball
    /// 12-15 in further out in front, about half of what full fastball timing would give.
    pub timing_adjust: f64,
    /// The same for the location: share of the misjudged movement that a batter who has
    /// picked the pitch up corrects. Below 1 the batter still reads an off-speed pitch a
    /// little higher than it is, swings over it and beats it into the ground (Statcast 2025:
    /// curves, changeups and splitters leave the bat at 7-10 deg on average against 20.5
    /// for four-seamers, although their descent and spin alone would lift them).
    pub location_adjust: f64,
    /// Share of the movement misjudgement that applies when both the pitch and the
    /// expected one are fastballs. Statcast 2025: a sinker draws 13.0 % whiffs per swing
    /// from pitchers whose main fastball is a four-seamer and 14.1 % from sinker-first
    /// pitchers (four-seamers 21.7 % and 20.9 %), so taking one fastball for another costs
    /// the batter nothing measurable.
    pub fastball_misread: f64,
    /// How much easier (+) or harder (-) each pitch type is to pick up out of the hand
    /// than the batter's eye alone says, indexed by `PitchKind::index`. A curveball leaves
    /// the hand on an upward path that no fastball has; a slider or a splitter looks like
    /// a fastball for longer. Fitted to the whiff rates by pitch type. Fastballs are 0.
    pub kind_recognition: [f64; 8],
    /// Share of pitches of each type that a batter who has picked the type up out of the
    /// hand lets go whatever their location, by strikes (0, 1, 2), indexed by
    /// `PitchKind::index`. Hitters sit on the fastball early in the count and take the
    /// breaking ball they recognise; with two strikes they protect the plate. In the 2025
    /// Statcast sample, at equal distance from the zone edge and no strikes, curves deep in
    /// the zone are swung at 39.7 % of the time and four-seamers 62.5 %; with two strikes
    /// 93.1 % and 95.3 %. Four-seamers are 0.
    pub take_kind: [[f64; 3]; 8],
    /// Share of the difference between a pitch's movement and the batter's prior (the
    /// usage-weighted mean of the pitcher's pitches in the count, `Expectation::prior_x` /
    /// `prior_z`) that he has not seen when he decides to swing, horizontal and vertical,
    /// with fewer than two strikes and with two. Only the decision uses it. In the 2025
    /// Statcast sample with two strikes, a sinker on the outside edge to a batter on the
    /// pitcher's side (one that comes back over the corner) is swung at 38 % of the time and
    /// one on the inside edge 90 %; a four-seamer on the bottom edge 40 %, on the top 89 %.
    pub prior_pull: [[f64; 2]; 2],
    /// Batters adapt only partly to a fastball's "ride": share of the difference between
    /// its induced vertical movement and the generic fastball that ends up as barrel error.
    pub ride_bias: f64,
    /// Induced vertical movement (whole flight) of the generic fastball, m.
    pub ride_reference: f64,
    /// How far a four-seamer or a sinker is taken for the generic fastball rather than
    /// for what it is (0 = told apart perfectly, 1 = not at all).
    pub ride_confusion: f64,
    /// Batters time a pitcher's fastball from the fastballs they see every day and adapt
    /// to his own only in part: this share of the difference in flight time between the
    /// league's typical fastball and this pitcher's stays in the swing as a timing error,
    /// late against hard throwers and early against soft ones. 2025 Statcast bat
    /// tracking: contact is 0.13-0.30 in deeper per mph of a pitcher's average fastball,
    /// against 3 in per mph with no adaptation at all.
    pub velocity_lag: f64,
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
    /// Against a pitcher whose fastball gets there sooner than the league's typical one,
    /// the batter commits to every swing earlier and sees less of each pitch: the timing
    /// and barrel errors of every pitch type grow by (typical fastball flight time /
    /// expected flight time) to this power. 2025 Statcast sample: whiffs per swing rise
    /// 1.31 +- 0.24 points per mph of the pitcher's mean four-seamer across pitchers, by
    /// 1.1-2.0 points on every pitch type, while within a type a harder slider, curve or
    /// splitter is missed hardly more often (pitch level +0.0 to +0.5 points per mph).
    pub commit_exp: f64,
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
    /// Aim under (+) or over (-) the ball's centre by pitch type, m, on top of the
    /// batter's own aim; indexed by `PitchKind::index`, four-seamers 0.
    pub aim_kind: [f64; 8],
    /// Factor on the batter's barrel error by pitch type (four-seamers 1): how hard each
    /// type is to square up, fitted to its rate of hard contact, home runs and whiffs
    /// (`docs/physics-model.md` 5.3).
    pub barrel_kind: [f64; 8],
    /// Weight of the straight-line timing geometry in the vertical error. At 1 the barrel
    /// and the ball keep to their paths, so contact a distance `dy` from the planned point
    /// misses by `(tan descent - tan attack) dy`; at 0 the barrel meets the ball's height
    /// wherever it meets the ball.
    pub plane_mismatch: f64,
    /// Noise of the correction that takes the barrel to the ball's height on a mistimed
    /// swing, per metre of correction needed (the straight-line miss above): the larger
    /// the correction, the less precise it is (signal-dependent motor noise; Harris and
    /// Wolpert, Nature 394, 780 (1998)).
    pub mismatch_noise: f64,
    /// Turn of the ball off the bat toward the pull side, rad, velocity and spin together.
    /// An empirical correction: rigid-body collisions with the bat placed as bat tracking
    /// measures it send the ball 9-15 deg less toward the pull side than it goes (2025
    /// Statcast, both sides of the plate), and in the laboratory a square bat face sends
    /// a centred ball 7.6 deg toward the pull side (Kidokoro & Yanai, Jpn. J. Phys. Educ.
    /// Health Sport Sci. 62, 475 (2017)). No mechanism was found. 4 deg brings the side
    /// spin against direction onto Nathan's Statcast fit; more pushes it past that fit.
    pub spray_turn: f64,
    /// The AI manager shades its infield toward the pull side of left-handed (right-handed)
    /// batters whose timing bias is below this, s (earlier = pulls more). Set so that
    /// generated hitters (timing bias 2.6 +- 2.7 ms) are shaded as often as 2025 MLB
    /// infields shaded (Statcast `if_fielding_alignment`: 46 % of left-handed batters'
    /// pitches, 8 % of right-handed).
    pub shade_left_below: f64,
    pub shade_right_below: f64,
    /// An AI manager with `ManagerStyle::full_shift`, where the rules allow it (MLB before
    /// 2023), puts three infielders on the pull side of left-handed (right-handed) batters
    /// whose timing bias is below this, s.
    /// Set so that generated hitters (timing bias 2.6 +- 2.7 ms) see the full shift as often
    /// as 2022 MLB batters did (Statcast `if_fielding_alignment` "Infield shift" at the first
    /// pitch of the plate appearance, six days of 2022: 50.9 % of left-handed batters,
    /// 18.7 % of right-handed). Not fitted to any outcome.
    pub full_shift_left_below: f64,
    pub full_shift_right_below: f64,
    /// Every defence leans toward the batter's pull side, deg: corner infielders, middle
    /// infielders, outfielders. Chosen among the values tried by the batting average of
    /// ground balls by bearing in the 2025 Statcast sample: with these the pull-line side
    /// matches (.234 against .238 from 30 deg pull-side out), and larger leans open the
    /// opposite field further (already .60 against .42 beyond 30 deg there, because
    /// simulated ground balls are pulled about 6 deg less than measured ones). The
    /// outfield lean opened the opposite-field line and doubled the triples, so it is 0.
    pub lean_corner: f64,
    pub lean_middle: f64,
    pub lean_outfield: f64,
    /// Pitch of the swing plane toward the pitcher: shift from the batter's average
    /// attack angle (rad), its change per metre of pitch height above the middle of the
    /// zone, per metre of depth of contact (+ = out in front), per metre of inside
    /// location (rad/m), and by pitch type (rad, indexed by `PitchKind::index`,
    /// four-seamers 0).
    pub attack_shift: f64,
    pub attack_height: f64,
    pub attack_depth: f64,
    pub attack_inside: f64,
    pub attack_kind: [f64; 8],
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
    /// A bat that faces further round than it travels slides along the ball and puts
    /// hook spin on it; with a lead of 0.118 rad the batted balls carried about 1,000 rpm
    /// more hook spin than Nathan's Statcast fit at every direction, so there is none.
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
    /// Share of balls hit 45 deg and higher that fall in although a fielder could reach
    /// them (ramping in from 35 deg): lost in the sun or the wind, or left by two fielders
    /// to each other. Without it the engine caught nearly every ball hit 40 deg and up
    /// (hits on 0.0-0.5 % against 0.9-4.6 % in the 2025 Statcast sample when both are
    /// flown alike and compared by the distance of the nearest fielder,
    /// `tools/statcast/catch_check.py`).
    pub high_fly_drop: f64,
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
    /// Fewer than two outs, the infield back and the runner on third not forced: the chance
    /// that he breaks for the plate on a ground ball the defence plays at first, fielded by the
    /// second baseman or the shortstop (`third_go_middle`) or by anyone else (pitcher, catcher,
    /// corners: `third_go_other`), and on one it plays at second for the force (`third_go_force`,
    /// a runner on first). Set (2026-11-19) to the share of such ground-ball outs on which a run
    /// scored in the 2025 Statcast sample: with first base empty 0.61 of 75 to the middle
    /// infielders and 0.29 of 55 to the others; with a runner on first 0.67-0.83 of the outs
    /// that were not double plays (docs/tactics.md 7). The runner who breaks scores; the infield
    /// does not throw home.
    pub third_go_middle: f64,
    pub third_go_other: f64,
    pub third_go_force: f64,
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
    /// SD of the drag coefficient from one batted ball to the next, standing in for wind,
    /// weather and ball-to-ball variation, which the flight model otherwise treats as
    /// fixed. Zero: the spread of the batted balls' own spin already gives distances at a
    /// given exit velocity and launch angle the spread Statcast measures (23-30 ft at
    /// 20-35 deg against 22-31 ft), and any more lifts the home-run rate of the slower
    /// fly balls above the measured one.
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

impl ModelParams {
    /// Lean of the defence toward the batter's pull side (corners, middle, outfield), deg.
    pub fn lean(&self) -> [f64; 3] {
        [self.lean_corner, self.lean_middle, self.lean_outfield]
    }
}

impl Default for ModelParams {
    fn default() -> Self {
        ModelParams {
            target_sd_x: 0.118,
            target_sd_z: 0.157,
            aim_adapt: 1.5,
            aim_reference: 6.784 * INCH,
            wild_fraction: 0.044,
            wild_factor: 2.0,
            release_sd_side: 2.1 * INCH,
            release_sd_height: 1.3 * INCH,
            zone_intent: [[0.20, -0.30, -0.95], [0.25, -0.05, -0.70], [0.50, 0.40, -0.15], [0.82, 0.90, 0.58]],
            fastball_bias: [[1.0; 3]; 4],
            #[rustfmt::skip]
            count_kind: [[[1.026, 0.901, 1.02], [0.969, 0.879, 0.976], [1.148, 0.948, 0.98], [1.988, 1.271, 1.119]], [[1.213, 0.942, 0.423], [1.218, 0.941, 0.541], [1.408, 1.125, 0.749], [1.478, 1.544, 1.097]], [[1.144, 0.926, 0.53], [1.25, 1.028, 0.575], [1.55, 1.252, 0.737], [0.76, 1.446, 1.03]], [[0.967, 1.014, 1.138], [0.982, 1.012, 1.097], [0.875, 1.013, 1.058], [0.221, 0.737, 1.015]], [[0.961, 1.121, 1.472], [0.799, 0.968, 1.307], [0.516, 0.813, 1.137], [0.102, 0.408, 0.835]], [[1.158, 0.931, 1.328], [0.762, 0.994, 1.294], [0.41, 0.795, 1.127], [0.034, 0.262, 0.731]], [[0.629, 1.278, 1.173], [1.0, 1.362, 1.292], [0.687, 1.107, 1.193], [0.135, 0.523, 0.816]], [[0.496, 1.319, 1.628], [0.763, 1.268, 1.657], [0.339, 0.823, 1.508], [0.049, 0.43, 0.792]]],
            fatigue_speed: 0.0004,
            fatigue_command: 0.004,

            ump_bias: 0.3 * INCH,
            ump_scale: 0.93 * INCH,

            swing_base: [[0.60607, 0.93393, 0.985], [0.71325, 0.96259, 0.985], [0.8093, 0.94741, 0.985], [0.14157, 0.8368, 0.985]],
            swing_d0: 0.030229,
            swing_d0_one_strike: 0.015674,
            swing_d0_two_strikes: 0.062162,
            swing_width: 0.05031,
            recognition_sd: 0.08,
            fooled_recognition: 0.5,
            noise_residual: 0.098279,
            recognition_scale: 0.22369,
            recognition_time_scale: 0.14501,
            timing_adjust: 0.68402,
            location_adjust: 0.98853,
            fastball_misread: 0.0,
            kind_recognition: [0.0, 0.0, 0.010371, -0.079913, 0.07369, 0.29486, -0.023278, -0.33115],
            take_kind: [
                [0.0, 0.0, 0.0],
                [0.19885, 0.10494, 0.11863],
                [-0.06985, 0.044837, -0.0046047],
                [0.081739, 0.023119, -0.017762],
                [0.27526, 0.10117, -0.0059464],
                [0.40885, 0.10145, -0.025499],
                [-0.060513, -0.050318, -0.015074],
                [0.065229, 0.030025, -0.046005],
            ],
            prior_pull: [[0.28366, 0.22544], [0.15755, 0.17194]],
            ride_bias: 0.085453,
            ride_reference: 0.375,
            ride_confusion: 0.6,
            velocity_lag: 0.10,
            expect_mix: 0.0,
            misread_rate: 0.12,
            misread_factor: 2.5,

            ref_plate_speed: 86.0 * MPH,
            timing_speed_exp: 3.9675,
            barrel_speed_exp: 3.7199,
            commit_exp: 4.8229,
            tracking_residual: 0.21647,
            barrel_outside_gain: 7.2635,
            barrel_outside_gain2: 70.0,
            barrel_outside_offset: 0.03,
            reach_limit: 0.40,
            height_bias: 0.076376,
            aim_kind: [0.0, -0.032548, -0.018124, -0.014379, 0.0061355, -0.031321, -0.028653, -0.013767],
            barrel_kind: [1.0, 0.99257, 1.45, 1.35, 0.95, 0.89343, 1.1298, 1.2133],
            plane_mismatch: 0.0,
            mismatch_noise: 1.7742,
            spray_turn: 0.0698,
            shade_left_below: 0.00233,
            shade_right_below: -0.00119,
            full_shift_left_below: 0.00266,
            full_shift_right_below: 0.00020,
            lean_corner: 3.0,
            lean_middle: 5.0,
            lean_outfield: 0.0,
            attack_shift: -0.034162,
            attack_height: 0.1898,
            attack_depth: 0.20017,
            attack_inside: -0.2150,
            attack_kind: [0.0, -0.0168, -0.0036, -0.02491, -0.02615, -0.03656, -0.01897, -0.01948],
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
            off_end_gain: 0.67564,
            off_end_late: 0.1,
            pivot_radius: 2.25,
            off_end_linear: 0.015232,
            off_end_max: 0.17847,
            face_gain: 1.3212,
            face_lead: 0.0,
            face_sd: 0.108,
            face_max: 0.424,
            roll_from: 0.184,
            roll_span: 0.261,
            roll_flatten: 0.757,
            swing_arc_radius: 1.0,
            arc_scale: 0.051319,
            arc_scale_early: 0.78898,
            arc_centre: 0.068661,
            arc_early_from: 0.29812,
            max_depth: 0.90,
            bat_speed_depth_cap: 0.15,
            partial_precision: 1.5,
            piece_rate: 0.53993,
            piece_reach: 0.046436,
            piece_band: 0.022,
            piece_depth: 0.37,
            piece_centre: 0.11,
            piece_two_strikes: 1.33,
            direction_sd: 0.115,
            bat_tilt_sd: 0.098,
            chop_foul_dist: 3.0,
            chop_foul_prob: 0.509,
            contact_along: 0.021778,
            reach_per_m: 0.15,
            foul_tip_catch: 0.55,
            foul_tip_angle: 0.3,
            two_strike_speed: 0.98,
            two_strike_precision: 0.78,

            batter_body_x: 0.63,
            hbp_width: 0.063,
            hbp_no_evade: 0.90,
            wild_pitch_prob: 0.13,

            catch_height: 2.6,
            catch_margin_scale: 0.18,
            high_fly_drop: 0.02,
            catch_margin_bias: 0.0,
            redirect_delay: 0.35,
            field_height: 1.5,
            gather_time: 0.10,
            force_margin: 0.40,
            third_go_middle: 0.61,
            third_go_other: 0.29,
            third_go_force: 0.80,
            advance_left_side: 0.42,
            feed_transfer: 0.80,
            pivot_time: 0.64,
            error_ground: 0.009,
            error_fly: 0.004,
            error_throw: 0.012,
            wall_play_delay: 1.0,
            tag_time: 0.12,
            play_noise: 0.12,
            drag_jitter: 0.0,
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
