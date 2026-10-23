import { useNavigate } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoAdd, IoChevronForward } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { SearchBar } from "@/components/SearchBar";
import { Toggle } from "@/components/Toggle";
import {
  appHasMissingInputSource,
  inputSourceSourcesAtom,
  useInputSourceState,
} from "@/features/inputSource";
import { pageSlide } from "@/features/navigation";

import {
  type AppRuleGroup,
  appDisplayName,
  filterAppListRows,
  groupRulesByApp,
} from "../../appRulesViewModel";
import { AppIconView, useAppPresentations } from "../appSettingsShared";
import * as styles from "./AppSettingsPage.styles";

type AppListRow = AppRuleGroup & {
  iconSrc?: string | null;
};

export function AppSettingsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { state, updateRule } = useInputSourceState();
  const rules = state?.rules ?? [];
  const languagePreferences = state?.languagePreferences ?? {};
  const sources = useAtomValue(inputSourceSourcesAtom);
  const groups = useMemo(() => groupRulesByApp(rules, t), [rules, t]);
  const appIds = useMemo(() => groups.map((group) => group.appId), [groups]);
  const presentations = useAppPresentations(appIds);
  const [search, setSearch] = useState("");

  const rows: AppListRow[] = useMemo(
    () =>
      groups.map((group) => {
        const presentation = presentations[group.appId];
        return {
          ...group,
          label: appDisplayName(group.appId, presentation?.displayName),
          iconSrc: presentation?.iconSrc,
        };
      }),
    [groups, presentations]
  );

  const visibleRows = useMemo(() => filterAppListRows(rows, search), [rows, search]);

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
          <AppIconView
            src={group.iconSrc}
            label={group.label}
            showWarningBadge={
              sources.status === "ready" &&
              appHasMissingInputSource(group.appId, rules, languagePreferences, sources.sources)
            }
          />
          <span className={styles.appName}>{group.label}</span>
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
    {
      id: "navigate",
      header: "",
      width: "2.5rem",
      headerClassName: styles.thChevron,
      cellClassName: (_row, selected) => styles.tdChevron({ selected }),
      cell: () => (
        <span className={styles.chevron} aria-hidden>
          <IoChevronForward />
        </span>
      ),
    },
  ];

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <p className={styles.intro}>{t("apps.intro")}</p>
        <div className={styles.toolbarActions}>
          <SearchBar
            value={search}
            onChange={setSearch}
            placeholder={t("apps.searchPlaceholder")}
            aria-label={t("apps.searchAria")}
          />
          <Button variant="secondary" iconLeft={<IoAdd />} onClick={() => undefined} disabled>
            {t("apps.addApp")}
          </Button>
        </div>
      </div>

      <DataTable
        columns={columns}
        rows={visibleRows}
        getRowId={(group) => group.appId}
        onRowSelect={(appId) => {
          void navigate({
            to: "/apps/$appId",
            params: { appId },
            viewTransition: pageSlide("forward"),
          });
        }}
        rowClassName={() => styles.row}
        emptyMessage={search.trim() !== "" ? t("apps.emptySearch") : t("apps.emptyList")}
        hideHeader
      />
    </div>
  );
}
