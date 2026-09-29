import { describe, expect, it } from "vitest";

import {
  getLanguageOptionLabel,
  languageFromOptionLabel,
  languageOptionLabels,
  SYSTEM_LANGUAGE_LABEL_KEY,
} from "@/features/settings/languageLabels";

describe("languageLabels", () => {
  const translate = (key: typeof SYSTEM_LANGUAGE_LABEL_KEY) => {
    if (key === "settings.languageSystem") {
      return "システム";
    }
    return key;
  };

  it("builds option labels from registry native names", () => {
    expect(languageOptionLabels(translate)).toEqual(["システム", "日本語", "English", "中文"]);
  });

  it("maps preference to display labels", () => {
    expect(getLanguageOptionLabel("system", translate)).toBe("システム");
    expect(getLanguageOptionLabel("ja", translate)).toBe("日本語");
    expect(getLanguageOptionLabel("en", translate)).toBe("English");
    expect(getLanguageOptionLabel("zh", translate)).toBe("中文");
  });

  it("resolves option labels back to preference", () => {
    expect(languageFromOptionLabel("システム", translate)).toBe("system");
    expect(languageFromOptionLabel("English", translate)).toBe("en");
    expect(languageFromOptionLabel("中文", translate)).toBe("zh");
  });
});
