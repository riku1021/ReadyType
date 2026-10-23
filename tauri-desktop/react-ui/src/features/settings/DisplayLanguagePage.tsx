import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { SearchBar } from "@/components/SearchBar";
import { useInputSourceState } from "@/features/inputSource";
import { isUiLocalePreference, type UiLocalePreference } from "@/locales/i18n";

import * as styles from "./DisplayLanguagePage.styles";
import { filterLanguageListRows, type LanguageListRow, languageListRows } from "./languageLabels";

export function DisplayLanguagePage() {
  const { t, i18n } = useTranslation();
  const { state, setUiLocale } = useInputSourceState();
  const [search, setSearch] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const preferenceRaw = state?.uiLocale ?? "";
  const currentPreference: UiLocalePreference = isUiLocalePreference(preferenceRaw)
    ? preferenceRaw
    : "system";

  const uiLocale = i18n.resolvedLanguage ?? i18n.language;
  const rows = useMemo(() => languageListRows(t, uiLocale), [t, uiLocale]);
  const visibleRows = useMemo(() => filterLanguageListRows(rows, search), [rows, search]);

  const selectLanguage = (preference: UiLocalePreference) => {
    if (pending || preference === currentPreference) {
      return;
    }
    setPending(true);
    setError(null);
    void setUiLocale(preference)
      .catch((updateError) => {
        setError(updateError instanceof Error ? updateError.message : t("common.saveFailed"));
      })
      .finally(() => {
        setPending(false);
      });
  };

  const columns: DataTableColumn<LanguageListRow>[] = [
    {
      id: "language",
      width: "100%",
      cellClassName: (_row, selected) => styles.td({ selected }),
      cell: (row) => <span className={styles.name}>{row.label}</span>,
    },
  ];

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <SearchBar
          fullWidth
          value={search}
          onChange={setSearch}
          placeholder={t("settings.languageSearchPlaceholder")}
          aria-label={t("settings.languageSearchAria")}
        />
      </div>
      <DataTable
        columns={columns}
        rows={visibleRows}
        getRowId={(row) => row.preference}
        selectedRowId={currentPreference}
        onRowSelect={(rowId) => {
          if (!isUiLocalePreference(rowId)) {
            return;
          }
          selectLanguage(rowId);
        }}
        rowClassName={() => styles.row}
        emptyMessage={t("settings.languageEmptySearch")}
        hideHeader
      />
      {error ? <p>{error}</p> : null}
      {pending ? <p>{t("common.saving")}</p> : null}
    </div>
  );
}
