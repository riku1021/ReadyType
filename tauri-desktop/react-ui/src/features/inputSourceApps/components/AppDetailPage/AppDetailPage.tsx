import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoEllipsisHorizontal, IoInformationCircle } from "react-icons/io5";

import { DataTable, type DataTableColumn } from "@/components/DataTable";
import { AlertDialog, SuccessDialog } from "@/components/Dialog";
import { SelectionMenu } from "@/components/SelectionMenu";
import { SettingsSection } from "@/components/SettingsSection";
import { Toggle } from "@/components/Toggle";
import {
  buildInputSourceOptions,
  type InputSourceDto,
  type Platform,
  type RuleDto,
  useInputSourceState,
} from "@/features/inputSource";

import {
  type AppRuleGroup,
  appDisplayName,
  groupRulesByApp,
  type RuleDisplay,
  resolvedInputSourceId,
  ruleTargetFromInputSourceId,
} from "../../appRulesViewModel";
import {
  AppIconView,
  LanguageSelect,
  RuleKindIcon,
  useAppPresentations,
} from "../appSettingsShared";
import * as styles from "./AppDetailPage.styles";

type AppDetailPageProps = {
  appId: string;
};

function RuleActionsMenu({ title, onDelete }: { title: string; onDelete: () => void }) {
  const { t } = useTranslation();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const deleteLabel = t("apps.deleteRule");
  const menuLabel = t("apps.ruleMenuAria", { title });

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        className={styles.iconButton}
        aria-label={menuLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <IoEllipsisHorizontal aria-hidden="true" />
      </button>
      <SelectionMenu
        isOpen={open}
        onClose={() => setOpen(false)}
        anchorRef={triggerRef}
        label={menuLabel}
        options={[deleteLabel]}
        currentSelection=""
        onSelectionChanged={(value) => {
          if (value === deleteLabel) {
            onDelete();
          }
        }}
      />
    </>
  );
}

function sanitizeRuleIdPart(value: string): string {
  return value.replace(/[^a-zA-Z0-9._-]+/g, "-");
}

function resolveAppPlatform(rules: readonly RuleDto[], appId: string): Platform {
  return rules.find((rule) => rule.appId === appId)?.platform ?? "macOs";
}

export function AppDetailPage({ appId }: AppDetailPageProps) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { state, createRule, updateRule, deleteRule, fetchInputSources } = useInputSourceState();
  const rules = state?.rules ?? [];
  const groups = useMemo(() => groupRulesByApp(rules, t), [rules, t]);
  const selectedBase = groups.find((group) => group.appId === appId) ?? null;
  const presentations = useAppPresentations(selectedBase ? [selectedBase.appId] : []);
  const selected = useMemo(() => {
    if (!selectedBase) {
      return null;
    }
    return {
      ...selectedBase,
      label: appDisplayName(selectedBase.appId, presentations[selectedBase.appId]?.displayName),
    };
  }, [presentations, selectedBase]);
  const [sources, setSources] = useState<InputSourceDto[]>([]);
  const [pendingDelete, setPendingDelete] = useState<RuleDisplay | null>(null);
  const [deletedTitle, setDeletedTitle] = useState<string | null>(null);
  const uiLanguage = i18n.language;

  // UI 言語変更時にネイティブ側の locale 付き list を取り直す
  useEffect(() => {
    let cancelled = false;
    const requestedLanguage = uiLanguage;
    void fetchInputSources(true)
      .then((next) => {
        if (!cancelled && requestedLanguage === uiLanguage) {
          setSources(next);
        }
      })
      .catch(() => {
        // ネイティブ未接続時は空のまま（セレクトは無効）
      });
    return () => {
      cancelled = true;
    };
  }, [fetchInputSources, uiLanguage]);

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

  const setDefaultInputLanguage = async (inputSourceId: string) => {
    if (!selected) {
      return;
    }
    if (inputSourceId.length === 0) {
      if (!selected.defaultRule) {
        return;
      }
      const onlyRuleForApp =
        selected.contextualRules.length === 0 &&
        rules.filter((rule) => rule.appId === selected.appId).length === 1;
      await deleteRule(selected.defaultRule.id);
      if (onlyRuleForApp) {
        void navigate({ to: "/apps" });
      }
      return;
    }
    if (selected.defaultRule) {
      await setRuleTarget(selected.defaultRule, inputSourceId);
      return;
    }
    const target = ruleTargetFromInputSourceId(inputSourceId, languagePreferences);
    await createRule({
      id: `app-default-${sanitizeRuleIdPart(selected.appId)}-${Date.now()}`,
      name: t("rules.defaultTitle"),
      enabled: selected.enabled,
      platform: resolveAppPlatform(rules, selected.appId),
      appId: selected.appId,
      priority: 0,
      conditions: [],
      target,
    });
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
        <RuleActionsMenu title={item.title} onDelete={() => setPendingDelete(item)} />
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
      <SettingsSection>
        <div className={styles.detailHeader}>
          <div className={styles.detailIdentity}>
            <AppIconView src={presentations[selected.appId]?.icon} label={selected.label} large />
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
        <div className={styles.cardSection}>
          <p className={styles.fieldLabel}>{t("apps.defaultInputLanguage")}</p>
          <LanguageSelect
            value={
              selected.defaultRule
                ? resolvedInputSourceId(selected.defaultRule.target, languagePreferences)
                : ""
            }
            label={t("apps.defaultInputLanguage")}
            options={languageOptions}
            wide
            noneLabel={t("apps.defaultInputLanguageNone")}
            onChange={(value) => {
              void setDefaultInputLanguage(value);
            }}
          />
        </div>
      </SettingsSection>

      <div className={styles.fieldBlock}>
        <p className={styles.fieldLabel}>{t("apps.contextualRules")}</p>
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

      <AlertDialog
        isOpen={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title={t("apps.deleteRuleConfirmTitle")}
        text={t("apps.deleteRuleConfirmText", { title: pendingDelete?.title ?? "" })}
        confirmButtonText={t("apps.deleteRuleConfirmButton")}
        onConfirm={() => {
          if (!pendingDelete) {
            return;
          }
          const title = pendingDelete.title;
          void deleteRule(pendingDelete.rule.id).then(() => {
            setDeletedTitle(title);
          });
        }}
      />
      <SuccessDialog
        isOpen={deletedTitle !== null}
        onClose={() => setDeletedTitle(null)}
        title={deletedTitle ? t("apps.deleteRuleSuccess", { title: deletedTitle }) : ""}
      />
    </section>
  );
}
