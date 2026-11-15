import { useTranslation } from "react-i18next";
import type { Theme } from "../App";
import { store, useUi } from "../store";
import { Preferences } from "./TopBar";

const SPEEDS = [1, 2, 4];

export function Controls({ theme, onToggleTheme }: { theme: Theme; onToggleTheme: () => void }) {
  const { t } = useTranslation();
  const ui = useUi();
  const over = ui.state!.situation.over;
  const locked = ui.busy || over || ui.auto;
  return (
    <div className="controls">
      <div className="row">
        <button className="primary" disabled={locked} onClick={() => void store.advance("pitch")}>
          {t("controls.nextPitch")}
        </button>
        <button disabled={locked} onClick={() => void store.advance("plate_appearance")}>
          {t("controls.finishPa")}
        </button>
        <button disabled={locked} onClick={() => void store.advance("half_inning")} title={t("controls.quickNote")}>
          {t("controls.finishHalf")}
        </button>
        <button disabled={locked} onClick={() => void store.advance("game")} title={t("controls.quickNote")}>
          {t("controls.finishGame")}
        </button>
      </div>
      <div className="row">
        <button className={ui.auto ? "toggle on" : "toggle"} disabled={over} onClick={() => store.setAuto(!ui.auto)}>
          {ui.auto ? t("controls.pause") : t("controls.auto")}
        </button>
        <div className="segmented" role="group" aria-label={t("controls.speed")}>
          {SPEEDS.map((s) => (
            <button key={s} className={ui.speed === s ? "on" : ""} onClick={() => store.setSpeed(s)}>
              {s}×
            </button>
          ))}
        </div>
        <label className="check">
          <input type="checkbox" checked={ui.skip} onChange={(e) => store.setSkip(e.target.checked)} />
          <span>{t("controls.skip")}</span>
        </label>
        <span className="spacer" />
        <Preferences theme={theme} onToggleTheme={onToggleTheme} />
        <button className="ghost" onClick={() => store.leave()}>
          {t("controls.newGame")}
        </button>
      </div>
    </div>
  );
}
