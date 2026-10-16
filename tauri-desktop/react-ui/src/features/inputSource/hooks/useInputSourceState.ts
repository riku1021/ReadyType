import { listen } from "@tauri-apps/api/event";
import { useAtom, useSetAtom } from "jotai";
import { useCallback, useEffect } from "react";

import i18n, { isResolvedUiLocale } from "@/locales/i18n";
import {
  ACCESSIBILITY_PERMISSION_EVENT,
  applyNewerStateEvent,
  clearSwitchLogs as clearSwitchLogsApi,
  completeOnboarding as completeOnboardingApi,
  createRule as createRuleApi,
  deleteRule as deleteRuleApi,
  getAccessibilityPermission,
  getAutostartEnabled,
  getFocusSnapshot,
  getInputSourceSettings,
  getInputSourceSwitcherState,
  INPUT_SOURCE_STATE_EVENT,
  listInputSources,
  listRules,
  listSwitchLogs,
  openAccessibilitySettings as openAccessibilitySettingsApi,
  RESOLVED_UI_LOCALE_EVENT,
  reorderRules as reorderRulesApi,
  requestAccessibilityPermission,
  resetDefaultRules as resetDefaultRulesApi,
  resumeInputSourceSwitcher,
  setAutostartEnabled as setAutostartEnabledApi,
  setDebugLogging as setDebugLoggingApi,
  setInputSourceSwitcherEnabled,
  setSnooze as setSnoozeApi,
  setUiLocale as setUiLocaleApi,
  shouldApplyFetchedRules,
  shouldRefreshRulesOnStateEvent,
  snoozeInputSourceSwitcherIndefinitely as snoozeIndefinitelyApi,
  startDebugCapture as startDebugCaptureApi,
  stopDebugCapture as stopDebugCaptureApi,
  updateRule as updateRuleApi,
} from "../api";
import { type InputSourceFeatureState, inputSourceStateAtom } from "../atoms/inputSourceStateAtom";
import type {
  AccessibilityPermission,
  FocusSnapshotDto,
  InputSourceDto,
  InputSourceSettingsDto,
  LanguageInputPreferences,
  ResolvedUiLocaleEventDto,
  RuleDto,
  StateEventDto,
  SwitcherStateDto,
  SwitchLogDto,
} from "../types";
import { prepareUiLocaleSync } from "../uiLocaleSync";

export type InputSourceViewState = SwitcherStateDto & {
  accessibilityTrusted: boolean;
  permission: AccessibilityPermission | null;
  languagePreferences: LanguageInputPreferences;
  inputSourceDisplayNames: Record<string, string>;
  debugLogging: boolean;
  uiLocale: string | null;
  rules: RuleDto[];
  initError: string | null;
  ready: boolean;
};

type FeatureSetter = (
  updater: InputSourceFeatureState | ((current: InputSourceFeatureState) => InputSourceFeatureState)
) => void;

function applySwitcher(setFeature: FeatureSetter, switcher: SwitcherStateDto) {
  setFeature((current) => ({ ...current, switcher, initError: null, ready: true }));
}

async function applySettingsToI18n(settings: InputSourceSettingsDto): Promise<void> {
  const resolved = prepareUiLocaleSync(settings);
  if (resolved != null && i18n.language !== resolved) {
    await i18n.changeLanguage(resolved);
  }
}

