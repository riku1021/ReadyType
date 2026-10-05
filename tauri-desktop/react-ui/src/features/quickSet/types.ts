import type {
  InputSourceDto,
  LanguageInputPreferences,
  RuleConditionDto,
  RuleTarget,
} from "@/features/inputSource";

export type QuickSetSessionDto = {
  appId: string;
  appDisplayName: string;
  platform: "macOs" | "windows" | "linux";
  pid: number;
  proposedName: string;
  proposedConditions: RuleConditionDto[];
  matchedRuleId: string | null;
  matchedTarget: RuleTarget | null;
  sources: InputSourceDto[];
  languagePreferences: LanguageInputPreferences;
};

export const QUICK_SET_EVENT = "readytype://quick-set";
export const NAVIGATE_EVENT = "readytype://navigate";
