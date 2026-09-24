import { Button } from "@/components/Button";
import { Toggle } from "@/components/Toggle";
import { useInputSourceState } from "@/features/inputSource";

import * as styles from "./HomePage.styles";
import { toStatusViewModel } from "./statusViewModel";

export function HomePage() {
  const { state, setEnabled, requestPermission, openAccessibilitySettings } = useInputSourceState();
  const view = toStatusViewModel(state);

  return (
    <div className={styles.page}>
      {state && !state.accessibilityTrusted && state.permission !== "unsupported" ? (
        <section className={styles.banner} aria-live="polite">
          <h2 className={styles.bannerTitle}>アクセシビリティ権限が必要です</h2>
          <p className={styles.bannerText}>
            フォーカス監視と入力ソース切替には、システム設定での許可が必要です。
          </p>
          <div className={styles.bannerActions}>
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
          </div>
        </section>
      ) : null}

      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <h2 className={styles.headline}>{view.headline}</h2>
          <Toggle
            checked={state?.enabled ?? false}
            disabled={!state || state.permission === "unsupported"}
            aria-label="自動切替"
            onChange={(enabled) => {
              void setEnabled(enabled);
            }}
          />
        </div>
        <div className={styles.row}>
          <span className={styles.label}>現在のアプリ</span>
          <span className={styles.value}>{view.currentAppLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>一致ルール</span>
          <span className={styles.value}>{view.matchedRuleLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>現在の入力ソース</span>
          <span className={styles.value}>{view.currentSourceLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>切替先</span>
          <span className={styles.value}>{view.targetSourceLabel}</span>
        </div>
        <div className={styles.row}>
          <span className={styles.label}>直近の結果</span>
          <span className={styles.value}>{view.lastDecisionLabel}</span>
        </div>
      </section>
    </div>
  );
}
