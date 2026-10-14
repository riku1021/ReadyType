import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  Children,
  isValidElement,
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { FiTrash2 } from "react-icons/fi";
import { MdClose, MdSettings } from "react-icons/md";

import { BrandLogo } from "@/components/BrandLogo";
import { CompletionCheckbox } from "@/components/CompletionCheckbox";
import { AlertDialog, SuccessDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { deleteRule } from "@/features/inputSource/api";
import { AppIconView, appDisplayName, useAppPresentations } from "@/features/inputSourceApps";
import { cx } from "@/styled-system/css";

import { applyQuickSetSource, getQuickSetSession, hideQuickSet, openQuickSetAdvanced } from "./api";
import {
  appIdentity,
  appName,
  closeButton,
  divider,
  emptyState,
  footerBlock,
  headerBlock,
  nameField,
  nameInput,
  nameLabel,
  optionButton,
  optionButtonCurrent,
  optionsScroll,
  optionText,
  optionTitle,
  prompt,
  shell,
} from "./QuickSetPage.styles";
import { buildQuickSetOptions, type QuickSetUiPhase } from "./quickSetViewModel";
import { QUICK_SET_EVENT, type QuickSetOpenEventDto, type QuickSetSessionDto } from "./types";

function itemKey(child: ReactNode, index: number): string {
  if (isValidElement(child) && child.key != null) {
    return String(child.key);
  }
  return `quick-set-item-${index}`;
}

function withDividers(items: ReactNode[]): ReactNode[] {
  return Children.toArray(items).flatMap((child, index) => {
    if (index === 0) {
      return [child];
    }
    const key = itemKey(child, index);
    return [<hr key={`divider-${key}`} className={divider} />, child];
  });
}

export function QuickSetPage() {
  const { t } = useTranslation();
  const [session, setSession] = useState<QuickSetSessionDto | null>(null);
  const [unavailable, setUnavailable] = useState(false);
  const [ruleName, setRuleName] = useState("");
  const [phase, setPhase] = useState<QuickSetUiPhase>({ kind: "idle" });
  const [error, setError] = useState<string | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const resetInteraction = useCallback(() => {
    setPhase({ kind: "idle" });
    setError(null);
  }, []);

  const adoptSession = useCallback(
    (next: QuickSetSessionDto | null) => {
      setSession(next);
      setUnavailable(false);
      setRuleName(next?.proposedName ?? "");
      resetInteraction();
    },
    [resetInteraction]
  );

  const adoptOpenEvent = useCallback(
    (event: QuickSetOpenEventDto) => {
      if (event.kind === "ready") {
        adoptSession(event.session);
        return;
      }
      setSession(null);
      setUnavailable(true);
      setRuleName("");
      resetInteraction();
    },
    [adoptSession, resetInteraction]
  );

  const loadSession = useCallback(async () => {
    try {
      const next = await getQuickSetSession();
      adoptSession(next);
    } catch {
      setError(t("quickSet.loadFailed"));
    }
  }, [adoptSession, t]);

  useEffect(() => {
    void loadSession();
    // emit_to(quick-set) と対になる webview 専用 listen
    const unlistenPromise = getCurrentWebviewWindow().listen<QuickSetOpenEventDto>(
      QUICK_SET_EVENT,
      (event) => {
        adoptOpenEvent(event.payload);
      }
    );
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [adoptOpenEvent, loadSession]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && phase.kind !== "success") {
        event.preventDefault();
        void hideQuickSet();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [phase.kind]);

  const closeAfterSuccess = useCallback(() => {
    setPhase({ kind: "idle" });
    void hideQuickSet();
  }, []);

  const onClose = useCallback(() => {
    if (phase.kind === "applying") {
      return;
    }
    void hideQuickSet();
  }, [phase.kind]);

  const options = useMemo(() => (session ? buildQuickSetOptions(session) : []), [session]);
  const presentations = useAppPresentations(session ? [session.appId] : []);
  const appLabel = session
    ? appDisplayName(
        session.appId,
        presentations[session.appId]?.displayName || session.appDisplayName
      )
    : "";
  const hasExisting = Boolean(session?.matchedRuleId);
  const isIdle = phase.kind === "idle";
  const successLabel = phase.kind === "success" ? phase.languageLabel : null;

  const onSelect = async (sourceId: string, label: string) => {
    if (phase.kind !== "idle") {
      return;
    }
    setPhase({ kind: "applying" });
    setError(null);
    try {
      await applyQuickSetSource(sourceId, ruleName);
      setPhase({ kind: "success", languageLabel: label });
    } catch {
      setError(t("quickSet.saveFailed"));
      setPhase({ kind: "idle" });
    }
  };

  const onAdvanced = () => {
    if (!session || !isIdle) {
      return;
    }
    void openQuickSetAdvanced(session.appId);
  };

  const onDeleteConfirm = () => {
    const ruleId = session?.matchedRuleId;
    if (!ruleId || phase.kind === "applying") {
      return;
    }
    setPhase({ kind: "applying" });
    setError(null);
    void deleteRule(ruleId)
      .then(() => {
        setDeleteOpen(false);
        void hideQuickSet();
      })
      .catch(() => {
        setError(t("quickSet.deleteFailed"));
        setPhase({ kind: "idle" });
      });
  };

  const optionItems: ReactNode[] =
    options.length === 0
      ? [
          <p key="empty" className={emptyState}>
            {t("quickSet.noLanguages")}
          </p>,
        ]
      : options.map((option) => (
          <button
            key={option.sourceId}
            type="button"
            className={cx(optionButton, option.isCurrent ? optionButtonCurrent : undefined)}
            disabled={!isIdle}
            aria-pressed={option.isCurrent}
            onClick={() => void onSelect(option.sourceId, option.label)}
          >
            <CompletionCheckbox checked={option.isCurrent} />
            <span className={optionText}>
              <span className={optionTitle}>
                {hasExisting && !option.isCurrent
                  ? t("quickSet.changeTo", { language: option.label })
                  : option.label}
              </span>
            </span>
          </button>
        ));

  return (
    <div className={shell}>
      <button
        type="button"
        className={closeButton}
        onClick={onClose}
        disabled={phase.kind === "applying"}
        aria-label={t("common.closeAria")}
      >
        <MdClose />
      </button>
      {session ? (
        <>
          <div className={headerBlock}>
            <BrandLogo />
            <div className={appIdentity}>
              <AppIconView src={presentations[session.appId]?.icon} label={appLabel} />
              <p className={appName}>{appLabel}</p>
            </div>
            <p className={prompt}>
              {hasExisting ? t("quickSet.currentSetting") : t("quickSet.chooseLanguage")}
            </p>
          </div>
          <hr className={divider} />
          <div className={nameField}>
            <label className={nameLabel} htmlFor="quick-set-rule-name">
              {t("quickSet.ruleName")}
            </label>
            <input
              id="quick-set-rule-name"
              className={nameInput}
              type="text"
              value={ruleName}
              disabled={!isIdle}
              onChange={(event) => setRuleName(event.target.value)}
            />
          </div>
          <hr className={divider} />
          <div className={optionsScroll}>{withDividers(optionItems)}</div>
          {error ? (
            <>
              <hr className={divider} />
              <p className={emptyState}>{error}</p>
            </>
          ) : null}
          <hr className={divider} />
          <div className={footerBlock}>
            {withDividers([
              <SettingItem
                key="advanced"
                icon={MdSettings}
                title={hasExisting ? t("quickSet.editRule") : t("quickSet.advanced")}
                type="navigation"
                onClick={isIdle ? onAdvanced : undefined}
              />,
              ...(hasExisting
                ? [
                    <SettingItem
                      key="delete"
                      icon={FiTrash2}
                      title={t("quickSet.deleteRule")}
                      type="action"
                      danger
                      onClick={isIdle ? () => setDeleteOpen(true) : undefined}
                    />,
                  ]
                : []),
            ])}
          </div>
        </>
      ) : (
        <div className={headerBlock}>
          <BrandLogo />
          <p className={emptyState}>
            {unavailable ? t("quickSet.noFocusTarget") : (error ?? t("quickSet.waiting"))}
          </p>
        </div>
      )}
      <SuccessDialog
        isOpen={successLabel !== null}
        onClose={closeAfterSuccess}
        title={successLabel ? t("quickSet.saved", { language: successLabel }) : ""}
      />
      <AlertDialog
        isOpen={deleteOpen}
        onClose={() => setDeleteOpen(false)}
        title={t("apps.deleteRuleConfirmTitle")}
        text={t("apps.deleteRuleConfirmText", { title: ruleName || t("quickSet.ruleName") })}
        confirmButtonText={t("apps.deleteRuleConfirmButton")}
        onConfirm={onDeleteConfirm}
      />
    </div>
  );
}
