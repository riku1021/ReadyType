/** Playwright 用の固定 IPC レスポンス（ブラウザ単体起動向け） */

export const mockSwitcherState = {
  version: 1,
  rulesVersion: 1,
  enabled: true,
  snoozedUntilUnixMs: null,
  currentApp: null,
  matchedRuleId: null,
  currentInputSourceId: null,
  targetInputSourceId: null,
  lastDecision: null,
} as const;

export const mockSettings = {
  languagePreferences: {
    en: "com.apple.keylayout.ABC",
    ja: "com.apple.inputmethod.Kotoeri.RomajiTyping",
  },
  uiLocale: "system",
  resolvedUiLocale: "ja",
  quickSetShortcut: "CommandOrControl+Shift+L",
  quickSetShortcutRegistered: true,
  onboardingCompleted: true,
  showMenuBarIcon: true,
  inputSourceDisplayNames: {
    "com.apple.keylayout.ABC": "ABC",
    "com.apple.inputmethod.Kotoeri.RomajiTyping": "ひらがな",
  },
} as const;

const axContains = (attribute: string, value: string) => ({
  attribute,
  operator: "contains" as const,
  scope: "focusedAndAncestors" as const,
  value,
});

export const mockRules = [
  {
    id: "preset-terminal",
    name: "Terminal",
    enabled: true,
    platform: "macOs" as const,
    appId: "com.apple.Terminal",
    priority: 0,
    conditions: [],
    target: { kind: "preferredLanguage" as const, language: "en" },
  },
  {
    id: "preset-cursor-composer",
    name: "Cursor Composer",
    enabled: true,
    platform: "macOs" as const,
    appId: "com.todesktop.230313mzl4w4u92",
    priority: 111,
    conditions: [axContains("domClassList", "composer-")],
    target: { kind: "preferredLanguage" as const, language: "ja" },
  },
  {
    id: "preset-cursor-editor",
    name: "Cursor Editor",
    enabled: true,
    platform: "macOs" as const,
    appId: "com.todesktop.230313mzl4w4u92",
    priority: 130,
    conditions: [axContains("domIdentifier", "workbench.parts.editor")],
    target: { kind: "preferredLanguage" as const, language: "en" },
  },
  {
    id: "preset-cursor-terminal-en",
    name: "Cursor Terminal",
    enabled: true,
    platform: "macOs" as const,
    appId: "com.todesktop.230313mzl4w4u92",
    priority: 100,
    conditions: [axContains("role", "text"), axContains("description", "terminal")],
    target: { kind: "preferredLanguage" as const, language: "en" },
  },
];
