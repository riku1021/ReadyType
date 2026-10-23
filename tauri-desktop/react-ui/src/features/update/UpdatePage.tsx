import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";
import type { UpdatePhase } from "./appUpdateController";
import * as styles from "./UpdatePage.styles";
import { parseLastCheckedAt } from "./updatePreferences";
import { useAppUpdate } from "./useAppUpdate";

function formatLastChecked(date: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
}

function isBusy(phase: UpdatePhase): boolean {
  return phase.kind === "checking" || phase.kind === "downloading" || phase.kind === "installing";
}

export function UpdatePage() {
  const { t, i18n } = useTranslation();
  const { preferences, phase, installReady, setAutoCheck, setAutoDownload, checkNow, install } =
    useAppUpdate();

  const lastCheckedLabel = useMemo(() => {
    const checkedAt = parseLastCheckedAt(preferences.lastCheckedAt);
    const datetime =
      checkedAt != null ? formatLastChecked(checkedAt, i18n.language) : t("common.emDash");
    return t("update.lastChecked", { datetime });
  }, [i18n.language, preferences.lastCheckedAt, t]);

  const description = useMemo(() => {
    if (
      phase.kind === "available" ||
      phase.kind === "downloading" ||
      phase.kind === "downloaded" ||
      phase.kind === "installing"
    ) {
      return t("update.available", { version: phase.version });
    }
    if (phase.kind === "error") {
      switch (phase.messageKey) {
        case "desktopOnly":
          return t("update.errors.desktopOnly");
        case "checkFailed":
          return t("update.errors.checkFailed");
        case "downloadFailed":
          return t("update.errors.downloadFailed");
        case "installFailed":
          return t("update.errors.installFailed");
        case "restartFailed":
          return t("update.errors.restartFailed");
      }
    }
    return lastCheckedLabel;
  }, [lastCheckedLabel, phase, t]);

  const buttonLabel = (() => {
    switch (phase.kind) {
      case "checking":
        return t("update.checking");
      case "downloading":
        return t("update.downloading");
      case "installing":
        return t("update.installing");
      default:
        return installReady ? t("update.installButton") : t("update.checkButton");
    }
  })();

  return (
    <div className={styles.page}>
      <SettingsSection>
        <SettingItem
          type="toggle"
          title={t("update.autoCheck")}
          toggleValue={preferences.autoCheck}
          onToggleChanged={setAutoCheck}
        />
        <SettingItem
          type="toggle"
          title={t("update.autoDownload")}
          toggleValue={preferences.autoDownload}
          onToggleChanged={setAutoDownload}
        />
      </SettingsSection>

      <SettingsSection>
        <SettingItem
          type="button"
          title={t("update.checkNow")}
          description={description}
          buttonLabel={buttonLabel}
          disabled={isBusy(phase)}
          onClick={() => {
            if (installReady) {
              void install();
              return;
            }
            void checkNow();
          }}
        />
      </SettingsSection>
    </div>
  );
}
