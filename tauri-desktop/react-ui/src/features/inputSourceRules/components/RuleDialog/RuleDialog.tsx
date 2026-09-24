import { useState } from "react";

import { Button } from "@/components/Button";
import { FormModal } from "@/components/FormModal";
import type { RuleConditionDto, RuleDto } from "@/features/inputSource";

import {
  emptyRuleForm,
  type RuleFormState,
  ruleToForm,
  toRuleDto,
  validateRuleForm,
} from "../../ruleForm";
import * as styles from "./RuleDialog.styles";

type RuleDialogProps = {
  isOpen: boolean;
  mode: "create" | "edit";
  initialRule?: RuleDto;
  initialCondition?: RuleConditionDto;
  nextPriority: number;
  onClose: () => void;
  onSubmit: (rule: RuleDto) => Promise<void>;
};

const ATTRIBUTES = [
  "role",
  "subrole",
  "roleDescription",
  "title",
  "description",
  "identifier",
  "domIdentifier",
  "domClassList",
  "help",
] as const;

export function RuleDialog({
  isOpen,
  mode,
  initialRule,
  initialCondition,
  nextPriority,
  onClose,
  onSubmit,
}: RuleDialogProps) {
  const [form, setForm] = useState<RuleFormState>(() => {
    const withPrefill = (condition: RuleConditionDto | undefined, base: RuleFormState) => {
      if (!condition) {
        return base;
      }
      return {
        ...base,
        conditions: [
          ...base.conditions,
          {
            key: crypto.randomUUID(),
            ...condition,
          },
        ],
      };
    };
    if (initialRule) {
      return withPrefill(initialCondition, ruleToForm(initialRule));
    }
    return withPrefill(initialCondition, emptyRuleForm(crypto.randomUUID(), nextPriority));
  });
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const update = (patch: Partial<RuleFormState>) => {
    setForm((current) => ({ ...current, ...patch }));
  };

  const handleSubmit = async () => {
    const validation = validateRuleForm(form);
    if (validation) {
      setError(validation.message);
      return;
    }
    setPending(true);
    setError(null);
    try {
      await onSubmit(toRuleDto(form));
      onClose();
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : "保存に失敗しました");
    } finally {
      setPending(false);
    }
  };

  return (
    <FormModal
      isOpen={isOpen}
      onClose={onClose}
      title={mode === "create" ? "ルールを追加" : "ルールを編集"}
      footer={
        <div className={styles.footer}>
          <Button variant="secondary" onClick={onClose} disabled={pending}>
            キャンセル
          </Button>
          <Button
            disabled={pending}
            onClick={() => {
              void handleSubmit();
            }}
          >
            保存
          </Button>
        </div>
      }
    >
      <div className={styles.form}>
        <label className={styles.field}>
          <span className={styles.label}>名前</span>
          <input
            className={styles.input}
            value={form.name}
            onChange={(event) => update({ name: event.target.value })}
          />
        </label>
        <label className={styles.field}>
          <span className={styles.label}>アプリ識別子</span>
          <input
            className={styles.input}
            value={form.appId}
            onChange={(event) => update({ appId: event.target.value })}
          />
        </label>
        <label className={styles.field}>
          <span className={styles.label}>切替先</span>
          <select
            className={styles.input}
            value={form.targetKind}
            onChange={(event) =>
              update({
                targetKind: event.target.value as RuleFormState["targetKind"],
              })
            }
          >
            <option value="preferredEnglish">既定の英数</option>
            <option value="preferredJapanese">既定の日本語</option>
            <option value="exact">具体的な入力ソース</option>
          </select>
        </label>
        {form.targetKind === "exact" ? (
          <label className={styles.field}>
            <span className={styles.label}>入力ソースID</span>
            <input
              className={styles.input}
              value={form.exactInputSourceId}
              onChange={(event) => update({ exactInputSourceId: event.target.value })}
            />
          </label>
        ) : null}

        {form.conditions.map((condition, index) => (
          <div key={condition.key} className={styles.conditionBlock}>
            <label className={styles.field}>
              <span className={styles.label}>属性</span>
              <select
                className={styles.input}
                value={condition.attribute}
                onChange={(event) => {
                  const attribute = event.target.value as (typeof ATTRIBUTES)[number];
                  setForm((current) => {
                    const conditions = [...current.conditions];
                    const existing = conditions[index];
                    if (!existing) {
                      return current;
                    }
                    conditions[index] = { ...existing, attribute };
                    return { ...current, conditions };
                  });
                }}
              >
                {ATTRIBUTES.map((attribute) => (
                  <option key={attribute} value={attribute}>
                    {attribute}
                  </option>
                ))}
              </select>
            </label>
            <label className={styles.field}>
              <span className={styles.label}>比較</span>
              <select
                className={styles.input}
                value={condition.operator}
                onChange={(event) => {
                  const operator = event.target.value as "equals" | "contains";
                  setForm((current) => {
                    const conditions = [...current.conditions];
                    const existing = conditions[index];
                    if (!existing) {
                      return current;
                    }
                    conditions[index] = { ...existing, operator };
                    return { ...current, conditions };
                  });
                }}
              >
                <option value="equals">equals</option>
                <option value="contains">contains</option>
              </select>
            </label>
            <label className={styles.field}>
              <span className={styles.label}>範囲</span>
              <select
                className={styles.input}
                value={condition.scope}
                onChange={(event) => {
                  const scope = event.target.value as "focusedOnly" | "focusedAndAncestors";
                  setForm((current) => {
                    const conditions = [...current.conditions];
                    const existing = conditions[index];
                    if (!existing) {
                      return current;
                    }
                    conditions[index] = { ...existing, scope };
                    return { ...current, conditions };
                  });
                }}
              >
                <option value="focusedOnly">フォーカス要素のみ</option>
                <option value="focusedAndAncestors">フォーカスと親要素</option>
              </select>
            </label>
            <label className={styles.field}>
              <span className={styles.label}>比較値</span>
              <input
                className={styles.input}
                value={condition.value}
                onChange={(event) => {
                  const value = event.target.value;
                  setForm((current) => {
                    const conditions = [...current.conditions];
                    const existing = conditions[index];
                    if (!existing) {
                      return current;
                    }
                    conditions[index] = { ...existing, value };
                    return { ...current, conditions };
                  });
                }}
              />
            </label>
            <Button
              variant="secondary"
              onClick={() => {
                setForm((current) => ({
                  ...current,
                  conditions: current.conditions.filter((_, itemIndex) => itemIndex !== index),
                }));
              }}
            >
              条件を削除
            </Button>
          </div>
        ))}

        <Button
          variant="secondary"
          onClick={() => {
            setForm((current) => ({
              ...current,
              conditions: [
                ...current.conditions,
                {
                  key: crypto.randomUUID(),
                  attribute: "role",
                  operator: "equals",
                  scope: "focusedAndAncestors",
                  value: "",
                },
              ],
            }));
          }}
        >
          条件を追加
        </Button>

        {error ? <p className={styles.error}>{error}</p> : null}
      </div>
    </FormModal>
  );
}
