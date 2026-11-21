// The state of the interface: the match as received from the engine, what is
// shown while events are being played back, and the user's standing orders.

import { useSyncExternalStore } from "react";
import { api, ApiError } from "./api";
import { eventEntry, pitchEntry, plateAppearanceEntry, type LogEntry } from "./commentary";
import type { FieldScene, IdleState } from "./field/scene";
import type { DefenseOrders, Delegate, EventView, MatchState, MatchView, Mode, NewMatch, OffenseOrders, PitchView, RatingScale, SubRequest } from "./types";

/** The situation as shown while a batch of events is played back. */
export interface Live {
  inning: number;
  top: boolean;
  outs: number;
  balls: number;
  strikes: number;
  score: [number, number];
  bases: (number | null)[];
  batter: number;
  pitcher: number;
}

export interface Ui {
  view: MatchView | null;
  state: MatchState | null;
  live: Live | null;
  log: LogEntry[];
  /** Pitches of the plate appearance being shown. */
  paPitches: PitchView[];
  busy: boolean;
  starting: boolean;
  auto: boolean;
  speed: number;
  skip: boolean;
  offense: OffenseOrders;
  defense: DefenseOrders;
  delegate: Delegate;
  /** Translation key and message of the last error. */
  error: { code: string; message: string } | null;
  /** Translation key of the result flashed over the field. */
  banner: string | null;
  /** Scale on which ratings are shown. */
  scale: RatingScale;
  /** Player whose profile is open: `[team, index]`. */
  card: [number, number] | null;
}

function initialScale(): RatingScale {
  try {
    return localStorage.getItem("bm.scale") === "80" ? "80" : "20";
  } catch {
    return "20";
  }
}

const defaultOffense = (): OffenseOrders => ({ steal: "auto" });
const defaultDefense = (): DefenseOrders => ({ intentional_walk: false });
export const defaultDelegate = (): Delegate => ({ pitching: false, bench: false, offense: false, defense: true });

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

function liveFrom(state: MatchState): Live {
  const s = state.situation;
  return { inning: s.inning, top: s.top, outs: s.outs, balls: s.balls, strikes: s.strikes, score: s.score, bases: [...s.bases], batter: s.batter, pitcher: s.pitcher };
}

class MatchStore {
  private ui: Ui = {
    view: null,
    state: null,
    live: null,
    log: [],
    paPitches: [],
    busy: false,
    starting: false,
    auto: false,
    speed: 1,
    skip: false,
    offense: defaultOffense(),
    defense: defaultDefense(),
    delegate: defaultDelegate(),
    error: null,
    banner: null,
    scale: initialScale(),
    card: null,
  };
  private listeners = new Set<() => void>();
  private scene: FieldScene | null = null;
  private pitchNo = 0;
  private lastPitch: PitchView | null = null;
  private autoTimer: ReturnType<typeof setTimeout> | null = null;
  private bannerTimer: ReturnType<typeof setTimeout> | null = null;

  subscribe = (fn: () => void) => {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  };

  get = () => this.ui;

  private set(patch: Partial<Ui>) {
    this.ui = { ...this.ui, ...patch };
    this.listeners.forEach((fn) => fn());
  }

  // ---- preferences and the player profile ----

  toggleScale() {
    const scale: RatingScale = this.ui.scale === "20" ? "80" : "20";
    try {
      localStorage.setItem("bm.scale", scale);
    } catch {
      // Storage can be unavailable; the choice then lasts for this session only.
    }
    this.set({ scale });
  }

  showCard(card: [number, number] | null) {
    this.set({ card });
  }

  // ---- scene ----

  attachScene(scene: FieldScene | null) {
    this.scene = scene;
    if (scene) {
      scene.speed = this.ui.speed;
      this.syncScene();
    }
  }

  private idle(state: MatchState, live: Live): IdleState | null {
    const view = this.ui.view;
    if (!view) return null;
    const s = state.situation;
    const bat = view.teams[s.batting];
    const batter = bat.players[live.batter];
    // A switch hitter stands on the side opposite to the pitcher's arm.
    const pitcher = view.teams[s.fielding].players[live.pitcher];
    const side = batter.bats === "S" ? (pitcher.throws === "R" ? "L" : "R") : batter.bats;
    return {
      positions: state.positions,
      runners: live.bases.map((p) => (p == null ? null : bat.players[p].number)),
      batter: s.over ? null : { number: batter.number, hand: side },
      fielding: { primary: view.teams[s.fielding].club.colors[0], secondary: view.teams[s.fielding].club.colors[1] },
      batting: { primary: bat.club.colors[1], secondary: bat.club.colors[0] },
    };
  }

