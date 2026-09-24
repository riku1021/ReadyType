import { useEffect } from "react";

import { Button } from "@/components/Button";
import { useInputSourceState } from "@/features/inputSource";

import * as styles from "./AccessibilityPage.styles";
import { toAccessibilityViewModel } from "./accessibilityViewModel";

const STATUS_LABELS = {
  loading: "確認中",
  granted: "許可済み",
  denied: "未許可",
  unsupported: "未対応",
  error: "エラー",
} as const;

export function AccessibilityPage() {
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

  const view = toAccessibilityViewModel({
    ready: state?.ready ?? false,
    permission: state?.permission ?? null,
    initError: state?.initError ?? null,
  });

  return (
    <div className={styles.page}>
      <section className={styles.card} aria-live="polite">
        <h2 className={styles.headline}>{view.headline}</h2>
        <p className={styles.description}>{view.description}</p>
        <div className={styles.statusRow}>
          <span className={styles.label}>現在の状態</span>
          <span className={styles.value}>{STATUS_LABELS[view.kind]}</span>
        </div>
        <div className={styles.actions}>
          {view.showPromptActions ? (
            <>
              <Button
                onClick={() => {
                  void requestPermission();
                }}
              >
                権限を要求
              </Button>
              <Button
                variant="secondary"
                onClick={() => {
                  void openAccessibilitySettings();
                }}
              >
                システム設定を開く
              </Button>
            </>
          ) : null}
          <Button
            variant="secondary"
            onClick={() => {
              void refreshPermission();
            }}
          >
            状態を再確認
          </Button>
        </div>
      </section>
    </div>
  );
}
