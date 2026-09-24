import { useRouterState } from "@tanstack/react-router";
import { useMemo, useState } from "react";

import { Button } from "@/components/Button";
import { AlertDialog } from "@/components/Dialog";
import { Toggle } from "@/components/Toggle";
import { type RuleConditionDto, type RuleDto, useInputSourceState } from "@/features/inputSource";

import { duplicateRule, moveRule, summarizeConditions, summarizeTarget } from "../../ruleForm";
import { RuleDialog } from "../RuleDialog/RuleDialog";
import * as styles from "./RulesPage.styles";

type RulesLocationState = {
  conditionPrefill?: RuleConditionDto;
};

export function RulesPage() {
  const { state, createRule, updateRule, deleteRule, reorderRules, resetDefaultRules } =
    useInputSourceState();
  const locationState = useRouterState({
    select: (routerState) => routerState.location.state as RulesLocationState | undefined,
  });
  const rules = state?.rules ?? [];
  const [editor, setEditor] = useState<{
    mode: "create" | "edit";
    rule?: RuleDto;
    condition?: RuleConditionDto;
  } | null>(
    locationState?.conditionPrefill
      ? { mode: "create", condition: locationState.conditionPrefill }
      : null
  );
  const [deleteTarget, setDeleteTarget] = useState<RuleDto | null>(null);
  const [resetOpen, setResetOpen] = useState(false);

  const nextPriority = useMemo(
    () => (rules.length === 0 ? 0 : Math.max(...rules.map((rule) => rule.priority)) + 1),
    [rules]
  );

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <Button
          onClick={() => {
            setEditor({ mode: "create" });
          }}
        >
          ルールを追加
        </Button>
        <Button variant="secondary" onClick={() => setResetOpen(true)}>
          既定ルールへ戻す
        </Button>
      </div>

      <div className={styles.list}>
        {rules.map((rule, index) => (
          <article key={rule.id} className={styles.card}>
            <div className={styles.cardHeader}>
              <h2 className={styles.title}>{rule.name}</h2>
              <Toggle
                checked={rule.enabled}
                aria-label={`${rule.name} を有効化`}
                onChange={(enabled) => {
                  void updateRule({ ...rule, enabled });
                }}
              />
            </div>
            <p className={styles.meta}>{rule.appId}</p>
            <p className={styles.meta}>{summarizeConditions(rule.conditions)}</p>
            <p className={styles.meta}>切替先: {summarizeTarget(rule.target)}</p>
            <div className={styles.actions}>
              <Button
                variant="secondary"
                disabled={index === 0}
                onClick={() => {
                  const reordered = moveRule(rules, rule.id, -1);
                  void reorderRules(reordered.map((item) => item.id));
                }}
              >
                上へ
              </Button>
              <Button
                variant="secondary"
                disabled={index === rules.length - 1}
                onClick={() => {
                  const reordered = moveRule(rules, rule.id, 1);
                  void reorderRules(reordered.map((item) => item.id));
                }}
              >
                下へ
              </Button>
              <Button
                variant="secondary"
                onClick={() => {
                  setEditor({ mode: "edit", rule });
                }}
              >
                編集
              </Button>
              <Button
                variant="secondary"
                onClick={() => {
                  void createRule(duplicateRule(rule, crypto.randomUUID()));
                }}
              >
                複製
              </Button>
              <Button variant="danger" onClick={() => setDeleteTarget(rule)}>
                削除
              </Button>
            </div>
          </article>
        ))}
      </div>

      {editor ? (
        <RuleDialog
          key={`${editor.mode}-${editor.rule?.id ?? "new"}-${editor.condition?.value ?? ""}`}
          isOpen
          mode={editor.mode}
          initialRule={editor.rule}
          initialCondition={editor.condition}
          nextPriority={nextPriority}
          onClose={() => setEditor(null)}
          onSubmit={async (rule) => {
            if (editor.mode === "create") {
              await createRule(rule);
            } else {
              await updateRule(rule);
            }
          }}
        />
      ) : null}

      <AlertDialog
        isOpen={deleteTarget != null}
        onClose={() => setDeleteTarget(null)}
        title="ルールを削除しますか？"
        text={deleteTarget ? `${deleteTarget.name} を削除します。` : undefined}
        confirmButtonText="削除"
        onConfirm={() => {
          if (deleteTarget) {
            void deleteRule(deleteTarget.id);
          }
        }}
      />

      <AlertDialog
        isOpen={resetOpen}
        onClose={() => setResetOpen(false)}
        title="既定ルールへ戻しますか？"
        text="カスタムルールを含む現在のルール一覧を既定プリセットで置き換えます。"
        confirmButtonText="戻す"
        onConfirm={() => {
          void resetDefaultRules();
        }}
      />
    </div>
  );
}
