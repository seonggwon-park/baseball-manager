//! Field geometry: bases, outfield fence, foul-territory boundary, playing surfaces.

use crate::ground::Surface;
use crate::units::FT;
use crate::vec3::Vec3;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// Distance between bases (90 ft).
pub const BASE_PATH: f64 = 90.0 * FT;
/// Distance from the back of home plate to the centre of the pitching rubber.
pub const MOUND_Y: f64 = 60.5 * FT;
/// Radius of the skinned infield arc measured from the pitching rubber (95 ft).
pub const INFIELD_ARC: f64 = 95.0 * FT;

const S: f64 = BASE_PATH * std::f64::consts::FRAC_1_SQRT_2;

/// Position of a base: 0 = home, 1 = first, 2 = second, 3 = third.
#[inline]
pub fn base_position(base: usize) -> Vec3 {
    match base & 3 {
        0 => Vec3::ZERO,
        1 => Vec3::new(S, S, 0.0),
        2 => Vec3::new(0.0, 2.0 * S, 0.0),
        _ => Vec3::new(-S, S, 0.0),
    }
}

/// Horizontal bearing of a point as seen from home plate: 0 toward second base,
/// positive toward first base / right field.
#[inline]
pub fn bearing(pos: Vec3) -> f64 {
    pos.x.atan2(pos.y)
}

