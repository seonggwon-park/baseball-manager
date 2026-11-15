import { useEffect, useState } from "react";
import { MatchScreen } from "./components/MatchScreen";
import { StartScreen } from "./components/StartScreen";
import { useUi } from "./store";

export type Theme = "dark" | "light";

function initialTheme(): Theme {
  try {
    return localStorage.getItem("bm.theme") === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}

export function App() {
  const ui = useUi();
  const [theme, setTheme] = useState<Theme>(initialTheme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("bm.theme", theme);
    } catch {
      // the choice then lasts for this session only
    }
  }, [theme]);
  const toggleTheme = () => setTheme(theme === "dark" ? "light" : "dark");
  return ui.view && ui.state ? <MatchScreen theme={theme} onToggleTheme={toggleTheme} /> : <StartScreen theme={theme} onToggleTheme={toggleTheme} />;
}