  private syncScene() {
    const { view, state, live } = this.ui;
    if (!this.scene || !view || !state || !live) return;
    this.scene.setPark(view.park);
    const idle = this.idle(state, live);
    if (idle) this.scene.showIdle(idle);
  }

  // ---- match ----

  async start(cfg: NewMatch) {
    if (this.ui.starting) return;
    this.stopAuto();
    this.set({ starting: true, error: null });
    try {
      const { view, state } = await api.newMatch(cfg);
      this.pitchNo = 0;
      this.lastPitch = null;
      this.set({
        view,
        state,
        live: liveFrom(state),
        log: [],
        paPitches: [],
        offense: defaultOffense(),
        defense: defaultDefense(),
        delegate: cfg.delegate,
        starting: false,
        auto: false,
        banner: null,
      });
      this.syncScene();
    } catch (e) {
      this.fail(e);
      this.set({ starting: false });
    }
  }

  leave() {
    this.stopAuto();
    this.scene?.finish();
    this.set({ view: null, state: null, live: null, log: [], paPitches: [], busy: false, banner: null, error: null });
  }

  private fail(e: unknown) {
    const err = e instanceof ApiError ? { code: e.code, message: e.message } : { code: "generic", message: String(e) };
    this.set({ error: err });
  }

  clearError() {
    this.set({ error: null });
  }

  private flash(key: string) {
    if (this.bannerTimer) clearTimeout(this.bannerTimer);
    this.set({ banner: key });
    this.bannerTimer = setTimeout(() => this.set({ banner: null }), 1100 / this.ui.speed);
  }

  private pushLog(entry: LogEntry | null) {
    if (!entry) return;
    const log = this.ui.log.length > 600 ? this.ui.log.slice(-500) : this.ui.log;
    this.set({ log: [...log, entry] });
  }

  private async play(ev: EventView, animate: boolean) {
    const live = { ...this.ui.live! };
    const pause = (ms: number) => (animate ? sleep(ms / this.ui.speed) : Promise.resolve());
    switch (ev.type) {
      case "half_inning":
        live.inning = ev.inning;
        live.top = ev.top;
        live.outs = 0;
        live.balls = 0;
        live.strikes = 0;
        live.bases = [null, null, null];
        this.set({ live, paPitches: [] });
        this.pushLog(eventEntry(ev, live.score));
        await pause(350);
        break;
      case "pitch": {
        if (this.pitchNo === 0) this.set({ paPitches: [] });
        this.pitchNo += 1;
        live.batter = ev.batter;
        live.pitcher = ev.pitcher;
        this.set({ live });
        if (animate && this.scene) {
          await this.scene.playPitch(ev, {
            basesBefore: live.bases,
            numberOf: (p) => this.ui.view!.teams[ev.top ? 0 : 1].players[p].number,
          });
        }
        const bunt = ev.approach === "sac_bunt" || ev.approach === "bunt_for_hit";
        if (ev.result === "ball") live.balls += 1;
        else if (ev.result === "called_strike" || ev.result === "swinging_strike" || ev.result === "foul_tip") live.strikes += 1;
        else if (ev.result === "foul" && (live.strikes < 2 || bunt)) live.strikes += 1;
        if (ev.result === "in_play" && ev.play) {
          live.bases = [...ev.play.bases_after];
          live.outs = Math.min(live.outs + ev.play.outs, 3);
        }
        this.lastPitch = ev;
        this.set({ live, paPitches: [...this.ui.paPitches, ev] });
        this.pushLog(pitchEntry(ev, this.pitchNo));
        if (animate && ev.result !== "in_play") this.flash(`result.${ev.result}`);
        await pause(ev.result === "in_play" ? 150 : 250);
        break;
      }
      case "plate_appearance": {
        live.outs = ev.outs_after;
        live.score = ev.score_after;
        live.balls = 0;
        live.strikes = 0;
        this.pitchNo = 0;
        this.set({ live });
        const last = this.lastPitch && this.lastPitch.batter === ev.batter && this.lastPitch.top === ev.top ? this.lastPitch : null;
        this.pushLog(plateAppearanceEntry(ev, last, live.score));
        await pause(450);
        break;
      }
      case "steal": {
        const from = ev.to_base - 2;
        live.bases = [...live.bases];
        // On a double steal the trailing runner moves up behind the lead runner.
        if (from === 1 && live.bases[0] != null && live.bases[0] !== ev.runner) {
          live.bases[1] = live.bases[0];
          live.bases[0] = null;
        } else {
          live.bases[from] = null;
        }
        if (ev.safe) live.bases[from + 1] = ev.runner;
        else live.outs = Math.min(live.outs + 1, 3);
        this.set({ live });
        this.pushLog(eventEntry(ev, live.score));
        await pause(450);
        break;
      }
      case "wild_pitch":
        live.bases = [null, live.bases[0], live.bases[1]];
        this.set({ live });
        this.pushLog(eventEntry(ev, live.score));
        await pause(350);
        break;
      case "intentional_walk":
        break;
      case "substitution":
      case "game_over":
        this.pushLog(eventEntry(ev, live.score));
        await pause(300);
        break;
    }
  }

