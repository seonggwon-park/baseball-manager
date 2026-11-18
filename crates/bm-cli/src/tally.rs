//! Aggregation of simulated games into the same statistics that were measured
//! from Statcast and the MLB Stats API, for side-by-side validation.

use bm_engine::{Approach, BattedInfo, GameEvent, Hand, Miss, Observer, PaEvent, PaOutcome, PitchEvent, PitchKind, PitchResult, PlayKind};
use bm_physics::units::{FT, INCH, MPH, RPM};
use bm_physics::FlightModel;

pub const DIST_EDGES: [f64; 17] = [-12.0, -8.0, -6.0, -4.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 14.0, 20.0, 40.0];
pub const CALL_EDGES: [f64; 16] = [-12.0, -6.0, -4.0, -3.0, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0];
pub const NB: usize = DIST_EDGES.len() - 1;
pub const NC: usize = CALL_EDGES.len() - 1;

fn bin(edges: &[f64], x: f64) -> Option<usize> {
    if x < edges[0] || x >= edges[edges.len() - 1] {
        return None;
    }
    Some(edges.partition_point(|&e| e <= x) - 1)
}

#[derive(Clone, Copy, Default)]
pub struct Mean {
    pub n: f64,
    pub sum: f64,
    pub sq: f64,
}

impl Mean {
    pub fn add(&mut self, x: f64) {
        self.n += 1.0;
        self.sum += x;
        self.sq += x * x;
    }
    pub fn mean(&self) -> f64 {
        if self.n > 0.0 {
            self.sum / self.n
        } else {
            f64::NAN
        }
    }
    pub fn sd(&self) -> f64 {
        if self.n > 1.0 {
            (self.sq / self.n - self.mean().powi(2)).max(0.0).sqrt()
        } else {
            f64::NAN
        }
    }
    fn merge(&mut self, o: &Mean) {
        self.n += o.n;
        self.sum += o.sum;
        self.sq += o.sq;
    }
}

/// Lower edges of the launch-angle classes used for the collision-efficiency table, deg.
pub const EFF_LA_EDGES: [f64; 8] = [-90.0, -20.0, 0.0, 10.0, 25.0, 40.0, 60.0, 90.0];
/// Bat-speed bands for the collision-efficiency table, mph.
pub const EFF_BAT_EDGES: [f64; 7] = [40.0, 60.0, 66.0, 70.0, 74.0, 78.0, 90.0];
const EFF_BIN: f64 = 0.005;

/// Exit velocity relative to the maximum Statcast's "squared-up" model allows for the
/// swing and the pitch: `1.23 * bat speed + 0.23 * pitch speed at the plate`.
#[derive(Clone)]
pub struct Efficiency {
    pub ratio: Mean,
    pub ev: Mean,
    pub bat: Mean,
    pub la: Mean,
    pub hist: Vec<f64>,
}

impl Default for Efficiency {
    fn default() -> Self {
        Efficiency { ratio: Mean::default(), ev: Mean::default(), bat: Mean::default(), la: Mean::default(), hist: vec![0.0; 280] }
    }
}

impl Efficiency {
    fn add(&mut self, ratio: f64, ev: f64, bat: f64, la: f64) {
        self.ratio.add(ratio);
        self.ev.add(ev);
        self.bat.add(bat);
        self.la.add(la);
        let i = ((ratio / EFF_BIN) as usize).min(self.hist.len() - 1);
        self.hist[i] += 1.0;
    }
    fn merge(&mut self, o: &Efficiency) {
        self.ratio.merge(&o.ratio);
        self.ev.merge(&o.ev);
        self.bat.merge(&o.bat);
        self.la.merge(&o.la);
        for (a, b) in self.hist.iter_mut().zip(&o.hist) {
            *a += *b;
        }
    }
    pub fn percentile(&self, q: f64) -> f64 {
        Tally::percentile(&self.hist, 0.0, EFF_BIN, q)
    }
    /// Share of balls at or above `x`.
    pub fn share_at_least(&self, x: f64) -> f64 {
        let i = (x / EFF_BIN).round() as usize;
        self.hist[i.min(self.hist.len())..].iter().sum::<f64>() / self.ratio.n.max(1.0)
    }
}

/// Running sums for a straight-line fit of y on x.
#[derive(Clone, Copy, Default)]
pub struct Fit {
    pub n: f64,
    sx: f64,
    sy: f64,
    sxx: f64,
    sxy: f64,
    syy: f64,
}

impl Fit {
    pub fn add(&mut self, x: f64, y: f64) {
        self.n += 1.0;
        self.sx += x;
        self.sy += y;
        self.sxx += x * x;
        self.sxy += x * y;
        self.syy += y * y;
    }
    fn merge(&mut self, o: &Fit) {
        self.n += o.n;
        self.sx += o.sx;
        self.sy += o.sy;
        self.sxx += o.sxx;
        self.sxy += o.sxy;
        self.syy += o.syy;
    }
    pub fn slope(&self) -> f64 {
        (self.n * self.sxy - self.sx * self.sy) / (self.n * self.sxx - self.sx * self.sx)
    }
    pub fn r(&self) -> f64 {
        (self.n * self.sxy - self.sx * self.sy) / ((self.n * self.sxx - self.sx * self.sx) * (self.n * self.syy - self.sy * self.sy)).sqrt()
    }
}

/// Edges of the contact-depth bins, inches ahead of the batter's usual contact point.
pub const DEPTH_EDGES: [f64; 11] = [-60.0, -25.0, -15.0, -8.0, -3.0, 1.0, 5.0, 10.0, 17.0, 25.0, 60.0];
pub const ND: usize = DEPTH_EDGES.len() - 1;

/// Contact depth of the full swings at one type of pitch: mean and spread, and a
/// histogram in steps of 2.5 in from -40 to +40 in of [swings, misses].
#[derive(Clone, Copy, Default)]
pub struct KindDepth {
    pub depth: Mean,
    pub hist: [[f64; 2]; 32],
}

/// Swing geometry (full swings, 60 mph and up, bunts excluded), for comparison with
/// Statcast bat tracking. Outcome index: 0 = whiff, 1 = foul, 2 = in play.
#[derive(Clone, Copy, Default)]
pub struct SwingGeo {
    /// Timing by pitch type.
    pub kind_depth: [KindDepth; 8],
    /// All swings and those under 60 mph, by outcome.
    pub all: [f64; 3],
    pub slow: [f64; 3],
    pub speed: [Mean; 3],
    /// Contact depth, inches.
    pub depth: [Mean; 3],
    /// Direction of the barrel's travel, degrees toward the pull side.
    pub direction: [Mean; 3],
    pub attack: Mean,
    /// Attack angle of swings and misses, fouls and balls in play, deg.
    pub attack_by: [Mean; 3],
    /// Fits on balls in play: bat speed, attack angle and direction on depth; pull-side
    /// spray on direction; depth, direction and spray on the inside location (in).
    pub fits: [Fit; 7],
    /// Outcomes by depth bin, and for balls in play: bat speed, attack angle, direction,
    /// launch angle, exit velocity, efficiency, pull-side spray.
    pub by_depth: [[f64; 3]; ND],
    pub depth_bip: [[Mean; 8]; ND],
    /// By launch-angle class (GB, LD, FB, PU): pull-side spray, direction, depth, attack angle.
    pub by_la: [[Mean; 4]; 4],
    pub spray: Mean,
    /// Well-timed swings (within 4 in of the usual depth and 5 deg of the usual
    /// direction): all, misses, caught tips; and by elevation of the ball off the bat:
    /// fouls and balls in play with their exit velocity.
    /// All swings by strikes: miss, caught tip, foul, in play, swings under 60 mph; and the bat speed.
    pub by_strikes: [[f64; 5]; 3],
    pub speed_by_strikes: [Mean; 3],
    pub centred: [f64; 3],
    /// Well-timed swings at pitches well inside the zone, on its edge and outside it: miss, foul, in play.
    pub centred_zone: [[f64; 3]; 3],
    /// The same three classes of location for all full swings, by depth bin.
    pub depth_zone: [[[f64; 3]; 3]; ND],
    /// Well-timed misses by cause.
    pub centred_miss: [f64; 6],
    pub centred_foul: [Mean; NE],
    pub centred_bip: [Mean; NE],
}

