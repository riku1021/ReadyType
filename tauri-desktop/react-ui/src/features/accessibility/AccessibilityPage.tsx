import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/Button";
import { useInputSourceState } from "@/features/inputSource";

import * as styles from "./AccessibilityPage.styles";
import { type AccessibilityStatusKind, toAccessibilityViewModel } from "./accessibilityViewModel";

const STATUS_KEYS = {
  loading: "accessibility.statusLoading",
  granted: "accessibility.statusGranted",
  denied: "accessibility.statusDenied",
  unsupported: "accessibility.statusUnsupported",
  error: "accessibility.statusError",
} as const satisfies Record<AccessibilityStatusKind, string>;

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

  return (
    <div className={styles.page}>
      <section className={styles.card} aria-live="polite">
        <h2 className={styles.headline}>{view.headline}</h2>
        <p className={styles.description}>{view.description}</p>
        <div className={styles.statusRow}>
          <span className={styles.label}>{t("accessibility.labelCurrentStatus")}</span>
          <span className={styles.value}>{t(STATUS_KEYS[view.kind])}</span>
        </div>
        <div className={styles.actions}>
          {view.showPromptActions ? (
            <>
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
            </>
          ) : null}
          <Button
            variant="secondary"
            onClick={() => {
              void refreshPermission();
            }}
          >
            {t("accessibility.recheckStatus")}
          </Button>
        </div>
      </section>
    </div>
  );
}
