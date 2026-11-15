// Names and translated messages for the current language.

import { useTranslation } from "react-i18next";
import type { Msg, Param } from "./commentary";
import type { Localized, MatchView, PlayerView } from "./types";

export function useLabels(view: MatchView | null) {
  const { t, i18n } = useTranslation();
  const lang: keyof Localized = i18n.language === "en" ? "en" : "ko";
  const loc = (l: Localized) => l[lang];
  const player = (team: number, idx: number): PlayerView => view!.teams[team].players[idx];
  const playerName = (team: number, idx: number) => loc(player(team, idx).name);
  const teamName = (team: number) => {
    const c = view!.teams[team].club;
    return `${loc(c.city)} ${loc(c.name)}`;
  };
  const param = (p: Param): string | number => {
    if (typeof p !== "object") return p;
    if ("player" in p) return playerName(p.player[0], p.player[1]);
    if ("team" in p) return teamName(p.team);
    return t(p.t);
  };
  const message = (m: Msg): string => {
    const params: Record<string, string | number> = {};
    for (const [k, v] of Object.entries(m.params ?? {})) params[k] = param(v);
    return t(m.key, params);
  };
  return { t, lang, loc, player, playerName, teamName, message };
}

/** Innings pitched from outs: 7 outs -> "2.1". */
export function innings(outs: number): string {
  return `${Math.floor(outs / 3)}.${outs % 3}`;
}
