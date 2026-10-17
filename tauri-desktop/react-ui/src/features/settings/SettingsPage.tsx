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
import { isUiLocalePreference, type UiLocalePreference } from "@/locales/i18n";

import {
  getLanguageOptionLabel,
  languageFromOptionLabel,
  languageOptionLabels,
} from "./languageLabels";
import * as styles from "./SettingsPage.styles";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { getThemeLabel, THEME_OPTIONS, themeFromTranslatedLabel } from "./themeLabels";

export function SettingsPage() {
  const { t } = useTranslation();
  const [theme, setTheme] = useAtom(themeAtom);
  const {
    state,
    setEnabled,
    setDebugLogging,
    setUiLocale,
    autostartEnabled,
    refreshAutostartEnabled,
    setAutostartEnabled,
    resetDefaultRules,
  } = useInputSourceState();

  const [pending, setPending] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  const themeOptions = THEME_OPTIONS.map((key) => t(key));
  const languageOptions = languageOptionLabels(t);
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
          onToggleChanged={(enabled) => {
            if (!state || state.permission === "unsupported") {
              return;
            }
            setPending(true);
            void setEnabled(enabled)
              .catch((updateError) => {
                setError(
                  updateError instanceof Error ? updateError.message : t("common.saveFailed")
                );
              })
              .finally(() => setPending(false));
          }}
        />
        <SettingItem
          icon={MdRocketLaunch}
          title={t("settings.launchAtLogin")}
          type="toggle"
          toggleValue={autostartEnabled}
          disabled={autostartEnabled === null || pending}
          onToggleChanged={(enabled) => {
            setPending(true);
            void setAutostartEnabled(enabled)
              .catch((updateError) => {
                setError(
                  updateError instanceof Error ? updateError.message : t("common.saveFailed")
                );
              })
              .finally(() => setPending(false));
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
          type="selection"
          value={getLanguageOptionLabel(currentPreference, t)}
          options={languageOptions}
          onSelectionChanged={(label) => {
            const nextLocale = languageFromOptionLabel(label, t);
            if (!nextLocale || nextLocale === currentPreference) {
              return;
            }
            setPending(true);
            void setUiLocale(nextLocale)
              .catch((updateError) => {
                setError(
                  updateError instanceof Error ? updateError.message : t("common.saveFailed")
                );
              })
              .finally(() => setPending(false));
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("settings.sectionShortcuts")}>
        <ShortcutRecorder />
      </SettingsSection>

      <SettingsSection title={t("settings.sectionOther")}>
        <SettingItem
          icon={MdBugReport}
          title={t("settings.debugLogging")}
          type="toggle"
          toggleValue={state?.debugLogging ?? false}
          onToggleChanged={(enabled) => {
            setPending(true);
            void setDebugLogging(enabled)
              .catch((updateError) => {
                setError(
                  updateError instanceof Error ? updateError.message : t("common.saveFailed")
                );
              })
              .finally(() => setPending(false));
          }}
        />
        <SettingItem
          icon={MdBugReport}
          title={t("settings.resetToDefaults")}
          type="action"
          danger
          onClick={() => setResetOpen(true)}
        />
      </SettingsSection>

      {error ? <p>{error}</p> : null}
      {pending ? <p>{t("common.saving")}</p> : null}

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