/// Edges of the elevation bins of well-timed contact, deg.
pub const ELEV_EDGES: [f64; 15] = [-90.0, -60.0, -40.0, -25.0, -10.0, 0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 91.0];
pub const NE: usize = ELEV_EDGES.len() - 1;

impl SwingGeo {
    fn merge(&mut self, o: &SwingGeo) {
        for i in 0..3 {
            self.all[i] += o.all[i];
            self.slow[i] += o.slow[i];
            self.speed[i].merge(&o.speed[i]);
            self.depth[i].merge(&o.depth[i]);
            self.direction[i].merge(&o.direction[i]);
        }
        self.attack.merge(&o.attack);
        for i in 0..3 {
            self.attack_by[i].merge(&o.attack_by[i]);
        }
        self.spray.merge(&o.spray);
        for i in 0..3 {
            for k in 0..5 {
                self.by_strikes[i][k] += o.by_strikes[i][k];
            }
            self.speed_by_strikes[i].merge(&o.speed_by_strikes[i]);
            self.centred[i] += o.centred[i];
            for z in 0..3 {
                self.centred_zone[z][i] += o.centred_zone[z][i];
                for d in 0..ND {
                    self.depth_zone[d][z][i] += o.depth_zone[d][z][i];
                }
            }
        }
        for i in 0..6 {
            self.centred_miss[i] += o.centred_miss[i];
        }
        for (a, b) in self.kind_depth.iter_mut().zip(&o.kind_depth) {
            a.depth.merge(&b.depth);
            for (x, y) in a.hist.iter_mut().zip(&b.hist) {
                add_arr(x, y);
            }
        }
        for i in 0..NE {
            self.centred_foul[i].merge(&o.centred_foul[i]);
            self.centred_bip[i].merge(&o.centred_bip[i]);
        }
        for (a, b) in self.fits.iter_mut().zip(&o.fits) {
            a.merge(b);
        }
        for i in 0..ND {
            for k in 0..3 {
                self.by_depth[i][k] += o.by_depth[i][k];
            }
            for k in 0..8 {
                self.depth_bip[i][k].merge(&o.depth_bip[i][k]);
            }
        }
        for i in 0..4 {
            for k in 0..4 {
                self.by_la[i][k].merge(&o.by_la[i][k]);
            }
        }
    }
}

/// Counts of the things managers call for.
#[derive(Clone, Copy, Default)]
pub struct Tactics {
    pub sac_bunts: f64,
    pub intentional_walks: f64,
    /// Bunt attempts: pulled back, missed, fouled off, put in play.
    pub bunt_pitches: [f64; 4],
    pub bunt_ev: Mean,
    pub bunt_la: Mean,
    pub bunt_popped: f64,
    pub bunt_fielder: [f64; 9],
    /// Bunts in play with a runner on first and fewer than two outs:
    /// n, sacrifice, hit, batter out without an advance, lead runner out, error.
    pub sac_situation: [f64; 6],
    /// Bunts in play with the bases empty: n, hits.
    pub empty_situation: [f64; 2],
    pub hit_and_run_pitches: f64,
    pub pinch_hitters: f64,
    pub pinch_runners: f64,
    /// Steals of third: attempts, successes.
    pub steals_of_third: [f64; 2],
    /// The plate appearance in progress ended with a bunt in play.
    bunt_in_play: bool,
}

impl Tactics {
    fn merge(&mut self, o: &Tactics) {
        self.sac_bunts += o.sac_bunts;
        self.intentional_walks += o.intentional_walks;
        self.bunt_popped += o.bunt_popped;
        self.hit_and_run_pitches += o.hit_and_run_pitches;
        self.pinch_hitters += o.pinch_hitters;
        self.pinch_runners += o.pinch_runners;
        self.bunt_ev.merge(&o.bunt_ev);
        self.bunt_la.merge(&o.bunt_la);
        let add = |a: &mut [f64], b: &[f64]| a.iter_mut().zip(b).for_each(|(x, y)| *x += *y);
        add(&mut self.bunt_pitches, &o.bunt_pitches);
        add(&mut self.bunt_fielder, &o.bunt_fielder);
        add(&mut self.sac_situation, &o.sac_situation);
        add(&mut self.empty_situation, &o.empty_situation);
        add(&mut self.steals_of_third, &o.steals_of_third);
    }
}

#[derive(Clone, Copy, Default)]
pub struct BbType {
    pub n: f64,
    pub ev: Mean,
    pub la: Mean,
    pub hits: f64,
    pub singles: f64,
    pub doubles: f64,
    pub triples: f64,
    pub hr: f64,
    pub pull: Mean,
}

#[derive(Clone, Copy, Default)]
pub struct LaBin {
    pub n: f64,
    pub ev: Mean,
    pub hits: f64,
    pub hr: f64,
    pub dist: Mean,
}

#[derive(Clone, Copy, Default)]
pub struct KindStat {
    pub n: f64,
    pub speed: Mean,
    pub swings: f64,
    pub whiffs: f64,
    pub zone: f64,
    pub out_swings: f64,
    pub out_n: f64,
    pub ev: Mean,
    pub la: Mean,
    pub gb: f64,
    pub bip: f64,
    /// Swings at pitches in the zone, and the misses among them; called strikes.
    pub zone_swings: f64,
    pub zone_whiffs: f64,
    pub called: f64,
    /// Swings and misses by cause: early, late, off the end, jammed, over, under.
    pub miss: [f64; 6],
    /// Swings by batter side: same side as the pitcher [swings, misses], opposite side.
    pub side: [[f64; 2]; 2],
    /// Induced vertical movement over the whole flight (in), arm-side movement (in), plate height (ft).
    pub rise: Mean,
    pub arm_side: Mean,
    pub height: Mean,
    /// Plate location toward the outside edge (ft), and [pitches, pitches in the zone],
    /// against same-side and opposite-side batters.
    pub side_x: [Mean; 2],
    pub side_zone: [[f64; 2]; 2],
    /// [pitches, swings] by strikes (0, 1, 2) and signed distance from the zone edge
    /// (`KIND_DIST_EDGES`, in), for swing rates by type at equal location (`bm validate --json`).
    pub dist: [[[f64; 2]; KIND_DIST_EDGES.len() - 1]; 3],
}

