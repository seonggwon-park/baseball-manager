import { useLabels } from "../labels";
import { useUi } from "../store";

/** Line score, count, outs and the bases. */
export function Scoreboard() {
  const ui = useUi();
  const { t, loc } = useLabels(ui.view);
  const view = ui.view!;
  const state = ui.state!;
  const live = ui.live!;
  const innings = Math.max(view.rules.innings, state.line_score.length);
  const over = state.situation.over;
  const half = (i: number, team: number) => {
    const row = state.line_score[i];
    if (!row) return "";
    // The half being played (or not yet played) shows nothing until it produces a run.
    const current = i === live.inning - 1 && !over;
    if (current && team === 1 && live.top) return "";
    if (over && i === state.line_score.length - 1 && team === 1 && state.situation.score[1] > state.situation.score[0] && row[1] === 0) return "x";
    return String(row[team]);
  };
  const park = view.park;
  return (
    <div className="scoreboard">
      <div className="venue" title={t("park.title")}>
        <span>{loc(park.name)}</span>
        <span className="muted">
          {t("park.elevation", { m: Math.round(park.elevation_m) })}
          {park.turf ? ` · ${t("park.turf")}` : ""}
        </span>
      </div>
      <table className="linescore">
        <thead>
          <tr>
            <th className="team-col" />
            {Array.from({ length: innings }, (_, i) => (
              <th key={i} className={i === live.inning - 1 && !over ? "now" : ""}>
                {i + 1}
              </th>
            ))}
            <th className="total">{t("score.r")}</th>
            <th className="total">{t("score.h")}</th>
            <th className="total">{t("score.e")}</th>
          </tr>
        </thead>
        <tbody>
          {[0, 1].map((team) => {
            const club = view.teams[team].club;
            const batting = !over && state.situation.batting === team;
            return (
              <tr key={team} className={batting ? "batting" : ""}>
                <td className="team-col">
                  <span className="chip" style={{ background: club.colors[0], color: club.colors[1] }}>
                    {club.abbr}
                  </span>
                  <span className="team-name">
                    {loc(club.city)} {loc(club.name)}
                  </span>
                  {view.user_team === team && <span className="you">●</span>}
                </td>
                {Array.from({ length: innings }, (_, i) => (
                  <td key={i} className={i === live.inning - 1 && !over ? "now" : ""}>
                    {half(i, team)}
                  </td>
                ))}
                <td className="total strong">{live.score[team]}</td>
                <td className="total">{state.boxes[team].hits}</td>
                <td className="total">{state.boxes[1 - team].errors}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="situation">
        <div className="inning">
          {over ? (
            t("score.final")
          ) : (
            <>
              {t("score.inning", { n: live.inning })}
              <span className="half">{live.top ? t("score.top") : t("score.bottom")}</span>
            </>
          )}
        </div>
        <svg className="diamond" viewBox="0 0 60 48" aria-hidden="true">
          {[
            [44, 26],
            [30, 12],
            [16, 26],
          ].map(([x, y], i) => (
            <rect key={i} x={x - 7} y={y - 7} width="14" height="14" transform={`rotate(45 ${x} ${y})`} className={live.bases[i] != null ? "base on" : "base"} />
          ))}
        </svg>
        <div className="count">
          {(
            [
              ["ball", live.balls, 3],
              ["strike", live.strikes, 2],
              ["out", live.outs, 2],
            ] as const
          ).map(([k, n, max]) => (
            <div key={k} className={`dots ${k}`}>
              <span className="lbl">{t(`score.${k}`)}</span>
              {Array.from({ length: max }, (_, i) => (
                <i key={i} className={i < n ? "on" : ""} />
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
