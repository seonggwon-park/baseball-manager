//! League rule sets and the strike zone.

use crate::params::ModelParams;
use crate::rng::sigmoid;
use bm_physics::units::INCH;
use bm_physics::{Vec3, PLATE_FRONT_Y, PLATE_HALF_WIDTH};

/// Who calls balls and strikes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneCall {
    /// Human plate umpire: probabilistic at the edges.
    Human,
    /// Automated ball-strike system: the rule is applied exactly.
    Automated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rules {
    /// Regulation innings.
    pub innings: u8,
    /// Last inning that may be played; a tie after it stands. `None` = play until decided.
    pub max_innings: Option<u8>,
    /// Automatic runner on second base from this inning on (`None` = never).
    pub auto_runner_from: Option<u8>,
    /// Designated hitter in use.
    pub dh: bool,
    pub zone_call: ZoneCall,
    /// Top and bottom of the strike zone as fractions of the batter's height.
    pub zone_top_frac: f64,
    pub zone_bottom_frac: f64,
    /// Extra half-width added to the plate on each side, m.
    pub zone_extra_half_width: f64,
    /// For automated zones: also require the height limits at the back of the plate.
    pub zone_check_back_plane: bool,
    /// Batters a pitcher has to face before he can be replaced, unless the half
    /// inning ends first.
    pub min_batters_faced: u8,
    /// Two infielders must stand on each side of second base, all on the infield.
    pub shift_restricted: bool,
}

impl Rules {
    /// MLB regular season, 2025: human umpires, universal DH, automatic runner
    /// in extra innings, three-batter minimum (since 2020) and restricted infield
    /// shifts (since 2023). The zone fractions reproduce the average `sz_top` /
    /// `sz_bot` reported by Statcast (3.44 ft / 1.60 ft for a 1.855 m batter).
    pub fn mlb_2025() -> Self {
        Rules {
            innings: 9,
            max_innings: None,
            auto_runner_from: Some(10),
            dh: true,
            zone_call: ZoneCall::Human,
            zone_top_frac: 0.565,
            zone_bottom_frac: 0.264,
            zone_extra_half_width: 0.0,
            zone_check_back_plane: false,
            min_batters_faced: 3,
            shift_restricted: true,
        }
    }

    /// KBO League, 2025: automated ball-strike system (top 55.75 %, bottom 27.04 %
    /// of the batter's height, plate width plus 2 cm on each side, height checked at
    /// the middle and the back of the plate), DH, ties after the 11th inning.
    /// Source: KBO, "2025 KBO 리그 규정·규칙 변화". Infield shifts have been restricted
    /// since 2024; the three-batter minimum was only trialled in the Futures League,
    /// so a reliever has to face one batter.
    pub fn kbo_2025() -> Self {
        Rules {
            innings: 9,
            max_innings: Some(11),
            auto_runner_from: None,
            dh: true,
            zone_call: ZoneCall::Automated,
            zone_top_frac: 0.5575,
            zone_bottom_frac: 0.2704,
            zone_extra_half_width: 0.02,
            zone_check_back_plane: true,
            min_batters_faced: 1,
            shift_restricted: true,
        }
    }

    /// NPB-style rules: human umpires, ties after the 12th inning. The DH flag is
    /// set per league by the caller (Pacific League uses it).
    pub fn npb_2025(dh: bool) -> Self {
        Rules {
            innings: 9,
            max_innings: Some(12),
            auto_runner_from: None,
            dh,
            zone_call: ZoneCall::Human,
            zone_top_frac: 0.565,
            zone_bottom_frac: 0.264,
            zone_extra_half_width: 0.0,
            zone_check_back_plane: false,
            min_batters_faced: 1,
            shift_restricted: false,
        }
    }
}

/// Strike zone of one batter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikeZone {
    pub top: f64,
    pub bottom: f64,
    pub half_width: f64,
}

impl StrikeZone {
    pub fn for_batter(height: f64, rules: &Rules) -> Self {
        StrikeZone { top: rules.zone_top_frac * height, bottom: rules.zone_bottom_frac * height, half_width: PLATE_HALF_WIDTH + rules.zone_extra_half_width }
    }

    #[inline]
    pub fn mid(&self) -> f64 {
        0.5 * (self.top + self.bottom)
    }

    #[inline]
    pub fn height(&self) -> f64 {
        self.top - self.bottom
    }

    /// Signed distance of the ball's centre outside the zone, counting a ball that
    /// touches the zone as inside: negative inside, positive outside, m.
    pub fn signed_distance(&self, x: f64, z: f64, ball_radius: f64) -> f64 {
        let dx = x.abs() - (self.half_width + ball_radius);
        let dz = (z - (self.top + ball_radius)).max((self.bottom - ball_radius) - z);
        if dx <= 0.0 && dz <= 0.0 {
            dx.max(dz)
        } else if dx <= 0.0 {
            dz
        } else if dz <= 0.0 {
            dx
        } else {
            (dx * dx + dz * dz).sqrt()
        }
    }