  async advance(mode: Mode) {
    const { state, busy } = this.ui;
    if (busy || !state || state.situation.over) return;
    this.set({ busy: true, error: null });
    try {
      const can = state.can;
      // Orders go with a pitch or a plate appearance; in the quick modes the bench decides.
      const stepwise = mode === "pitch" || mode === "plate_appearance";
      const res = await api.advance({
        mode,
        offense: stepwise && can.offense ? this.ui.offense : undefined,
        defense: stepwise && can.defense ? this.ui.defense : undefined,
      });
      const animate = !this.ui.skip && stepwise;
      for (const ev of res.events) await this.play(ev, animate);
      const live = liveFrom(res.state);
      const patch: Partial<Ui> = { state: res.state, live, busy: false };
      // Orders are for one plate appearance; an intentional walk and a steal sign for one pitch.
      if (!res.state.situation.in_plate_appearance) {
        patch.offense = defaultOffense();
        patch.defense = defaultDefense();
      } else {
        patch.offense = { ...this.ui.offense, steal: this.ui.offense.steal === "go" ? "auto" : this.ui.offense.steal };
        patch.defense = { ...this.ui.defense, intentional_walk: false };
      }
      if (res.state.situation.over) patch.auto = false;
      this.set(patch);
      this.syncScene();
    } catch (e) {
      this.fail(e);
      this.set({ busy: false, auto: false });
    }
    this.scheduleAuto();
  }

  private scheduleAuto() {
    if (this.autoTimer) clearTimeout(this.autoTimer);
    const { auto, state } = this.ui;
    if (!auto || !state || state.situation.over) return;
    this.autoTimer = setTimeout(() => void this.advance("pitch"), this.ui.skip ? 30 : 350 / this.ui.speed);
  }

  private stopAuto() {
    if (this.autoTimer) clearTimeout(this.autoTimer);
    this.autoTimer = null;
    if (this.ui.auto) this.set({ auto: false });
  }

  setAuto(on: boolean) {
    if (!on) {
      this.stopAuto();
      return;
    }
    this.set({ auto: true });
    if (!this.ui.busy) void this.advance("pitch");
  }

  setSpeed(speed: number) {
    this.set({ speed });
    if (this.scene) this.scene.speed = speed;
  }

  setSkip(skip: boolean) {
    this.set({ skip });
    if (skip) this.scene?.finish();
  }

  setOffense(patch: Partial<OffenseOrders>) {
    this.set({ offense: { ...this.ui.offense, ...patch } });
  }

  setDefense(patch: Partial<DefenseOrders>) {
    this.set({ defense: { ...this.ui.defense, ...patch } });
  }

  async substitute(req: SubRequest) {
    if (this.ui.busy) return;
    this.set({ busy: true, error: null });
    try {
      const state = await api.substitute(req);
      this.set({ state, live: liveFrom(state), busy: false });
      this.syncScene();
    } catch (e) {
      this.fail(e);
      this.set({ busy: false });
    }
  }

  async setDelegate(delegate: Delegate) {
    this.set({ delegate });
    try {
      const state = await api.delegate(delegate);
      if (!this.ui.busy) this.set({ state });
    } catch (e) {
      this.fail(e);
    }
  }
}

export const store = new MatchStore();

export function useUi(): Ui {
  return useSyncExternalStore(store.subscribe, store.get);
}
