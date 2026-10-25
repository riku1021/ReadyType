import { useNavigate } from "@tanstack/react-router";
import { atom, useAtom, useAtomValue } from "jotai";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoChevronForward, IoSwapVertical } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { SearchBar } from "@/components/SearchBar";
import {
  appHasMissingInputSource,
  type InputSourceOption,
  inputSourceSourcesAtom,
  useInputSourceState,
} from "@/features/inputSource";
import { pageSlide } from "@/features/navigation";

import {
  type AppListSort,
  type AppRuleGroup,
  appDisplayName,
  filterAppListRows,
  groupRulesByApp,
  sortAppListRows,
} from "../../appRulesViewModel";
import { AppIconView, LanguageSelect, useAppPresentations } from "../appSettingsShared";
import * as styles from "./AppSettingsPage.styles";

type AppListRow = AppRuleGroup & {
  iconSrc?: string | null;
};

/** 詳細へ遷移して戻っても、このセッションの並びを保つ。 */
const appListSortAtom = atom<AppListSort>("name");

export function AppSettingsPage() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { state } = useInputSourceState();
  const rules = state?.rules ?? [];
  const languagePreferences = state?.languagePreferences ?? {};
  const sources = useAtomValue(inputSourceSourcesAtom);
  const groups = useMemo(() => groupRulesByApp(rules, t), [rules, t]);
  const appIds = useMemo(() => groups.map((group) => group.appId), [groups]);
  const presentations = useAppPresentations(appIds);
  const [search, setSearch] = useState("");
  const [sort, setSort] = useAtom(appListSortAtom);
  const language = i18n.resolvedLanguage ?? i18n.language;

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

  const sortedRows = useMemo(() => sortAppListRows(rows, sort, language), [language, rows, sort]);
  const visibleRows = useMemo(() => filterAppListRows(sortedRows, search), [search, sortedRows]);

  const sortLabelKey = {
    name: "apps.sortName",
    added: "apps.sortAdded",
    ruleCount: "apps.sortRuleCount",
  } as const satisfies Record<AppListSort, string>;
  const sortOptions: InputSourceOption[] = (["name", "added", "ruleCount"] as const).map(
    (value) => ({
      value,
      label: t(sortLabelKey[value]),
      available: true,
    })
  );
  const currentSortLabel = t(sortLabelKey[sort]);

  const columns: DataTableColumn<AppListRow>[] = [
    {
      id: "app",
      header: t("apps.columnApp"),
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
        <SearchBar
          fullWidth
          value={search}
          onChange={setSearch}
          placeholder={t("apps.searchPlaceholder")}
          aria-label={t("apps.searchAria")}
        />
        <span className={styles.sortControl}>
          <span className={styles.sortVisual} aria-hidden="true">
            <Button
              variant="neutral"
              iconLeft={<IoSwapVertical />}
              tabIndex={-1}
              onClick={() => undefined}
            >
              {t("apps.sort")}
            </Button>
          </span>
          <LanguageSelect
            className={styles.sortSelectHit}
            value={sort}
            label={t("apps.sortAria", { order: currentSortLabel })}
            options={sortOptions}
            onChange={(value) => {
              if (value === "name" || value === "added" || value === "ruleCount") {
                setSort(value);
              }
            }}
          />
        </span>
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
