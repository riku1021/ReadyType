import { useNavigate } from "@tanstack/react-router";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAtom } from "jotai";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdBugReport, MdContrast, MdLanguage, MdRocketLaunch, MdSwapHoriz } from "react-icons/md";

import { themeAtom } from "@/atoms/theme";
import { AlertDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";
import { useInputSourceState } from "@/features/inputSource";
import { pageSlide } from "@/features/navigation";
import { isUiLocalePreference, type UiLocalePreference } from "@/locales/i18n";

import { getLanguageOptionLabel } from "./languageLabels";
import * as styles from "./SettingsPage.styles";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { getThemeLabel, THEME_OPTIONS, themeFromTranslatedLabel } from "./themeLabels";

type SettingsPendingKey = "enabled" | "autostart";

type SettingsPendingMap = Partial<Record<SettingsPendingKey, true>>;

export function SettingsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [theme, setTheme] = useAtom(themeAtom);
  const {
    state,
    setEnabled,
    autostartEnabled,
    refreshAutostartEnabled,
    setAutostartEnabled,
    resetDefaultRules,
  } = useInputSourceState();

  const [pending, setPending] = useState<SettingsPendingMap>({});
  const [resetOpen, setResetOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isPending = (key: SettingsPendingKey) => pending[key] === true;
  const anyPending = Object.keys(pending).length > 0;

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        await refreshAutostartEnabled();
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : t("common.loadSettingsFailed"));
        }
      }
    };
    void load();

    // System Settings でログイン項目を変えたあと、フォーカス復帰で OS 状態を取り直す。
    // 前回値は atom に残るので、再取得中に OFF フラッシュしない。
    const unlistenFocusPromise = isTauri()
      ? getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused) {
            void load();
          }
        })
      : undefined;

    return () => {
      cancelled = true;
      // `.then` 代入前に unmount しても解除できるよう Promise を待つ。
      void unlistenFocusPromise?.then((unlisten) => unlisten());
    };
  }, [refreshAutostartEnabled, t]);

  const runPending = (key: SettingsPendingKey, work: () => Promise<unknown>) => {
    if (isPending(key)) {
      return;
    }
    setPending((current) => ({ ...current, [key]: true }));
    void work()
      .catch((updateError) => {
        setError(updateError instanceof Error ? updateError.message : t("common.saveFailed"));
      })
      .finally(() => {
        setPending((current) => {
          const next = { ...current };
          delete next[key];
          return next;
        });
      });
  };

  const themeOptions = THEME_OPTIONS.map((key) => t(key));
  const preferenceRaw = state?.uiLocale ?? "";
  const currentPreference: UiLocalePreference = isUiLocalePreference(preferenceRaw)
    ? preferenceRaw
    : "system";

  return (
    <div className={styles.page}>
      <SettingsSection title={t("settings.sectionStartup")}>
        <SettingItem
          icon={MdSwapHoriz}
          title={t("settings.autoSwitch")}
          description={t("settings.autoSwitchDescription")}
          type="toggle"
          toggleValue={state?.enabled ?? false}
          disabled={isPending("enabled")}
          onToggleChanged={(enabled) => {
            if (!state || state.permission === "unsupported") {
              return;
            }
            runPending("enabled", () => setEnabled(enabled));
          }}
        />
        <SettingItem
          icon={MdRocketLaunch}
          title={t("settings.launchAtLogin")}
          type="toggle"
          toggleValue={autostartEnabled}
          disabled={autostartEnabled === null || isPending("autostart")}
          onToggleChanged={(enabled) => {
            runPending("autostart", () => setAutostartEnabled(enabled));
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("settings.sectionAppearance")}>
        <SettingItem
          icon={MdContrast}
          title={t("settings.theme")}
          type="selection"
          value={t(getThemeLabel(theme))}
          options={themeOptions}
          onSelectionChanged={(label) => {
            const nextTheme = themeFromTranslatedLabel(label, t);
            if (nextTheme) {
              setTheme(nextTheme);
            }
          }}
        />
        <SettingItem
          icon={MdLanguage}
          title={t("settings.displayLanguage")}
          type="navigation"
          value={getLanguageOptionLabel(currentPreference, t)}
          onClick={() => {
            void navigate({
              to: "/general/language",
              viewTransition: pageSlide("forward"),
            });
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("settings.sectionShortcuts")}>
        <ShortcutRecorder />
      </SettingsSection>

      <SettingsSection title={t("settings.sectionOther")}>
        <SettingItem
          icon={MdBugReport}
          title={t("settings.resetToDefaults")}
          type="action"
          danger
          onClick={() => setResetOpen(true)}
        />
      </SettingsSection>

      {error ? <p>{error}</p> : null}
      {anyPending ? <p>{t("common.saving")}</p> : null}

      <AlertDialog
        isOpen={resetOpen}
        onClose={() => setResetOpen(false)}
        title={t("settings.resetConfirmTitle")}
        text={t("settings.resetConfirmText")}
        confirmButtonText={t("settings.resetConfirmButton")}
        onConfirm={() => {
          void resetDefaultRules().catch((resetError) => {
            setError(resetError instanceof Error ? resetError.message : t("common.resetFailed"));
          });
        }}
      />
    </div>
  );
}
