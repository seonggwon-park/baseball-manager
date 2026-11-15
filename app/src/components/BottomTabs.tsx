import { useEffect, useRef, useState } from "react";
import { innings, useLabels } from "../labels";
import { useUi } from "../store";
import { POSITION_CODES } from "../types";

type Tab = "log" | "box" | "lineup";

function Log() {
  const ui = useUi();
  const { message } = useLabels(ui.view);
  const end = useRef<HTMLDivElement>(null);
  useEffect(() => {
    end.current?.scrollIntoView({ block: "end" });
  }, [ui.log.length]);
  return (
    <div className="log">
      {ui.log.map((e) => (
        <div key={e.id} className={`entry ${e.cls}`}>
          {e.msgs.map((m, i) => (
            <span key={i}>{message(m)}</span>
          ))}
        </div>
      ))}
      <div ref={end} />
    </div>
  );
}

function BoxScore() {
  const ui = useUi();
  const { t, loc, player, teamName } = useLabels(ui.view);
  const state = ui.state!;
  const result = state.result;
  const mark = (team: number, idx: number) => {
    if (!result) return "";
    if (result.win?.[0] === team && result.win[1] === idx) return ` (${t("box.win")})`;
    if (result.loss?.[0] === team && result.loss[1] === idx) return ` (${t("box.loss")})`;
    if (result.save?.[0] === team && result.save[1] === idx) return ` (${t("box.save")})`;
    return "";
  };
  return (
    <div className="box">
      {[0, 1].map((team) => {
        const b = state.boxes[team];
        const order = state.lineups[team];
        return (
          <div key={team} className="box-team">
            <h4>
              {teamName(team)} <span className="muted small">{t("box.lob", { n: b.left_on_base })}</span>
            </h4>
            <table>
              <thead>
                <tr>
                  <th className="left">{t("box.batting")}</th>
                  {["ab", "r", "h", "rbi", "bb", "so", "hr", "sb"].map((k) => (
                    <th key={k}>{t(`box.${k}`)}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {b.batting.map((l) => {
                  const p = player(team, l.player);
                  const starter = order.includes(l.player) || l.pa > 0;
                  return (
                    <tr key={l.player} className={order.includes(l.player) ? "" : "replaced"}>
                      <td className="left">
                        <span className="muted small">{t(`posShort.${p.position}`)}</span> {loc(p.name)}
                      </td>
                      {starter ? [l.ab, l.r, l.h, l.rbi, l.bb, l.so, l.hr, l.sb].map((v, i) => <td key={i}>{v}</td>) : <td colSpan={8} />}
                    </tr>
                  );
                })}
              </tbody>
            </table>
            <table>
              <thead>
                <tr>
                  <th className="left">{t("box.pitching")}</th>
                  {["ip", "h", "r", "er", "bb", "so", "hr", "np"].map((k) => (
                    <th key={k}>{t(`box.${k}`)}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {b.pitching.map((l) => (
                  <tr key={l.player}>
                    <td className="left">
                      {loc(player(team, l.player).name)}
                      {mark(team, l.player)}
                    </td>
                    {[innings(l.outs), l.h, l.r, l.er, l.bb, l.so, l.hr, l.pitches].map((v, i) => (
                      <td key={i}>{v}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
      })}
    </div>
  );
}

function Lineups() {
  const ui = useUi();
  const { t, loc, player, teamName } = useLabels(ui.view);
  const view = ui.view!;
  const state = ui.state!;
  return (
    <div className="lineups">
      {[0, 1].map((team) => {
        const order = state.lineups[team];
        const defense = state.defense[team];
        const inGame = new Set([...order, ...defense]);
        const used = new Set(state.boxes[team].batting.map((l) => l.player).concat(state.boxes[team].pitching.map((l) => l.player)));
        const others = view.teams[team].players.filter((p) => !inGame.has(p.idx));
        const posOf = (idx: number) => {
          const i = defense.indexOf(idx);
          return i >= 0 ? POSITION_CODES[i] : "DH";
        };
        return (
          <div key={team} className="lineup-team">
            <h4>{teamName(team)}</h4>
            <ol>
              {order.map((idx, slot) => {
                const p = player(team, idx);
                const up = state.situation.batting === team && state.batting_slot[team] === slot && !state.situation.over;
                return (
                  <li key={slot} className={up ? "up" : ""}>
                    <span className="pos">{t(`posShort.${posOf(idx)}`)}</span>
                    <span className="num">{p.number}</span> {loc(p.name)}
                    <span className="muted small"> {t(`hand.${p.bats}`)}</span>
                  </li>
                );
              })}
              <li className="pitcher-row">
                <span className="pos">{t("posShort.P")}</span>
                <span className="num">{player(team, defense[0]).number}</span> {loc(player(team, defense[0]).name)}
              </li>
            </ol>
            {(["bench", "bullpen"] as const).map((kind) => {
              const list = others.filter((p) => (kind === "bullpen" ? p.pitching != null : p.pitching == null));
              return (
                <div key={kind} className="reserve">
                  <span className="muted small">{t(`lineup.${kind}`)}</span>
                  {list.map((p) => (
                    <span key={p.idx} className={used.has(p.idx) ? "tag used" : "tag"} title={used.has(p.idx) ? t("lineup.used") : undefined}>
                      {loc(p.name)}
                    </span>
                  ))}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

export function BottomTabs() {
  const { t } = useLabels(null);
  const [tab, setTab] = useState<Tab>("log");
  return (
    <section className="panel bottom">
      <div className="tabs" role="tablist">
        {(["log", "box", "lineup"] as Tab[]).map((k) => (
          <button key={k} role="tab" aria-selected={tab === k} className={tab === k ? "on" : ""} onClick={() => setTab(k)}>
            {t(`tabs.${k}`)}
          </button>
        ))}
      </div>
      <div className="tab-body" key={tab}>
        {tab === "log" && <Log />}
        {tab === "box" && <BoxScore />}
        {tab === "lineup" && <Lineups />}
      </div>
    </section>
  );
}
