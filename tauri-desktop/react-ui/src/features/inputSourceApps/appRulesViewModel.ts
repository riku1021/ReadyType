import type { RuleDto, RuleTarget } from "@/features/inputSource";
import { summarizeConditions } from "@/features/inputSourceRules/ruleForm";

const APP_LABELS: Record<string, string> = {
  "com.todesktop.230313mzl4w4u92": "Cursor",
  "com.microsoft.VSCode": "Visual Studio Code",
  "com.apple.Terminal": "Terminal",
};

export type RuleDisplay = {
  rule: RuleDto;
  title: string;
  subtitle: string;
};

export type AppRuleGroup = {
  appId: string;
  label: string;
  enabled: boolean;
  ruleCount: number;
  defaultRule: RuleDto | null;
  contextualRules: RuleDisplay[];
};

export function appDisplayName(appId: string): string {
  return APP_LABELS[appId] ?? appId;
}

export function displayRuleLabel(rule: RuleDto): Omit<RuleDisplay, "rule"> {
  const id = rule.id.toLowerCase();
  if (rule.conditions.length === 0) {
    return {
      title: "デフォルト",
      subtitle: "このアプリのすべての入力欄で使う既定の言語",
    };
  }
  if (id.includes("chat") || id.includes("composer") || id.includes("ai-input")) {
    return {
      title: "Chat（AIチャット欄）",
      subtitle: "右サイドバーのチャット入力欄",
    };
  }
  if (id.includes("editor")) {
    return {
      title: "Editor（コードエディタ）",
      subtitle: "メインのエディタ入力欄",
    };
  }
  if (id.includes("terminal")) {
    return {
      title: "Terminal（内蔵ターミナル）",
      subtitle: "ターミナルのコマンド入力欄",
    };
  }
  if (id.includes("rename")) {
    return {
      title: "Rename（ファイル名変更）",
      subtitle: "ファイル名の入力欄",
    };
  }
  return {
    title: rule.name,
    subtitle: summarizeConditions(rule.conditions),
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

export function groupRulesByApp(rules: readonly RuleDto[]): AppRuleGroup[] {
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
        .map((rule) => ({ rule, ...displayRuleLabel(rule) }));
      return {
        appId,
        label: appDisplayName(appId),
        enabled: appRules.some((rule) => rule.enabled),
        ruleCount: appRules.length,
        defaultRule,
        contextualRules,
      };
    })
    .sort((left, right) => left.label.localeCompare(right.label, "ja"));
}

export function summarizeLanguageTarget(target: RuleTarget): string {
  switch (target.kind) {
    case "preferredEnglish":
      return "English（英語）";
    case "preferredJapanese":
      return "日本語";
    case "exact":
      return target.inputSourceId;
  }
}

export function languageTargetFromLabel(label: string): RuleTarget | null {
  if (label === "English（英語）" || label === "preferredEnglish") {
    return { kind: "preferredEnglish" };
  }
  if (label === "日本語" || label === "preferredJapanese") {
    return { kind: "preferredJapanese" };
  }
  return null;
}

export const LANGUAGE_TARGET_OPTIONS = ["English（英語）", "日本語"] as const;
