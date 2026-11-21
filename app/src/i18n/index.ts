import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ko from "./ko.json";
import en from "./en.json";

export const LANGUAGES = ["ko", "en"] as const;
export type Language = (typeof LANGUAGES)[number];

const stored = typeof localStorage !== "undefined" ? localStorage.getItem("bm.lang") : null;
const initial: Language = stored === "en" ? "en" : "ko";

void i18n.use(initReactI18next).init({
  resources: { ko: { translation: ko }, en: { translation: en } },
  lng: initial,
  fallbackLng: "ko",
  interpolation: { escapeValue: false },
});

export function setLanguage(lang: Language) {
  void i18n.changeLanguage(lang);
  document.documentElement.lang = lang;
  try {
    localStorage.setItem("bm.lang", lang);
  } catch {
    // storage may be unavailable; the choice then lasts for this session only
  }
}

document.documentElement.lang = initial;

export default i18n;
