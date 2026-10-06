import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdDescription, MdMailOutline } from "react-icons/md";
import { FaFileShield } from "react-icons/fa6";

import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";

import * as styles from "./AboutPage.styles";
import {
  ABOUT_PRIVACY_POLICY_URL,
  ABOUT_SUPPORT_MAILTO,
  ABOUT_TERMS_OF_USE_URL,
} from "./placeholders";

async function openExternal(url: string): Promise<void> {
  if (isTauri()) {
    await openUrl(url);
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

export function AboutPage() {
  const { t } = useTranslation();
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    if (!isTauri()) {
      return;
    }
    void getVersion()
      .then((next) => {
        if (!cancelled) {
          setVersion(next);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setVersion(null);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const versionLabel = version != null ? t("about.version", { version }) : t("common.emDash");

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <img className={styles.appIcon} src="/pngs/logo.png" alt="" width={96} height={96} />
        <h1 className={styles.appName}>{t("about.appName")}</h1>
        <p className={styles.version}>{versionLabel}</p>
        <p className={styles.copyright}>{t("about.copyright")}</p>
      </header>

      <SettingsSection>
        <SettingItem
          icon={FaFileShield}
          title={t("about.privacyPolicy")}
          type="externalLink"
          onClick={() => {
            void openExternal(ABOUT_PRIVACY_POLICY_URL);
          }}
        />
        <SettingItem
          icon={MdDescription}
          title={t("about.termsOfUse")}
          type="externalLink"
          onClick={() => {
            void openExternal(ABOUT_TERMS_OF_USE_URL);
          }}
        />
        <SettingItem
          icon={MdMailOutline}
          title={t("about.support")}
          type="externalLink"
          onClick={() => {
            void openExternal(ABOUT_SUPPORT_MAILTO);
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("about.sectionCredits")}>
        <div className={styles.creditRow}>
          <span className={styles.creditLabel}>{t("about.license")}</span>
          <span className={styles.creditValue}>{t("about.licenseValue")}</span>
        </div>
      </SettingsSection>
    </div>
  );
}
