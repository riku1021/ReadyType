import type { InputSourceDto, LanguageInputPreferences, RuleDto, RuleTarget } from "./types";

/** ルール target を言語別設定で解決した入力ソース ID（未設定は空文字）。 */
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

/** 空 ID（未設定）は欠損扱いしない。一覧に無い / available=false は欠損。 */
export function isSourceMissing(id: string, sources: readonly InputSourceDto[]): boolean {
  if (id.length === 0) {
    return false;
  }
  const source = sources.find((candidate) => candidate.id === id);
  return source === undefined || !source.available;
}

/** ルール target と言語別設定から、設定済みの入力ソース ID を重複なしで集める。 */
export function collectConfiguredInputSourceIds(
  rules: readonly RuleDto[],
  preferences: LanguageInputPreferences
): string[] {
  const ids = new Set<string>();
  for (const rule of rules) {
    ids.add(resolvedInputSourceId(rule.target, preferences));
  }
  for (const id of Object.values(preferences)) {
    if (id != null) {
      ids.add(id);
    }
  }
  ids.delete("");
  return [...ids];
}

export function appHasMissingInputSource(
  appId: string,
  rules: readonly RuleDto[],
  preferences: LanguageInputPreferences,
  sources: readonly InputSourceDto[]
): boolean {
  return rules.some(
    (rule) =>
      rule.appId === appId &&
      isSourceMissing(resolvedInputSourceId(rule.target, preferences), sources)
  );
}

export function anyAppHasMissingInputSource(
  rules: readonly RuleDto[],
  preferences: LanguageInputPreferences,
  sources: readonly InputSourceDto[]
): boolean {
  return rules.some((rule) =>
    isSourceMissing(resolvedInputSourceId(rule.target, preferences), sources)
  );
}