/// Bins of signed distance from the zone edge (in; negative inside) for `KindStat::dist`.
pub const KIND_DIST_EDGES: [f64; 9] = [-24.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 8.0, 40.0];

#[derive(Clone)]
pub struct Tally {
    /// Flight model used to project the landing point of balls that left the park.
    pub flight: Option<FlightModel>,
    /// The park of the current game, for distances beyond its fence.
    pub park: Option<bm_physics::Park>,
    pub games: f64,
    pub team_games: f64,
    pub innings: f64,
    pub runs: f64,
    pub ties: f64,
    // pitches
    pub pitches: f64,
    pub res: [f64; 7],
    /// Swinging strikes by cause: early, late, off the end, jammed, over, under.
    pub miss: [f64; 6],
    pub zone: f64,
    pub swings: f64,
    pub z_swings: f64,
    pub o_swings: f64,
    pub z_n: f64,
    pub o_n: f64,
    pub contact: f64,
    pub z_contact: f64,
    pub o_contact: f64,
    pub fouls_of_contact: f64,
    pub takes_z: f64,
    pub takes_o: f64,
    pub cs_z: f64,
    pub cs_o: f64,
    pub count_n: [[f64; 3]; 4],
    pub count_zone: [[f64; 3]; 4],
    pub count_swing: [[f64; 3]; 4],
    pub count_zswing: [[f64; 3]; 4],
    pub count_oswing: [[f64; 3]; 4],
    pub count_zn: [[f64; 3]; 4],
    pub count_fast: [[f64; 3]; 4],
    pub swing_dist_n: [[f64; NB]; 4],
    pub swing_dist_s: [[f64; NB]; 4],
    pub contact_dist_n: [f64; NB],
    pub contact_dist_c: [f64; NB],
    pub call_n: [f64; NC],
    pub call_s: [f64; NC],
    pub kinds: [KindStat; 8],
    /// Pitches by type, count before the pitch (balls * 3 + strikes) and result
    /// (`PITCH_CLASSES`), to value each pitch type with a count-by-result table of run values
    /// (`bm validate --json`).
    pub pitch_classes: Vec<[[f64; PITCH_CLASSES.len()]; 12]>,
    /// The same for the pitches in the strike zone only.
    pub pitch_classes_zone: Vec<[[f64; PITCH_CLASSES.len()]; 12]>,
    pub bat_speed_all: Mean,
    pub bat_speed_bip: Mean,
    pub eff_all: Efficiency,
    pub eff_la: Vec<Efficiency>,
    pub eff_bat: Vec<Efficiency>,
    pub whiff_fb_speed: [[f64; 2]; 8],
    // batted balls
    pub bip: f64,
    pub ev: Mean,
    pub la: Mean,
    pub ev_hist: Vec<f64>,
    pub la_hist: Vec<f64>,
    pub hard: f64,
    pub sweet: f64,
    pub barrel: f64,
    pub types: [BbType; 4],
    pub la_bins: [LaBin; 11],
    pub hr_ev_n: [f64; 7],
    pub hr_ev_hr: [f64; 7],
    pub hr_ev_dist: [Mean; 7],
    /// Launch angle 15 deg and up: [line, alley, centre][20-ft bin from 300 ft] = (balls, home runs)
    /// by projected distance and the bearing of the projected landing point.
    pub hr_dist: [[(f64, f64); 8]; 3],
    /// The same balls by margin beyond the park's fence at the landing bearing:
    /// [sector][10-ft bin from -40 ft] = (balls, home runs). Filled by the caller.
    pub hr_margin: [[(f64, f64); 9]; 3],
    /// Home-run rate by launch angle (10-15, 15-20, 20-25, 25-30, 30-35, 35-40, 40-50 deg) and
    /// exit velocity (90-95, 95-100, 100-105, 105-110, 110-120 mph): (balls, home runs).
    pub hr_cell: [[(f64, f64); 5]; 7],
    /// Home runs that did not leave the park on the fly: (inside the park, all).
    pub hr_inside: (f64, f64),
    pub spray_pull: [Mean; 2],
    pub pull_n: [[f64; 3]; 2],
    pub spin_fit: [f64; 5],
    /// Sums for the regression of backspin on launch angle and pull-side direction (LA 0-40 deg):
    /// n, la, pull, y, la^2, pull^2, la*pull, la*y, pull*y.
    pub spin_fit3: [f64; 9],
    /// Regression sums of "hook" sidespin (toward the pull-side line) on the launch
    /// direction measured toward the pull side.
    pub side_fit: [f64; 5],
    pub base_state: [f64; 8],
    pub third_lt2: f64,
    pub first_lt2: f64,
    pub sf_opp: f64,
    /// Runner advancement: [opportunities, successes] for
    /// 0: first to third on a single, 1: second scores on a single, 2: first scores on a double,
    /// 3: second to third on a ground out, 4: second to third on an air out,
    /// 5..=7: ground ball with a runner on first and fewer than two outs -> double play / force out / out at first.
    pub adv: [[f64; 2]; 8],
    /// Outs on balls in play by the position of the fielder who made the play, for
    /// ground balls / line drives / fly balls / pop-ups.
    pub outs_by_fielder: [[f64; 9]; 4],
    pub la_height_n: [f64; 6],
    pub la_height_la: [f64; 6],
    pub whiff_height: [[[f64; 2]; 8]; 3],
    // plate appearances
    pub pa: f64,
    pub so: f64,
    pub bb: f64,
    pub hbp: f64,
    pub singles: f64,
    pub doubles: f64,
    pub triples: f64,
    pub hr: f64,
    pub roe: f64,
    pub gidp: f64,
    pub sf: f64,
    pub outs_in_play: f64,
    pub pa_pitches: f64,
    pub tactics: Tactics,
    pub geo: SwingGeo,
    pub sb: f64,
    pub cs: f64,
    pub wp: f64,
    pub pitchers_used: f64,
    pub runner_outs: f64,
    pub first_to_third_n: f64,
    pub first_to_third: f64,
    pub second_home_n: f64,
    pub second_home: f64,
    pub dp_opp_gb: f64,
    pub dp_made: f64,
}

