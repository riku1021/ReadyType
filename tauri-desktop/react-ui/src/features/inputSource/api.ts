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
export const snoozeInputSourceSwitcherIndefinitely = () =>
  invoke<SwitcherStateDto>("snooze_input_source_switcher_indefinitely");
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
export const setUiLocale = (locale: string) =>
  invoke<InputSourceSettingsDto>("set_ui_locale", { locale });
export const completeOnboarding = () => invoke<InputSourceSettingsDto>("complete_onboarding");
export const getFocusSnapshot = () => invoke<FocusSnapshotDto | null>("get_focus_snapshot");
/** Lab 切替プリセットを既存ルールへ upsert。 */
export const ensureLabSwitchRules = () => invoke<SwitcherStateDto>("ensure_lab_switch_rules");
/** Lab 切替テスト用のネイティブ入力窓を開く。 */
export const openLabSwitchWindow = () => invoke<void>("open_lab_switch_window");
export const listSwitchLogs = () => invoke<SwitchLogDto[]>("list_switch_logs");
export const clearSwitchLogs = () => invoke<SwitcherStateDto>("clear_switch_logs");
export const getAutostartEnabled = () => invoke<boolean>("get_autostart_enabled");
export const setAutostartEnabled = (enabled: boolean) =>
  invoke<boolean>("set_autostart_enabled", { enabled });
/** IPC payload from `get_app_presentations` (filesystem path + revision; not a UI src). */
export type AppPresentationDto = {
  icon: string | null;
  iconRevision: string | null;
  displayName: string | null;
};

/** bundle id ごとのキャッシュ PNG パス + Finder 表示名（解決不可フィールドは null）。 */
export const getAppPresentations = (bundleIds: readonly string[]) =>
  invoke<Record<string, AppPresentationDto>>("get_app_presentations", {
    bundleIds: [...bundleIds],
  });

export function applyNewerStateEvent(
  current: SwitcherStateDto | null,
  event: StateEventDto
): SwitcherStateDto | null {
  return current && event.version <= current.version ? current : event.state;
}

/** state イベント採用後に rules を再取得すべきか（rulesVersion 変化時のみ）。 */
export function shouldRefreshRulesOnStateEvent(
  previousRulesVersion: number | undefined,
  nextSwitcher: SwitcherStateDto | null,
  eventAccepted: boolean
): boolean {
  if (!eventAccepted || nextSwitcher === null) {
    return false;
  }
  return nextSwitcher.rulesVersion !== previousRulesVersion;
}

/** listRules 完了時、より新しい state イベントが既に来ていれば破棄する。 */
export function shouldApplyFetchedRules(
  currentRulesVersion: number | undefined,
  expectedRulesVersion: number
): boolean {
  return currentRulesVersion === expectedRulesVersion;
}

export function withConfiguredSource(
  sources: readonly InputSourceDto[],
  configuredId: string
): InputSourceDto[] {
  if (sources.some((source) => source.id === configuredId)) {
    return [...sources];
  }
  return [
    ...sources,
    { id: configuredId, name: configuredId, languageCode: null, available: false },
  ];
}
