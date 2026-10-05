import { beforeAll, describe, expect, it } from "vitest";
import { buildQuickSetOptions, targetHeading } from "@/features/quickSet/quickSetViewModel";
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
  sources: [
    {
      id: "com.apple.inputmethod.Kotoeri.Japanese",
      name: "Hiragana",
      languageCode: "ja",
      available: true,
    },
    {
      id: "com.apple.keylayout.ABC",
      name: "ABC",
      languageCode: "en",
      available: true,
    },
    {
      id: "com.apple.inputmethod.Korean",
      name: "2-Set Korean",
      languageCode: "ko",
      available: true,
    },
  ],
  languagePreferences: {
    en: "com.apple.keylayout.ABC",
    ja: "com.apple.inputmethod.Kotoeri.Japanese",
  },
});

describe("quickSetViewModel", () => {
  beforeAll(async () => {
    await i18n.changeLanguage("ja");
  });

  it("builds target heading from app display name only", () => {
    expect(targetHeading(baseSession())).toBe("cursor");
  });

  it("lists the same input sources as app settings (including non-preferred languages)", () => {
    const options = buildQuickSetOptions(baseSession());
    expect(options.map((option) => option.sourceId).sort()).toEqual([
      "com.apple.inputmethod.Korean",
      "com.apple.inputmethod.Kotoeri.Japanese",
      "com.apple.keylayout.ABC",
    ]);
  });

  it("marks current source when matched preferredLanguage", () => {
    const session = {
      ...baseSession(),
      matchedRuleId: "rule-1",
      matchedTarget: { kind: "preferredLanguage" as const, language: "ja" },
    };
    const options = buildQuickSetOptions(session);
    expect(
      options.find((option) => option.sourceId === "com.apple.inputmethod.Kotoeri.Japanese")
        ?.isCurrent
    ).toBe(true);
    expect(options.find((option) => option.sourceId === "com.apple.keylayout.ABC")?.isCurrent).toBe(
      false
    );
  });

  it("has no current when creating a new rule", () => {
    const options = buildQuickSetOptions(baseSession());
    expect(options.every((option) => !option.isCurrent)).toBe(true);
  });
});
