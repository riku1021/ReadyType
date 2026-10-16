import { beforeAll, describe, expect, it } from "vitest";
import {
  buildInputSourceOptions,
  visibleInputSourceOptions,
} from "@/features/inputSource/inputSourceOptions";
import i18n from "@/locales/i18n";

describe("buildInputSourceOptions", () => {
  beforeAll(async () => {
    await i18n.changeLanguage("ja");
  });
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

  it("uses last-known display name when a configured source is missing", () => {
    expect(
      buildInputSourceOptions(
        [{ id: "en", name: "ABC", languageCode: "en", available: true }],
        "com.google.inputmethod.Japanese.base",
        { "com.google.inputmethod.Japanese.base": "ひらがな（Google）" }
      )
    ).toContainEqual({
      value: "com.google.inputmethod.Japanese.base",
      label: "ひらがな（Google）（利用不可）",
      available: false,
    });
  });

  it("uses OS localized names as labels", () => {
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
          id: "com.google.inputmethod.Japanese.Roman",
          name: "英数（Google）",
          languageCode: "en",
          available: true,
        },
        {
          id: "com.apple.inputmethod.Kotoeri.Japanese",
          name: "日本語",
          languageCode: "ja",
          available: true,
        },
      ],
      []
    );
    expect(options.map((option) => ({ value: option.value, label: option.label }))).toEqual([
      { value: "com.apple.keylayout.ABC", label: "ABC" },
      { value: "com.google.inputmethod.Japanese.base", label: "ひらがな（Google）" },
      { value: "com.google.inputmethod.Japanese.Roman", label: "英数（Google）" },
      { value: "com.apple.inputmethod.Kotoeri.Japanese", label: "日本語" },
    ]);
  });

  it("keeps distinct OS names when language codes collide", () => {
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
      "2セット韓国語",
      "3セット韓国語",
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

describe("visibleInputSourceOptions", () => {
  const options = [
    { value: "en", label: "ABC", available: true },
    { value: "removed", label: "removed（利用不可）", available: false },
    { value: "ja", label: "ひらがな（Google）", available: true },
  ];

  it("hides unavailable options that are not the current value", () => {
    expect(visibleInputSourceOptions(options, "ja").map((option) => option.value)).toEqual([
      "en",
      "ja",
    ]);
  });

  it("keeps an unavailable option when it is the current value", () => {
    expect(visibleInputSourceOptions(options, "removed").map((option) => option.value)).toEqual([
      "en",
      "removed",
      "ja",
    ]);
  });
});
