import { describe, expect, it } from "vitest";

import {
  getThemeLabel,
  THEME_OPTIONS,
  themeFromLabel,
  themeFromTranslatedLabel,
} from "./themeLabels";

describe("themeLabels", () => {
  it("テーマ値を表示ラベルキーへ変換する", () => {
    expect(getThemeLabel("light")).toBe("theme.light");
    expect(getThemeLabel("dark")).toBe("theme.dark");
    expect(getThemeLabel("system")).toBe("theme.system");
  });

  it("表示ラベルキーをテーマ値へ変換する", () => {
    expect(themeFromLabel("theme.light")).toBe("light");
    expect(themeFromLabel("theme.dark")).toBe("dark");
    expect(themeFromLabel("theme.system")).toBe("system");
    expect(themeFromLabel("不明")).toBeUndefined();
  });

  it("翻訳済みラベルからテーマを解決する", () => {
    const translate = (key: (typeof THEME_OPTIONS)[number]) => {
      switch (key) {
        case "theme.light":
          return "ライト";
        case "theme.dark":
          return "ダーク";
        case "theme.system":
          return "システムの設定を使う";
      }
    };
    expect(themeFromTranslatedLabel("ダーク", translate)).toBe("dark");
    expect(themeFromTranslatedLabel("不明", translate)).toBeUndefined();
  });

  it("選択肢は3つ", () => {
    expect(THEME_OPTIONS).toHaveLength(3);
  });
});
