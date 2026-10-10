import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BsUniversalAccessCircle } from "react-icons/bs";
import { FiExternalLink, FiLock } from "react-icons/fi";
import { IoAlertCircle, IoCheckmarkCircle } from "react-icons/io5";

import { Button } from "@/components/Button";
import { useInputSourceState } from "@/features/inputSource";

import * as styles from "./AccessibilityPage.styles";
import { toAccessibilityViewModel } from "./accessibilityViewModel";

export function AccessibilityPage() {
  const { t } = useTranslation();
  const { state, requestPermission, openAccessibilitySettings, refreshPermission } =
    useInputSourceState();

  useEffect(() => {
    void refreshPermission();

    const onVisible = () => {
      if (document.visibilityState === "visible") {
        void refreshPermission();
      }
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [refreshPermission]);

  const view = toAccessibilityViewModel(
    {
      ready: state?.ready ?? false,
      permission: state?.permission ?? null,
      initError: state?.initError ?? null,
    },
    t
  );

  const bodyDescription =
    view.detailMessage != null ? view.detailMessage : t("accessibility.usageDescription");

  const handleOpenSettings = async () => {
    if (view.requestBeforeOpenSettings) {
      await requestPermission();
    }
    await openAccessibilitySettings();
  };

  return (
    <div className={styles.page}>
      <section className={styles.card({ tone: view.cardTone })} aria-live="polite">
        <div className={styles.topRow}>
          <div className={styles.mainColumn}>
            <span className={styles.featureIcon({ tone: view.cardTone })} aria-hidden="true">
              <BsUniversalAccessCircle />
            </span>
            <div className={styles.textStack}>
              <h2 className={styles.title}>{view.permissionPane}</h2>
              {view.showRequiredStatus ? (
                <p className={styles.requiredStatus}>
                  <IoAlertCircle aria-hidden="true" />
                  {t("accessibility.statusRequired")}
                </p>
              ) : null}
              <p className={styles.description}>{bodyDescription}</p>
            </div>
          </div>

          <div className={styles.actionsColumn}>
            {view.showGrantedBadge ? (
              <span className={styles.grantedBadge}>
                <IoCheckmarkCircle aria-hidden="true" />
                {t("accessibility.statusGranted")}
              </span>
            ) : null}
            {view.showOpenSettings ? (
              <Button
                variant={view.openSettingsVariant}
                iconRight={<FiExternalLink />}
                onClick={() => {
                  void handleOpenSettings();
                }}
              >
                {t("accessibility.openSystemSettings")}
              </Button>
            ) : null}
          </div>
        </div>

        {view.showPrivacy ? (
          <>
            <hr className={styles.divider} />
            <div className={styles.privacyRow}>
              <span className={styles.privacyIcon} aria-hidden="true">
                <FiLock />
              </span>
              <div className={styles.privacyStack}>
                <h3 className={styles.privacyTitle}>{t("accessibility.privacyTitle")}</h3>
                <p className={styles.description}>{t("accessibility.privacyDescription")}</p>
              </div>
            </div>
          </>
        ) : null}
      </section>
    </div>
  );
}
