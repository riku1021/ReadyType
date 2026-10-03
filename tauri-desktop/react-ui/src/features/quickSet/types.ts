import type { RuleConditionDto, RuleTarget } from "@/features/inputSource";

export type QuickSetLanguageDto = {
  language: string;
  preferredSourceId: string;
  preferredSourceName: string;
};

export type QuickSetSessionDto = {
  appId: string;
  appDisplayName: string;
  platform: "macOs" | "windows" | "linux";
  pid: number;
  fieldLabel: string;
  proposedName: string;
  proposedConditions: RuleConditionDto[];
  matchedRuleId: string | null;
  matchedTarget: RuleTarget | null;
  languages: QuickSetLanguageDto[];
};

export const QUICK_SET_EVENT = "readytype://quick-set";
export const NAVIGATE_EVENT = "readytype://navigate";
