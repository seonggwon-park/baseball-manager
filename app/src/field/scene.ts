// The 2D top-down match view (PixiJS): the park, the fielders, the runners and
// the ball, and the animation of a pitch and of the play that follows it.

// The desktop shell runs under a content security policy without `unsafe-eval`;
// this module makes PixiJS build its shader bindings without generated code.
import "pixi.js/unsafe-eval";
import { Application, Container, Graphics, Text } from "pixi.js";
import type { ParkView, PitchView, PlayView } from "../types";
import { alongBases, BASES, fenceLine, infieldGrass, infieldSkin, MOUND, parkOutline } from "./geometry";

const COLORS = {
  grass: 0x2f6f3e,
  grassDark: 0x2a6438,
  foul: 0x27573a,
  dirt: 0xa9774a,
  line: 0xf2f0e6,
  fence: 0xe0b84c,
  base: 0xffffff,
  ball: 0xffffff,
  shadow: 0x000000,
  trail: 0xfff3c4,
  out: 0xe5534b,
};

type Pt = [number, number];

interface Token {
  node: Container;
  disc: Graphics;
  label: Text;
}

interface Segment {
  t0: number;
  t1: number;
  from: Pt;
  to: Pt;
}

export interface TeamColors {
  primary: string;
  secondary: string;
}

/** Who stands where between pitches. */
export interface IdleState {
  positions: Pt[];
  /** Uniform numbers of the runners on first, second, third (null = empty). */
  runners: (number | null)[];
  batter: { number: number; hand: "R" | "L" } | null;
  fielding: TeamColors;
  batting: TeamColors;
}

/** What the scene needs to know about the players involved in a pitch. */
export interface PitchCast {
  /** Player index (batting team) of the runners on first, second, third before the pitch. */
  basesBefore: (number | null)[];
  numberOf: (player: number) => number;
}

const hex = (c: string) => parseInt(c.replace("#", ""), 16);
const lerp = (a: number, b: number, u: number) => a + (b - a) * u;
const ease = (u: number) => (u <= 0 ? 0 : u >= 1 ? 1 : u * u * (3 - 2 * u));

function segmentsAt(start: Pt, segs: Segment[], t: number): Pt {
  let pos = start;
  for (const s of segs) {
    if (t <= s.t0) break;
    const u = ease((t - s.t0) / Math.max(s.t1 - s.t0, 1e-3));
    pos = [lerp(s.from[0], s.to[0], u), lerp(s.from[1], s.to[1], u)];
  }
  return pos;
}

export class FieldScene {
  private app = new Application();
  private ready = false;
  private disposed = false;
  private ground = new Graphics();
  private labels = new Container();
  private trail = new Graphics();
  private marks = new Graphics();
  private tokens = new Container();
  private fielders: Token[] = [];
  private runners: Token[] = [];
  private ballShadow = new Graphics();
  private ball = new Graphics();
  private park: ParkView | null = null;
  private idle: IdleState | null = null;
  private positionLabels: string[] = [];
  private width = 800;
  private height = 600;
  private scale = 4;
  private ox = 400;
  private oy = 560;
  private anim: { t: number; duration: number; update: (t: number) => void; done: () => void } | null = null;
  speed = 1;

  /** Name of the renderer in use ("webgl", "webgpu" or "canvas"). */
  renderer = "";

  async init(canvas: HTMLCanvasElement, width: number, height: number) {
    // WebGL where it works, the 2D canvas renderer where it does not (remote
    // desktops, virtual machines, old graphics drivers). `VITE_RENDERER=canvas`
    // forces the fallback for testing.
    const forced = import.meta.env.VITE_RENDERER === "canvas";
    await this.app.init({
      canvas,
      width,
      height,
      antialias: true,
      backgroundAlpha: 0,
      resolution: Math.min(window.devicePixelRatio || 1, 2),
      autoDensity: true,
      preference: forced ? "canvas" : ["webgl", "canvas"],
      failIfMajorPerformanceCaveat: false,
    });
    this.renderer = this.app.renderer.name;
    canvas.dataset.renderer = this.renderer;
    if (this.disposed) {
      this.app.destroy();
      return;
    }
    this.app.stage.addChild(this.ground, this.labels, this.trail, this.marks, this.tokens);
    for (let i = 0; i < 9; i++) this.fielders.push(this.makeToken());
    for (let i = 0; i < 4; i++) this.runners.push(this.makeToken());
    this.tokens.addChild(this.ballShadow, this.ball);
    this.app.ticker.add((ticker) => this.tick(ticker.deltaMS / 1000));
    this.ready = true;
    this.resize(width, height);
  }

