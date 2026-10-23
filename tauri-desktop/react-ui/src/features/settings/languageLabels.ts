import {
  LANGUAGE_PREFERENCES,
  nativeNameForLocale,
  SYSTEM_LANGUAGE_PREFERENCE,
  type UiLocalePreference,
} from "@/locales/registry";

/** i18n key for the "System" preference label. */
export const SYSTEM_LANGUAGE_LABEL_KEY = "settings.languageSystem" as const;

type SystemLabelTranslator = (key: typeof SYSTEM_LANGUAGE_LABEL_KEY) => string;

export type LanguageListRow = {
  preference: UiLocalePreference;
  label: string;
  /** 自称・英語名・いまの表示言語での名前・コード。システムは翻訳ラベルとコード。 */
  searchTerms: readonly string[];
};

export function getLanguageOptionLabel(
  preference: UiLocalePreference,
  translate: SystemLabelTranslator
): string {
  if (preference === SYSTEM_LANGUAGE_PREFERENCE) {
    return translate(SYSTEM_LANGUAGE_LABEL_KEY);
  }
  return nativeNameForLocale(preference) ?? preference;
}

export function languageListRows(
  translate: SystemLabelTranslator,
  uiLocale: string
): LanguageListRow[] {
  const inUiLocale = new Intl.DisplayNames([uiLocale], { type: "language" });
  const inEnglish = new Intl.DisplayNames(["en"], { type: "language" });
  return LANGUAGE_PREFERENCES.map((preference) => {
    const label = getLanguageOptionLabel(preference, translate);
    const searchTerms = [label, preference];
    if (preference !== SYSTEM_LANGUAGE_PREFERENCE) {
      const englishName = inEnglish.of(preference);
      const uiName = inUiLocale.of(preference);
      if (englishName) {
        searchTerms.push(englishName);
      }
      if (uiName) {
        searchTerms.push(uiName);
      }
    }
    return { preference, label, searchTerms };
  });
}

/** 自称・英語名・いまの表示言語での名前・コードの部分一致で絞る。 */
export function filterLanguageListRows(
  rows: readonly LanguageListRow[],
  search: string
): LanguageListRow[] {
  const query = search.trim().toLocaleLowerCase();
  if (query === "") {
    return [...rows];
  }
  return rows.filter((row) =>
    row.searchTerms.some((term) => term.toLocaleLowerCase().includes(query))
  );
}
