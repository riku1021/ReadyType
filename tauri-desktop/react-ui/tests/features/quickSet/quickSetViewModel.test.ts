import { beforeAll, describe, expect, it } from "vitest";
import {
  buildQuickSetOptions,
  currentLanguageFromTarget,
  targetHeading,
} from "@/features/quickSet/quickSetViewModel";
import type { QuickSetSessionDto } from "@/features/quickSet/types";
import i18n from "@/locales/i18n";

const baseSession = (): QuickSetSessionDto => ({
  appId: "com.todesktop.cursor",
  appDisplayName: "cursor",
  platform: "macOs",
  pid: 1,
  proposedName: "AI Chat",
  proposedConditions: [],
  matchedRuleId: null,
  matchedTarget: null,
  languages: [
    {
      language: "ja",
      preferredSourceId: "com.apple.inputmethod.Kotoeri.Japanese",
      preferredSourceName: "Hiragana",
    },
    {
      language: "en",
      preferredSourceId: "com.apple.keylayout.ABC",
      preferredSourceName: "ABC",
    },
  ],
});

describe("quickSetViewModel", () => {
  beforeAll(async () => {
    await i18n.changeLanguage("ja");
  });

  it("builds target heading from app display name only", () => {
    expect(targetHeading(baseSession())).toBe("cursor");
  });

  it("marks current language when matched preferredLanguage", () => {
    const session = {
      ...baseSession(),
      matchedRuleId: "rule-1",
      matchedTarget: { kind: "preferredLanguage" as const, language: "ja" },
    };
    const options = buildQuickSetOptions(session);
    expect(currentLanguageFromTarget(session.matchedTarget)).toBe("ja");
    expect(options.find((option) => option.language === "ja")?.isCurrent).toBe(true);
    expect(options.find((option) => option.language === "en")?.isCurrent).toBe(false);
  });

  it("has no current when creating a new rule", () => {
    const options = buildQuickSetOptions(baseSession());
    expect(options.every((option) => !option.isCurrent)).toBe(true);
  });
});
