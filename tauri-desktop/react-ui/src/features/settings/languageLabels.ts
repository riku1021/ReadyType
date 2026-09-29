import type { UiLocalePreference } from "@/locales/i18n";

export const LANGUAGE_OPTIONS = [
  "settings.languageSystem",
  "settings.languageJa",
  "settings.languageEn",
] as const;

export type LanguageOptionLabel = (typeof LANGUAGE_OPTIONS)[number];

const preferenceToLabel: Record<UiLocalePreference, LanguageOptionLabel> = {
  system: "settings.languageSystem",
  ja: "settings.languageJa",
  en: "settings.languageEn",
};

const labelToPreference: Record<LanguageOptionLabel, UiLocalePreference> = {
  "settings.languageSystem": "system",
  "settings.languageJa": "ja",
  "settings.languageEn": "en",
};

export function getLanguageLabel(preference: UiLocalePreference): LanguageOptionLabel {
  return preferenceToLabel[preference];
}

/** 表示ラベル（翻訳済み）から preference を解決する */
export function languageFromTranslatedLabel(
  label: string,
  translate: (key: LanguageOptionLabel) => string
): UiLocalePreference | undefined {
  const key = LANGUAGE_OPTIONS.find((option) => translate(option) === label);
  return key ? labelToPreference[key] : undefined;
}
