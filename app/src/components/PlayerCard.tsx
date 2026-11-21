import { useLabels } from "../labels";
import { store, useUi } from "../store";
import type { AttrGroup } from "../types";
import { tier } from "./Matchup";

const PITCHER_GROUPS: AttrGroup[] = ["pitching", "mental", "physical", "fielding"];
const CATCHER_GROUPS: AttrGroup[] = ["batting", "catching", "fielding", "running", "mental", "physical"];
const HITTER_GROUPS: AttrGroup[] = ["batting", "fielding", "running", "mental", "physical"];

/** A player's profile: who he is and every attribute, grouped. */
export function PlayerCard() {
  const ui = useUi();
  const { t, loc, teamName } = useLabels(ui.view);
  if (!ui.card || !ui.view) return null;
  const [team, idx] = ui.card;
  const p = ui.view.teams[team].players[idx];
  const scale = ui.scale;
  const pitcher = p.repertoire.length > 0;
  const groups = pitcher ? PITCHER_GROUPS : p.position === "C" ? CATCHER_GROUPS : HITTER_GROUPS;
  const shown = (a: { r20: number; r80: number }) => (scale === "20" ? a.r20 : a.r80);
  const close = () => store.showCard(null);
  return (
    <div className="modal-back" onClick={close}>
      <div className="modal card" role="dialog" aria-label={loc(p.name)} onClick={(e) => e.stopPropagation()}>
        <header className="card-head">
          <div>
            <div className="card-name">
              <span className="num">{p.number}</span> {loc(p.name)}
            </div>
            <div className="muted small">
              {teamName(team)} · {t(`pos.${p.position}`)} · {t("hand.throws", { hand: t(`hand.${p.throws}`) })} {t("hand.bats", { hand: t(`hand.${p.bats}`) })} ·{" "}
              {t("card.body", { cm: p.height_cm, kg: p.weight_kg })}
            </div>
          </div>
          <button className="ghost" onClick={close}>
            {t("common.close")}
          </button>
        </header>
        {pitcher && (
          <table className="card-pitches">
            <thead>
              <tr>
                <th>{t("card.pitch")}</th>
                <th>{t("card.speed")}</th>
                <th>{t("card.usage")}</th>
                <th>{t("card.grade")}</th>
              </tr>
            </thead>
            <tbody>
              {p.repertoire.map((r) => (
                <tr key={r.kind}>
                  <td>{t(`pitch.${r.kind}`)}</td>
                  <td>{Math.round(r.speed_kmh)} km/h</td>
                  <td>{Math.round(r.usage * 100)}%</td>
                  <td>
                    <b className={tier(shown(r), scale)}>{shown(r)}</b>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div className="card-groups">
          {groups.map((g) => (
            <section key={g}>
              <h5>{t(`attrGroup.${g}`)}</h5>
              {p.attrs
                .filter((a) => a.group === g)
                .map((a) => (
                  <div key={a.key} className="card-attr">
                    <span>{t(`attr.${a.key}`)}</span>
                    <b className={tier(shown(a), scale)}>{shown(a)}</b>
                  </div>
                ))}
            </section>
          ))}
        </div>
        <div className="muted tiny">{t(scale === "20" ? "card.scale20" : "card.scale80")}</div>
      </div>
    </div>
  );
}
