import { describe, expect, it } from "vitest";

import type { InputSourceViewState } from "@/features/inputSource";

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
    settingsPreferredEnglish: "en",
    settingsPreferredJapanese: "ja",
    debugLogging: false,
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
    ["running", "自動切替は有効です"],
    ["snoozed", "一時停止中"],
    ["permissionRequired", "アクセシビリティ権限が必要です"],
    ["unsupported", "このOSでは未対応です"],
  ] as const)("formats %s", (status, expected) => {
    expect(toStatusViewModel(stateWithStatus(status)).headline).toBe(expected);
  });

  it("formats empty current app and rule", () => {
    const view = toStatusViewModel(stateWithStatus("running"));
    expect(view.currentAppLabel).toBe("なし");
    expect(view.matchedRuleLabel).toBe("なし");
  });

  it("formats last decision labels", () => {
    const view = toStatusViewModel({
      ...stateWithStatus("running"),
      lastDecision: "noRule",
    });
    expect(view.lastDecisionLabel).toBe("一致ルールなし");
  });
});
