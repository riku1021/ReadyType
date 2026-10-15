import {
  isResolvedUiLocale,
  isUiLocalePreference,
  persistLanguage,
  type ResolvedUiLocale,
  resolveUiLocale,
} from "@/locales/i18n";

import type { InputSourceSettingsDto } from "./types";

/** Resolve the concrete UI pack from settings (IPC resolved value preferred). */
export function resolvedUiLocaleFromSettings(settings: InputSourceSettingsDto): ResolvedUiLocale {
  if (isResolvedUiLocale(settings.resolvedUiLocale)) {
    return settings.resolvedUiLocale;
  }
  return resolveUiLocale(settings.uiLocale);
}

/**
 * Persist preference and return the language pack to apply to i18n.
 * Returns `null` when `uiLocale` is not a known preference.
 */
export function prepareUiLocaleSync(settings: InputSourceSettingsDto): ResolvedUiLocale | null {
  if (!isUiLocalePreference(settings.uiLocale)) {
    return null;
  }
  persistLanguage(settings.uiLocale);
  return resolvedUiLocaleFromSettings(settings);
}
