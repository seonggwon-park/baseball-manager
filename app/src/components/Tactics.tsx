import { useState } from "react";
import { useLabels } from "../labels";
import { store, useUi } from "../store";
import type { Approach, Delegate, InfieldDepth, OutfieldDepth, Shift, StealOrder, SubRequest } from "../types";
import { POSITION_CODES } from "../types";
import { PlayerRatings } from "./Matchup";

type SubMode = "pinch_hitter" | "pinch_runner" | "pitcher" | "fielder";

function Segmented<T extends string>({ value, options, onChange, disabled }: { value: T; options: [T, string][]; onChange: (v: T) => void; disabled?: boolean }) {
  return (
    <div className="segmented wrap">
      {options.map(([v, label]) => (
        <button key={v} className={value === v ? "on" : ""} disabled={disabled} onClick={() => onChange(v)}>
          {label}
        </button>
      ))}
    </div>
  );
}

/** Orders for the user's team: tactics for the pitch, substitutions, what the bench handles. */
export function Tactics() {
  const ui = useUi();
  const { t, loc, player } = useLabels(ui.view);
  const [sub, setSub] = useState<SubMode | null>(null);
  const [target, setTarget] = useState<number | null>(null);
  const view = ui.view!;
  const state = ui.state!;
  const team = view.user_team;
  if (team == null) {
    return (
      <section className="panel tactics">
        <h3>{t("tactics.title")}</h3>
        <p className="muted">{t("tactics.watching")}</p>
      </section>
    );
  }
  const can = state.can;
  const s = state.situation;
  const locked = ui.busy || ui.auto || s.over;
  const off = ui.offense;
  const def = ui.defense;
  const bench = "bench" as const;
  const bunting = off.approach === "sac_bunt" || off.approach === "bunt_for_hit";
  const runners = s.bases.some((b) => b != null);

  const closeSub = () => {
    setSub(null);
    setTarget(null);
  };
  const doSub = (req: SubRequest) => {
    closeSub();
    void store.substitute(req);
  };
  const pool = sub === "pitcher" ? can.bullpen : can.bench;

  return (
    <section className="panel tactics">
      <h3>{t("tactics.title")}</h3>

      {can.offense && (
        <div className="group">
          <div className="group-title">{t("tactics.offense")}</div>
          <label>{t("tactics.approach")}</label>
          <Segmented<Approach | typeof bench>
            value={off.approach ?? bench}
            disabled={locked}
            onChange={(v) => store.setOffense({ approach: v === bench ? undefined : v })}
            options={[
              [bench, t("tactics.bench")],
              ["normal", t("tactics.normal")],
              ["sac_bunt", t("tactics.sac_bunt")],
              ["bunt_for_hit", t("tactics.bunt_for_hit")],
              ["hit_and_run", t("tactics.hit_and_run")],
              ["take", t("tactics.take")],
            ]}
          />
          {bunting && (
            <>
              <label>{t("tactics.buntSide")}</label>
              <Segmented<"third" | "first">
                value={off.bunt_toward_third === false ? "first" : "third"}
                disabled={locked}
                onChange={(v) => store.setOffense({ bunt_toward_third: v === "third" })}
                options={[
                  ["third", t("tactics.third")],
                  ["first", t("tactics.first")],
                ]}
              />
            </>
          )}
          {runners && (
            <>
              <label>{t("tactics.steal")}</label>
              <Segmented<StealOrder>
                value={off.steal}
                disabled={locked}
                onChange={(v) => store.setOffense({ steal: v })}
                options={[
                  ["auto", t("tactics.stealAuto")],
                  ["hold", t("tactics.stealHold")],
                  ["go", t("tactics.stealGo")],
                ]}
              />
            </>
          )}
        </div>
      )}

      {can.defense && (
        <div className="group">
          <div className="group-title">{t("tactics.defense")}</div>
          <label>{t("tactics.infield")}</label>
          <Segmented<InfieldDepth | typeof bench>
            value={def.infield ?? bench}
            disabled={locked}
            onChange={(v) => store.setDefense({ infield: v === bench ? undefined : v })}
            options={[
              [bench, t("tactics.bench")],
              ["normal", t("tactics.infieldNormal")],
              ["double_play", t("tactics.infieldDp")],
              ["in", t("tactics.infieldIn")],
              ["corners_in", t("tactics.infieldCorners")],
            ]}
          />
          <label>{t("tactics.outfield")}</label>
          <Segmented<OutfieldDepth | typeof bench>
            value={def.outfield ?? bench}
            disabled={locked}
            onChange={(v) => store.setDefense({ outfield: v === bench ? undefined : v })}
            options={[
              [bench, t("tactics.bench")],
              ["normal", t("tactics.outfieldNormal")],
              ["shallow", t("tactics.outfieldShallow")],
              ["deep", t("tactics.outfieldDeep")],
            ]}
          />
          <label>{t("tactics.shift")}</label>
          <Segmented<Shift | typeof bench>
            value={def.shift ?? bench}
            disabled={locked}
            onChange={(v) => store.setDefense({ shift: v === bench ? undefined : v })}
            options={[
              [bench, t("tactics.bench")],
              ["none", t("tactics.shiftNone")],
              ["shade", t("tactics.shiftShade")],
              ["full", t("tactics.shiftFull")],
            ]}
          />
          {def.shift === "full" && view.rules.shift_restricted && <p className="hint">{t("tactics.shiftRestricted")}</p>}
          <button
            className="wide"
            disabled={locked}
            title={t("tactics.ibbHint")}
            onClick={() => {
              store.setDefense({ intentional_walk: true });
              void store.advance("pitch");
            }}
          >
            {t("tactics.ibb")}
          </button>
        </div>
      )}

      <div className="group" hidden={s.over}>
        <div className="group-title">{t("tactics.subs")}</div>
        {can.needs_pitcher && <p className="hint warn">{t("tactics.needPitcher")}</p>}
        {!can.substitute && !s.over && <p className="hint">{t("tactics.subsOnlyBetween")}</p>}
        <div className="row">
          {can.offense && (
            <>
              <button disabled={locked || !can.substitute || can.bench.length === 0} onClick={() => setSub("pinch_hitter")}>
                {t("tactics.pinchHit")}
              </button>
              <button disabled={locked || !can.substitute || can.bench.length === 0 || !runners} onClick={() => setSub("pinch_runner")}>
                {t("tactics.pinchRun")}
              </button>
            </>
          )}
          {can.defense && (
            <>
              <button
                disabled={locked || !can.substitute || can.bullpen.length === 0 || !can.pitcher_removable}
                title={!can.pitcher_removable && can.substitute ? t("tactics.mustFace") : undefined}
                onClick={() => setSub("pitcher")}
              >
                {t("tactics.changePitcher")}
              </button>
              <button disabled={locked || !can.substitute || can.bench.length === 0} onClick={() => setSub("fielder")}>
                {t("tactics.defSub")}
              </button>
            </>
          )}
        </div>
      </div>

      <div className="group">
        <div className="group-title">{t("start.delegate")}</div>
        <div className="checks">
          {(["pitching", "bench", "offense", "defense"] as (keyof Delegate)[]).map((d) => (
            <label key={d} className="check">
              <input type="checkbox" checked={ui.delegate[d]} disabled={ui.busy} onChange={(e) => void store.setDelegate({ ...ui.delegate, [d]: e.target.checked })} />
              <span>{t(`delegate.${d}`)}</span>
            </label>
          ))}
        </div>
      </div>

      {sub && (
        <div className="modal-back" onClick={closeSub}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>
              {t(
                {
                  pinch_hitter: "tactics.pinchHit",
                  pinch_runner: "tactics.pinchRun",
                  pitcher: "tactics.changePitcher",
                  fielder: "tactics.defSub",
                }[sub],
              )}
            </h3>
            {sub === "pinch_runner" && (
              <>
                <label>{t("tactics.chooseBase")}</label>
                <div className="segmented wrap">
                  {s.bases.map((p, i) =>
                    p == null ? null : (
                      <button key={i} className={target === i + 1 ? "on" : ""} onClick={() => setTarget(i + 1)}>
                        {t("tactics.forRunner", { base: i + 1 })}: {loc(player(team, p).name)}
                      </button>
                    ),
                  )}
                </div>
              </>
            )}
            {sub === "fielder" && (
              <>
                <label>{t("tactics.choosePosition")}</label>
                <div className="segmented wrap">
                  {state.defense[team].map((p, i) =>
                    i === 0 ? null : (
                      <button key={i} className={target === i ? "on" : ""} onClick={() => setTarget(i)}>
                        {t(`posShort.${POSITION_CODES[i]}`)} {loc(player(team, p).name)}
                      </button>
                    ),
                  )}
                </div>
              </>
            )}
            <label>{t("tactics.choosePlayer")}</label>
            {pool.length === 0 && <p className="muted">{sub === "pitcher" ? t("tactics.noPen") : t("tactics.noBench")}</p>}
            <ul className="pick">
              {pool.map((idx) => {
                const p = player(team, idx);
                const needsTarget = sub === "pinch_runner" || sub === "fielder";
                return (
                  <li key={idx}>
                    <button
                      disabled={needsTarget && target == null}
                      onClick={() => {
                        if (sub === "pinch_hitter") doSub({ kind: "pinch_hitter", player: idx });
                        else if (sub === "pitcher") doSub({ kind: "pitcher", player: idx });
                        else if (sub === "pinch_runner" && target != null) doSub({ kind: "pinch_runner", base: target, player: idx });
                        else if (sub === "fielder" && target != null) doSub({ kind: "fielder", position: target, player: idx });
                      }}
                    >
                      <span className="pick-name">
                        <span className="num">{p.number}</span> {loc(p.name)}
                        <span className="muted small">
                          {" "}
                          {t(`posShort.${p.position}`)} · {p.repertoire.length > 0 ? t("hand.throws", { hand: t(`hand.${p.throws}`) }) : t("hand.bats", { hand: t(`hand.${p.bats}`) })}
                        </span>
                      </span>
                      <PlayerRatings p={p} pitching={sub === "pitcher"} />
                    </button>
                  </li>
                );
              })}
            </ul>
            <div className="row end">
              <button onClick={closeSub}>{t("common.cancel")}</button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
