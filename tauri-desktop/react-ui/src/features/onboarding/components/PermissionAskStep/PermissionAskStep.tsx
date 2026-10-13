import { Trans, useTranslation } from "react-i18next";
import { MdOutlineLock } from "react-icons/md";

import * as styles from "./PermissionAskStep.styles";

export function PermissionAskStep() {
  const { t } = useTranslation();

  return (
    <div className={styles.root}>
      <div className={styles.hero} aria-hidden="true">
        <div className={styles.trafficRow}>
          <span className={styles.traffic({ tone: "red" })} />
          <span className={styles.traffic({ tone: "yellow" })} />
          <span className={styles.traffic({ tone: "green" })} />
        </div>
        <div className={styles.askField}>
          {t("onboarding.permissionAsk.heroPlaceholder")}
          <span className={styles.caret} />
        </div>
        <div className={styles.spark}>
          <span className={styles.sparkMark({ index: "0" })} />
          <span className={styles.sparkMark({ index: "1" })} />
          <span className={styles.sparkMark({ index: "2" })} />
        </div>
        <span className={styles.heroTail} />
      </div>

      <h1 className={styles.title}>
        <Trans
          i18nKey="onboarding.permissionAsk.heading"
          components={{ accent: <span className={styles.titleAccent} /> }}
        />
      </h1>
      <p className={styles.lead}>
        {t("onboarding.permissionAsk.description", { appName: t("about.appName") })}
      </p>

      <section className={styles.flow} aria-hidden="true">
        <div className={styles.chat}>
          <div className={styles.chatTraffic}>
            <span className={styles.traffic({ tone: "red", size: "sm" })} />
            <span className={styles.traffic({ tone: "yellow", size: "sm" })} />
            <span className={styles.traffic({ tone: "green", size: "sm" })} />
          </div>
          <div className={styles.chatInner}>
            <div className={styles.chatTitle}>
              <span className={styles.chatBubbleIcon} />
              {t("onboarding.permissionAsk.flowChatTitle")}
            </div>
            <div className={styles.chatField}>
              {t("onboarding.permissionAsk.flowChatText")}
              <span className={styles.caret} />
            </div>
          </div>
        </div>

        <span className={styles.arrow} />

        <div className={styles.center}>
          <img className={styles.centerLogo} src="/pngs/logo.png" alt="" width={46} />
          <b className={styles.centerName}>{t("about.appName")}</b>
          <span className={styles.focusBadge}>{t("onboarding.permissionAsk.focusBadge")}</span>
        </div>

        <span className={styles.arrow} />

        <div className={styles.resultCard}>
          <div className={styles.langPill}>
            <span className={styles.langGlyph}>{t("onboarding.permissionAsk.langGlyph")}</span>
            {t("onboarding.permissionAsk.langLabel")}
          </div>
          <p className={styles.resultNote}>{t("onboarding.permissionAsk.resultNote")}</p>
        </div>
      </section>

      <section className={styles.privacy}>
        <span className={styles.lockIcon} aria-hidden="true">
          <MdOutlineLock />
        </span>
        <div className={styles.privacyCopy}>
          <h2 className={styles.privacyTitle}>{t("onboarding.permissionAsk.privacyTitle")}</h2>
          <p className={styles.privacyBody}>
            {t("onboarding.permissionAsk.privacyBody", { appName: t("about.appName") })}
          </p>
        </div>
      </section>
    </div>
  );
}
