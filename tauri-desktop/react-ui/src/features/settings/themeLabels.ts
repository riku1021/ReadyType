import type { Theme } from "@/atoms/theme";

export const THEME_OPTIONS = ["ライト", "ダーク", "システムの設定を使う"] as const;

export type ThemeOptionLabel = (typeof THEME_OPTIONS)[number];

const themeToLabel: Record<Theme, ThemeOptionLabel> = {
  light: "ライト",
  dark: "ダーク",
  system: "システムの設定を使う",
};

const labelToTheme: Record<ThemeOptionLabel, Theme> = {
  ライト: "light",
  ダーク: "dark",
  システムの設定を使う: "system",
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