export function useInputSourceBootstrap() {
  const setFeature = useSetAtom(inputSourceStateAtom);

  useEffect(() => {
    let cancelled = false;
    let unlistenState: (() => void) | undefined;
    let unlistenLocale: (() => void) | undefined;
    let unlistenPermission: (() => void) | undefined;

    const bootstrap = async () => {
      try {
        const [switcher, permission, settings, rules] = await Promise.all([
          getInputSourceSwitcherState(),
          getAccessibilityPermission(),
          getInputSourceSettings(),
          listRules(),
        ]);
        if (cancelled) {
          return;
        }
        await applySettingsToI18n(settings);
        setFeature({
          switcher,
          permission,
          settings,
          rules,
          initError: null,
          ready: true,
        });
      } catch (error) {
        if (cancelled) {
          return;
        }
        const message =
          error instanceof Error ? error.message : i18n.t("common.initSwitcherFailed");
        setFeature((current) => ({
          ...current,
          initError: message,
          ready: true,
        }));
      }

      try {
        unlistenState = await listen<StateEventDto>(INPUT_SOURCE_STATE_EVENT, (event) => {
          let shouldRefreshRules = false;
          setFeature((current) => {
            const previousRulesVersion = current.switcher?.rulesVersion;
            const nextSwitcher = applyNewerStateEvent(current.switcher, event.payload);
            const eventAccepted = nextSwitcher !== current.switcher;
            shouldRefreshRules = shouldRefreshRulesOnStateEvent(
              previousRulesVersion,
              nextSwitcher,
              eventAccepted
            );
            if (!eventAccepted) {
              return current;
            }
            return { ...current, switcher: nextSwitcher };
          });
          if (!shouldRefreshRules || cancelled) {
            return;
          }
          const expectedRulesVersion = event.payload.state.rulesVersion;
          void listRules()
            .then((rules) => {
              if (cancelled) {
                return;
              }
              setFeature((current) => {
                if (
                  !shouldApplyFetchedRules(current.switcher?.rulesVersion, expectedRulesVersion)
                ) {
                  return current;
                }
                return { ...current, rules };
              });
            })
            .catch(() => {
              // 一時的な IPC 失敗は次回イベント / 手動操作に委ねる
            });
        });
        unlistenLocale = await listen<ResolvedUiLocaleEventDto>(
          RESOLVED_UI_LOCALE_EVENT,
          (event) => {
            const resolved = event.payload.resolvedUiLocale;
            if (!isResolvedUiLocale(resolved)) {
              return;
            }
            setFeature((current) => {
              if (current.settings == null) {
                return current;
              }
              return {
                ...current,
                settings: { ...current.settings, resolvedUiLocale: resolved },
              };
            });
            if (!cancelled && i18n.language !== resolved) {
              void i18n.changeLanguage(resolved);
            }
          }
        );
        unlistenPermission = await listen<AccessibilityPermission>(
          ACCESSIBILITY_PERMISSION_EVENT,
          (event) => {
            const permission = event.payload;
            if (
              permission !== "granted" &&
              permission !== "denied" &&
              permission !== "unsupported"
            ) {
              return;
            }
            setFeature((current) => {
              if (current.permission === permission) {
                return current;
              }
              return { ...current, permission };
            });
          }
        );
      } catch {
        // ブラウザ単体起動など
      }
    };

    void bootstrap();

    return () => {
      cancelled = true;
      unlistenState?.();
      unlistenLocale?.();
      unlistenPermission?.();
    };
  }, [setFeature]);
}

