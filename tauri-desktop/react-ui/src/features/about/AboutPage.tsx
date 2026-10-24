import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaFileShield, FaYoutube } from "react-icons/fa6";
import { MdBugReport, MdDescription, MdLightbulbOutline, MdPublic } from "react-icons/md";

import { ErrorDialog, SuccessDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";
import { getHostOs } from "@/platform/hostOs";
import { getHostOsVersion } from "@/platform/hostOsVersion";

import * as styles from "./AboutPage.styles";
import {
  ABOUT_INTRO_VIDEO_URL,
  ABOUT_PRIVACY_POLICY_URL,
  ABOUT_TERMS_OF_USE_URL,
  ABOUT_WEBSITE_URL,
} from "./placeholders";
import {
  buildSupportMailDraft,
  hostOsDisplayName,
  openSupportMailDraft,
  SUPPORT_EMAIL,
  SUPPORT_MAIL_SUBJECT,
  type SupportMailEnv,
  type SupportMailKind,
  supportMailBodyKey,
} from "./supportMail";

async function openExternal(url: string): Promise<void> {
  if (isTauri()) {
    await openUrl(url);
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

export function AboutPage() {
  const { t, i18n } = useTranslation();
  const [version, setVersion] = useState<string | null>(null);
  const [mailFallback, setMailFallback] = useState<"copied" | "failed" | null>(null);
  const mailOpenRef = useRef(false);

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

  const openFeedback = (kind: SupportMailKind) => {
    if (mailOpenRef.current) {
      return;
    }
    mailOpenRef.current = true;
    const subject = SUPPORT_MAIL_SUBJECT[kind];
    const osVersion = getHostOsVersion();
    const env: SupportMailEnv = {
      appName: t("about.appName"),
      version: version ?? t("common.emDash"),
      osName: hostOsDisplayName(getHostOs()),
      osVersion: osVersion ?? t("common.emDash"),
      locale: i18n.resolvedLanguage ?? i18n.language,
    };
    const draft = buildSupportMailDraft({
      kind,
      prompt: t(supportMailBodyKey(kind)),
      environment: t("about.feedbackEnvironment", env),
      clipboardLead: t("about.feedbackClipboardLead", { email: SUPPORT_EMAIL, subject }),
    });
    void openSupportMailDraft(draft, {
      openUrl: openExternal,
      writeClipboard: (text) => navigator.clipboard.writeText(text),
    })
      .then((result) => {
        if (result === "copied") {
          setMailFallback("copied");
        }
        if (result === "failed") {
          setMailFallback("failed");
        }
      })
      .finally(() => {
        mailOpenRef.current = false;
      });
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <img className={styles.appIcon} src="/pngs/logo.png" alt="" width={96} />
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
          description={t("about.feedbackDraftDescription")}
          type="action"
          onClick={() => {
            openFeedback("feature");
          }}
        />
        <SettingItem
          icon={MdBugReport}
          title={t("about.bugReport")}
          description={t("about.feedbackDraftDescription")}
          type="action"
          onClick={() => {
            openFeedback("bug");
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

      <SuccessDialog
        isOpen={mailFallback === "copied"}
        onClose={() => setMailFallback(null)}
        title={t("about.feedbackCopiedTitle")}
        text={t("about.feedbackCopiedText")}
      />
      <ErrorDialog
        isOpen={mailFallback === "failed"}
        onClose={() => setMailFallback(null)}
        title={t("about.feedbackCopyFailedTitle")}
        text={t("about.feedbackCopyFailedText", { email: SUPPORT_EMAIL })}
      />
    </div>
  );
}
