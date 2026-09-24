import type {
  AxAttribute,
  MatchOperator,
  RuleConditionDto,
  RuleDto,
  RuleTarget,
  SearchScope,
} from "@/features/inputSource";

export type RuleFormCondition = {
  key: string;
  attribute: AxAttribute;
  operator: MatchOperator;
  scope: SearchScope;
  value: string;
};

export type RuleFormState = {
  id: string;
  name: string;
  enabled: boolean;
  platform: "macOs" | "windows" | "linux";
  appId: string;
  priority: number;
  conditions: RuleFormCondition[];
  targetKind: "preferredEnglish" | "preferredJapanese" | "exact";
  exactInputSourceId: string;
};

export type RuleFormError = {
  field: string;
  message: string;
};

export function emptyRuleForm(id: string, priority: number): RuleFormState {
  return {
    id,
    name: "",
    enabled: true,
    platform: "macOs",
    appId: "",
    priority,
    conditions: [],
    targetKind: "preferredEnglish",
    exactInputSourceId: "",
  };
}

export function ruleToForm(rule: RuleDto): RuleFormState {
  return {
    id: rule.id,
    name: rule.name,
    enabled: rule.enabled,
    platform: rule.platform,
    appId: rule.appId,
    priority: rule.priority,
    conditions: rule.conditions.map((condition) => ({
      key: crypto.randomUUID(),
      ...condition,
    })),
    targetKind: rule.target.kind,
    exactInputSourceId: rule.target.kind === "exact" ? rule.target.inputSourceId : "",
  };
}

export function validateRuleForm(form: RuleFormState): RuleFormError | null {
  if (!form.name.trim()) {
    return { field: "name", message: "名前を入力してください" };
  }
  if (!form.appId.trim()) {
    return { field: "appId", message: "アプリ識別子を入力してください" };
  }
  if (form.targetKind === "exact" && !form.exactInputSourceId.trim()) {
    return { field: "exactInputSourceId", message: "入力ソースIDを入力してください" };
  }
  for (const [index, condition] of form.conditions.entries()) {
    if (!condition.value.trim()) {
      return {
        field: `conditions.${index}.value`,
        message: "比較値を入力してください",
      };
    }
  }
  return null;
}

export function toRuleDto(form: RuleFormState): RuleDto {
  const target: RuleTarget =
    form.targetKind === "exact"
      ? { kind: "exact", inputSourceId: form.exactInputSourceId.trim() }
      : { kind: form.targetKind };

  return {
    id: form.id,
    name: form.name.trim(),
    enabled: form.enabled,
    platform: form.platform,
    appId: form.appId.trim(),
    priority: form.priority,
    conditions: form.conditions.map(
      (condition): RuleConditionDto => ({
        attribute: condition.attribute,
        operator: condition.operator,
        scope: condition.scope,
        value: condition.value.trim(),
      })
    ),
    target,
  };
}

export function duplicateRule(rule: RuleDto, id: string): RuleDto {
  return {
    ...rule,
    id,
    name: `${rule.name} のコピー`,
    priority: rule.priority + 1,
  };
}

export function moveRule(rules: readonly RuleDto[], id: string, offset: -1 | 1): RuleDto[] {
  const from = rules.findIndex((rule) => rule.id === id);
  const to = from + offset;
  if (from < 0 || to < 0 || to >= rules.length) {
    return [...rules];
  }
  const reordered = [...rules];
  const left = reordered[from];
  const right = reordered[to];
  if (!left || !right) {
    return [...rules];
  }
  reordered[from] = right;
  reordered[to] = left;
  return reordered.map((rule, priority) => ({ ...rule, priority }));
}

export function summarizeConditions(conditions: readonly RuleConditionDto[]): string {
  if (conditions.length === 0) {
    return "条件なし（アプリ一致）";
  }
  return conditions
    .map((condition) => `${condition.attribute} ${condition.operator} ${condition.value}`)
    .join(" / ");
}

export function summarizeTarget(target: RuleTarget): string {
  switch (target.kind) {
    case "preferredEnglish":
      return "既定の英数";
    case "preferredJapanese":
      return "既定の日本語";
    case "exact":
      return target.inputSourceId;
  }
}
