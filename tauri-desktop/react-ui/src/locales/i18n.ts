import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import en from "@/locales/en";
import ja from "@/locales/ja";

export const DEFAULT_LANGUAGE_PREFERENCE = "system";
export const DEFAULT_RESOLVED_LANGUAGE = "ja";
export const LANGUAGE_STORE_KEY = "readytype_ui_locale";
export const SUPPORTED_LANGUAGES = ["ja", "en"] as const;
export const LANGUAGE_PREFERENCES = ["system", "ja", "en"] as const;

export type ResolvedUiLocale = (typeof SUPPORTED_LANGUAGES)[number];
export type UiLocalePreference = (typeof LANGUAGE_PREFERENCES)[number];

/** @deprecated Prefer UiLocalePreference; kept for call sites that mean preference. */
export type UiLocale = UiLocalePreference;

export function isResolvedUiLocale(value: string): value is ResolvedUiLocale {
  return (SUPPORTED_LANGUAGES as readonly string[]).includes(value);
}

export function isUiLocalePreference(value: string): value is UiLocalePreference {
  return (LANGUAGE_PREFERENCES as readonly string[]).includes(value);
}

/** @deprecated Use isUiLocalePreference */
export function isUiLocale(value: string): value is UiLocalePreference {
  return isUiLocalePreference(value);
}

export function resolveUiLocale(preference: string): ResolvedUiLocale {
  if (preference === "en" || preference === "ja") {
    return preference;
  }
  if (typeof navigator !== "undefined") {
    for (const tag of navigator.languages ?? [navigator.language]) {
      const primary = tag.split(/[-_]/)[0]?.toLowerCase();
      if (primary === "en" || primary === "ja") {
        return primary;
      }
    }
  }
  return DEFAULT_RESOLVED_LANGUAGE;
}

export function readStoredLanguage(): UiLocalePreference {
  try {
    if (typeof localStorage === "undefined" || typeof localStorage.getItem !== "function") {
      return DEFAULT_LANGUAGE_PREFERENCE;
    }
    const stored = localStorage.getItem(LANGUAGE_STORE_KEY);
    if (stored && isUiLocalePreference(stored)) {
      return stored;
    }
  } catch {
    // Node / restricted environments may expose a non-functional localStorage.
  }
  return DEFAULT_LANGUAGE_PREFERENCE;
}

export function persistLanguage(locale: UiLocalePreference): void {
  try {
    if (typeof localStorage === "undefined" || typeof localStorage.setItem !== "function") {
      return;
    }
    localStorage.setItem(LANGUAGE_STORE_KEY, locale);
  } catch {
    // Ignore persistence failures in non-browser environments.
  }
}

const preference = readStoredLanguage();

void i18n.use(initReactI18next).init({
  resources: { ja, en },
  lng: resolveUiLocale(preference),
  fallbackLng: DEFAULT_RESOLVED_LANGUAGE,
  defaultNS: "translation",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