impl Default for Tally {
    fn default() -> Self {
        Tally {
            flight: None,
            park: None,
            games: 0.0,
            team_games: 0.0,
            innings: 0.0,
            runs: 0.0,
            ties: 0.0,
            pitches: 0.0,
            res: [0.0; 7],
            miss: [0.0; 6],
            zone: 0.0,
            swings: 0.0,
            z_swings: 0.0,
            o_swings: 0.0,
            z_n: 0.0,
            o_n: 0.0,
            contact: 0.0,
            z_contact: 0.0,
            o_contact: 0.0,
            fouls_of_contact: 0.0,
            takes_z: 0.0,
            takes_o: 0.0,
            cs_z: 0.0,
            cs_o: 0.0,
            count_n: [[0.0; 3]; 4],
            count_zone: [[0.0; 3]; 4],
            count_swing: [[0.0; 3]; 4],
            count_zswing: [[0.0; 3]; 4],
            count_oswing: [[0.0; 3]; 4],
            count_zn: [[0.0; 3]; 4],
            count_fast: [[0.0; 3]; 4],
            swing_dist_n: [[0.0; NB]; 4],
            swing_dist_s: [[0.0; NB]; 4],
            contact_dist_n: [0.0; NB],
            contact_dist_c: [0.0; NB],
            call_n: [0.0; NC],
            call_s: [0.0; NC],
            kinds: [KindStat::default(); 8],
            pitch_classes: vec![[[0.0; PITCH_CLASSES.len()]; 12]; 8],
            pitch_classes_zone: vec![[[0.0; PITCH_CLASSES.len()]; 12]; 8],
            bat_speed_all: Mean::default(),
            bat_speed_bip: Mean::default(),
            eff_all: Efficiency::default(),
            eff_la: vec![Efficiency::default(); EFF_LA_EDGES.len() - 1],
            eff_bat: vec![Efficiency::default(); EFF_BAT_EDGES.len() - 1],
            whiff_fb_speed: [[0.0; 2]; 8],
            bip: 0.0,
            ev: Mean::default(),
            la: Mean::default(),
            ev_hist: vec![0.0; 130],
            la_hist: vec![0.0; 181],
            hard: 0.0,
            sweet: 0.0,
            barrel: 0.0,
            types: [BbType::default(); 4],
            la_bins: [LaBin::default(); 11],
            hr_ev_n: [0.0; 7],
            hr_ev_hr: [0.0; 7],
            hr_ev_dist: [Mean::default(); 7],
            hr_dist: [[(0.0, 0.0); 8]; 3],
            hr_margin: [[(0.0, 0.0); 9]; 3],
            hr_inside: (0.0, 0.0),
            hr_cell: [[(0.0, 0.0); 5]; 7],
            spray_pull: [Mean::default(); 2],
            pull_n: [[0.0; 3]; 2],
            spin_fit: [0.0; 5],
            spin_fit3: [0.0; 9],
            side_fit: [0.0; 5],
            base_state: [0.0; 8],
            third_lt2: 0.0,
            first_lt2: 0.0,
            sf_opp: 0.0,
            adv: [[0.0; 2]; 8],
            outs_by_fielder: [[0.0; 9]; 4],
            la_height_n: [0.0; 6],
            la_height_la: [0.0; 6],
            whiff_height: [[[0.0; 2]; 8]; 3],
            pa: 0.0,
            so: 0.0,
            bb: 0.0,
            hbp: 0.0,
            singles: 0.0,
            doubles: 0.0,
            triples: 0.0,
            hr: 0.0,
            roe: 0.0,
            gidp: 0.0,
            sf: 0.0,
            outs_in_play: 0.0,
            pa_pitches: 0.0,
            tactics: Tactics::default(),
            geo: SwingGeo::default(),
            sb: 0.0,
            cs: 0.0,
            wp: 0.0,
            pitchers_used: 0.0,
            runner_outs: 0.0,
            first_to_third_n: 0.0,
            first_to_third: 0.0,
            second_home_n: 0.0,
            second_home: 0.0,
            dp_opp_gb: 0.0,
            dp_made: 0.0,
        }
    }
}

fn add_arr<const N: usize>(a: &mut [f64; N], b: &[f64; N]) {
    for (x, y) in a.iter_mut().zip(b) {
        *x += *y;
    }
}
fn add_grid<const N: usize, const M: usize>(a: &mut [[f64; N]; M], b: &[[f64; N]; M]) {
    for (x, y) in a.iter_mut().zip(b) {
        add_arr(x, y);
    }
}

/// Statcast's barrel definition: at least 98 mph, with a launch-angle window that
/// starts at 26-30 degrees and widens as the exit velocity rises (to 8-50 degrees at 116 mph).
/// What a pitch came to, for its run value: the same classes as Statcast's descriptions
/// and events give.
pub const PITCH_CLASSES: [&str; 12] = ["ball", "called", "swinging", "foul_tip", "foul", "hbp", "out", "single", "double", "triple", "home_run", "error"];

/// The class (`PITCH_CLASSES`) of a pitch.
pub fn pitch_class(e: &PitchEvent) -> usize {
    match e.result {
        PitchResult::Ball => 0,
        PitchResult::CalledStrike => 1,
        PitchResult::SwingingStrike => 2,
        PitchResult::FoulTip => 3,
        PitchResult::Foul => 4,
        PitchResult::HitByPitch => 5,
        PitchResult::InPlay => match e.play.map(|p| p.kind) {
            Some(PlayKind::Single) => 7,
            Some(PlayKind::Double | PlayKind::GroundRuleDouble) => 8,
            Some(PlayKind::Triple) => 9,
            Some(PlayKind::HomeRun) => 10,
            Some(PlayKind::Error) => 11,
            Some(PlayKind::Foul) => 4,
            _ => 6,
        },
    }
}

pub fn is_barrel(ev_mph: f64, la_deg: f64) -> bool {
    if ev_mph < 98.0 {
        return false;
    }
    let k = ((ev_mph - 98.0) / 18.0).min(1.0);
    la_deg >= 26.0 - 18.0 * k && la_deg <= 30.0 + 20.0 * k
}

pub fn bb_type(la_deg: f64) -> usize {
    if la_deg < 10.0 {
        0
    } else if la_deg < 25.0 {
        1
    } else if la_deg < 50.0 {
        2
    } else {
        3
    }
}

