import { useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { IoAdd, IoSearch } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { AlertDialog } from "@/components/Dialog";
import { Toggle } from "@/components/Toggle";
import { useInputSourceState } from "@/features/inputSource";

import { type AppRuleGroup, groupRulesByApp } from "../../appRulesViewModel";
import { AppIconView, useAppIcons } from "../appSettingsShared";
import * as styles from "./AppSettingsPage.styles";

type AppListRow = AppRuleGroup & {
  iconSrc?: string | null;
};

export function AppSettingsPage() {
  const navigate = useNavigate();
  const { state, updateRule, resetDefaultRules } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules), [rules]);
  const appIds = useMemo(() => groups.map((group) => group.appId), [groups]);
  const icons = useAppIcons(appIds);
  const [resetOpen, setResetOpen] = useState(false);

  const rows: AppListRow[] = useMemo(
    () => groups.map((group) => ({ ...group, iconSrc: icons[group.appId] })),
    [groups, icons]
  );

  const setAppEnabled = async (group: AppRuleGroup, enabled: boolean) => {
    const targets = rules.filter((rule) => rule.appId === group.appId);
    await Promise.all(targets.map((rule) => updateRule({ ...rule, enabled })));
  };

  const columns: DataTableColumn<AppListRow>[] = [
    {
      id: "app",
      header: "アプリ",
      width: "55%",
      headerClassName: styles.thApp,
      cellClassName: (_row, selected) => styles.tdApp({ selected }),
      cell: (group) => (
        <div className={styles.appCell}>
          <AppIconView src={group.iconSrc} label={group.label} />
          <span className={styles.appCellText}>
            <span className={styles.appName}>{group.label}</span>
            <span className={styles.appMeta}>{group.appId}</span>
          </span>
        </div>
      ),
    },
    {
      id: "rules",
      header: "ルール",
      width: "6.5rem",
      headerClassName: styles.th,
      cellClassName: (_row, selected) => styles.td({ selected }),
      cell: (group) => <span className={styles.ruleCount}>{group.ruleCount}個</span>,
    },
    {
      id: "enabled",
      header: "有効",
      width: "5.5rem",
      headerClassName: styles.thEnabled,
      cellClassName: (_row, selected) => styles.tdEnabled({ selected }),
      stopPropagation: true,
      cell: (group) => (
        <Toggle
          checked={group.enabled}
          aria-label={`${group.label} を有効化`}
          onChange={(enabled) => {
            void setAppEnabled(group, enabled);
          }}
        />
      ),
    },
  ];

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <p className={styles.intro}>アプリや入力欄に応じて、入力言語を自動で切り替えます。</p>
        <div className={styles.toolbarActions}>
          <span className={styles.searchBox}>
            <IoSearch className={styles.searchIcon} aria-hidden="true" />
            <input
              className={styles.searchInput}
              type="search"
              placeholder="アプリを検索..."
              aria-label="アプリを検索"
              disabled
            />
          </span>
          <Button variant="secondary" iconLeft={<IoAdd />} onClick={() => undefined} disabled>
            アプリを追加
          </Button>
          <Button variant="secondary" onClick={() => setResetOpen(true)}>
            既定ルールへ戻す
          </Button>
        </div>
      </div>

      <DataTable
        columns={columns}
        rows={rows}
        getRowId={(group) => group.appId}
        onRowSelect={(appId) => {
          void navigate({ to: "/apps/$appId", params: { appId } });
        }}
        rowClassName={() => styles.row}
        emptyMessage="表示できるアプリ設定がありません。既定ルールへ戻してください。"
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
