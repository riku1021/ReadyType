import { useNavigate } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdBugReport, MdContrast, MdInfoOutline, MdLanguage, MdRocketLaunch } from "react-icons/md";

import { themeAtom } from "@/atoms/theme";
import { AlertDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";
import { useInputSourceState } from "@/features/inputSource";
import { isUiLocalePreference, type UiLocalePreference } from "@/locales/i18n";

import { getLanguageLabel, LANGUAGE_OPTIONS, languageFromTranslatedLabel } from "./languageLabels";
import * as styles from "./SettingsPage.styles";
import { getThemeLabel, THEME_OPTIONS, themeFromTranslatedLabel } from "./themeLabels";

export function SettingsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [theme, setTheme] = useAtom(themeAtom);
  const {
    state,
    setDebugLogging,
    setUiLocale,
    fetchAutostartEnabled,
    setAutostartEnabled,
    resetDefaultRules,
  } = useInputSourceState();

  const [autostartEnabled, setAutostartEnabledState] = useState(false);
  const [pending, setPending] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const nextAutostart = await fetchAutostartEnabled();
        if (!cancelled) {
          setAutostartEnabledState(nextAutostart);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : t("common.loadSettingsFailed"));
        }
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [fetchAutostartEnabled, t]);

  const themeOptions = THEME_OPTIONS.map((key) => t(key));
  const languageOptions = LANGUAGE_OPTIONS.map((key) => t(key));
  const preferenceRaw = state?.uiLocale ?? "";
  const currentPreference: UiLocalePreference = isUiLocalePreference(preferenceRaw)
    ? preferenceRaw
    : "system";

  return (
    <div className={styles.page}>
      <SettingsSection title={t("settings.sectionStartup")}>
        <SettingItem
          icon={MdRocketLaunch}
          title={t("settings.launchAtLogin")}
          type="toggle"
          toggleValue={autostartEnabled}
          onToggleChanged={(enabled) => {
            setPending(true);
            void setAutostartEnabled(enabled)
              .then(setAutostartEnabledState)
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
          value={t(getLanguageLabel(currentPreference))}
          options={languageOptions}
          onSelectionChanged={(label) => {
            const nextLocale = languageFromTranslatedLabel(label, t);
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
          icon={MdInfoOutline}
          title={t("settings.aboutApp")}
          type="navigation"
          onClick={() => {
            void navigate({ to: "/about" });
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
