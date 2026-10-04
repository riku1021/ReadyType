import type { Page } from "@playwright/test";

import {
  DEFAULT_RESOLVED_LANGUAGE,
  LANGUAGE_PREFERENCES,
  SUPPORTED_LANGUAGES,
} from "../../src/locales/i18n";
import { mockRules, mockSettings, mockSwitcherState } from "./mockData";

type MockPayload = {
  switcher: typeof mockSwitcherState;
  settings: typeof mockSettings;
  rules: typeof mockRules;
  languagePreferences: readonly string[];
  supportedLanguages: readonly string[];
  defaultResolved: string;
};

/**
 * ページ読み込み前に Tauri IPC をモックする。
 * `goto` より先に呼ぶこと。
 */
export async function installTauriMocks(page: Page): Promise<void> {
  const payload: MockPayload = {
    switcher: mockSwitcherState,
    settings: mockSettings,
    rules: mockRules,
    languagePreferences: [...LANGUAGE_PREFERENCES],
    supportedLanguages: [...SUPPORTED_LANGUAGES],
    defaultResolved: DEFAULT_RESOLVED_LANGUAGE,
  };

  await page.addInitScript((data: MockPayload) => {
    type InvokeArgs = Record<string, unknown> | undefined;

    const callbacks = new Map<number, (value: unknown) => void>();
    let nextCallbackId = 1;

    const registerCallback = (callback: (value: unknown) => void, once = false) => {
      const id = nextCallbackId;
      nextCallbackId += 1;
      callbacks.set(id, (value) => {
        if (once) {
          callbacks.delete(id);
        }
        callback(value);
      });
      return id;
    };

    const invoke = async (cmd: string, args?: InvokeArgs): Promise<unknown> => {
      switch (cmd) {
        case "get_input_source_switcher_state":
        case "set_input_source_switcher_enabled":
        case "set_snooze":
        case "snooze_input_source_switcher_indefinitely":
        case "resume_input_source_switcher":
        case "create_rule":
        case "update_rule":
        case "delete_rule":
        case "reorder_rules":
        case "reset_default_rules":
        case "start_debug_capture":
        case "stop_debug_capture":
        case "set_debug_logging":
        case "clear_switch_logs":
          return { ...data.switcher };
        case "get_accessibility_permission":
        case "request_accessibility_permission":
          return "granted";
        case "get_input_source_settings":
          return { ...data.settings };
        case "set_ui_locale": {
          const locale =
            typeof args?.locale === "string" && data.languagePreferences.includes(args.locale)
              ? args.locale
              : "system";
          const resolvedUiLocale = data.supportedLanguages.includes(locale)
            ? locale
            : data.defaultResolved;
          data.settings = { ...data.settings, uiLocale: locale, resolvedUiLocale };
          return { ...data.settings };
        }
        case "list_rules":
          return data.rules.map((rule) => ({ ...rule, conditions: [...rule.conditions] }));
        case "list_input_sources":
          return [
            {
              id: data.settings.languagePreferences.en,
              name: "ABC",
              languageCode: "en",
              available: true,
            },
            {
              id: data.settings.languagePreferences.ja,
              name: "日本語",
              languageCode: "ja",
              available: true,
            },
          ];
        case "list_switch_logs":
          return [];
        case "get_focus_snapshot":
          return null;
        case "open_accessibility_settings":
        case "sync_window_chrome_theme":
        case "quit_app":
          return null;
        case "should_show_main_window_on_launch":
          return true;
        case "get_autostart_enabled":
        case "set_autostart_enabled":
          return false;
        case "get_app_presentations": {
          const ids = Array.isArray(args?.bundleIds) ? args.bundleIds : [];
          const presentations: Record<string, { icon: string; displayName: string }> = {};
          const pixel =
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
          for (const id of ids) {
            if (typeof id === "string") {
              presentations[id] = {
                icon: pixel,
                displayName: id.split(".").at(-1) ?? id,
              };
            }
          }
          return presentations;
        }
        case "plugin:event|listen":
        case "plugin:event|unlisten":
        case "plugin:event|emit":
        case "plugin:event|emit_to":
          return null;
        default:
          if (typeof cmd === "string" && cmd.startsWith("plugin:window|")) {
            return null;
          }
          console.warn(`[e2e] unmocked invoke: ${cmd}`, args);
          return null;
      }
    };

    const internals = {
      invoke,
      transformCallback: registerCallback,
      unregisterCallback: (id: number) => {
        callbacks.delete(id);
      },
      runCallback: (id: number, value: unknown) => {
        callbacks.get(id)?.(value);
      },
      callbacks,
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {
        ...internals,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { windowLabel: "main", label: "main" },
        },
      },
    });
    // isTauri() は window.isTauri を見る。ブラウザ E2E では false のままにし、
    // ネイティブ window.show を走らせない。
    Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
      configurable: true,
      value: {
        unregisterListener: () => undefined,
      },
    });
  }, payload);
}
