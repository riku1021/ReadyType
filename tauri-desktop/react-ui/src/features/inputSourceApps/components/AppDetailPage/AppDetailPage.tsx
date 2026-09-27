import { useMemo } from "react";
import { IoAdd, IoEllipsisHorizontal, IoInformationCircle } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { Toggle } from "@/components/Toggle";
import { type RuleDto, useInputSourceState } from "@/features/inputSource";

import {
  type AppRuleGroup,
  groupRulesByApp,
  languageTargetFromLabel,
  type RuleDisplay,
  summarizeLanguageTarget,
} from "../../appRulesViewModel";
import { AppIconView, LanguageSelect, RuleKindIcon, useAppIcons } from "../appSettingsShared";
import * as styles from "./AppDetailPage.styles";

function languageSelectValue(target: RuleDto["target"]): string {
  return summarizeLanguageTarget(target);
}

type AppDetailPageProps = {
  appId: string;
};

export function AppDetailPage({ appId }: AppDetailPageProps) {
  const { state, updateRule } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules), [rules]);
  const selected = groups.find((group) => group.appId === appId) ?? null;
  const icons = useAppIcons(selected ? [selected.appId] : []);

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

  const ruleColumns: DataTableColumn<RuleDisplay>[] = [
    {
      id: "rule",
      header: "ルール",
      width: "50%",
      headerClassName: styles.thRule,
      cellClassName: styles.tdRule,
      cell: (item) => (
        <div className={styles.ruleCell}>
          <RuleKindIcon kind={item.kind} />
          <div className={styles.ruleText}>
            <p className={styles.ruleTitle}>{item.title}</p>
            <p className={styles.appMeta}>{item.subtitle}</p>
          </div>
        </div>
      ),
    },
    {
      id: "language",
      header: "入力言語",
      width: "10rem",
      headerClassName: styles.th,
      cellClassName: styles.td,
      stopPropagation: true,
      cell: (item) => (
        <LanguageSelect
          value={languageSelectValue(item.rule.target)}
          label={`${item.title} の入力言語`}
          onChange={(value) => {
            void setRuleTarget(item.rule, value);
          }}
        />
      ),
    },
    {
      id: "actions",
      header: "操作",
      width: "4rem",
      headerClassName: styles.thActions,
      cellClassName: styles.tdActions,
      stopPropagation: true,
      cell: (item) => (
        <button
          type="button"
          className={styles.iconButton}
          aria-label={`${item.title} の詳細メニュー`}
          disabled
        >
          <IoEllipsisHorizontal aria-hidden="true" />
        </button>
      ),
    },
  ];

  if (!selected) {
    return (
      <div className={styles.page}>
        <p className={styles.empty}>指定されたアプリの設定が見つかりません。</p>
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <section className={styles.detail} aria-label="アプリ詳細">
        <div className={styles.detailHeader}>
          <div className={styles.detailIdentity}>
            <AppIconView src={icons[selected.appId]} label={selected.label} large />
            <div className={styles.detailTitleBlock}>
              <h2 className={styles.detailName}>{selected.label}</h2>
              <p className={styles.detailPath}>{selected.appId}</p>
            </div>
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
              このアプリで、特にルールが設定されていない入力欄に使用する言語です。
            </p>
            <LanguageSelect
              value={languageSelectValue(selected.defaultRule.target)}
              label="デフォルトの入力言語"
              wide
              onChange={(value) => {
                if (selected.defaultRule) {
                  void setRuleTarget(selected.defaultRule, value);
                }
              }}
            />
            <div className={styles.infoBox}>
              <IoInformationCircle className={styles.infoIcon} aria-hidden="true" />
              <span>個別のルールで指定がある入力欄は、そちらの設定が優先されます。</span>
            </div>
          </div>
        ) : null}

        <div className={styles.fieldBlock}>
          <div className={styles.sectionHeader}>
            <p className={styles.fieldLabel}>個別のルール</p>
            <Button variant="secondary" iconLeft={<IoAdd />} onClick={() => undefined} disabled>
              ルールを追加
            </Button>
          </div>
          <p className={styles.fieldHelp}>特定の入力欄に対して、異なる言語を設定できます。</p>
          <DataTable
            className={styles.rulesTable}
            columns={ruleColumns}
            rows={selected.contextualRules}
            getRowId={(item) => item.rule.id}
            rowClassName={() => styles.row}
            emptyMessage="個別ルールはありません。"
          />
        </div>

        <div className={styles.infoBox}>
          <IoInformationCircle className={styles.infoIcon} aria-hidden="true" />
          <span>
            Chat / Editor / Terminal
            は分かりやすさのための表示で、実際の判定は各ルールの条件（AX）で行います。
          </span>
        </div>
      </section>
    </div>
  );
}