/// True if a point on the ground is in fair territory (on or between the foul lines).
#[inline]
pub fn is_fair(pos: Vec3) -> bool {
    pos.y >= 0.0 && pos.x.abs() <= pos.y + 1e-9
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FencePoint {
    /// Bearing, rad (-pi/4 = left-field line, +pi/4 = right-field line).
    pub angle: f64,
    /// Distance from home plate, m.
    pub distance: f64,
    /// Wall height, m.
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Grass,
    Dirt,
    Synthetic,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Park {
    /// Outfield fence from the left-field line to the right-field line, ascending in angle.
    pub fence: Vec<FencePoint>,
    /// Width of foul territory beside the plate (foul line to stands), m.
    pub foul_width_near: f64,
    /// Width of foul territory at the outfield corners, m.
    pub foul_width_far: f64,
    /// Distance from home plate to the backstop, m.
    pub backstop: f64,
    /// Coefficient of restitution of the outfield wall.
    pub wall_cor: f64,
    /// True if the playing field is synthetic turf (infield skin cut-outs remain dirt).
    pub synthetic: bool,
    pub grass: Surface,
    pub dirt: Surface,
    pub turf: Surface,
}

impl Park {
    /// Symmetric park from five fence distances (m) and a uniform wall height (m).
    pub fn symmetric(line: f64, alley: f64, center: f64, wall_height: f64) -> Self {
        Park::from_fence(&[line, alley, center, alley, line], wall_height)
    }

    /// Park from fence distances sampled at evenly spaced bearings between the foul lines.
    pub fn from_fence(distances: &[f64], wall_height: f64) -> Self {
        assert!(distances.len() >= 2);
        let n = distances.len();
        let fence = distances
            .iter()
            .enumerate()
            .map(|(i, &d)| FencePoint { angle: -FRAC_PI_4 + FRAC_PI_2 * i as f64 / (n - 1) as f64, distance: d, height: wall_height })
            .collect();
        Park {
            fence,
            foul_width_near: 16.0,
            foul_width_far: 5.0,
            backstop: 16.0,
            wall_cor: 0.35,
            synthetic: false,
            grass: Surface::grass(),
            dirt: Surface::dirt(),
            turf: Surface::synthetic(),
        }
    }

    /// Roughly the average MLB outfield: 330 ft down the lines, 377 ft to the
    /// alleys, 404 ft to centre, 8.5 ft wall.
    pub fn mlb_average() -> Self {
        Park::symmetric(330.0 * FT, 377.0 * FT, 404.0 * FT, 8.5 * FT)
    }

    /// Fence distance and wall height at a fair bearing (clamped to the foul lines).
    pub fn fence_at(&self, angle: f64) -> (f64, f64) {
        let f = &self.fence;
        let a = angle.clamp(f[0].angle, f[f.len() - 1].angle);
        let span = f[f.len() - 1].angle - f[0].angle;
        let pos = (a - f[0].angle) / span * (f.len() - 1) as f64;
        let i = (pos.floor() as usize).min(f.len() - 2);
        let t = pos - i as f64;
        (f[i].distance + t * (f[i + 1].distance - f[i].distance), f[i].height + t * (f[i + 1].height - f[i].height))
    }

    /// Distance from home plate to the edge of the playing field along any bearing,
    /// and the height of the barrier there.
    pub fn boundary(&self, angle: f64) -> (f64, f64) {
        let a = angle.abs();
        if a <= FRAC_PI_4 {
            return self.fence_at(angle);
        }
        let (pole, h) = self.fence_at(angle.signum() * FRAC_PI_4);
        // Stands roughly parallel to the foul line, closer in toward the outfield corner.
        let t = ((a - FRAC_PI_4) / FRAC_PI_4).clamp(0.0, 1.0);
        let width = self.foul_width_far + t * (self.foul_width_near - self.foul_width_far);
        let side = width / (a - FRAC_PI_4).sin().max(1e-6);
        let mut d = pole.min(side);
        if a > FRAC_PI_2 {
            d = d.min(self.backstop / (-a.cos()).max(1e-6));
        }
        (d, h.max(1.0))
    }

    /// Smallest distance to any boundary (used to skip boundary tests cheaply).
    pub fn min_boundary(&self) -> f64 {
        let fence_min = self.fence.iter().map(|p| p.distance).fold(f64::INFINITY, f64::min);
        fence_min.min(self.backstop).min(self.foul_width_far)
    }

    /// Which kind of surface a point of the field is.
    pub fn surface_kind_at(&self, pos: Vec3) -> SurfaceKind {
        let (x, y) = (pos.x, pos.y);
        let d_home = (x * x + y * y).sqrt();
        let dm = (x * x + (y - MOUND_Y) * (y - MOUND_Y)).sqrt();
        // Home-plate circle (13 ft radius) and mound (9 ft radius) are dirt.
        if d_home < 13.0 * FT || dm < 9.0 * FT {
            return SurfaceKind::Dirt;
        }
        let base_kind = if self.synthetic { SurfaceKind::Synthetic } else { SurfaceKind::Grass };
        // Inside the diamond (inset about 1 m for the base paths) the infield is grass.
        let diamond = x.abs() + (y - S).abs();
        if diamond < S - 1.0 {
            return base_kind;
        }
        // The skinned area: between the diamond and the 95 ft arc, in fair territory or
        // along the base lines.
        if dm < INFIELD_ARC && y > -1.0 && x.abs() <= y + 2.0 {
            return SurfaceKind::Dirt;
        }
        base_kind
    }

    #[inline]
    pub fn surface_at(&self, pos: Vec3) -> &Surface {
        match self.surface_kind_at(pos) {
            SurfaceKind::Grass => &self.grass,
            SurfaceKind::Dirt => &self.dirt,
            SurfaceKind::Synthetic => &self.turf,
        }
    }
}

/// Normalise an angle to (-pi, pi].
#[inline]
pub fn wrap_angle(a: f64) -> f64 {
    let mut a = a % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bases_form_a_square() {
        let h = base_position(0);
        let b1 = base_position(1);
        let b2 = base_position(2);
        let b3 = base_position(3);
        for (a, b) in [(h, b1), (b1, b2), (b2, b3), (b3, h)] {
            assert!(((a - b).norm() - BASE_PATH).abs() < 1e-9);
        }
        assert!((b2.y - 127.279 * FT).abs() < 1e-3);
    }

    #[test]
    fn fence_interpolation_and_foul_boundary() {
        let p = Park::mlb_average();
        let (c, h) = p.fence_at(0.0);
        assert!((c - 404.0 * FT).abs() < 1e-9 && (h - 8.5 * FT).abs() < 1e-9);
        let (l, _) = p.fence_at(-FRAC_PI_4);
        assert!((l - 330.0 * FT).abs() < 1e-9);
        let (mid, _) = p.fence_at(FRAC_PI_4 / 4.0);
        assert!(mid < c && mid > 377.0 * FT);
        // Foul territory narrows from the plate area toward the corner; the backstop is close.
        let (side, _) = p.boundary(FRAC_PI_2);
        assert!(side > 15.0 && side < 30.0, "beside the plate: {side}");
        let (back, _) = p.boundary(PI);
        assert!((back - p.backstop).abs() < 1e-9);
        let (just_foul, _) = p.boundary(FRAC_PI_4 + 0.02);
        assert!((just_foul - 330.0 * FT).abs() < 1e-6);
    }

    #[test]
    fn surfaces() {
        let p = Park::mlb_average();
        assert_eq!(p.surface_kind_at(Vec3::new(0.0, 2.0, 0.0)), SurfaceKind::Dirt); // plate area
        assert_eq!(p.surface_kind_at(Vec3::new(0.0, 10.0, 0.0)), SurfaceKind::Grass); // infield grass
        assert_eq!(p.surface_kind_at(Vec3::new(8.0, 36.0, 0.0)), SurfaceKind::Dirt); // behind second base side
        assert_eq!(p.surface_kind_at(Vec3::new(0.0, 80.0, 0.0)), SurfaceKind::Grass); // outfield
        assert!(is_fair(Vec3::new(10.0, 10.0, 0.0)) && !is_fair(Vec3::new(10.1, 10.0, 0.0)));
    }
}
