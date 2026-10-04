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
import { MdClose, MdSettings } from "react-icons/md";

import { BrandLogo } from "@/components/BrandLogo";
import { SuccessDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { cx } from "@/styled-system/css";

import {
  applyQuickSetLanguage,
  getQuickSetSession,
  hideQuickSet,
  openQuickSetAdvanced,
} from "./api";
import {
  closeButton,
  divider,
  emptyState,
  headerBlock,
  optionButton,
  optionButtonCurrent,
  optionSubtitle,
  optionText,
  optionTitle,
  prompt,
  radio,
  radioChecked,
  shell,
  targetLabel,
} from "./QuickSetPage.styles";
import { buildQuickSetOptions, type QuickSetUiPhase, targetHeading } from "./quickSetViewModel";
import { QUICK_SET_EVENT, type QuickSetSessionDto } from "./types";

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
  const [phase, setPhase] = useState<QuickSetUiPhase>({ kind: "idle" });
  const [error, setError] = useState<string | null>(null);

  const resetInteraction = useCallback(() => {
    setPhase({ kind: "idle" });
    setError(null);
  }, []);

  const loadSession = useCallback(async () => {
    try {
      const next = await getQuickSetSession();
      setSession(next);
      resetInteraction();
    } catch {
      setError(t("quickSet.loadFailed"));
    }
  }, [resetInteraction, t]);

  useEffect(() => {
    void loadSession();
    let unlisten: (() => void) | undefined;
    // emit_to(quick-set) と対になる webview 専用 listen
    void getCurrentWebviewWindow()
      .listen<QuickSetSessionDto>(QUICK_SET_EVENT, (event) => {
        setSession(event.payload);
        resetInteraction();
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, [loadSession, resetInteraction]);

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
  const hasExisting = Boolean(session?.matchedRuleId);
  const isIdle = phase.kind === "idle";
  const successLabel = phase.kind === "success" ? phase.languageLabel : null;

  const onSelect = async (language: string, label: string) => {
    if (phase.kind !== "idle") {
      return;
    }
    setPhase({ kind: "applying" });
    setError(null);
    try {
      await applyQuickSetLanguage(language);
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

  const rows: ReactNode[] = session
    ? [
        <div key="header" className={headerBlock}>
          <BrandLogo />
          <p className={targetLabel}>{targetHeading(session, t)}</p>
          <p className={prompt}>
            {hasExisting ? t("quickSet.currentSetting") : t("quickSet.chooseLanguage")}
          </p>
        </div>,
        ...(options.length === 0
          ? [
              <p key="empty" className={emptyState}>
                {t("quickSet.noLanguages")}
              </p>,
            ]
          : options.map((option) => (
              <button
                key={option.language}
                type="button"
                className={cx(optionButton, option.isCurrent ? optionButtonCurrent : undefined)}
                disabled={!isIdle}
                aria-pressed={option.isCurrent}
                onClick={() => void onSelect(option.language, option.label)}
              >
                <span
                  className={cx(radio, option.isCurrent ? radioChecked : undefined)}
                  aria-hidden
                />
                <span className={optionText}>
                  <span className={optionTitle}>
                    {hasExisting && !option.isCurrent
                      ? t("quickSet.changeTo", { language: option.label })
                      : option.label}
                  </span>
                  <span className={optionSubtitle}>{option.sourceName}</span>
                </span>
              </button>
            ))),
        ...(error
          ? [
              <p key="error" className={emptyState}>
                {error}
              </p>,
            ]
          : []),
        <SettingItem
          key="advanced"
          icon={MdSettings}
          title={hasExisting ? t("quickSet.editRule") : t("quickSet.advanced")}
          type="navigation"
          onClick={isIdle ? onAdvanced : undefined}
        />,
      ]
    : [
        <p key="waiting" className={emptyState}>
          {error ?? t("quickSet.waiting")}
        </p>,
      ];

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
      {withDividers(rows)}
      <SuccessDialog
        isOpen={successLabel !== null}
        onClose={closeAfterSuccess}
        title={successLabel ? t("quickSet.saved", { language: successLabel }) : ""}
      />
    </div>
  );
}
