import type { TFunction } from "i18next";

import type { RuleDto, RuleTarget } from "@/features/inputSource";

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
  kind: RuleKind;
};

export type AppRuleGroup = {
  appId: string;
  label: string;
  enabled: boolean;
  ruleCount: number;
  /** ルール配列でこのアプリが最初に現れた位置。追加順のキー。 */
  addedIndex: number;
  defaultRule: RuleDto | null;
  contextualRules: RuleDisplay[];
};

/** 一覧の並び。名前順が既定。 */
export type AppListSort = "name" | "added" | "ruleCount";

type AppListSortable = {
  label: string;
  ruleCount: number;
  addedIndex: number;
};

/** OS 表示名 → オフライン用フォールバック → bundle id。 */
export function appDisplayName(appId: string, osDisplayName?: string | null): string {
  const trimmed = osDisplayName?.trim();
  if (trimmed) {
    return trimmed;
  }
  return FALLBACK_APP_LABELS[appId] ?? appId;
}

function ruleKind(rule: RuleDto): RuleKind {
  if (rule.conditions.length === 0) {
    return "default";
  }
  const id = rule.id.toLowerCase();
  if (id.includes("chat") || id.includes("composer") || id.includes("ai-input")) {
    return "chat";
  }
  if (id.includes("editor")) {
    return "editor";
  }
  if (id.includes("terminal")) {
    return "terminal";
  }
  if (id.includes("rename")) {
    return "rename";
  }
  return "other";
}

/** Contextual rules use persisted `rule.name` (user edit is truth). Defaults stay i18n. */
export function displayRuleLabel(rule: RuleDto, t: TFunction): Omit<RuleDisplay, "rule"> {
  const kind = ruleKind(rule);
  if (kind === "default") {
    return {
      title: t("rules.defaultTitle"),
      kind,
    };
  }
  return {
    title: rule.name,
    kind,
  };
}

function contextSortKey(rule: RuleDto): number {
  switch (ruleKind(rule)) {
    case "chat":
      return 0;
    case "editor":
      return 1;
    case "terminal":
      return 2;
    case "rename":
      return 3;
    default:
      return 10 + rule.priority;
  }
}

export function groupRulesByApp(rules: readonly RuleDto[], t: TFunction): AppRuleGroup[] {
  const byApp = new Map<string, { addedIndex: number; rules: RuleDto[] }>();
  for (const [index, rule] of rules.entries()) {
    const bucket = byApp.get(rule.appId);
    if (bucket) {
      bucket.rules.push(rule);
    } else {
      byApp.set(rule.appId, { addedIndex: index, rules: [rule] });
    }
  }

  // 初出順。表示用の並びは sortAppListRows が解決済みの表示名で決める。
  return [...byApp.entries()].map(([appId, { addedIndex, rules: appRules }]) => {
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
      addedIndex,
      defaultRule,
      contextualRules,
    };
  });
}

function compareByName(left: AppListSortable, right: AppListSortable, language: string): number {
  const byLabel = left.label.localeCompare(right.label, language, {
    numeric: true,
    sensitivity: "accent",
  });
  if (byLabel !== 0) {
    return byLabel;
  }
  return left.addedIndex - right.addedIndex;
}

/** 表示名が解決された行を並べる。同点は追加順で安定させる。 */
export function sortAppListRows<T extends AppListSortable>(
  rows: readonly T[],
  sort: AppListSort,
  language: string
): T[] {
  return [...rows].sort((left, right) => {
    if (sort === "added") {
      return left.addedIndex - right.addedIndex;
    }
    if (sort === "ruleCount") {
      const byCount = right.ruleCount - left.ruleCount;
      if (byCount !== 0) {
        return byCount;
      }
    }
    return compareByName(left, right, language);
  });
}

export { resolvedInputSourceId } from "@/features/inputSource";

/** 表示名の部分一致（大文字小文字無視）。空ならそのまま返す。 */
export function filterAppListRows<T extends { label: string }>(rows: T[], search: string): T[] {
  const keyword = search.trim().toLowerCase();
  if (keyword === "") {
    return rows;
  }
  return rows.filter((row) => row.label.toLowerCase().includes(keyword));
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
