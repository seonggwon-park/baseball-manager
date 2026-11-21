// Turns engine events into play-by-play entries. Entries hold translation keys
// and parameters, not text, so the log follows the interface language.

import type { EventView, PitchView, PlateAppearanceView } from "./types";
import { POSITION_CODES } from "./types";
import { sector } from "./field/geometry";

/** A parameter that is resolved when the entry is rendered. */
export type Param = string | number | { player: [number, number] } | { team: number } | { t: string };

export interface Msg {
  key: string;
  params?: Record<string, Param>;
}

export type LogClass = "inning" | "pitch" | "play" | "score" | "note" | "final";

export interface LogEntry {
  id: number;
  cls: LogClass;
  msgs: Msg[];
}

let nextId = 1;
const entry = (cls: LogClass, ...msgs: Msg[]): LogEntry => ({ id: nextId++, cls, msgs });

export function halfInningEntry(inning: number, top: boolean): LogEntry {
  return entry("inning", { key: top ? "log.halfTop" : "log.halfBottom", params: { n: inning, team: { team: top ? 0 : 1 } } });
}

export function pitchEntry(p: PitchView, n: number): LogEntry {
  const bunt = p.approach === "sac_bunt" || p.approach === "bunt_for_hit";
  let result = `log.${p.result}`;
  if (bunt && p.result === "foul") result = "log.buntFoul";
  if (bunt && p.result === "swinging_strike") result = "log.buntMiss";
  const msgs: Msg[] = [{ key: "log.pitch", params: { n, kind: { t: `pitch.${p.kind}` }, speed: Math.round(p.speed_kmh) } }];
  if (p.result !== "in_play") msgs.push({ key: result });
  return entry("pitch", ...msgs);
}

/** The outcome of a plate appearance; `last` is its final pitch. */
export function plateAppearanceEntry(pa: PlateAppearanceView, last: PitchView | null, score: [number, number]): LogEntry | null {
  if (pa.outcome === "interrupted") return null;
  const team = pa.top ? 0 : 1;
  const name: Param = { player: [team, pa.batter] };
  const play = last?.play ?? null;
  const batted = last?.batted ?? null;
  const pos: Param = { t: `pos.${POSITION_CODES[play?.fielder ?? 0]}` };
  const spot = play ? sector(play.field_pos[0], play.field_pos[1]) : "center";
  const dir: Param = { t: `dir.${spot}` };
  let msg: Msg;
  switch (pa.outcome) {
    case "strikeout": {
      const bunt = last != null && last.result === "foul" && (last.approach === "sac_bunt" || last.approach === "bunt_for_hit");
      msg = { key: bunt ? "log.strikeoutBunt" : last?.result === "called_strike" ? "log.strikeoutLooking" : "log.strikeout", params: { name } };
      break;
    }
    case "walk":
    case "intentional_walk":
      msg = { key: `log.${pa.outcome}`, params: { name } };
      break;
    case "hit_by_pitch":
      msg = { key: "log.hbp", params: { name } };
      break;
    case "single": {
      const infield = play != null && play.fielder <= 5 && Math.hypot(play.field_pos[0], play.field_pos[1]) < 50;
      msg = batted?.bunt ? { key: "log.buntSingle", params: { name } } : infield ? { key: "log.infieldSingle", params: { name } } : { key: "log.single", params: { name, dir } };
      break;
    }
    case "double":
    case "triple":
      msg = { key: `log.${pa.outcome}`, params: { name, dir } };
      break;
    case "home_run": {
      const end = batted?.path[batted.path.length - 1];
      const over = end ? sector(end[1], end[2]) : spot;
      msg = { key: "log.home_run", params: { name, dir: { t: `dirOver.${over}` }, dist: Math.round(batted?.distance ?? 0) } };
      break;
    }
    case "double_play":
      msg = { key: play?.gidp ? "log.double_play" : "log.lineDouble", params: { name, pos } };
      break;
    default:
      msg = { key: `log.${pa.outcome}`, params: { name, pos } };
  }
  const msgs = [msg];
  // A line or fly out on which a runner was doubled off.
  if (play && play.outs === 2 && !play.gidp && pa.outcome !== "double_play") msgs.push({ key: "log.runnerOut" });
  if (batted && !batted.bunt && last?.result === "in_play") {
    msgs.push({ key: "log.detail", params: { ev: Math.round(batted.exit_speed_kmh), la: Math.round(batted.launch_angle_deg) } });
  }
  if (pa.runs > 0) {
    msgs.push({ key: "log.runs", params: { count: pa.runs, n: pa.runs } });
    msgs.push({ key: "log.scoreNow", params: { away: { team: 0 }, a: score[0], home: { team: 1 }, h: score[1] } });
  }
  return entry(pa.runs > 0 ? "score" : "play", ...msgs);
}

export function eventEntry(ev: EventView, score: [number, number]): LogEntry | null {
  switch (ev.type) {
    case "half_inning":
      return halfInningEntry(ev.inning, ev.top);
    case "steal":
      return entry("note", { key: ev.safe ? "log.stealSafe" : "log.stealOut", params: { name: { player: [ev.team, ev.runner] }, base: ev.to_base } });
    case "wild_pitch":
      return entry("note", { key: "log.wildPitch" });
    case "substitution": {
      const key = { pitcher: "log.subPitcher", pinch_hitter: "log.subPinchHitter", pinch_runner: "log.subPinchRunner", fielder: "log.subFielder" }[ev.kind];
      return entry("note", { key, params: { old: { player: [ev.team, ev.replaced] }, new: { player: [ev.team, ev.player] } } });
    }
    case "game_over": {
      const msgs: Msg[] = [{ key: "log.gameOver", params: { away: { team: 0 }, a: score[0], home: { team: 1 }, h: score[1] } }];
      if (score[0] === score[1]) msgs.push({ key: "log.tie" });
      return entry("final", ...msgs);
    }
    default:
      return null;
  }
}
