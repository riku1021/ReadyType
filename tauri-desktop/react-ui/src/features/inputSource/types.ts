export type Platform = "macOs" | "windows" | "linux";
export type AccessibilityPermission = "granted" | "denied" | "unsupported";
export type SwitchDecision =
  | "ignored"
  | "disabled"
  | "snoozed"
  | "noRule"
  | "alreadySelected"
  | "switched";

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
  | "help";
export type MatchOperator = "equals" | "contains";
export type SearchScope = "focusedOnly" | "focusedAndAncestors";

export interface RuleConditionDto {
  attribute: AxAttribute;
  operator: MatchOperator;
  scope: SearchScope;
  value: string;
}

export type RuleTarget =
  | { kind: "preferredEnglish" }
  | { kind: "preferredJapanese" }
  | { kind: "exact"; inputSourceId: string };

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
  preferredEnglish: string;
  preferredJapanese: string;
  debugLogging: boolean;
}

export interface SwitchLogDto {
  timestampUnixMs: number;
  appId: string;
  ruleId: string;
  from: string;
  to: string;
  success: boolean;
  reason: string;
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
  help: string;
}

export interface FocusSnapshotDto {
  app: AppDto;
  generation: number;
  elements: AxElementDto[];
}
