import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoAdd, IoEllipsisHorizontal, IoInformationCircle } from "react-icons/io5";

import { Button } from "@/components/Button";
import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { Toggle } from "@/components/Toggle";
import {
  buildInputSourceOptions,
  type InputSourceDto,
  type RuleDto,
  useInputSourceState,
} from "@/features/inputSource";

import {
  type AppRuleGroup,
  groupRulesByApp,
  type RuleDisplay,
  resolvedInputSourceId,
  ruleTargetFromInputSourceId,
} from "../../appRulesViewModel";
import { AppIconView, LanguageSelect, RuleKindIcon, useAppIcons } from "../appSettingsShared";
import * as styles from "./AppDetailPage.styles";

type AppDetailPageProps = {
  appId: string;
};

export function AppDetailPage({ appId }: AppDetailPageProps) {
  const { t } = useTranslation();
  const { state, updateRule, fetchInputSources } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules, t), [rules, t]);
  const selected = groups.find((group) => group.appId === appId) ?? null;
  const icons = useAppIcons(selected ? [selected.appId] : []);
  const [sources, setSources] = useState<InputSourceDto[]>([]);

  useEffect(() => {
    let cancelled = false;
    void fetchInputSources(true)
      .then((next) => {
        if (!cancelled) {
          setSources(next);
        }
      })
      .catch(() => {
        // ネイティブ未接続時は空のまま（セレクトは無効）
      });
    return () => {
      cancelled = true;
    };
  }, [fetchInputSources]);

  const languagePreferences = state?.languagePreferences ?? {};
  const preferredIds = useMemo(
    () =>
      Object.values(languagePreferences).filter(
        (id): id is string => typeof id === "string" && id.length > 0
      ),
    [languagePreferences]
  );

  const languageOptions = useMemo(() => {
    const configuredIds = [
      ...preferredIds,
      ...(selected?.defaultRule
        ? [resolvedInputSourceId(selected.defaultRule.target, languagePreferences)]
        : []),
      ...(selected?.contextualRules.map((item) =>
        resolvedInputSourceId(item.rule.target, languagePreferences)
      ) ?? []),
    ];
    return buildInputSourceOptions(sources, configuredIds);
  }, [languagePreferences, preferredIds, selected, sources]);

  const setAppEnabled = async (group: AppRuleGroup, enabled: boolean) => {
    const targets = rules.filter((rule) => rule.appId === group.appId);
    await Promise.all(targets.map((rule) => updateRule({ ...rule, enabled })));
  };

  const setRuleTarget = async (rule: RuleDto, inputSourceId: string) => {
    const target = ruleTargetFromInputSourceId(inputSourceId, languagePreferences);
    await updateRule({ ...rule, target });
  };

  const ruleColumns: DataTableColumn<RuleDisplay>[] = [
    {
      id: "rule",
      header: t("apps.columnRule"),
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
      header: t("apps.columnInputLanguage"),
      width: "10rem",
      headerClassName: styles.th,
      cellClassName: styles.td,
      stopPropagation: true,
      cell: (item) => (
        <LanguageSelect
          value={resolvedInputSourceId(item.rule.target, languagePreferences)}
          label={t("apps.ruleLanguageAria", { title: item.title })}
          options={languageOptions}
          onChange={(value) => {
            void setRuleTarget(item.rule, value);
          }}
        />
      ),
    },
    {
      id: "actions",
      header: t("apps.columnActions"),
      width: "4rem",
      headerClassName: styles.thActions,
      cellClassName: styles.tdActions,
      stopPropagation: true,
      cell: (item) => (
        <button
          type="button"
          className={styles.iconButton}
          aria-label={t("apps.ruleMenuAria", { title: item.title })}
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
        <p className={styles.empty}>{t("apps.notFound")}</p>
      </div>
    );
  }

  return (
    <section className={styles.page} aria-label={t("apps.detailAria")}>
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
          aria-label={t("common.enableAppAria", { label: selected.label })}
          onChange={(enabled) => {
            void setAppEnabled(selected, enabled);
          }}
        />
      </div>

      {selected.defaultRule ? (
        <div className={styles.fieldBlock}>
          <p className={styles.fieldLabel}>{t("apps.defaultInputLanguage")}</p>
          <p className={styles.fieldHelp}>{t("apps.defaultInputLanguageHelp")}</p>
          <LanguageSelect
            value={resolvedInputSourceId(selected.defaultRule.target, languagePreferences)}
            label={t("apps.defaultInputLanguage")}
            options={languageOptions}
            wide
            onChange={(value) => {
              if (selected.defaultRule) {
                void setRuleTarget(selected.defaultRule, value);
              }
            }}
          />
          <div className={styles.infoBox}>
            <IoInformationCircle className={styles.infoIcon} aria-hidden="true" />
            <span>{t("apps.contextualOverrideNote")}</span>
          </div>
        </div>
      ) : null}

      <div className={styles.fieldBlock}>
        <div className={styles.sectionHeader}>
          <p className={styles.fieldLabel}>{t("apps.contextualRules")}</p>
          <Button variant="secondary" iconLeft={<IoAdd />} onClick={() => undefined} disabled>
            {t("apps.addRule")}
          </Button>
        </div>
        <p className={styles.fieldHelp}>{t("apps.contextualRulesHelp")}</p>
      </div>

      <DataTable
        columns={ruleColumns}
        rows={selected.contextualRules}
        getRowId={(item) => item.rule.id}
        rowClassName={() => styles.row}
        emptyMessage={t("apps.emptyContextualRules")}
      />

      <div className={styles.infoBox}>
        <IoInformationCircle className={styles.infoIcon} aria-hidden="true" />
        <span>{t("apps.kindDisplayNote")}</span>
      </div>
    </section>
  );
}
