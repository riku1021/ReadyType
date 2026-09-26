import { useEffect, useMemo, useState } from "react";

import { Button } from "@/components/Button";
import { AlertDialog } from "@/components/Dialog";
import { Toggle } from "@/components/Toggle";
import { type RuleDto, useInputSourceState } from "@/features/inputSource";
import { cx } from "@/styled-system/css";

import {
  type AppRuleGroup,
  groupRulesByApp,
  LANGUAGE_TARGET_OPTIONS,
  languageTargetFromLabel,
  summarizeLanguageTarget,
} from "../../appRulesViewModel";
import * as styles from "./AppSettingsPage.styles";

function languageSelectValue(target: RuleDto["target"]): string {
  return summarizeLanguageTarget(target);
}

export function AppSettingsPage() {
  const { state, updateRule, resetDefaultRules } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules), [rules]);
  const [selectedAppId, setSelectedAppId] = useState<string | null>(null);
  const [resetOpen, setResetOpen] = useState(false);

  useEffect(() => {
    if (groups.length === 0) {
      setSelectedAppId(null);
      return;
    }
    if (!selectedAppId || !groups.some((group) => group.appId === selectedAppId)) {
      setSelectedAppId(groups[0]?.appId ?? null);
    }
  }, [groups, selectedAppId]);

  const selected = groups.find((group) => group.appId === selectedAppId) ?? null;

  const setAppEnabled = async (group: AppRuleGroup, enabled: boolean) => {
    const targets = rules.filter((rule) => rule.appId === group.appId);
    await Promise.all(targets.map((rule) => updateRule({ ...rule, enabled })));
  };

  const setRuleTarget = async (rule: RuleDto, label: string) => {
    const target = languageTargetFromLabel(label);
    if (!target) {
      return;
    }
    await updateRule({ ...rule, target });
  };

  return (
    <div className={styles.page}>
      <p className={styles.intro}>アプリや入力欄に応じて、入力言語を自動で切り替えます。</p>

      <div className={styles.toolbar}>
        <Button variant="secondary" onClick={() => setResetOpen(true)}>
          既定ルールへ戻す
        </Button>
      </div>

      {groups.length === 0 ? (
        <p className={styles.empty}>
          表示できるアプリ設定がありません。既定ルールへ戻してください。
        </p>
      ) : (
        <div className={styles.layout}>
          <section className={styles.appList} aria-label="アプリ一覧">
            <div className={styles.appListHeader}>
              <h2 className={styles.sectionTitle}>アプリごとの設定</h2>
            </div>
            {groups.map((group) => (
              <div
                key={group.appId}
                className={cx(
                  styles.appRow,
                  selectedAppId === group.appId && styles.appRowSelected
                )}
              >
                <button
                  type="button"
                  className={styles.appRowMain}
                  onClick={() => setSelectedAppId(group.appId)}
                >
                  <p className={styles.appName}>{group.label}</p>
                  <p className={styles.appMeta}>
                    {group.ruleCount}個のルール · {group.appId}
                  </p>
                </button>
                <Toggle
                  checked={group.enabled}
                  aria-label={`${group.label} を有効化`}
                  onChange={(enabled) => {
                    void setAppEnabled(group, enabled);
                  }}
                />
              </div>
            ))}
          </section>

          <section className={styles.detail} aria-label="アプリ詳細">
            {selected ? (
              <>
                <div className={styles.detailHeader}>
                  <div className={styles.detailTitleBlock}>
                    <h2 className={styles.sectionTitle}>{selected.label}</h2>
                    <p className={styles.appMeta}>{selected.appId}</p>
                  </div>
                  <Toggle
                    checked={selected.enabled}
                    aria-label={`${selected.label} を有効化`}
                    onChange={(enabled) => {
                      void setAppEnabled(selected, enabled);
                    }}
                  />
                </div>

                {selected.defaultRule ? (
                  <div className={styles.fieldBlock}>
                    <p className={styles.fieldLabel}>デフォルトの入力言語</p>
                    <p className={styles.fieldHelp}>
                      このアプリのすべての入力欄で使用する既定の言語です。
                    </p>
                    <select
                      className={styles.select}
                      aria-label="デフォルトの入力言語"
                      value={languageSelectValue(selected.defaultRule.target)}
                      onChange={(event) => {
                        if (selected.defaultRule) {
                          void setRuleTarget(selected.defaultRule, event.target.value);
                        }
                      }}
                    >
                      {LANGUAGE_TARGET_OPTIONS.map((option) => (
                        <option key={option} value={option}>
                          {option}
                        </option>
                      ))}
                      {(LANGUAGE_TARGET_OPTIONS as readonly string[]).includes(
                        summarizeLanguageTarget(selected.defaultRule.target)
                      ) ? null : (
                        <option value={summarizeLanguageTarget(selected.defaultRule.target)}>
                          {summarizeLanguageTarget(selected.defaultRule.target)}
                        </option>
                      )}
                    </select>
                  </div>
                ) : null}

                <div className={styles.fieldBlock}>
                  <p className={styles.fieldLabel}>個別のルール</p>
                  <p className={styles.fieldHelp}>
                    特定の入力欄に対して、異なる言語を設定できます。
                  </p>
                  {selected.contextualRules.length === 0 ? (
                    <p className={styles.empty}>個別ルールはありません。</p>
                  ) : (
                    <div className={styles.ruleList}>
                      {selected.contextualRules.map((item) => (
                        <div key={item.rule.id} className={styles.ruleRow}>
                          <div className={styles.ruleText}>
                            <p className={styles.appName}>{item.title}</p>
                            <p className={styles.appMeta}>{item.subtitle}</p>
                          </div>
                          <select
                            className={styles.select}
                            aria-label={`${item.title} の入力言語`}
                            value={languageSelectValue(item.rule.target)}
                            onChange={(event) => {
                              void setRuleTarget(item.rule, event.target.value);
                            }}
                          >
                            {LANGUAGE_TARGET_OPTIONS.map((option) => (
                              <option key={option} value={option}>
                                {option}
                              </option>
                            ))}
                            {(LANGUAGE_TARGET_OPTIONS as readonly string[]).includes(
                              summarizeLanguageTarget(item.rule.target)
                            ) ? null : (
                              <option value={summarizeLanguageTarget(item.rule.target)}>
                                {summarizeLanguageTarget(item.rule.target)}
                              </option>
                            )}
                          </select>
                          <Toggle
                            checked={item.rule.enabled}
                            aria-label={`${item.title} を有効化`}
                            onChange={(enabled) => {
                              void updateRule({ ...item.rule, enabled });
                            }}
                          />
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                <p className={styles.hint}>
                  ヒント:
                  アプリのデフォルト言語を設定し、特定の入力欄のみ個別ルールを追加すると、効率的に管理できます。名前の
                  Chat / Editor / Terminal
                  は分かりやすさのための表示で、実際の判定は各ルールの条件（AX）で行います。
                </p>
              </>
            ) : (
              <p className={styles.empty}>アプリを選択してください。</p>
            )}
          </section>
        </div>
      )}

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