export function useInputSourceState() {
  const [feature, setFeature] = useAtom(inputSourceStateAtom);

  const refreshRules = useCallback(async () => {
    const rules = await listRules();
    setFeature((current) => ({ ...current, rules }));
    return rules;
  }, [setFeature]);

  const refreshPermission = useCallback(async () => {
    const permission = await getAccessibilityPermission();
    setFeature((current) => ({ ...current, permission }));
    return permission;
  }, [setFeature]);

  const refreshSettings = useCallback(async () => {
    const settings = await getInputSourceSettings();
    setFeature((current) => ({ ...current, settings }));
    return settings;
  }, [setFeature]);

  const setEnabled = useCallback(
    async (enabled: boolean) => {
      const switcher = await setInputSourceSwitcherEnabled(enabled);
      applySwitcher(setFeature, switcher);
      return switcher;
    },
    [setFeature]
  );

  const setSnooze = useCallback(
    async (minutes: number) => {
      const switcher = await setSnoozeApi(minutes);
      applySwitcher(setFeature, switcher);
      return switcher;
    },
    [setFeature]
  );

  const snoozeIndefinitely = useCallback(async () => {
    const switcher = await snoozeIndefinitelyApi();
    applySwitcher(setFeature, switcher);
    return switcher;
  }, [setFeature]);

  const resume = useCallback(async () => {
    const switcher = await resumeInputSourceSwitcher();
    applySwitcher(setFeature, switcher);
    return switcher;
  }, [setFeature]);

  const requestPermission = useCallback(async () => {
    const permission = await requestAccessibilityPermission();
    setFeature((current) => ({ ...current, permission }));
    return permission;
  }, [setFeature]);

  const openAccessibilitySettings = useCallback(async () => {
    await openAccessibilitySettingsApi();
  }, []);

  const createRule = useCallback(
    async (rule: RuleDto) => {
      const switcher = await createRuleApi(rule);
      applySwitcher(setFeature, switcher);
      await refreshRules();
      return switcher;
    },
    [refreshRules, setFeature]
  );

  const updateRule = useCallback(
    async (rule: RuleDto) => {
      const switcher = await updateRuleApi(rule);
      applySwitcher(setFeature, switcher);
      await refreshRules();
      return switcher;
    },
    [refreshRules, setFeature]
  );

  const deleteRule = useCallback(
    async (id: string) => {
      const switcher = await deleteRuleApi(id);
      applySwitcher(setFeature, switcher);
      await refreshRules();
      return switcher;
    },
    [refreshRules, setFeature]
  );

  const reorderRules = useCallback(
    async (ids: readonly string[]) => {
      const switcher = await reorderRulesApi(ids);
      applySwitcher(setFeature, switcher);
      await refreshRules();
      return switcher;
    },
    [refreshRules, setFeature]
  );

  const resetDefaultRules = useCallback(async () => {
    const switcher = await resetDefaultRulesApi();
    applySwitcher(setFeature, switcher);
    await refreshRules();
    return switcher;
  }, [refreshRules, setFeature]);

  const setDebugLogging = useCallback(
    async (enabled: boolean) => {
      const switcher = await setDebugLoggingApi(enabled);
      applySwitcher(setFeature, switcher);
      await refreshSettings();
      return switcher;
    },
    [refreshSettings, setFeature]
  );

  const setUiLocale = useCallback(
    async (locale: string) => {
      const settings = await setUiLocaleApi(locale);
      setFeature((current) => ({ ...current, settings }));
      await applySettingsToI18n(settings);
      return settings;
    },
    [setFeature]
  );

  const completeOnboarding = useCallback(async () => {
    const settings = await completeOnboardingApi();
    setFeature((current) => ({ ...current, settings }));
    return settings;
  }, [setFeature]);

  const startDebugCapture = useCallback(async () => {
    const switcher = await startDebugCaptureApi();
    applySwitcher(setFeature, switcher);
    return switcher;
  }, [setFeature]);

  const stopDebugCapture = useCallback(async () => {
    const switcher = await stopDebugCaptureApi();
    applySwitcher(setFeature, switcher);
    return switcher;
  }, [setFeature]);

  const clearSwitchLogs = useCallback(async () => {
    const switcher = await clearSwitchLogsApi();
    applySwitcher(setFeature, switcher);
    return switcher;
  }, [setFeature]);

  const fetchInputSources = useCallback(
    async (refresh = false): Promise<InputSourceDto[]> => listInputSources(refresh),
    []
  );

  const fetchSwitchLogs = useCallback(async (): Promise<SwitchLogDto[]> => listSwitchLogs(), []);

  const fetchFocusSnapshot = useCallback(
    async (): Promise<FocusSnapshotDto | null> => getFocusSnapshot(),
    []
  );

  const fetchAutostartEnabled = useCallback(async () => getAutostartEnabled(), []);

  const setAutostartEnabled = useCallback(
    async (enabled: boolean) => setAutostartEnabledApi(enabled),
    []
  );

  const languagePreferences = feature.settings?.languagePreferences ?? {};
  const inputSourceDisplayNames = feature.settings?.inputSourceDisplayNames ?? {};

  const state: InputSourceViewState | null = feature.switcher
    ? {
        ...feature.switcher,
        accessibilityTrusted: feature.permission === "granted",
        permission: feature.permission,
        languagePreferences,
        inputSourceDisplayNames,
        debugLogging: feature.settings?.debugLogging ?? false,
        uiLocale: feature.settings?.uiLocale ?? null,
        rules: feature.rules,
        initError: feature.initError,
        ready: feature.ready,
      }
    : feature.ready
      ? {
          version: 0,
          rulesVersion: 0,
          enabled: false,
          snoozedUntilUnixMs: null,
          currentApp: null,
          matchedRuleId: null,
          currentInputSourceId: null,
          targetInputSourceId: null,
          lastDecision: null,
          accessibilityTrusted: feature.permission === "granted",
          permission: feature.permission,
          languagePreferences,
          inputSourceDisplayNames,
          debugLogging: feature.settings?.debugLogging ?? false,
          uiLocale: feature.settings?.uiLocale ?? null,
          rules: feature.rules,
          initError: feature.initError,
          ready: feature.ready,
        }
      : null;

  return {
    state,
    setEnabled,
    setSnooze,
    snoozeIndefinitely,
    resume,
    requestPermission,
    openAccessibilitySettings,
    refreshPermission,
    createRule,
    updateRule,
    deleteRule,
    reorderRules,
    resetDefaultRules,
    setDebugLogging,
    setUiLocale,
    completeOnboarding,
    startDebugCapture,
    stopDebugCapture,
    clearSwitchLogs,
    fetchInputSources,
    fetchSwitchLogs,
    fetchFocusSnapshot,
    fetchAutostartEnabled,
    setAutostartEnabled,
    refreshRules,
    refreshSettings,
  };
}
