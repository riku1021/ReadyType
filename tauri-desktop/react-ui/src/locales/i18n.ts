import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import {
  isResolvedUiLocale,
  isUiLocalePreference,
  LANGUAGE_PREFERENCES,
  type ResolvedUiLocale,
  SUPPORTED_LANGUAGES,
  SYSTEM_LANGUAGE_PREFERENCE,
  UI_LOCALE_REGISTRY,
  type UiLocalePreference,
} from "@/locales/registry";

export {
  isResolvedUiLocale,
  isUiLocalePreference,
  LANGUAGE_PREFERENCES,
  type ResolvedUiLocale,
  SUPPORTED_LANGUAGES,
  SYSTEM_LANGUAGE_PREFERENCE,
  UI_LOCALE_REGISTRY,
  type UiLocalePreference,
};

export const DEFAULT_LANGUAGE_PREFERENCE = UI_LOCALE_REGISTRY.defaultPreference;
export const DEFAULT_RESOLVED_LANGUAGE = UI_LOCALE_REGISTRY.defaultResolved;
export const LANGUAGE_STORE_KEY = "readytype_ui_locale";

/** @deprecated Prefer UiLocalePreference; kept for call sites that mean preference. */
export type UiLocale = UiLocalePreference;

/** @deprecated Use isUiLocalePreference */
export function isUiLocale(value: string): value is UiLocalePreference {
  return isUiLocalePreference(value);
}

export function resolveUiLocale(preference: string): ResolvedUiLocale {
  if (isResolvedUiLocale(preference)) {
    return preference;
  }
  if (typeof navigator !== "undefined") {
    for (const tag of navigator.languages ?? [navigator.language]) {
      const primary = tag.split(/[-_]/)[0]?.toLowerCase();
      if (primary && isResolvedUiLocale(primary)) {
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

type LocalePack = { translation: Record<string, unknown> };

const localePackModules = import.meta.glob<{ default: LocalePack }>("./*/index.ts", {
  eager: true,
});

function buildResources(): Record<string, LocalePack> {
  const resources: Record<string, LocalePack> = {};
  for (const code of SUPPORTED_LANGUAGES) {
    const modulePath = `./${code}/index.ts`;
    const pack = localePackModules[modulePath]?.default;
    if (!pack) {
      throw new Error(`Missing UI locale pack for registry code "${code}" at ${modulePath}`);
    }
    resources[code] = pack;
  }
  return resources;
}

const preference = readStoredLanguage();

void i18n.use(initReactI18next).init({
  resources: buildResources(),
  lng: resolveUiLocale(preference),
  fallbackLng: DEFAULT_RESOLVED_LANGUAGE,
  defaultNS: "translation",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