  destroy() {
    this.disposed = true;
    if (this.anim) {
      this.anim.done();
      this.anim = null;
    }
    if (this.ready) this.app.destroy();
  }

  private makeToken(): Token {
    const node = new Container();
    const disc = new Graphics();
    const label = new Text({ text: "", style: { fontFamily: "system-ui, 'Malgun Gothic', 'Noto Sans CJK KR', sans-serif", fontSize: 11, fontWeight: "700", fill: 0xffffff } });
    label.anchor.set(0.5);
    node.addChild(disc, label);
    node.visible = false;
    this.tokens.addChild(node);
    return { node, disc, label };
  }

  // ---- coordinates ----

  private sx(x: number) {
    return this.ox + x * this.scale;
  }

  private sy(y: number) {
    return this.oy - y * this.scale;
  }

  resize(width: number, height: number) {
    this.width = Math.max(width, 200);
    this.height = Math.max(height, 200);
    if (!this.ready) return;
    this.app.renderer.resize(this.width, this.height);
    this.layout();
  }

  setPark(park: ParkView) {
    this.park = park;
    this.layout();
  }

  setPositionLabels(labels: string[]) {
    this.positionLabels = labels;
    if (this.idle && !this.anim) this.showIdle(this.idle);
  }

  private layout() {
    if (!this.ready || !this.park) return;
    const outline = parkOutline(this.park);
    let [minX, maxX, minY, maxY] = [Infinity, -Infinity, Infinity, -Infinity];
    for (const [x, y] of outline) {
      minX = Math.min(minX, x);
      maxX = Math.max(maxX, x);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
    }
    const pad = 14;
    this.scale = Math.min((this.width - 2 * pad) / (maxX - minX), (this.height - 2 * pad) / (maxY - minY));
    this.ox = this.width / 2 - ((minX + maxX) / 2) * this.scale;
    this.oy = this.height / 2 + ((minY + maxY) / 2) * this.scale;
    this.drawGround();
    if (this.idle && !this.anim) this.showIdle(this.idle);
  }

  private poly(points: Pt[]): number[] {
    const out: number[] = [];
    for (const [x, y] of points) out.push(this.sx(x), this.sy(y));
    return out;
  }

