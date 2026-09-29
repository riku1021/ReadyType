import { describe, expect, it } from "vitest";

import type { InputSourceViewState } from "@/features/inputSource";
import type i18n from "@/locales/i18n";

import { toStatusViewModel } from "./statusViewModel";

function stateWithStatus(
  status: "running" | "snoozed" | "permissionRequired" | "unsupported"
): InputSourceViewState {
  const base: InputSourceViewState = {
    version: 1,
    rulesVersion: 1,
    enabled: true,
    snoozedUntilUnixMs: null,
    currentApp: null,
    matchedRuleId: null,
    currentInputSourceId: "en",
    targetInputSourceId: "en",
    lastDecision: "switched",
    accessibilityTrusted: true,
    permission: "granted",
    languagePreferences: { en: "en", ja: "ja" },
    debugLogging: false,
    uiLocale: "ja",
    rules: [],
    initError: null,
    ready: true,
  };

  switch (status) {
    case "running":
      return base;
    case "snoozed":
      return { ...base, snoozedUntilUnixMs: Date.now() + 60_000 };
    case "permissionRequired":
      return { ...base, accessibilityTrusted: false, permission: "denied" };
    case "unsupported":
      return { ...base, accessibilityTrusted: false, permission: "unsupported" };
  }
}

describe("toStatusViewModel", () => {
  it.each([
    ["running", "home.headlineRunning"],
    ["snoozed", "home.headlineSnoozed"],
    ["permissionRequired", "home.headlinePermissionRequired"],
    ["unsupported", "home.headlineUnsupported"],
  ] as const)("formats %s", (status, expectedKey) => {
    const t = ((key: string) => key) as typeof i18n.t;
    expect(toStatusViewModel(stateWithStatus(status), t).headline).toBe(expectedKey);
  });

  it("formats empty current app and rule", () => {
    const t = ((key: string) => key) as typeof i18n.t;
    const view = toStatusViewModel(stateWithStatus("running"), t);
    expect(view.currentAppLabel).toBe("common.none");
    expect(view.matchedRuleLabel).toBe("common.none");
  });

  it("formats last decision labels", () => {
    const t = ((key: string) => key) as typeof i18n.t;
    const view = toStatusViewModel(
      {
        ...stateWithStatus("running"),
        lastDecision: "noRule",
      },
      t
    );
    expect(view.lastDecisionLabel).toBe("home.decisionNoRule");
  });
});
