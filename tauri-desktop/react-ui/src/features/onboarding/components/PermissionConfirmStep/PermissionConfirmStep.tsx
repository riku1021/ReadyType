import { Trans, useTranslation } from "react-i18next";
import { MdOutlineLock } from "react-icons/md";

import CheckAnimation from "@/animations/CheckAnimation";

import * as styles from "./PermissionConfirmStep.styles";

type PermissionConfirmStepProps = {
  granted: boolean;
};

export function PermissionConfirmStep({ granted }: PermissionConfirmStepProps) {
  const { t } = useTranslation();

  return (
    <div className={styles.root}>
      <div className={styles.badge} aria-hidden={granted ? true : undefined}>
        {granted ? (
          <div className={styles.checkAnimationWrap}>
            <CheckAnimation loop={false} />
          </div>
        ) : (
          <span className={styles.pendingBadge} aria-hidden="true">
            <MdOutlineLock />
          </span>
        )}
      </div>

      {granted ? (
        <h1 className={styles.title}>
          <Trans
            i18nKey="onboarding.permissionConfirm.headingGranted"
            components={{ accent: <span className={styles.titleAccent} /> }}
          />
        </h1>
      ) : (
        <h1 className={styles.titlePending}>{t("onboarding.permissionConfirm.title")}</h1>
      )}

      {granted ? (
        <p className={styles.lead}>
          {t("onboarding.permissionConfirm.leadGranted", { appName: t("about.appName") })}
        </p>
      ) : (
        <p className={styles.leadPending}>{t("onboarding.permissionConfirm.description")}</p>
      )}

      <section className={styles.statusCard} aria-live="polite">
        <div className={styles.fieldIcon} aria-hidden="true">
          <span className={styles.fieldIconBack} />
          <span className={styles.fieldIconFront} />
          {granted ? (
            <>
              <span className={styles.fieldSpark({ index: "0" })} />
              <span className={styles.fieldSpark({ index: "1" })} />
              <span className={styles.fieldSpark({ index: "2" })} />
            </>
          ) : null}
        </div>
        <span className={styles.statusSep} aria-hidden="true" />
        <div className={styles.okRow}>
          <span
            className={styles.okCheck({ tone: granted ? "granted" : "pending" })}
            aria-hidden="true"
          >
            {granted ? null : <MdOutlineLock />}
          </span>
          <div>
            <b className={styles.okTitle}>{t("onboarding.permissionConfirm.statusFeature")}</b>
            <small className={styles.okSubtitle}>
              {granted
                ? t("onboarding.permissionConfirm.statusReady")
                : t("accessibility.statusRequired")}
            </small>
          </div>
        </div>
      </section>
    </div>
  );
}
