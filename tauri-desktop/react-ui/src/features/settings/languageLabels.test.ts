import { describe, expect, it } from "vitest";

import { getLanguageLabel, languageFromTranslatedLabel } from "./languageLabels";

describe("languageLabels", () => {
  it("maps preference to i18n keys", () => {
    expect(getLanguageLabel("system")).toBe("settings.languageSystem");
    expect(getLanguageLabel("ja")).toBe("settings.languageJa");
    expect(getLanguageLabel("en")).toBe("settings.languageEn");
  });

  it("resolves translated labels back to preference", () => {
    const translate = (key: string) => {
      switch (key) {
        case "settings.languageSystem":
          return "システム";
        case "settings.languageJa":
          return "日本語";
        case "settings.languageEn":
          return "English";
        default:
          return key;
      }
    };
    expect(languageFromTranslatedLabel("システム", translate)).toBe("system");
    expect(languageFromTranslatedLabel("English", translate)).toBe("en");
  });
});
