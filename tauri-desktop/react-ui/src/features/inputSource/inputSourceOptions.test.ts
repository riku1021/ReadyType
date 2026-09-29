import { describe, expect, it } from "vitest";

import "@/locales/i18n";

import { buildInputSourceOptions } from "./inputSourceOptions";

describe("buildInputSourceOptions", () => {
  it("keeps a removed configured source visible and unavailable", () => {
    expect(
      buildInputSourceOptions(
        [{ id: "en", name: "ABC", languageCode: "en", available: true }],
        "removed"
      )
    ).toContainEqual({
      value: "removed",
      label: "removed（利用不可）",
      available: false,
    });
  });

  it("uses localized language names when unique", () => {
    const options = buildInputSourceOptions(
      [
        { id: "com.apple.keylayout.ABC", name: "ABC", languageCode: "en", available: true },
        {
          id: "com.google.inputmethod.Japanese.base",
          name: "ひらがな（Google）",
          languageCode: "ja",
          available: true,
        },
        {
          id: "com.apple.inputmethod.Korean.2SetKorean",
          name: "2セット韓国語",
          languageCode: "ko",
          available: true,
        },
      ],
      []
    );
    expect(options.map((option) => ({ value: option.value, label: option.label }))).toEqual([
      { value: "com.apple.keylayout.ABC", label: "英語" },
      { value: "com.apple.inputmethod.Korean.2SetKorean", label: "韓国語" },
      { value: "com.google.inputmethod.Japanese.base", label: "日本語" },
    ]);
  });

  it("disambiguates colliding language codes with localized source names", () => {
    const options = buildInputSourceOptions(
      [
        {
          id: "com.apple.inputmethod.Korean.2SetKorean",
          name: "2セット韓国語",
          languageCode: "ko",
          available: true,
        },
        {
          id: "com.apple.inputmethod.Korean.3SetKorean",
          name: "3セット韓国語",
          languageCode: "ko",
          available: true,
        },
      ],
      []
    );
    expect(options.map((option) => option.label).sort()).toEqual([
      "韓国語（2セット韓国語）",
      "韓国語（3セット韓国語）",
    ]);
  });

  it("falls back to localized name when language code is missing", () => {
    const options = buildInputSourceOptions(
      [{ id: "hex", name: "Unicode Hex Input", languageCode: null, available: true }],
      []
    );
    expect(options).toEqual([{ value: "hex", label: "Unicode Hex Input", available: true }]);
  });

  it("merges multiple configured ids that are missing from OS list", () => {
    const options = buildInputSourceOptions(
      [{ id: "en", name: "ABC", languageCode: "en", available: true }],
      ["missing.a", "missing.b"]
    );
    expect(options.map((option) => option.value).sort()).toEqual(["en", "missing.a", "missing.b"]);
  });
});