  private drawGround() {
    const park = this.park!;
    const g = this.ground;
    const s = this.scale;
    g.clear();
    // Playing area with foul territory, then fair territory on top.
    g.poly(this.poly(parkOutline(park))).fill(COLORS.foul);
    const fence = fenceLine(park);
    g.poly(this.poly([[0, 0], ...fence])).fill(COLORS.grass);
    // Mowing stripes: alternating rings.
    for (let r = 50; r < 130; r += 12) {
      const ring: Pt[] = [];
      for (const [x, y] of fence) {
        const d = Math.hypot(x, y);
        const k = Math.min(r + 6, d) / d;
        ring.push([x * k, y * k]);
      }
      const inner: Pt[] = [];
      for (const [x, y] of [...fence].reverse()) {
        const d = Math.hypot(x, y);
        const k = Math.min(r, d) / d;
        inner.push([x * k, y * k]);
      }
      g.poly(this.poly([...ring, ...inner])).fill({ color: COLORS.grassDark, alpha: 0.55 });
    }
    // Warning track.
    const inner = fenceLine(park, 4.5).reverse();
    g.poly(this.poly([...fence, ...inner])).fill(COLORS.dirt);
    // Infield skin, the grass inside the base paths, the mound and the plate area.
    g.poly(this.poly(infieldSkin())).fill(COLORS.dirt);
    g.poly(this.poly(infieldGrass())).fill(COLORS.grass);
    g.circle(this.sx(MOUND[0]), this.sy(MOUND[1]), 2.74 * s).fill(COLORS.dirt);
    g.circle(this.sx(0), this.sy(0), 4.0 * s).fill(COLORS.dirt);
    // Foul lines and the fence.
    const left = fence[0];
    const right = fence[fence.length - 1];
    g.moveTo(this.sx(0), this.sy(0)).lineTo(this.sx(left[0]), this.sy(left[1])).stroke({ width: 1.5, color: COLORS.line, alpha: 0.9 });
    g.moveTo(this.sx(0), this.sy(0)).lineTo(this.sx(right[0]), this.sy(right[1])).stroke({ width: 1.5, color: COLORS.line, alpha: 0.9 });
    g.poly(this.poly(fence), false).stroke({ width: 3, color: COLORS.fence });
    // Bases, rubber, plate.
    const b = Math.max(0.55 * s, 3);
    for (let i = 1; i <= 3; i++) {
      const [x, y] = BASES[i];
      g.poly([this.sx(x), this.sy(y) - b, this.sx(x) + b, this.sy(y), this.sx(x), this.sy(y) + b, this.sx(x) - b, this.sy(y)]).fill(COLORS.base);
    }
    g.rect(this.sx(MOUND[0]) - b, this.sy(MOUND[1]) - b * 0.25, 2 * b, b * 0.5).fill(COLORS.base);
    g.poly([this.sx(0) - b * 0.7, this.sy(0) - b, this.sx(0) + b * 0.7, this.sy(0) - b, this.sx(0) + b * 0.7, this.sy(0) - b * 0.35, this.sx(0), this.sy(0) + b * 0.35, this.sx(0) - b * 0.7, this.sy(0) - b * 0.35]).fill(COLORS.base);

    // Distances on the fence.
    this.labels.removeChildren().forEach((c) => c.destroy());
    for (const deg of [-45, -22.5, 0, 22.5, 45]) {
      const a = (deg * Math.PI) / 180;
      const i = Math.round(((deg + 45) / 90) * (fence.length - 1));
      const d = Math.hypot(fence[i][0], fence[i][1]);
      const t = new Text({ text: `${Math.round(d)}m`, style: { fontFamily: "system-ui, sans-serif", fontSize: 11, fill: COLORS.fence, fontWeight: "600" } });
      t.anchor.set(0.5);
      t.position.set(this.sx((d - 9) * Math.sin(a)), this.sy((d - 9) * Math.cos(a)));
      this.labels.addChild(t);
    }
  }

  // ---- tokens ----

  private paint(token: Token, colors: TeamColors, text: string) {
    const r = Math.min(Math.max(1.5 * this.scale, 10), 14);
    token.disc.clear();
    token.disc.circle(0, 0, r).fill(hex(colors.primary)).stroke({ width: 1.5, color: hex(colors.secondary) });
    token.label.text = text;
    token.label.style.fontSize = Math.max(Math.round(r * (text.length > 1 ? 0.82 : 1.0)), 9);
    token.label.style.fill = hex(colors.secondary);
  }

  private place(token: Token, p: Pt | null) {
    if (!p) {
      token.node.visible = false;
      return;
    }
    token.node.visible = true;
    token.node.position.set(this.sx(p[0]), this.sy(p[1]));
  }

  private drawBall(p: [number, number, number] | null) {
    this.ball.clear();
    this.ballShadow.clear();
    if (!p) return;
    const [x, y, z] = p;
    const r = Math.max(0.42 * this.scale, 3) * (1 + Math.min(z, 30) * 0.02);
    this.ballShadow.ellipse(this.sx(x), this.sy(y), r * 0.9, r * 0.6).fill({ color: COLORS.shadow, alpha: 0.35 });
    this.ball.circle(this.sx(x), this.sy(y) - z * this.scale * 0.5, r).fill(COLORS.ball).stroke({ width: 1, color: 0x9a1b1b });
  }