impl Tally {
    pub fn merge(&mut self, o: &Tally) {
        for (a, b) in self.pitch_classes.iter_mut().zip(&o.pitch_classes).chain(self.pitch_classes_zone.iter_mut().zip(&o.pitch_classes_zone)) {
            for (x, y) in a.iter_mut().zip(b) {
                add_arr(x, y);
            }
        }
        macro_rules! addf { ($($f:ident),*) => { $( self.$f += o.$f; )* } }
        addf!(
            games,
            team_games,
            innings,
            runs,
            ties,
            pitches,
            zone,
            swings,
            z_swings,
            o_swings,
            z_n,
            o_n,
            contact,
            z_contact,
            o_contact,
            fouls_of_contact,
            takes_z,
            takes_o,
            cs_z,
            cs_o,
            bip,
            hard,
            sweet,
            barrel,
            pa,
            so,
            bb,
            hbp,
            singles,
            doubles,
            triples,
            hr,
            roe,
            gidp,
            sf,
            outs_in_play,
            pa_pitches,
            sb,
            cs,
            wp,
            pitchers_used,
            runner_outs,
            first_to_third_n,
            first_to_third,
            second_home_n,
            second_home,
            dp_opp_gb,
            dp_made
        );
        add_arr(&mut self.res, &o.res);
        add_arr(&mut self.miss, &o.miss);
        add_grid(&mut self.count_n, &o.count_n);
        add_grid(&mut self.count_zone, &o.count_zone);
        add_grid(&mut self.count_swing, &o.count_swing);
        add_grid(&mut self.count_zswing, &o.count_zswing);
        add_grid(&mut self.count_oswing, &o.count_oswing);
        add_grid(&mut self.count_zn, &o.count_zn);
        add_grid(&mut self.count_fast, &o.count_fast);
        add_grid(&mut self.swing_dist_n, &o.swing_dist_n);
        add_grid(&mut self.swing_dist_s, &o.swing_dist_s);
        add_arr(&mut self.contact_dist_n, &o.contact_dist_n);
        add_arr(&mut self.contact_dist_c, &o.contact_dist_c);
        add_arr(&mut self.call_n, &o.call_n);
        add_arr(&mut self.call_s, &o.call_s);
        add_grid(&mut self.whiff_fb_speed, &o.whiff_fb_speed);
        add_arr(&mut self.hr_ev_n, &o.hr_ev_n);
        add_arr(&mut self.hr_ev_hr, &o.hr_ev_hr);
        add_grid(&mut self.pull_n, &o.pull_n);
        add_arr(&mut self.spin_fit, &o.spin_fit);
        add_arr(&mut self.spin_fit3, &o.spin_fit3);
        add_arr(&mut self.side_fit, &o.side_fit);
        add_arr(&mut self.base_state, &o.base_state);
        self.third_lt2 += o.third_lt2;
        self.first_lt2 += o.first_lt2;
        self.sf_opp += o.sf_opp;
        add_grid(&mut self.adv, &o.adv);
        add_grid(&mut self.outs_by_fielder, &o.outs_by_fielder);
        add_arr(&mut self.la_height_n, &o.la_height_n);
        add_arr(&mut self.la_height_la, &o.la_height_la);
        for (a, b) in self.whiff_height.iter_mut().zip(&o.whiff_height) {
            add_grid(a, b);
        }
        for (a, b) in self.ev_hist.iter_mut().zip(&o.ev_hist) {
            *a += *b;
        }
        for (a, b) in self.la_hist.iter_mut().zip(&o.la_hist) {
            *a += *b;
        }
        self.ev.merge(&o.ev);
        self.la.merge(&o.la);
        self.bat_speed_all.merge(&o.bat_speed_all);
        self.bat_speed_bip.merge(&o.bat_speed_bip);
        self.tactics.merge(&o.tactics);
        self.geo.merge(&o.geo);
        self.eff_all.merge(&o.eff_all);
        for (a, b) in self.eff_la.iter_mut().zip(&o.eff_la) {
            a.merge(b);
        }
        for (a, b) in self.eff_bat.iter_mut().zip(&o.eff_bat) {
            a.merge(b);
        }
        for (a, b) in self.hr_cell.iter_mut().flatten().zip(o.hr_cell.iter().flatten()) {
            a.0 += b.0;
            a.1 += b.1;
        }
        self.hr_inside.0 += o.hr_inside.0;
        self.hr_inside.1 += o.hr_inside.1;
        for (a, b) in self.hr_margin.iter_mut().flatten().zip(o.hr_margin.iter().flatten()) {
            a.0 += b.0;
            a.1 += b.1;
        }
        for (a, b) in self.hr_dist.iter_mut().flatten().zip(o.hr_dist.iter().flatten()) {
            a.0 += b.0;
            a.1 += b.1;
        }
        for (a, b) in self.hr_ev_dist.iter_mut().zip(&o.hr_ev_dist) {
            a.merge(b);
        }
        for (a, b) in self.spray_pull.iter_mut().zip(&o.spray_pull) {
            a.merge(b);
        }
        for (a, b) in self.kinds.iter_mut().zip(&o.kinds) {
            a.n += b.n;
            a.swings += b.swings;
            a.whiffs += b.whiffs;
            a.zone += b.zone;
            a.out_swings += b.out_swings;
            a.out_n += b.out_n;
            a.gb += b.gb;
            a.bip += b.bip;
            a.rise.merge(&b.rise);
            a.arm_side.merge(&b.arm_side);
            a.height.merge(&b.height);
            for (x, y) in a.side_x.iter_mut().zip(&b.side_x) {
                x.merge(y);
            }
            for (x, y) in a.side_zone.iter_mut().zip(&b.side_zone) {
                add_arr(x, y);
            }
            for (x, y) in a.dist.iter_mut().zip(&b.dist) {
                for (u, v) in x.iter_mut().zip(y) {
                    add_arr(u, v);
                }
            }
            a.zone_swings += b.zone_swings;
            a.zone_whiffs += b.zone_whiffs;
            a.called += b.called;
            add_arr(&mut a.miss, &b.miss);
            for (x, y) in a.side.iter_mut().zip(&b.side) {
                add_arr(x, y);
            }
            a.speed.merge(&b.speed);
            a.ev.merge(&b.ev);
            a.la.merge(&b.la);
        }
        for (a, b) in self.types.iter_mut().zip(&o.types) {
            a.n += b.n;
            a.hits += b.hits;
            a.singles += b.singles;
            a.doubles += b.doubles;
            a.triples += b.triples;
            a.hr += b.hr;
            a.ev.merge(&b.ev);
            a.la.merge(&b.la);
            a.pull.merge(&b.pull);
        }
        for (a, b) in self.la_bins.iter_mut().zip(&o.la_bins) {
            a.n += b.n;
            a.hits += b.hits;
            a.hr += b.hr;
            a.ev.merge(&b.ev);
            a.dist.merge(&b.dist);
        }
    }

    pub fn percentile(hist: &[f64], offset: f64, width: f64, q: f64) -> f64 {
        let total: f64 = hist.iter().sum();
        let mut acc = 0.0;
        for (i, &h) in hist.iter().enumerate() {
            if acc + h >= q * total {
                let f = if h > 0.0 { (q * total - acc) / h } else { 0.0 };
                return offset + width * (i as f64 + f);
            }
            acc += h;
        }
        offset + width * hist.len() as f64
    }

