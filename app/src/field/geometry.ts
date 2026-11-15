// Field geometry in metres (engine coordinates: origin at the back tip of home
// plate, x toward first base, y toward the pitcher).

import type { ParkView } from "../types";

export const FT = 0.3048;
export const BASE_PATH = 90 * FT;
const S = BASE_PATH * Math.SQRT1_2;

/** Home, first, second, third, home again (index = base number, 4 = scoring). */
export const BASES: [number, number][] = [
  [0, 0],
  [S, S],
  [0, 2 * S],
  [-S, S],
  [0, 0],
];

export const MOUND: [number, number] = [0, 60.5 * FT];
/** Radius of the infield skin, measured from the pitching rubber. */
export const INFIELD_ARC = 95 * FT;

/** Fence distance at a bearing (rad, 0 = centre field, + toward right field), by linear interpolation. */
export function fenceAt(park: ParkView, angle: number): number {
  const f = park.fence;
  if (angle <= f[0][0]) return f[0][1];
  for (let i = 1; i < f.length; i++) {
    if (angle <= f[i][0]) {
      const [a0, d0] = f[i - 1];
      const [a1, d1] = f[i];
      return d0 + ((d1 - d0) * (angle - a0)) / (a1 - a0);
    }
  }
  return f[f.length - 1][1];
}

/** Points along the outfield fence from the left-field line to the right-field line. */
export function fenceLine(park: ParkView, inset = 0, steps = 60): [number, number][] {
  const pts: [number, number][] = [];
  for (let i = 0; i <= steps; i++) {
    const a = -Math.PI / 4 + (i / steps) * (Math.PI / 2);
    const d = fenceAt(park, a) - inset;
    pts.push([d * Math.sin(a), d * Math.cos(a)]);
  }
  return pts;
}

/** Outline of the whole playing area, foul territory included. */
export function parkOutline(park: ParkView): [number, number][] {
  const fence = fenceLine(park);
  const left = fence[0];
  const right = fence[fence.length - 1];
  const far = park.foul_width_far;
  const near = park.foul_width_near;
  const k = Math.SQRT1_2;
  // Offsets perpendicular to the foul lines, away from fair territory.
  return [
    ...fence,
    [right[0] + far * k, right[1] - far * k],
    [near * 1.6, -park.backstop * 0.55],
    [near * 0.7, -park.backstop],
    [-near * 0.7, -park.backstop],
    [-near * 1.6, -park.backstop * 0.55],
    [left[0] - far * k, left[1] - far * k],
  ];
}

/** The infield skin: the part of the circle around the rubber that lies in fair territory. */
export function infieldSkin(steps = 48): [number, number][] {
  const [mx, my] = MOUND;
  // Intersections of the circle with the foul lines y = |x|.
  const pts: [number, number][] = [[0, 0]];
  const onLine = (sign: number): [number, number] => {
    // Solve (t*sign - mx)^2 + (t - my)^2 = R^2 for the far intersection.
    const b = -2 * my;
    const c = my * my - INFIELD_ARC * INFIELD_ARC;
    const t = (-b + Math.sqrt(b * b - 8 * c)) / 4;
    return [sign * t, t];
  };
  const r = onLine(1);
  const l = onLine(-1);
  const a0 = Math.atan2(r[1] - my, r[0] - mx);
  let a1 = Math.atan2(l[1] - my, l[0] - mx);
  if (a1 < a0) a1 += 2 * Math.PI;
  for (let i = 0; i <= steps; i++) {
    const a = a0 + ((a1 - a0) * i) / steps;
    pts.push([mx + INFIELD_ARC * Math.cos(a), my + INFIELD_ARC * Math.sin(a)]);
  }
  return pts;
}

/** The grass inside the base paths. */
export function infieldGrass(): [number, number][] {
  const inset = 2.6;
  const c = S;
  const h = S - inset;
  return [
    [0, c - h],
    [h, c],
    [0, c + h],
    [-h, c],
  ];
}

/** Bearing sector of a spot on the field, for commentary. */
export function sector(x: number, y: number): "left" | "left_center" | "center" | "right_center" | "right" {
  const deg = (Math.atan2(x, y) * 180) / Math.PI;
  if (deg < -27) return "left";
  if (deg < -9) return "left_center";
  if (deg < 9) return "center";
  if (deg < 27) return "right_center";
  return "right";
}

/** Position of a runner who is `f` (0..1) of the way from base `from` to base `to` along the base paths. */
export function alongBases(from: number, to: number, f: number): [number, number] {
  if (to <= from) return BASES[from];
  const total = to - from;
  const d = Math.min(Math.max(f, 0), 1) * total;
  const leg = Math.min(Math.floor(d), total - 1);
  const u = d - leg;
  const a = BASES[from + leg];
  const b = BASES[from + leg + 1];
  return [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
}
