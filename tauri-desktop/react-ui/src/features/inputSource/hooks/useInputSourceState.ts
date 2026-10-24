import { listen } from "@tauri-apps/api/event";
import { useAtom, useSetAtom } from "jotai";
import { useCallback, useEffect } from "react";

import i18n, { isResolvedUiLocale } from "@/locales/i18n";
import {
  ACCESSIBILITY_PERMISSION_EVENT,
  applyNewerStateEvent,
  completeOnboarding as completeOnboardingApi,
  createRule as createRuleApi,
  deleteRule as deleteRuleApi,
  getAccessibilityPermission,
  getAutostartEnabled,
  getInputSourceSettings,
  getInputSourceSwitcherState,
  INPUT_SOURCE_STATE_EVENT,
  listInputSources,
  listRules,
  openAccessibilitySettings as openAccessibilitySettingsApi,
  RESOLVED_UI_LOCALE_EVENT,
  reorderRules as reorderRulesApi,
  requestAccessibilityPermission,
  resetDefaultRules as resetDefaultRulesApi,
  resumeInputSourceSwitcher,
  setAutostartEnabled as setAutostartEnabledApi,
  setInputSourceSwitcherEnabled,
  setShowMenuBarIcon as setShowMenuBarIconApi,
  setSnooze as setSnoozeApi,
  setUiLocale as setUiLocaleApi,
  shouldApplyFetchedRules,
  shouldRefreshRulesOnStateEvent,
  snoozeInputSourceSwitcherIndefinitely as snoozeIndefinitelyApi,
  updateRule as updateRuleApi,
} from "../api";
import { type InputSourceFeatureState, inputSourceStateAtom } from "../atoms/inputSourceStateAtom";
import type {
  AccessibilityPermission,
  InputSourceDto,
  InputSourceSettingsDto,
  LanguageInputPreferences,
  ResolvedUiLocaleEventDto,
  RuleDto,
  StateEventDto,
  SwitcherStateDto,
} from "../types";
import { prepareUiLocaleSync } from "../uiLocaleSync";

export type InputSourceViewState = SwitcherStateDto & {
  accessibilityTrusted: boolean;
  permission: AccessibilityPermission | null;
  languagePreferences: LanguageInputPreferences;
  inputSourceDisplayNames: Record<string, string>;
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
        setFeature((current) => ({
          ...current,
          switcher,
          permission,
          settings,
          rules,
          initError: null,
          ready: true,
        }));
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

  const setShowMenuBarIcon = useCallback(
    async (visible: boolean) => {
      const settings = await setShowMenuBarIconApi(visible);
      setFeature((current) => ({ ...current, settings }));
      return settings;
    },
    [setFeature]
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

  const fetchInputSources = useCallback(
    async (refresh = false): Promise<InputSourceDto[]> => listInputSources(refresh),
    []
  );

  const refreshAutostartEnabled = useCallback(async () => {
    const enabled = await getAutostartEnabled();
    setFeature((current) =>
      current.autostartEnabled === enabled ? current : { ...current, autostartEnabled: enabled }
    );
    return enabled;
  }, [setFeature]);

  const setAutostartEnabled = useCallback(
    async (enabled: boolean) => {
      const next = await setAutostartEnabledApi(enabled);
      setFeature((current) => ({ ...current, autostartEnabled: next }));
      return next;
    },
    [setFeature]
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
    showMenuBarIcon: feature.settings?.showMenuBarIcon ?? true,
    setShowMenuBarIcon,
    setUiLocale,
    completeOnboarding,
    fetchInputSources,
    autostartEnabled: feature.autostartEnabled,
    refreshAutostartEnabled,
    setAutostartEnabled,
    refreshRules,
    refreshSettings,
  };
}
