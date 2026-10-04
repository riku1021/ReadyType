import { useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoAdd, IoSearch } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { AlertDialog } from "@/components/Dialog";
import { Toggle } from "@/components/Toggle";
import { useInputSourceState } from "@/features/inputSource";

import { type AppRuleGroup, appDisplayName, groupRulesByApp } from "../../appRulesViewModel";
import { AppIconView, useAppPresentations } from "../appSettingsShared";
import * as styles from "./AppSettingsPage.styles";

type AppListRow = AppRuleGroup & {
  iconSrc?: string | null;
};

export function AppSettingsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { state, updateRule, resetDefaultRules } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules, t), [rules, t]);
  const appIds = useMemo(() => groups.map((group) => group.appId), [groups]);
  const presentations = useAppPresentations(appIds);
  const [resetOpen, setResetOpen] = useState(false);

  const rows: AppListRow[] = useMemo(
    () =>
      groups.map((group) => {
        const presentation = presentations[group.appId];
        return {
          ...group,
          label: appDisplayName(group.appId, presentation?.displayName),
          iconSrc: presentation?.icon,
        };
      }),
    [groups, presentations]
  );

  const setAppEnabled = async (group: AppRuleGroup, enabled: boolean) => {
    const targets = rules.filter((rule) => rule.appId === group.appId);
    await Promise.all(targets.map((rule) => updateRule({ ...rule, enabled })));
  };

  const columns: DataTableColumn<AppListRow>[] = [
    {
      id: "app",
      header: t("apps.columnApp"),
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
      header: t("apps.columnRules"),
      width: "6.5rem",
      headerClassName: styles.th,
      cellClassName: (_row, selected) => styles.td({ selected }),
      cell: (group) => (
        <span className={styles.ruleCount}>
          {t("common.ruleCount", { count: group.ruleCount })}
        </span>
      ),
    },
    {
      id: "enabled",
      header: t("apps.columnEnabled"),
      width: "5.5rem",
      headerClassName: styles.thEnabled,
      cellClassName: (_row, selected) => styles.tdEnabled({ selected }),
      stopPropagation: true,
      cell: (group) => (
        <Toggle
          checked={group.enabled}
          aria-label={t("common.enableAppAria", { label: group.label })}
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
        <p className={styles.intro}>{t("apps.intro")}</p>
        <div className={styles.toolbarActions}>
          <span className={styles.searchBox}>
            <IoSearch className={styles.searchIcon} aria-hidden="true" />
            <input
              className={styles.searchInput}
              type="search"
              placeholder={t("apps.searchPlaceholder")}
              aria-label={t("apps.searchAria")}
              disabled
            />
          </span>
          <Button variant="secondary" iconLeft={<IoAdd />} onClick={() => undefined} disabled>
            {t("apps.addApp")}
          </Button>
          <Button variant="secondary" onClick={() => setResetOpen(true)}>
            {t("apps.resetToDefaultRules")}
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
        emptyMessage={t("apps.emptyList")}
      />

      <AlertDialog
        isOpen={resetOpen}
        onClose={() => setResetOpen(false)}
        title={t("apps.resetConfirmTitle")}
        text={t("apps.resetConfirmText")}
        confirmButtonText={t("apps.resetConfirmButton")}
        onConfirm={() => {
          void resetDefaultRules();
        }}
      />
    </div>
  );
}
