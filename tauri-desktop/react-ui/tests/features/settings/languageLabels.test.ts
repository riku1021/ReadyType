import { describe, expect, it } from "vitest";

import {
  filterLanguageListRows,
  getLanguageOptionLabel,
  languageListRows,
  type SYSTEM_LANGUAGE_LABEL_KEY,
} from "@/features/settings/languageLabels";

describe("languageLabels", () => {
  const translate = (key: typeof SYSTEM_LANGUAGE_LABEL_KEY) => {
    if (key === "settings.languageSystem") {
      return "システム";
    }
    return key;
  };

  const rows = () => languageListRows(translate, "ja");

  it("lists system first, then registry native names", () => {
    expect(rows().map((row) => row.label)).toEqual(["システム", "日本語", "English", "中文"]);
    expect(rows().map((row) => row.preference)).toEqual(["system", "ja", "en", "zh"]);
  });

  it("maps preference to display labels", () => {
    expect(getLanguageOptionLabel("system", translate)).toBe("システム");
    expect(getLanguageOptionLabel("ja", translate)).toBe("日本語");
    expect(getLanguageOptionLabel("en", translate)).toBe("English");
    expect(getLanguageOptionLabel("zh", translate)).toBe("中文");
  });

  it("returns every row when the query is blank", () => {
    expect(filterLanguageListRows(rows(), "  ")).toEqual(rows());
  });

  it("filters by native name, English name, current UI name, and code", () => {
    expect(filterLanguageListRows(rows(), "日本語").map((row) => row.preference)).toEqual(["ja"]);
    expect(filterLanguageListRows(rows(), "Ja").map((row) => row.preference)).toEqual(["ja"]);
    expect(filterLanguageListRows(rows(), "Japanese").map((row) => row.preference)).toEqual(["ja"]);
    expect(filterLanguageListRows(rows(), "Chinese").map((row) => row.preference)).toEqual(["zh"]);
    expect(filterLanguageListRows(rows(), "システム").map((row) => row.preference)).toEqual([
      "system",
    ]);
    expect(filterLanguageListRows(rows(), "EN").map((row) => row.preference)).toEqual(["en"]);
    expect(filterLanguageListRows(rows(), "中国語").map((row) => row.preference)).toEqual(["zh"]);
    expect(filterLanguageListRows(rows(), "英語").map((row) => row.preference)).toEqual(["en"]);
    expect(
      filterLanguageListRows(languageListRows(translate, "zh"), "日语").map((row) => row.preference)
    ).toEqual(["ja"]);
  });

  it("returns no rows when nothing matches", () => {
    expect(filterLanguageListRows(rows(), "fr")).toEqual([]);
  });
});
