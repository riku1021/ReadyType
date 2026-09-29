import {
  LANGUAGE_PREFERENCES,
  nativeNameForLocale,
  SYSTEM_LANGUAGE_PREFERENCE,
  UI_LOCALE_REGISTRY,
  type UiLocalePreference,
} from "@/locales/registry";

/** i18n key for the "System" preference label. */
export const SYSTEM_LANGUAGE_LABEL_KEY = "settings.languageSystem" as const;

type SystemLabelTranslator = (key: typeof SYSTEM_LANGUAGE_LABEL_KEY) => string;

export function getLanguageOptionLabel(
  preference: UiLocalePreference,
  translate: SystemLabelTranslator
): string {
  if (preference === SYSTEM_LANGUAGE_PREFERENCE) {
    return translate(SYSTEM_LANGUAGE_LABEL_KEY);
  }
  return nativeNameForLocale(preference) ?? preference;
}

export function languageOptionLabels(translate: SystemLabelTranslator): string[] {
  return LANGUAGE_PREFERENCES.map((preference) => getLanguageOptionLabel(preference, translate));
}

/** Resolve a displayed option label back to a preference. */
export function languageFromOptionLabel(
  label: string,
  translate: SystemLabelTranslator
): UiLocalePreference | undefined {
  if (label === translate(SYSTEM_LANGUAGE_LABEL_KEY)) {
    return SYSTEM_LANGUAGE_PREFERENCE;
  }
  const matched = UI_LOCALE_REGISTRY.locales.find((locale) => locale.nativeName === label);
  return matched ? (matched.code as UiLocalePreference) : undefined;
}