    /// Is this pitch a strike if not swung at? `plate_pos` / `plate_vel` are the
    /// ball's state at the front of the plate; `u` is a uniform random number
    /// (only used by human umpires). `framing` shifts the human zone (m).
    #[allow(clippy::too_many_arguments)]
    pub fn called_strike(&self, rules: &Rules, params: &ModelParams, plate_pos: Vec3, plate_vel: Vec3, ball_radius: f64, framing: f64, u: f64) -> bool {
        match rules.zone_call {
            ZoneCall::Human => {
                let d = self.signed_distance(plate_pos.x, plate_pos.z, ball_radius);
                u < sigmoid(-(d + params.ump_bias - framing) / params.ump_scale)
            }
            ZoneCall::Automated => {
                // Evaluate where the ball is over the middle (and optionally the back) of the plate.
                let at = |y: f64| {
                    let dt = if plate_vel.y.abs() > 1e-9 { (y - plate_pos.y) / plate_vel.y } else { 0.0 };
                    plate_pos + plate_vel * dt
                };
                let mid = at(0.5 * PLATE_FRONT_Y);
                let inside = |p: Vec3| p.z <= self.top + ball_radius && p.z >= self.bottom - ball_radius;
                let lateral = mid.x.abs() <= self.half_width + ball_radius;
                let vertical = inside(mid) && (!rules.zone_check_back_plane || inside(at(0.0)));
                lateral && vertical
            }
        }
    }
}

/// Ball radius used for zone geometry when only the rules are at hand (m).
pub const ZONE_BALL_RADIUS: f64 = 1.45 * INCH;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_matches_statcast_average() {
        let z = StrikeZone::for_batter(1.855, &Rules::mlb_2025());
        assert!((z.top / 0.3048 - 3.44).abs() < 0.02 && (z.bottom / 0.3048 - 1.60).abs() < 0.02);
    }

    #[test]
    fn signed_distance() {
        let z = StrikeZone { top: 1.05, bottom: 0.49, half_width: PLATE_HALF_WIDTH };
        let r = ZONE_BALL_RADIUS;
        assert!(z.signed_distance(0.0, 0.77, r) < -0.2);
        assert!((z.signed_distance(PLATE_HALF_WIDTH + r, 0.77, r)).abs() < 1e-12);
        assert!((z.signed_distance(0.0, 1.05 + r + 0.05, r) - 0.05).abs() < 1e-12);
        let corner = z.signed_distance(PLATE_HALF_WIDTH + r + 0.03, 0.49 - r - 0.04, r);
        assert!((corner - 0.05).abs() < 1e-12);
    }

    #[test]
    fn human_zone_is_soft_and_automated_zone_is_exact() {
        let p = ModelParams::default();
        let r = ZONE_BALL_RADIUS;
        let human = Rules::mlb_2025();
        let z = StrikeZone::for_batter(1.855, &human);
        let vel = Vec3::new(0.0, -38.0, -4.0);
        let rate = |d: f64| {
            let pos = Vec3::new(z.half_width + r + d, PLATE_FRONT_Y, z.mid());
            (0..1000).filter(|i| z.called_strike(&human, &p, pos, vel, r, 0.0, (*i as f64 + 0.5) / 1000.0)).count() as f64 / 1000.0
        };
        // 2025 Statcast: ~42 % at the edge, ~8 % two inches off, ~86 % two inches inside.
        assert!((rate(0.0) - 0.42).abs() < 0.05, "edge {}", rate(0.0));
        assert!((rate(2.0 * INCH) - 0.08).abs() < 0.04, "2 in off {}", rate(2.0 * INCH));
        assert!((rate(-2.0 * INCH) - 0.86).abs() < 0.05, "2 in inside {}", rate(-2.0 * INCH));

        let abs = Rules::kbo_2025();
        let z = StrikeZone::for_batter(1.80, &abs);
        let just_in = Vec3::new(z.half_width + r - 0.001, PLATE_FRONT_Y, z.mid());
        let just_out = Vec3::new(z.half_width + r + 0.001, PLATE_FRONT_Y, z.mid());
        let flat = Vec3::new(0.0, -38.0, 0.0);
        assert!(z.called_strike(&abs, &p, just_in, flat, r, 0.0, 0.99));
        assert!(!z.called_strike(&abs, &p, just_out, flat, r, 0.0, 0.01));
        // A pitch that clips the top at the front but drops below the bottom by the back plane
        // cannot happen; one that is in at the middle but under the bottom at the back is a ball.
        let low = Vec3::new(0.0, PLATE_FRONT_Y, z.bottom - r + 0.03);
        assert!(!z.called_strike(&abs, &p, low, Vec3::new(0.0, -36.0, -6.0), r, 0.0, 0.5));
    }
}