  /** Where the batter is drawn: beside the plate, far enough out not to cover the catcher. */
  private batterSpot(hand: "R" | "L"): Pt {
    const off = Math.max(1.0, 15 / this.scale);
    return [hand === "R" ? -off : off, 0.6];
  }

  /** Where a fielder is drawn; the catcher is pushed back a little so that his disc clears the plate. */
  private shown(i: number, p: Pt): Pt {
    return i === 1 ? [p[0], Math.min(p[1], -14 / this.scale)] : p;
  }

  /** Everybody in place for the next pitch. */
  showIdle(idle: IdleState) {
    this.idle = idle;
    if (!this.ready || this.anim) return;
    this.trail.clear();
    this.marks.clear();
    idle.positions.forEach((p, i) => {
      this.paint(this.fielders[i], idle.fielding, this.positionLabels[i] ?? "");
      this.place(this.fielders[i], this.shown(i, p));
    });
    for (let i = 0; i < 3; i++) {
      const n = idle.runners[i];
      if (n == null) {
        this.place(this.runners[i], null);
      } else {
        this.paint(this.runners[i], idle.batting, String(n));
        this.place(this.runners[i], BASES[i + 1]);
      }
    }
    if (idle.batter) {
      this.paint(this.runners[3], idle.batting, String(idle.batter.number));
      this.place(this.runners[3], this.batterSpot(idle.batter.hand));
    } else {
      this.place(this.runners[3], null);
    }
    this.drawBall(null);
  }

  // ---- animation ----

  private tick(dt: number) {
    const a = this.anim;
    if (!a) return;
    a.t = Math.min(a.t + dt * this.speed, a.duration);
    a.update(a.t);
    if (a.t >= a.duration) {
      this.anim = null;
      a.done();
    }
  }

  /** Stop the running animation and jump to its end. */
  finish() {
    const a = this.anim;
    if (!a) return;
    this.anim = null;
    a.update(a.duration);
    a.done();
  }

  /** Animate a pitch and, if the ball was put in play, the play. Resolves when it is over. */
  playPitch(p: PitchView, cast: PitchCast): Promise<void> {
    if (!this.ready || !this.idle) return Promise.resolve();
    this.finish();
    const idle = this.idle;
    this.trail.clear();
    this.marks.clear();
    const T = p.flight_time;
    // Constant-acceleration path through the release point that arrives with the plate velocity.
    const acc = [0, 1, 2].map((i) => (2 * (p.release[i] - p.plate[i] + p.plate_vel[i] * T)) / (T * T));
    const pitchAt = (t: number): [number, number, number] => {
      const u = T - Math.min(t, T);
      return [0, 1, 2].map((i) => p.plate[i] - p.plate_vel[i] * u + 0.5 * acc[i] * u * u) as [number, number, number];
    };
    p.positions.forEach((pos, i) => {
      this.paint(this.fielders[i], idle.fielding, this.positionLabels[i] ?? "");
      this.place(this.fielders[i], this.shown(i, pos));
    });

    const play = p.result === "in_play" || (p.batted && p.play) ? p.play : null;
    const batted = p.batted;
    let update: (t: number) => void;
    let duration: number;
    if (batted && batted.path.length > 1) {
      const live = play && play.kind !== "foul" ? play : null;
      const after = this.buildPlay(p, live, cast, idle);
      duration = T + after.duration;
      update = (t) => {
        if (t < T) this.drawBall(pitchAt(t));
        else after.update(t - T);
      };
    } else {
      // No contact, or tipped back: the ball stays with the catcher for a moment.
      duration = T + 0.45;
      update = (t) => this.drawBall(t < T ? pitchAt(t) : [p.plate[0], -0.6, 0.5]);
    }
    // The phase is exposed on the canvas for automated tests of the interface.
    const canvas = this.app.canvas;
    const inPlay = batted != null && batted.path.length > 1;
    return new Promise((resolve) => {
      const phased = (t: number) => {
        canvas.dataset.phase = t < T ? "pitch" : inPlay ? "play" : "result";
        update(t);
      };
      this.anim = {
        t: 0,
        duration,
        update: phased,
        done: () => {
          delete canvas.dataset.phase;
          resolve();
        },
      };
    });
  }

