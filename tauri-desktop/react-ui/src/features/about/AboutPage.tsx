import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaFileShield, FaYoutube } from "react-icons/fa6";
import { MdBugReport, MdDescription, MdLightbulbOutline, MdPublic } from "react-icons/md";

import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";

import * as styles from "./AboutPage.styles";
import {
  ABOUT_BUG_REPORT_URL,
  ABOUT_FEATURE_REQUEST_URL,
  ABOUT_INTRO_VIDEO_URL,
  ABOUT_PRIVACY_POLICY_URL,
  ABOUT_TERMS_OF_USE_URL,
  ABOUT_WEBSITE_URL,
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

      <SettingsSection title={t("about.sectionProduct")}>
        <SettingItem
          icon={MdPublic}
          title={t("about.website")}
          type="externalLink"
          onClick={() => {
            void openExternal(ABOUT_WEBSITE_URL);
          }}
        />
        <SettingItem
          icon={FaYoutube}
          title={t("about.introVideo")}
          type="externalLink"
          onClick={() => {
            void openExternal(ABOUT_INTRO_VIDEO_URL);
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("about.sectionFeedback")}>
        <SettingItem
          icon={MdLightbulbOutline}
          title={t("about.featureRequest")}
          type="action"
          onClick={() => {
            void openExternal(ABOUT_FEATURE_REQUEST_URL);
          }}
        />
        <SettingItem
          icon={MdBugReport}
          title={t("about.bugReport")}
          type="action"
          onClick={() => {
            void openExternal(ABOUT_BUG_REPORT_URL);
          }}
        />
      </SettingsSection>

      <SettingsSection title={t("about.sectionLegal")}>
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
