import { invoke } from "@tauri-apps/api/core";
import type {
  AccessibilityPermission,
  FocusSnapshotDto,
  InputSourceDto,
  InputSourceSettingsDto,
  RuleDto,
  StateEventDto,
  SwitcherStateDto,
  SwitchLogDto,
} from "./types";

export const INPUT_SOURCE_STATE_EVENT = "input-source-switcher://state";

export const getInputSourceSwitcherState = () =>
  invoke<SwitcherStateDto>("get_input_source_switcher_state");
export const setInputSourceSwitcherEnabled = (enabled: boolean) =>
  invoke<SwitcherStateDto>("set_input_source_switcher_enabled", { enabled });
export const setSnooze = (minutes: number) => invoke<SwitcherStateDto>("set_snooze", { minutes });
export const resumeInputSourceSwitcher = () =>
  invoke<SwitcherStateDto>("resume_input_source_switcher");
export const listInputSources = (refresh = false) =>
  invoke<InputSourceDto[]>("list_input_sources", { refresh });
export const listRules = () => invoke<RuleDto[]>("list_rules");
export const createRule = (rule: RuleDto) => invoke<SwitcherStateDto>("create_rule", { rule });
export const updateRule = (rule: RuleDto) => invoke<SwitcherStateDto>("update_rule", { rule });
export const deleteRule = (id: string) => invoke<SwitcherStateDto>("delete_rule", { id });
export const reorderRules = (ids: readonly string[]) =>
  invoke<SwitcherStateDto>("reorder_rules", { ids: [...ids] });
export const resetDefaultRules = () => invoke<SwitcherStateDto>("reset_default_rules");
export const updateInputSourcePreferences = (preferredEnglish: string, preferredJapanese: string) =>
  invoke<SwitcherStateDto>("update_input_source_preferences", {
    preferredEnglish,
    preferredJapanese,
  });
export const getInputSourceSettings = () =>
  invoke<InputSourceSettingsDto>("get_input_source_settings");
export const getAccessibilityPermission = () =>
  invoke<AccessibilityPermission>("get_accessibility_permission");
export const requestAccessibilityPermission = () =>
  invoke<AccessibilityPermission>("request_accessibility_permission");
export const openAccessibilitySettings = () => invoke<void>("open_accessibility_settings");
export const startDebugCapture = () => invoke<SwitcherStateDto>("start_debug_capture");
export const stopDebugCapture = () => invoke<SwitcherStateDto>("stop_debug_capture");
export const setDebugLogging = (enabled: boolean) =>
  invoke<SwitcherStateDto>("set_debug_logging", { enabled });
export const getFocusSnapshot = () => invoke<FocusSnapshotDto | null>("get_focus_snapshot");
export const listSwitchLogs = () => invoke<SwitchLogDto[]>("list_switch_logs");
export const clearSwitchLogs = () => invoke<SwitcherStateDto>("clear_switch_logs");
export const getAutostartEnabled = () => invoke<boolean>("get_autostart_enabled");
export const setAutostartEnabled = (enabled: boolean) =>
  invoke<boolean>("set_autostart_enabled", { enabled });
/** bundle id ごとの PNG data URL（解決不可は null）。 */
export const getAppIcons = (bundleIds: readonly string[]) =>
  invoke<Record<string, string | null>>("get_app_icons", { bundleIds: [...bundleIds] });

export function applyNewerStateEvent(
  current: SwitcherStateDto | null,
  event: StateEventDto
): SwitcherStateDto | null {
  return current && event.version <= current.version ? current : event.state;
}

export function withConfiguredSource(
  sources: readonly InputSourceDto[],
  configuredId: string
): InputSourceDto[] {
  if (sources.some((source) => source.id === configuredId)) {
    return [...sources];
  }
  return [...sources, { id: configuredId, name: configuredId, available: false }];
}
