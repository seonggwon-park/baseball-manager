import { useTranslation } from "react-i18next";
import type { Theme } from "../App";
import { setLanguage } from "../i18n";

/** Theme and language switches, shared by every screen. */
export function Preferences({ theme, onToggleTheme }: { theme: Theme; onToggleTheme: () => void }) {
  const { t, i18n } = useTranslation();
  const lang = i18n.language === "en" ? "en" : "ko";
  return (
    <div className="prefs">
      <button className="ghost" onClick={onToggleTheme} title={t("theme.toggle")}>
        {theme === "dark" ? t("theme.light") : t("theme.dark")}
      </button>
      <button className="ghost" onClick={() => setLanguage(lang === "ko" ? "en" : "ko")} title={t("lang.toggle")}>
        {lang === "ko" ? t("lang.en") : t("lang.ko")}
      </button>
    </div>
  );
}
