import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import {
  applyNewerStateEvent,
  clearSwitchLogs,
  createRule,
  deleteRule,
  getAccessibilityPermission,
  getAppPresentations,
  getAutostartEnabled,
  getFocusSnapshot,
  getInputSourceSettings,
  getInputSourceSwitcherState,
  listInputSources,
  listRules,
  listSwitchLogs,
  openAccessibilitySettings,
  reorderRules,
  requestAccessibilityPermission,
  resetDefaultRules,
  resumeInputSourceSwitcher,
  setAutostartEnabled,
  setDebugLogging,
  setInputSourceSwitcherEnabled,
  setSnooze,
  shouldApplyFetchedRules,
  shouldRefreshRulesOnStateEvent,
  snoozeInputSourceSwitcherIndefinitely,
  startDebugCapture,
  stopDebugCapture,
  updateRule,
  withConfiguredSource,
} from "@/features/inputSource/api";
import type { RuleDto, SwitcherStateDto } from "@/features/inputSource/types";

beforeAll(() => {
  Object.defineProperty(globalThis, "window", {
    value: globalThis,
    configurable: true,
  });
});

const stateAtVersion = (version: number): SwitcherStateDto => ({
  version,
  rulesVersion: 1,
  enabled: true,
  snoozedUntilUnixMs: null,
  currentApp: null,
  matchedRuleId: null,
  currentInputSourceId: null,
  targetInputSourceId: null,
  lastDecision: null,
});

const validRule: RuleDto = {
  id: "custom-rule",
  name: "Custom",
  enabled: true,
  platform: "macOs",
  appId: "com.example.Editor",
  priority: 1,
  conditions: [],
  target: { kind: "preferredLanguage", language: "en" },
};

afterEach(() => clearMocks());

describe("input source IPC", () => {
  it("sends a validated create_rule payload", async () => {
    mockIPC((command, args) => {
      expect(command).toBe("create_rule");
      expect(args).toEqual({ rule: validRule });
      return stateAtVersion(4);
    });
    await expect(createRule(validRule)).resolves.toMatchObject({ version: 4 });
  });

  it("uses the exact command names and payloads", async () => {
    const calls: Array<{ command: string; args: object | undefined }> = [];
    mockIPC((command, args) => {
      calls.push({ command, args });
      if (command.startsWith("get_autostart") || command.startsWith("set_autostart")) {
        return true;
      }
      if (command.includes("permission")) {
        return "granted";
      }
      if (
        command === "list_input_sources" ||
        command === "list_rules" ||
        command === "list_switch_logs"
      ) {
        return [];
      }
      if (command === "get_input_source_settings") {
        return {
          languagePreferences: { en: "en", ja: "ja" },
          debugLogging: false,
          uiLocale: "ja",
          resolvedUiLocale: "ja",
          quickSetShortcut: "CommandOrControl+Shift+L",
          quickSetShortcutRegistered: true,
        };
      }
      if (command === "get_focus_snapshot" || command === "open_accessibility_settings") {
        return null;
      }
      if (command === "get_app_presentations") {
        return {
          "com.apple.Terminal": {
            icon: "data:image/png;base64,AA==",
            displayName: "Terminal",
          },
        };
      }
      return stateAtVersion(1);
    });

    await getInputSourceSwitcherState();
    await setInputSourceSwitcherEnabled(false);
    await setSnooze(15);
    await snoozeInputSourceSwitcherIndefinitely();
    await resumeInputSourceSwitcher();
    await listInputSources(true);
    await listRules();
    await updateRule(validRule);
    await deleteRule("custom-rule");
    await reorderRules(["a", "b"]);
    await resetDefaultRules();
    await getInputSourceSettings();
    await getAccessibilityPermission();
    await requestAccessibilityPermission();
    await openAccessibilitySettings();
    await startDebugCapture();
    await stopDebugCapture();
    await setDebugLogging(true);
    await getFocusSnapshot();
    await listSwitchLogs();
    await clearSwitchLogs();
    await getAutostartEnabled();
    await setAutostartEnabled(true);
    await getAppPresentations(["com.apple.Terminal"]);

    expect(calls.map((call) => call.command)).toEqual([
      "get_input_source_switcher_state",
      "set_input_source_switcher_enabled",
      "set_snooze",
      "snooze_input_source_switcher_indefinitely",
      "resume_input_source_switcher",
      "list_input_sources",
      "list_rules",
      "update_rule",
      "delete_rule",
      "reorder_rules",
      "reset_default_rules",
      "get_input_source_settings",
      "get_accessibility_permission",
      "request_accessibility_permission",
      "open_accessibility_settings",
      "start_debug_capture",
      "stop_debug_capture",
      "set_debug_logging",
      "get_focus_snapshot",
      "list_switch_logs",
      "clear_switch_logs",
      "get_autostart_enabled",
      "set_autostart_enabled",
      "get_app_presentations",
    ]);
    expect(calls[calls.length - 1]?.args).toEqual({ bundleIds: ["com.apple.Terminal"] });
    expect(calls[1]?.args).toEqual({ enabled: false });
    expect(calls[9]?.args).toEqual({ ids: ["a", "b"] });
  });

  it("propagates invoke errors", async () => {
    mockIPC(() => {
      throw new Error("native failure");
    });
    await expect(listRules()).rejects.toThrow("native failure");
  });
});

describe("state helpers", () => {
  it("ignores state events that are not newer", () => {
    const current = stateAtVersion(5);
    expect(applyNewerStateEvent(current, { version: 4, state: stateAtVersion(4) })).toBe(current);
    expect(applyNewerStateEvent(current, { version: 6, state: stateAtVersion(6) })?.version).toBe(
      6
    );
  });

  it("marks a configured but absent source unavailable", () => {
    const available = [{ id: "present", name: "Present", languageCode: "en", available: true }];
    const merged = withConfiguredSource(available, "removed.id");
    expect(merged[merged.length - 1]).toEqual({
      id: "removed.id",
      name: "removed.id",
      languageCode: null,
      available: false,
    });
  });

  it("refreshes rules only when an accepted event changes rulesVersion", () => {
    const next = { ...stateAtVersion(6), rulesVersion: 2 };
    expect(shouldRefreshRulesOnStateEvent(1, next, true)).toBe(true);
    expect(shouldRefreshRulesOnStateEvent(2, next, true)).toBe(false);
    expect(shouldRefreshRulesOnStateEvent(1, next, false)).toBe(false);
    expect(shouldRefreshRulesOnStateEvent(undefined, next, true)).toBe(true);
    expect(shouldRefreshRulesOnStateEvent(1, null, true)).toBe(false);
  });

  it("applies fetched rules only for the still-current rulesVersion", () => {
    expect(shouldApplyFetchedRules(3, 3)).toBe(true);
    expect(shouldApplyFetchedRules(4, 3)).toBe(false);
    expect(shouldApplyFetchedRules(undefined, 3)).toBe(false);
  });
});