    fn batted(&mut self, e: &PitchEvent, info: &BattedInfo, kind: PlayKind, landing_dist_ft: f64, landing_bearing_deg: f64, field_bearing_deg: f64) {
        let ev = info.exit_speed / MPH;
        let la = info.launch_angle.to_degrees();
        self.bip += 1.0;
        self.ev.add(ev);
        self.la.add(la);
        self.ev_hist[(ev.clamp(0.0, 129.0)) as usize] += 1.0;
        self.la_hist[((la + 90.0).clamp(0.0, 180.0)) as usize] += 1.0;
        if ev >= 95.0 {
            self.hard += 1.0;
        }
        if (8.0..=32.0).contains(&la) {
            self.sweet += 1.0;
        }
        if is_barrel(ev, la) {
            self.barrel += 1.0;
        }
        // Bunts are not swings: they stay out of the bat-speed and efficiency figures.
        if !info.bunt {
            self.bat_speed_bip.add(info.bat_speed / MPH);
            let bat = info.bat_speed / MPH;
            let ratio = ev / (1.23 * bat + 0.23 * e.thrown.flight.plate_vel.norm() / MPH);
            self.eff_all.add(ratio, ev, bat, la);
            if let Some(i) = bin(&EFF_LA_EDGES, la) {
                self.eff_la[i].add(ratio, ev, bat, la);
            }
            if let Some(i) = bin(&EFF_BAT_EDGES, bat) {
                self.eff_bat[i].add(ratio, ev, bat, la);
            }
        }
        let hit = kind.is_hit();
        let hr = kind == PlayKind::HomeRun;
        // Like Statcast's hit coordinates: the bearing of the spot where the ball was fielded.
        let pull_side = field_bearing_deg * e.batter_hand.pull_sign();
        let t = &mut self.types[bb_type(la)];
        t.n += 1.0;
        t.ev.add(ev);
        t.la.add(la);
        t.pull.add(pull_side);
        if hit {
            t.hits += 1.0;
        }
        match kind {
            PlayKind::Single => t.singles += 1.0,
            PlayKind::Double | PlayKind::GroundRuleDouble => t.doubles += 1.0,
            PlayKind::Triple => t.triples += 1.0,
            PlayKind::HomeRun => t.hr += 1.0,
            _ => {}
        }
        if (-40.0..70.0).contains(&la) {
            let b = &mut self.la_bins[((la + 40.0) / 10.0) as usize];
            b.n += 1.0;
            b.ev.add(ev);
            b.dist.add(landing_dist_ft);
            if hit {
                b.hits += 1.0;
            }
            if hr {
                b.hr += 1.0;
            }
        }
        if la >= 15.0 && (300.0..460.0).contains(&landing_dist_ft) && landing_bearing_deg.abs() <= 45.0 {
            let deg = landing_bearing_deg.abs();
            let sector = if deg >= 33.75 {
                0
            } else if deg >= 11.25 {
                1
            } else {
                2
            };
            let c = &mut self.hr_dist[sector][((landing_dist_ft - 300.0) / 20.0) as usize];
            c.0 += 1.0;
            if hr {
                c.1 += 1.0;
            }
        }
        if !info.bunt && (10.0..50.0).contains(&la) && (90.0..120.0).contains(&ev) {
            let r = if la < 40.0 { ((la - 10.0) / 5.0) as usize } else { 6 };
            let c = if ev < 110.0 { ((ev - 90.0) / 5.0) as usize } else { 4 };
            self.hr_cell[r][c].0 += 1.0;
            if hr {
                self.hr_cell[r][c].1 += 1.0;
            }
        }
        if (20.0..35.0).contains(&la) && (80.0..115.0).contains(&ev) {
            let i = ((ev - 80.0) / 5.0) as usize;
            self.hr_ev_n[i] += 1.0;
            self.hr_ev_dist[i].add(landing_dist_ft);
            if hr {
                self.hr_ev_hr[i] += 1.0;
            }
        }
        let h = if e.batter_hand == Hand::Right { 0 } else { 1 };
        self.spray_pull[h].add(pull_side);
        let slot = if pull_side > 15.0 {
            0
        } else if pull_side >= -15.0 {
            1
        } else {
            2
        };
        self.pull_n[h][slot] += 1.0;
        if (0.0..40.0).contains(&la) {
            let y = info.backspin / RPM;
            self.spin_fit[0] += 1.0;
            self.spin_fit[1] += la;
            self.spin_fit[2] += y;
            self.spin_fit[3] += la * la;
            self.spin_fit[4] += la * y;
            let pull = info.spray.to_degrees() * e.batter_hand.pull_sign();
            let f = &mut self.spin_fit3;
            f[0] += 1.0;
            f[1] += la;
            f[2] += pull;
            f[3] += y;
            f[4] += la * la;
            f[5] += pull * pull;
            f[6] += la * pull;
            f[7] += la * y;
            f[8] += pull * y;
        }
        if (10.0..40.0).contains(&la) {
            let pull = e.batter_hand.pull_sign();
            let x = info.spray.to_degrees() * pull;
            let y = -info.sidespin / RPM * pull;
            self.side_fit[0] += 1.0;
            self.side_fit[1] += x;
            self.side_fit[2] += y;
            self.side_fit[3] += x * x;
            self.side_fit[4] += x * y;
        }
        let z_ft = e.thrown.flight.plate_pos.z / FT;
        if (1.0..4.0).contains(&z_ft) {
            let i = ((z_ft - 1.0) / 0.5) as usize;
            self.la_height_n[i] += 1.0;
            self.la_height_la[i] += la;
        }
        let k = &mut self.kinds[e.thrown.kind.index()];
        k.bip += 1.0;
        k.ev.add(ev);
        k.la.add(la);
        if la < 10.0 {
            k.gb += 1.0;
        }
    }
}

