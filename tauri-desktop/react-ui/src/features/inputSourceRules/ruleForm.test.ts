import { describe, expect, it } from "vitest";

import type { RuleDto } from "@/features/inputSource";

import { duplicateRule, emptyRuleForm, moveRule, toRuleDto, validateRuleForm } from "./ruleForm";

function ruleAtPriority(priority: number): RuleDto {
  return {
    id: `rule-${priority}`,
    name: "Rule",
    enabled: true,
    platform: "macOs",
    appId: "com.example",
    priority,
    conditions: [],
    target: { kind: "preferredEnglish" },
  };
}

function formWithCondition(value: string) {
  const form = emptyRuleForm("id", 0);
  form.name = "Name";
  form.appId = "com.example";
  form.conditions = [
    {
      key: crypto.randomUUID(),
      attribute: "role",
      operator: "equals",
      scope: "focusedOnly",
      value,
    },
  ];
  return form;
}

describe("ruleForm", () => {
  it("rejects blank condition values", () => {
    expect(validateRuleForm(formWithCondition("   "))).toEqual({
      field: "conditions.0.value",
      message: "比較値を入力してください",
    });
  });

  it("rejects blank name and app id", () => {
    expect(validateRuleForm(emptyRuleForm("id", 0))?.field).toBe("name");
    const form = emptyRuleForm("id", 0);
    form.name = "Name";
    expect(validateRuleForm(form)?.field).toBe("appId");
  });

  it("rejects exact target without id", () => {
    const form = emptyRuleForm("id", 0);
    form.name = "Name";
    form.appId = "com.example";
    form.targetKind = "exact";
    expect(validateRuleForm(form)?.field).toBe("exactInputSourceId");
  });

  it("duplicates with a fresh id and adjacent priority", () => {
    const copy = duplicateRule(ruleAtPriority(2), "new-id");
    expect(copy).toMatchObject({
      id: "new-id",
      priority: 3,
      name: "Rule のコピー",
    });
  });

  it("moves rules up and down with stable priorities", () => {
    const rules = [ruleAtPriority(0), ruleAtPriority(1), ruleAtPriority(2)];
    const moved = moveRule(rules, "rule-1", -1);
    expect(moved.map((rule) => rule.id)).toEqual(["rule-1", "rule-0", "rule-2"]);
    expect(moved.map((rule) => rule.priority)).toEqual([0, 1, 2]);
  });

  it("converts a valid form to a DTO", () => {
    const form = formWithCondition("AXTextArea");
    expect(toRuleDto(form).conditions[0]?.value).toBe("AXTextArea");
  });
});
