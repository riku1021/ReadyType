import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import {
  applyNewerStateEvent,
  clearSwitchLogs,
  createRule,
  deleteRule,
  getAccessibilityPermission,
  getAppIcons,
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
  snoozeInputSourceSwitcherIndefinitely,
  startDebugCapture,
  stopDebugCapture,
  updateInputSourcePreferences,
  updateRule,
  withConfiguredSource,
} from "./api";
import type { RuleDto, SwitcherStateDto } from "./types";

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
  target: { kind: "preferredEnglish" },
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
          preferredEnglish: "en",
          preferredJapanese: "ja",
          debugLogging: false,
        };
      }
      if (command === "get_focus_snapshot" || command === "open_accessibility_settings") {
        return null;
      }
      if (command === "get_app_icons") {
        return { "com.apple.Terminal": "data:image/png;base64,AA==" };
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
    await updateInputSourcePreferences("en", "ja");
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
    await getAppIcons(["com.apple.Terminal"]);

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
      "update_input_source_preferences",
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
      "get_app_icons",
    ]);
    expect(calls[calls.length - 1]?.args).toEqual({ bundleIds: ["com.apple.Terminal"] });
    expect(calls[1]?.args).toEqual({ enabled: false });
    expect(calls[9]?.args).toEqual({ ids: ["a", "b"] });
    expect(calls[11]?.args).toEqual({
      preferredEnglish: "en",
      preferredJapanese: "ja",
    });
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
});