impl Observer for Tally {
    fn pitch(&mut self, e: &PitchEvent) {
        self.pitches += 1.0;
        let count = (e.balls.min(3) as usize) * 3 + e.strikes.min(2) as usize;
        self.pitch_classes[e.thrown.kind.index()][count][pitch_class(e)] += 1.0;
        if e.zone_distance <= 0.0 {
            self.pitch_classes_zone[e.thrown.kind.index()][count][pitch_class(e)] += 1.0;
        }
        if e.approach.is_bunt() {
            let t = &mut self.tactics;
            let i = match e.result {
                PitchResult::Ball | PitchResult::CalledStrike | PitchResult::HitByPitch => 0,
                PitchResult::SwingingStrike => 1,
                PitchResult::Foul | PitchResult::FoulTip => 2,
                PitchResult::InPlay => 3,
            };
            t.bunt_pitches[i] += 1.0;
            if let (PitchResult::InPlay, Some(info)) = (e.result, e.batted) {
                t.bunt_ev.add(info.exit_speed / MPH);
                t.bunt_la.add(info.launch_angle.to_degrees());
                if info.launch_angle.to_degrees() > 20.0 {
                    t.bunt_popped += 1.0;
                }
                if let Some(play) = e.play {
                    t.bunt_fielder[play.fielder.min(8)] += 1.0;
                }
                t.bunt_in_play = true;
            }
        } else if e.approach == Approach::HitAndRun {
            self.tactics.hit_and_run_pitches += 1.0;
        }
        if let (Some(shape), false) = (e.swing, e.approach.is_bunt()) {
            let outcome = match e.result {
                PitchResult::SwingingStrike | PitchResult::FoulTip => 0,
                PitchResult::Foul => 1,
                _ => 2,
            };
            let g = &mut self.geo;
            let mph = shape.bat_speed / MPH;
            g.all[outcome] += 1.0;
            let s = e.strikes.min(2) as usize;
            g.by_strikes[s][match e.result {
                PitchResult::SwingingStrike => 0,
                PitchResult::FoulTip => 1,
                PitchResult::Foul => 2,
                _ => 3,
            }] += 1.0;
            g.speed_by_strikes[s].add(mph);
            if mph < 60.0 {
                g.by_strikes[s][4] += 1.0;
            }
            if mph < 60.0 {
                g.slow[outcome] += 1.0;
            } else {
                let depth = shape.depth / INCH;
                let dir = shape.direction.to_degrees();
                g.speed[outcome].add(mph);
                g.depth[outcome].add(depth);
                g.direction[outcome].add(dir);
                if shape.tilt > 0.0 {
                    g.attack_by[outcome].add(shape.attack.to_degrees());
                }
                let kd = &mut g.kind_depth[e.thrown.kind.index()];
                kd.depth.add(depth);
                let h = (((depth + 40.0) / 2.5).floor().clamp(0.0, 31.0)) as usize;
                kd.hist[h][0] += 1.0;
                if outcome == 0 {
                    kd.hist[h][1] += 1.0;
                }
                let bin_i = bin(&DEPTH_EDGES, depth);
                // Ball centre more than 8.5 cm inside the edge of the zone, within 3.7 cm of leaving it, or beyond.
                let zone = if e.zone_distance < -0.085 {
                    0
                } else if e.zone_distance <= -0.037 {
                    1
                } else {
                    2
                };
                if let Some(i) = bin_i {
                    g.by_depth[i][outcome] += 1.0;
                    g.depth_zone[i][zone][outcome] += 1.0;
                }
                if depth.abs() < 4.0 && dir.abs() < 5.0 {
                    g.centred[0] += 1.0;
                    g.centred_zone[zone][outcome] += 1.0;
                    if let Some(m) = e.miss {
                        g.centred_miss[m as usize] += 1.0;
                    }
                    match e.result {
                        PitchResult::SwingingStrike => g.centred[1] += 1.0,
                        PitchResult::FoulTip => g.centred[2] += 1.0,
                        _ => {
                            if let Some(i) = bin(&ELEV_EDGES, shape.elevation.to_degrees()) {
                                let c = if outcome == 1 { &mut g.centred_foul[i] } else { &mut g.centred_bip[i] };
                                c.add(shape.exit_speed / MPH);
                            }
                        }
                    }
                }
                if let (2, Some(info)) = (outcome, e.batted) {
                    let pull = e.batter_hand.pull_sign();
                    let spray = info.spray.to_degrees() * pull;
                    let inside = e.thrown.flight.plate_pos.x * pull / INCH;
                    let (ev, la, attack) = (info.exit_speed / MPH, info.launch_angle.to_degrees(), info.attack_angle.to_degrees());
                    let ratio = ev / (1.23 * mph + 0.23 * e.thrown.flight.plate_vel.norm() / MPH);
                    g.attack.add(attack);
                    g.spray.add(spray);
                    g.fits[0].add(depth, mph);
                    g.fits[1].add(depth, attack);
                    g.fits[2].add(depth, dir);
                    g.fits[3].add(dir, spray);
                    g.fits[4].add(inside, depth);
                    g.fits[5].add(inside, dir);
                    g.fits[6].add(inside, spray);
                    if let Some(i) = bin_i {
                        for (k, v) in [mph, attack, dir, la, ev, ratio, spray, shape.along / INCH].into_iter().enumerate() {
                            g.depth_bip[i][k].add(v);
                        }
                    }
                    let c = &mut g.by_la[bb_type(la)];
                    c[0].add(spray);
                    c[1].add(dir);
                    c[2].add(depth);
                    c[3].add(attack);
                }
            }
        }
        let d_in = e.zone_distance / INCH;
        let inz = e.zone_distance <= 0.0;
        let (b, s) = (e.balls.min(3) as usize, e.strikes.min(2) as usize);
        let ri = match e.result {
            PitchResult::Ball => 0,
            PitchResult::CalledStrike => 1,
            PitchResult::SwingingStrike => 2,
            PitchResult::Foul => 3,
            PitchResult::FoulTip => 4,
            PitchResult::InPlay => 5,
            PitchResult::HitByPitch => 6,
        };
        self.res[ri] += 1.0;
        if let Some(m) = e.miss {
            self.miss[match m {
                Miss::Early => 0,
                Miss::Late => 1,
                Miss::OffTheEnd => 2,
                Miss::Jammed => 3,
                Miss::Over => 4,
                Miss::Under => 5,
            }] += 1.0;
        }
        let whiff = matches!(e.result, PitchResult::SwingingStrike | PitchResult::FoulTip);
        let contact = matches!(e.result, PitchResult::Foul | PitchResult::InPlay);
        self.count_n[b][s] += 1.0;
        let kind = e.thrown.kind;
        if matches!(kind, PitchKind::FourSeam | PitchKind::Sinker | PitchKind::Cutter) {
            self.count_fast[b][s] += 1.0;
        }
        let k = &mut self.kinds[kind.index()];
        k.n += 1.0;
        k.speed.add(e.thrown.spec.speed / MPH);
        k.rise.add(e.thrown.flight.move_z / INCH);
        k.arm_side.add(e.thrown.flight.move_x / INCH * if e.pitcher_hand == Hand::Right { -1.0 } else { 1.0 });
        k.height.add(e.thrown.flight.plate_pos.z / FT);
        let opposite = usize::from(e.batter_hand != e.pitcher_hand);
        k.side_x[opposite].add(e.thrown.flight.plate_pos.x * e.batter_hand.outside_sign() / FT);
        k.side_zone[opposite][0] += 1.0;
        if inz {
            k.side_zone[opposite][1] += 1.0;
        }
        if inz {
            self.zone += 1.0;
            self.z_n += 1.0;
            self.count_zone[b][s] += 1.0;
            self.count_zn[b][s] += 1.0;
            k.zone += 1.0;
        } else {
            self.o_n += 1.0;
            k.out_n += 1.0;
        }
        let groups = [0usize, 1 + s];
        if let Some(i) = bin(&DIST_EDGES, d_in) {
            for g in groups {
                self.swing_dist_n[g][i] += 1.0;
                if e.swung {
                    self.swing_dist_s[g][i] += 1.0;
                }
            }
            if e.swung {
                self.contact_dist_n[i] += 1.0;
                if contact {
                    self.contact_dist_c[i] += 1.0;
                }
            }
        }
        if e.result == PitchResult::CalledStrike {
            k.called += 1.0;
        }
        if let Some(i) = bin(&KIND_DIST_EDGES, d_in) {
            k.dist[s][i][0] += 1.0;
            if e.swung {
                k.dist[s][i][1] += 1.0;
            }
        }
        if e.swung {
            self.swings += 1.0;
            self.count_swing[b][s] += 1.0;
            k.swings += 1.0;
            if whiff {
                k.whiffs += 1.0;
            }
            if let Some(m) = e.miss {
                k.miss[m as usize] += 1.0;
            }
            let opposite = usize::from(e.batter_hand != e.pitcher_hand);
            k.side[opposite][0] += 1.0;
            if whiff {
                k.side[opposite][1] += 1.0;
            }
            if inz {
                k.zone_swings += 1.0;
                if whiff {
                    k.zone_whiffs += 1.0;
                }
                self.z_swings += 1.0;
                self.count_zswing[b][s] += 1.0;
            } else {
                self.o_swings += 1.0;
                self.count_oswing[b][s] += 1.0;
                k.out_swings += 1.0;
            }
            if contact {
                self.contact += 1.0;
                if inz {
                    self.z_contact += 1.0;
                } else {
                    self.o_contact += 1.0;
                }
                if e.result == PitchResult::Foul {
                    self.fouls_of_contact += 1.0;
                }
            }
            if let (Some(bs), false) = (e.bat_speed, e.approach.is_bunt()) {
                self.bat_speed_all.add(bs / MPH);
            }
            if kind == PitchKind::FourSeam {
                let v = e.thrown.spec.speed / MPH;
                if (88.0..104.0).contains(&v) {
                    let i = ((v - 88.0) / 2.0) as usize;
                    self.whiff_fb_speed[i][0] += 1.0;
                    if whiff {
                        self.whiff_fb_speed[i][1] += 1.0;
                    }
                }
            }
            let z_ft = e.thrown.flight.plate_pos.z / FT;
            if (0.5..4.5).contains(&z_ft) {
                let g = match kind.group() {
                    bm_engine::PitchGroup::Fastball => 0,
                    bm_engine::PitchGroup::Breaking => 1,
                    bm_engine::PitchGroup::Offspeed => 2,
                };
                let i = ((z_ft - 0.5) / 0.5) as usize;
                self.whiff_height[g][i][0] += 1.0;
                if whiff {
                    self.whiff_height[g][i][1] += 1.0;
                }
            }
        } else if e.result != PitchResult::HitByPitch {
            let called = e.result == PitchResult::CalledStrike;
            if inz {
                self.takes_z += 1.0;
                if called {
                    self.cs_z += 1.0;
                }
            } else {
                self.takes_o += 1.0;
                if called {
                    self.cs_o += 1.0;
                }
            }
            if let Some(i) = bin(&CALL_EDGES, d_in) {
                self.call_n[i] += 1.0;
                if called {
                    self.call_s[i] += 1.0;
                }
            }
        }
        if e.result == PitchResult::InPlay {
            if let (Some(info), Some(play), Some(path)) = (e.batted, e.play, e.path) {
                // Statcast reports the projected distance at field level: where the ball would
                // have come down had no wall or stand stopped it.
                let end = path.carry_point();
                let bearing = play.field_pos.x.atan2(play.field_pos.y).to_degrees();
                if play.kind == PlayKind::HomeRun {
                    self.hr_inside.1 += 1.0;
                    if !path.is_home_run() {
                        self.hr_inside.0 += 1.0;
                    }
                }
                if let Some(park) = &self.park {
                    let b = end.x.atan2(end.y);
                    if info.launch_angle.to_degrees() >= 15.0 && end.horiz() / FT >= 250.0 && b.abs() <= std::f64::consts::FRAC_PI_4 {
                        let m = (end.horiz() - park.fence_at(b).0) / FT;
                        let deg = b.to_degrees().abs();
                        let sector = if deg >= 33.75 {
                            0
                        } else if deg >= 11.25 {
                            1
                        } else {
                            2
                        };
                        let c = &mut self.hr_margin[sector][((m / 10.0).floor() as i64 + 4).clamp(0, 8) as usize];
                        c.0 += 1.0;
                        if play.kind == PlayKind::HomeRun {
                            c.1 += 1.0;
                        }
                    }
                }
                self.batted(e, info, play.kind, end.horiz() / FT, end.x.atan2(end.y).to_degrees(), bearing);
                if play.runner_thrown_out {
                    self.runner_outs += 1.0;
                }
                if play.outs > 0 && !play.kind.is_hit() {
                    self.outs_by_fielder[bb_type(info.launch_angle.to_degrees())][play.fielder.min(8)] += 1.0;
                }
                if info.launch_angle.to_degrees() < 10.0 && e.outs < 2 {
                    // Ground ball with a force at second in order (runner on first is tracked by the PA event).
                    if play.gidp {
                        self.dp_made += 1.0;
                    }
                }
            }
        }
    }

