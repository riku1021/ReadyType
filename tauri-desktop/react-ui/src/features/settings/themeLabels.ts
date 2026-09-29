import type { Theme } from "@/atoms/theme";

export const THEME_OPTIONS = ["theme.light", "theme.dark", "theme.system"] as const;

export type ThemeOptionLabel = (typeof THEME_OPTIONS)[number];

const themeToLabel: Record<Theme, ThemeOptionLabel> = {
  light: "theme.light",
  dark: "theme.dark",
  system: "theme.system",
};

const labelToTheme: Record<ThemeOptionLabel, Theme> = {
  "theme.light": "light",
  "theme.dark": "dark",
  "theme.system": "system",
};

export function getThemeLabel(theme: Theme): ThemeOptionLabel {
  return themeToLabel[theme];
}

export function themeFromLabel(label: string): Theme | undefined {
  if (label in labelToTheme) {
    return labelToTheme[label as ThemeOptionLabel];
  }
  return undefined;
}

/** 表示ラベル（翻訳済み）からテーマを解決する */
export function themeFromTranslatedLabel(
  label: string,
  translate: (key: ThemeOptionLabel) => string
): Theme | undefined {
  const key = THEME_OPTIONS.find((option) => translate(option) === label);
  return key ? labelToTheme[key] : undefined;
}
