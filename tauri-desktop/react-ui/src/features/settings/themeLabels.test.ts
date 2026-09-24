import { describe, expect, it } from "vitest";

import { getThemeLabel, THEME_OPTIONS, themeFromLabel } from "./themeLabels";

describe("themeLabels", () => {
  it("テーマ値を表示ラベルへ変換する", () => {
    expect(getThemeLabel("light")).toBe("ライト");
    expect(getThemeLabel("dark")).toBe("ダーク");
    expect(getThemeLabel("system")).toBe("システムの設定を使う");
  });

  it("表示ラベルをテーマ値へ変換する", () => {
    expect(themeFromLabel("ライト")).toBe("light");
    expect(themeFromLabel("ダーク")).toBe("dark");
    expect(themeFromLabel("システムの設定を使う")).toBe("system");
    expect(themeFromLabel("不明")).toBeUndefined();
  });

  it("選択肢は3つ", () => {
    expect(THEME_OPTIONS).toHaveLength(3);
  });
});
