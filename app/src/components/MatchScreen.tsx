import { useLabels } from "../labels";
import type { Theme } from "../App";
import { store, useUi } from "../store";
import { BottomTabs } from "./BottomTabs";
import { Controls } from "./Controls";
import { FieldView } from "./FieldView";
import { Matchup } from "./Matchup";
import { Scoreboard } from "./Scoreboard";
import { Tactics } from "./Tactics";
import { ZonePanel } from "./ZonePanel";

export function MatchScreen({ theme, onToggleTheme }: { theme: Theme; onToggleTheme: () => void }) {
  const ui = useUi();
  const { t } = useLabels(ui.view);
  return (
    <div className="match">
      <header className="match-head">
        <Scoreboard />
        <Controls theme={theme} onToggleTheme={onToggleTheme} />
      </header>
      {ui.error && (
        <div className="toast" role="alert" onClick={() => store.clearError()}>
          {t(`errors.${ui.error.code}`, { defaultValue: t("errors.generic", { message: ui.error.message }) })}
        </div>
      )}
      <div className="match-main">
        <FieldView />
        <aside className="side">
          <Matchup />
          <ZonePanel />
          <Tactics />
        </aside>
        <BottomTabs />
      </div>
    </div>
  );
}
