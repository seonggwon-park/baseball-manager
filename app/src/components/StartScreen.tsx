import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Theme } from "../App";
import { defaultDelegate, store, useUi } from "../store";
import type { Control, Delegate, RuleSet } from "../types";
import { Preferences } from "./TopBar";

const RULES: RuleSet[] = ["kbo", "mlb", "npb"];
const CONTROLS: Control[] = ["home", "away", "watch"];
const DELEGATES: (keyof Delegate)[] = ["pitching", "bench", "offense", "defense"];

export function StartScreen({ theme, onToggleTheme }: { theme: Theme; onToggleTheme: () => void }) {
  const { t } = useTranslation();
  const ui = useUi();
  const [rules, setRules] = useState<RuleSet>("kbo");
  const [control, setControl] = useState<Control>("home");
  const [seed, setSeed] = useState("");
  const [delegate, setDelegate] = useState<Delegate>(defaultDelegate);

  const begin = () => {
    const n = Number.parseInt(seed, 10);
    void store.start({ rules, control, delegate, seed: Number.isFinite(n) && n >= 0 ? n : undefined });
  };

  return (
    <div className="start">
      <header className="start-head">
        <div>
          <h1>{t("app.title")}</h1>
          <p className="muted">{t("app.tagline")}</p>
        </div>
        <Preferences theme={theme} onToggleTheme={onToggleTheme} />
      </header>
      <main className="start-card">
        <h2>{t("start.heading")}</h2>
        <p className="muted">{t("start.intro")}</p>

        <fieldset>
          <legend>{t("start.rules")}</legend>
          {RULES.map((r) => (
            <label key={r} className="choice">
              <input type="radio" name="rules" checked={rules === r} onChange={() => setRules(r)} />
              <span>{t(`rules.${r}`)}</span>
            </label>
          ))}
        </fieldset>

        <fieldset>
          <legend>{t("start.control")}</legend>
          {CONTROLS.map((c) => (
            <label key={c} className="choice">
              <input type="radio" name="control" checked={control === c} onChange={() => setControl(c)} />
              <span>{t(`control.${c}`)}</span>
            </label>
          ))}
        </fieldset>

        {control !== "watch" && (
          <fieldset>
            <legend>{t("start.delegate")}</legend>
            {DELEGATES.map((d) => (
              <label key={d} className="choice">
                <input type="checkbox" checked={delegate[d]} onChange={(e) => setDelegate({ ...delegate, [d]: e.target.checked })} />
                <span>{t(`delegate.${d}`)}</span>
              </label>
            ))}
          </fieldset>
        )}

        <label className="field">
          <span>{t("start.seed")}</span>
          <input inputMode="numeric" value={seed} onChange={(e) => setSeed(e.target.value.replace(/[^0-9]/g, ""))} placeholder="2025" />
          <small className="muted">{t("start.seedHelp")}</small>
        </label>

        {ui.error && <p className="error">{t("start.failed", { message: ui.error.message })}</p>}

        <button className="primary big" onClick={begin} disabled={ui.starting}>
          {ui.starting ? t("start.starting") : t("start.play")}
        </button>
      </main>
    </div>
  );
}
