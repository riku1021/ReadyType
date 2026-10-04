import type { TFunction } from "i18next";

import type { RuleConditionDto, RuleDto, RuleTarget } from "@/features/inputSource";
import i18n from "@/locales/i18n";

/** OS 表示名が取れないときだけ使う（ブラウザ単体 / ネイティブ未接続）。 */
const FALLBACK_APP_LABELS: Record<string, string> = {
  "com.todesktop.230313mzl4w4u92": "Cursor",
  "com.microsoft.VSCode": "Visual Studio Code",
  "com.apple.Terminal": "Terminal",
};

export type RuleKind = "chat" | "editor" | "terminal" | "rename" | "default" | "other";

export type RuleDisplay = {
  rule: RuleDto;
  title: string;
  subtitle: string;
  kind: RuleKind;
};

export type AppRuleGroup = {
  appId: string;
  label: string;
  enabled: boolean;
  ruleCount: number;
  defaultRule: RuleDto | null;
  contextualRules: RuleDisplay[];
};

/** OS 表示名 → オフライン用フォールバック → bundle id。 */
export function appDisplayName(appId: string, osDisplayName?: string | null): string {
  const trimmed = osDisplayName?.trim();
  if (trimmed) {
    return trimmed;
  }
  return FALLBACK_APP_LABELS[appId] ?? appId;
}

export function summarizeConditions(conditions: readonly RuleConditionDto[], t: TFunction): string {
  if (conditions.length === 0) {
    return t("rules.conditionsNone");
  }
  return conditions
    .map((condition) => `${condition.attribute} ${condition.operator} ${condition.value}`)
    .join(" / ");
}

export function displayRuleLabel(rule: RuleDto, t: TFunction): Omit<RuleDisplay, "rule"> {
  const id = rule.id.toLowerCase();
  if (rule.conditions.length === 0) {
    return {
      title: t("rules.defaultTitle"),
      subtitle: t("rules.defaultSubtitle"),
      kind: "default",
    };
  }
  if (id.includes("chat") || id.includes("composer") || id.includes("ai-input")) {
    return {
      title: t("rules.chatTitle"),
      subtitle: t("rules.chatSubtitle"),
      kind: "chat",
    };
  }
  if (id.includes("editor")) {
    return {
      title: t("rules.editorTitle"),
      subtitle: t("rules.editorSubtitle"),
      kind: "editor",
    };
  }
  if (id.includes("terminal")) {
    return {
      title: t("rules.terminalTitle"),
      subtitle: t("rules.terminalSubtitle"),
      kind: "terminal",
    };
  }
  if (id.includes("rename")) {
    return {
      title: t("rules.renameTitle"),
      subtitle: t("rules.renameSubtitle"),
      kind: "rename",
    };
  }
  return {
    title: rule.name,
    subtitle: summarizeConditions(rule.conditions, t),
    kind: "other",
  };
}

function contextSortKey(rule: RuleDto): number {
  const id = rule.id.toLowerCase();
  if (id.includes("chat") || id.includes("composer") || id.includes("ai-input")) {
    return 0;
  }
  if (id.includes("editor")) {
    return 1;
  }
  if (id.includes("terminal")) {
    return 2;
  }
  if (id.includes("rename")) {
    return 3;
  }
  return 10 + rule.priority;
}

export function groupRulesByApp(rules: readonly RuleDto[], t: TFunction): AppRuleGroup[] {
  const byApp = new Map<string, RuleDto[]>();
  for (const rule of rules) {
    const list = byApp.get(rule.appId) ?? [];
    list.push(rule);
    byApp.set(rule.appId, list);
  }

  return [...byApp.entries()]
    .map(([appId, appRules]) => {
      const sorted = [...appRules].sort((left, right) => left.priority - right.priority);
      const defaultRule = sorted.find((rule) => rule.conditions.length === 0) ?? null;
      const contextualRules = sorted
        .filter((rule) => rule.conditions.length > 0)
        .sort((left, right) => contextSortKey(left) - contextSortKey(right))
        .map((rule) => ({ rule, ...displayRuleLabel(rule, t) }));
      return {
        appId,
        label: appDisplayName(appId),
        enabled: appRules.some((rule) => rule.enabled),
        ruleCount: appRules.length,
        defaultRule,
        contextualRules,
      };
    })
    .sort((left, right) => left.label.localeCompare(right.label, i18n.language));
}

export function resolvedInputSourceId(
  target: RuleTarget,
  preferences: Record<string, string | null | undefined> | null | undefined
): string {
  switch (target.kind) {
    case "preferredLanguage":
      return preferences?.[target.language] ?? "";
    case "exact":
      return target.inputSourceId;
  }
}

export function ruleTargetFromInputSourceId(
  inputSourceId: string,
  preferences: Record<string, string | null | undefined> | null | undefined
): RuleTarget {
  if (preferences) {
    for (const [language, preferredId] of Object.entries(preferences)) {
      if (preferredId && inputSourceId === preferredId) {
        return { kind: "preferredLanguage", language };
      }
    }
  }
  return { kind: "exact", inputSourceId };
}
