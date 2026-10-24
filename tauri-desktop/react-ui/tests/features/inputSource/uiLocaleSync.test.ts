import { describe, expect, it } from "vitest";

import type { InputSourceSettingsDto } from "@/features/inputSource/types";
import {
  prepareUiLocaleSync,
  resolvedUiLocaleFromSettings,
} from "@/features/inputSource/uiLocaleSync";

const baseSettings = (): InputSourceSettingsDto => ({
  languagePreferences: {},
  uiLocale: "system",
  resolvedUiLocale: "ja",
  quickSetShortcut: "CommandOrControl+Shift+L",
  quickSetShortcutRegistered: true,
  onboardingCompleted: true,
  showMenuBarIcon: true,
  inputSourceDisplayNames: {},
});

describe("uiLocaleSync", () => {
  it("prefers IPC resolvedUiLocale when valid", () => {
    expect(
      resolvedUiLocaleFromSettings({
        ...baseSettings(),
        uiLocale: "system",
        resolvedUiLocale: "en",
      })
    ).toBe("en");
  });

  it("falls back to resolveUiLocale when IPC value is invalid", () => {
    expect(
      resolvedUiLocaleFromSettings({
        ...baseSettings(),
        uiLocale: "zh",
        resolvedUiLocale: "not-a-locale",
      })
    ).toBe("zh");
  });

  it("returns resolved pack for known preference", () => {
    expect(
      prepareUiLocaleSync({
        ...baseSettings(),
        uiLocale: "zh",
        resolvedUiLocale: "zh",
      })
    ).toBe("zh");
  });

  it("returns null for unknown preference", () => {
    expect(
      prepareUiLocaleSync({
        ...baseSettings(),
        uiLocale: "fr",
        resolvedUiLocale: "en",
      })
    ).toBeNull();
  });
});