    fn plate_appearance(&mut self, e: &PaEvent) {
        let bunted = std::mem::take(&mut self.tactics.bunt_in_play);
        if e.outcome == PaOutcome::Interrupted {
            return;
        }
        if e.intentional {
            self.tactics.intentional_walks += 1.0;
        }
        if let (true, PaOutcome::Play(k)) = (bunted, e.outcome) {
            let t = &mut self.tactics;
            if k == PlayKind::SacBunt {
                t.sac_bunts += 1.0;
            }
            if e.bases_before & 1 != 0 && e.outs_before < 2 {
                let i = match k {
                    PlayKind::SacBunt => 1,
                    _ if k.is_hit() => 2,
                    PlayKind::FieldersChoice | PlayKind::DoublePlay => 4,
                    PlayKind::Error => 5,
                    _ => 3,
                };
                t.sac_situation[0] += 1.0;
                t.sac_situation[i] += 1.0;
            } else if e.bases_before == 0 {
                t.empty_situation[0] += 1.0;
                if k.is_hit() {
                    t.empty_situation[1] += 1.0;
                }
            }
        }
        self.pa += 1.0;
        self.pa_pitches += e.pitches as f64;
        self.base_state[(e.bases_before & 7) as usize] += 1.0;
        if e.bases_before & 4 != 0 && e.outs_before < 2 {
            self.third_lt2 += 1.0;
            if e.outcome == PaOutcome::Play(PlayKind::SacFly) {
                self.sf_opp += 1.0;
            }
        }
        if e.bases_before & 1 != 0 && e.outs_before < 2 {
            self.first_lt2 += 1.0;
        }
        if let PaOutcome::Play(k) = e.outcome {
            let (b, a) = (e.bases_before, e.bases_after);
            let lt2 = e.outs_before < 2;
            let mut count = |i: usize, hit: bool| {
                self.adv[i][0] += 1.0;
                if hit {
                    self.adv[i][1] += 1.0;
                }
            };
            match k {
                PlayKind::Single if b == 1 => count(0, a & 4 != 0),
                PlayKind::Single if b == 2 => count(1, a & 4 == 0),
                PlayKind::Double if b == 1 => count(2, a & 4 == 0),
                PlayKind::GroundOut if b == 2 && lt2 => count(3, a & 4 != 0),
                PlayKind::FlyOut | PlayKind::LineOut if b == 2 && lt2 => count(4, a & 4 != 0),
                _ => {}
            }
            if e.ground_ball && b & 1 != 0 && lt2 && matches!(k, PlayKind::DoublePlay | PlayKind::FieldersChoice | PlayKind::GroundOut) {
                count(5, k == PlayKind::DoublePlay);
                count(6, k == PlayKind::FieldersChoice);
                count(7, k == PlayKind::GroundOut);
            }
        }
        match e.outcome {
            PaOutcome::Strikeout => self.so += 1.0,
            PaOutcome::Walk => self.bb += 1.0,
            PaOutcome::HitByPitch => self.hbp += 1.0,
            PaOutcome::Play(k) => {
                match k {
                    PlayKind::Single => self.singles += 1.0,
                    PlayKind::Double | PlayKind::GroundRuleDouble => self.doubles += 1.0,
                    PlayKind::Triple => self.triples += 1.0,
                    PlayKind::HomeRun => self.hr += 1.0,
                    PlayKind::Error => self.roe += 1.0,
                    PlayKind::SacFly => self.sf += 1.0,
                    PlayKind::DoublePlay => self.gidp += 1.0,
                    _ => {}
                }
                if !k.is_hit() && k != PlayKind::Error {
                    self.outs_in_play += 1.0;
                }
                if e.bases_before & 1 != 0 && e.outs_before < 2 && matches!(k, PlayKind::GroundOut | PlayKind::DoublePlay | PlayKind::FieldersChoice) {
                    self.dp_opp_gb += 1.0;
                }
            }
            PaOutcome::Interrupted => {}
        }
    }

    fn steal(&mut self, success: bool) {
        if success {
            self.sb += 1.0;
        } else {
            self.cs += 1.0;
        }
    }

    fn wild_pitch(&mut self) {
        self.wp += 1.0;
    }

    fn pitching_change(&mut self, _team: usize, _player: usize) {
        self.pitchers_used += 1.0;
    }

    fn event(&mut self, e: &GameEvent) {
        match *e {
            GameEvent::PinchHitter { .. } => self.tactics.pinch_hitters += 1.0,
            GameEvent::PinchRunner { .. } => self.tactics.pinch_runners += 1.0,
            GameEvent::Steal { to_base: 3, safe, .. } => {
                self.tactics.steals_of_third[0] += 1.0;
                if safe {
                    self.tactics.steals_of_third[1] += 1.0;
                }
            }
            _ => {}
        }
    }
}