  private buildPlay(p: PitchView, play: PlayView | null, cast: PitchCast, idle: IdleState) {
    const path = p.batted!.path;
    const pathEnd = path[path.length - 1][0];
    const fieldedT = play && play.fielded_t > 0 ? Math.min(play.fielded_t, pathEnd) : pathEnd;
    const ballOnPath = (t: number): [number, number, number] => {
      const tt = Math.min(t, fieldedT);
      let i = 1;
      while (i < path.length - 1 && path[i][0] < tt) i++;
      const a = path[i - 1];
      const b = path[i];
      const u = b[0] > a[0] ? Math.min(Math.max((tt - a[0]) / (b[0] - a[0]), 0), 1) : 1;
      return [lerp(a[1], b[1], u), lerp(a[2], b[2], u), Math.max(lerp(a[3], b[3], u), 0)];
    };

    // Fielders: the one who makes the play goes to the ball, the others cover bases.
    const fielderSegs: Segment[][] = p.positions.map(() => []);
    const throws = play?.throws ?? [];
    if (play && play.fielded_t > 0) {
      const f = play.fielder;
      fielderSegs[f].push({ t0: Math.min(0.2, fieldedT * 0.3), t1: fieldedT, from: p.positions[f], to: play.field_pos });
    } else if (path.length > 1) {
      // Nobody got to it (a foul, or a ball over the fence): the nearest outfielder gives chase for a few steps.
      const end = path[path.length - 1];
      let best = 6;
      for (let i = 2; i < 9; i++) {
        const d = Math.hypot(p.positions[i][0] - end[1], p.positions[i][1] - end[2]);
        if (d < Math.hypot(p.positions[best][0] - end[1], p.positions[best][1] - end[2])) best = i;
      }
      const from = p.positions[best];
      const u = 0.35;
      fielderSegs[best].push({ t0: 0.3, t1: Math.max(pathEnd, 0.8), from, to: [lerp(from[0], end[1], u), lerp(from[1], end[2], u)] });
    }
    for (const th of throws) {
      const base = BASES[th.base % 4 === 0 ? 0 : th.base];
      if (th.carried) {
        fielderSegs[th.by].push({ t0: th.t_release, t1: th.t_arrive, from: play!.field_pos, to: base });
      } else if (th.to !== th.by && fielderSegs[th.to].length === 0) {
        fielderSegs[th.to].push({ t0: 0.25, t1: Math.max(th.t_arrive - 0.15, 0.6), from: p.positions[th.to], to: base });
      }
    }
    const fielderAt = (i: number, t: number) => segmentsAt(p.positions[i], fielderSegs[i], t);

    // Ball after it has been fielded: with the fielder, then thrown from base to base.
    const ballAt = (t: number): [number, number, number] | null => {
      if (!play || play.fielded_t <= 0 || t <= fieldedT) {
        if (t > pathEnd + 0.4 && (!play || play.fielded_t <= 0)) return null;
        return ballOnPath(t);
      }
      let holder: Pt = fielderAt(play.fielder, t);
      for (const th of throws) {
        if (t < th.t_release) break;
        const base = BASES[th.base % 4 === 0 ? 0 : th.base];
        if (th.carried) {
          holder = fielderAt(th.by, t);
          continue;
        }
        if (t < th.t_arrive) {
          const u = (t - th.t_release) / Math.max(th.t_arrive - th.t_release, 1e-3);
          const from = th.from;
          return [lerp(from[0], base[0], u), lerp(from[1], base[1], u), 1.6 + 6 * u * (1 - u)];
        }
        holder = base;
      }
      return [holder[0], holder[1], 1.2];
    };

    // Runners: the batter and whoever was on base.
    interface Mover {
      token: Token;
      at: (t: number) => Pt | null;
      outAt: number | null;
      outPos: Pt;
    }
    const movers: Mover[] = [];
    const runs = play?.runs ?? [];
    const starters: { player: number; base: number; token: Token }[] = [];
    cast.basesBefore.forEach((player, i) => {
      if (player != null) starters.push({ player, base: i + 1, token: this.runners[i] });
    });
    starters.push({ player: p.batter, base: 0, token: this.runners[3] });
    for (const s of starters) {
      this.paint(s.token, idle.batting, String(cast.numberOf(s.player)));
      const run = runs.find((r) => r.player === s.player && r.from === s.base);
      const home: Pt = s.base === 0 ? this.batterSpot(p.batter_hand) : BASES[s.base];
      if (run && run.to > run.from) {
        const end = alongBases(run.from, run.to, 1);
        movers.push({
          token: s.token,
          outAt: run.out ? run.t_arrive : null,
          outPos: end,
          at: (t) => {
            if (t < run.t_start) return home;
            if (t >= run.t_arrive) return run.out || run.to >= 4 ? (t < run.t_arrive + 0.25 ? end : null) : end;
            return alongBases(run.from, run.to, (t - run.t_start) / (run.t_arrive - run.t_start));
          },
        });
      } else if (run) {
        // Caught off his base: he stays where he was and is out.
        movers.push({ token: s.token, outAt: run.t_arrive, outPos: home, at: (t) => (t < run.t_arrive + 0.25 ? home : null) });
      } else if (s.base === 0) {
        // The batter runs it out until the play is made on him (or stays on a foul).
        const foul = !play;
        const until = play && play.fielded_t > 0 ? Math.max(play.fielded_t, 1.2) : 1.5;
        movers.push({
          token: s.token,
          outAt: null,
          outPos: home,
          at: (t) => {
            if (foul) return home;
            if (t > until + 0.2) return null;
            return alongBases(0, 1, Math.min((Math.max(t - 0.3, 0) / 4.3) * 1, 0.95));
          },
        });
      } else {
        movers.push({ token: s.token, outAt: null, outPos: home, at: () => home });
      }
    }
    // Bases that were empty stay empty.
    cast.basesBefore.forEach((player, i) => {
      if (player == null) this.place(this.runners[i], null);
    });

    let last = Math.max(fieldedT, play && play.fielded_t > 0 ? 0 : pathEnd);
    for (const th of throws) last = Math.max(last, th.t_arrive);
    for (const r of runs) last = Math.max(last, Math.min(r.t_arrive, fieldedT + 9));
    const duration = Math.min(last + 0.7, 13);

    const trailPts: number[] = [];
    let trailT = -1;
    const update = (t: number) => {
      p.positions.forEach((_, i) => this.place(this.fielders[i], this.shown(i, fielderAt(i, t))));
      this.marks.clear();
      for (const m of movers) {
        this.place(m.token, m.at(t));
        if (m.outAt != null && t >= m.outAt) {
          const x = this.sx(m.outPos[0]);
          const y = this.sy(m.outPos[1]);
          const k = 7;
          this.marks.moveTo(x - k, y - k).lineTo(x + k, y + k).moveTo(x + k, y - k).lineTo(x - k, y + k).stroke({ width: 3, color: COLORS.out });
        }
      }
      const b = ballAt(t);
      this.drawBall(b);
      // Trail of the batted ball while it is on its own.
      if (b && t <= fieldedT && t - trailT > 0.03) {
        trailT = t;
        trailPts.push(this.sx(b[0]), this.sy(b[1]) - b[2] * this.scale * 0.5);
        if (trailPts.length >= 4) {
          this.trail.clear();
          this.trail.poly(trailPts, false).stroke({ width: 2, color: COLORS.trail, alpha: 0.75 });
        }
      }
    };
    return { duration, update };
  }
}
