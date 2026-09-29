import { useTranslation } from "react-i18next";

import { Button } from "@/components/Button";
import { Toggle } from "@/components/Toggle";
import { useInputSourceState } from "@/features/inputSource";

import * as styles from "./HomePage.styles";
import { toStatusViewModel } from "./statusViewModel";

export function HomePage() {
  const { t } = useTranslation();
  const { state, setEnabled, requestPermission, openAccessibilitySettings } = useInputSourceState();
  const view = toStatusViewModel(state, t);

  return (
    <div className={styles.page}>
      {state && !state.accessibilityTrusted && state.permission !== "unsupported" ? (
        <section className={styles.banner} aria-live="polite">
          <h2 className={styles.bannerTitle}>{t("home.permissionBannerTitle")}</h2>
          <p className={styles.bannerText}>{t("home.permissionBannerText")}</p>
          <div className={styles.bannerActions}>
            <Button
              onClick={() => {
                void requestPermission();
              }}
            >
              {t("common.requestPermission")}
            </Button>
            <Button
              variant="secondary"
              onClick={() => {
                void openAccessibilitySettings();
              }}
            >
              {t("common.openSystemSettings")}
            </Button>
          </div>
        </section>
      ) : null}

      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <h2 className={styles.headline}>{view.headline}</h2>
          <Toggle
            checked={state?.enabled ?? false}
            disabled={!state || state.permission === "unsupported"}
            aria-label={t("home.autoSwitchAria")}
            onChange={(enabled) => {
              void setEnabled(enabled);
            }}
          />
        </div>
        <div className={styles.row}>
          <span className={styles.label}>{t("home.labelCurrentApp")}</span>
          <span className={styles.value}>{view.currentAppLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>{t("home.labelMatchedRule")}</span>
          <span className={styles.value}>{view.matchedRuleLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>{t("home.labelCurrentSource")}</span>
          <span className={styles.value}>{view.currentSourceLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>{t("home.labelTargetSource")}</span>
          <span className={styles.value}>{view.targetSourceLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>{t("home.labelLastResult")}</span>
          <span className={styles.value}>{view.lastDecisionLabel}</span>
        </div>
      </section>
    </div>
  );
}
