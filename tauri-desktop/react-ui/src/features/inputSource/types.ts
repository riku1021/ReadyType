export type Platform = "macOs" | "windows" | "linux";
export type AccessibilityPermission = "granted" | "denied" | "unsupported";
export type SwitchDecision =
  | "ignored"
  | "disabled"
  | "snoozed"
  | "noRule"
  | "alreadySelected"
  | "switched"
  | "failed";

export interface AppDto {
  platform: Platform;
  appId: string;
  pid: number;
}

export interface SwitcherStateDto {
  version: number;
  rulesVersion: number;
  enabled: boolean;
  snoozedUntilUnixMs: number | null;
  currentApp: AppDto | null;
  matchedRuleId: string | null;
  currentInputSourceId: string | null;
  targetInputSourceId: string | null;
  lastDecision: SwitchDecision | null;
}

export interface StateEventDto {
  version: number;
  state: SwitcherStateDto;
}

export interface InputSourceDto {
  id: string;
  name: string;
  languageCode: string | null;
  available: boolean;
}

export type AxAttribute =
  | "role"
  | "subrole"
  | "roleDescription"
  | "title"
  | "description"
  | "identifier"
  | "domIdentifier"
  | "domClassList"
  | "help"
  | "placeholder";
export type MatchOperator = "equals" | "contains";
export type SearchScope = "focusedOnly" | "focusedAndAncestors";

export interface RuleConditionDto {
  attribute: AxAttribute;
  operator: MatchOperator;
  scope: SearchScope;
  value: string;
}

export type RuleTarget =
  | { kind: "preferredLanguage"; language: string }
  | { kind: "exact"; inputSourceId: string };

export type LanguageInputPreferences = Record<string, string | null>;

export interface RuleDto {
  id: string;
  name: string;
  enabled: boolean;
  platform: Platform;
  appId: string;
  priority: number;
  conditions: RuleConditionDto[];
  target: RuleTarget;
}

export interface InputSourceSettingsDto {
  languagePreferences: LanguageInputPreferences;
  debugLogging: boolean;
  /** Preference: system | packs from locales/registry.json (FE) / tray.json (Rust) */
  uiLocale: string;
  /** Resolved display language pack code */
  resolvedUiLocale: string;
  /** Quick Set グローバルショートカット（Tauri accelerator） */
  quickSetShortcut: string;
  /** 起動時登録を含め、現在 OS に登録されているか */
  quickSetShortcutRegistered: boolean;
  /** 初回オンボーディング完了済みか */
  onboardingCompleted: boolean;
}

export interface SwitchLogDto {
  timestampUnixMs: number;
  appId: string;
  pid: number;
  generation: number;
  ruleId: string;
  decision: SwitchDecision;
  from: string;
  to: string;
  success: boolean;
  attempted: boolean;
  reason: string;
  detail: string;
  processingMicros: number;
}

export interface AxElementDto {
  role: string;
  subrole: string;
  roleDescription: string;
  title: string;
  description: string;
  identifier: string;
  domIdentifier: string;
  domClassList: string;
  help: string;
  placeholder: string;
}

export interface FocusSnapshotDto {
  app: AppDto;
  generation: number;
  elements: AxElementDto[];
}
