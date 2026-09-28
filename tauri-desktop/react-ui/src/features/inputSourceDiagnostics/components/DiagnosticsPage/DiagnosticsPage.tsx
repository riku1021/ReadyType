import { useEffect, useState } from "react";

import { Button } from "@/components/Button";
import {
  type FocusSnapshotDto,
  type SwitchLogDto,
  useInputSourceState,
} from "@/features/inputSource";

import {
  flattenSnapshotAttributes,
  formatLatencyMicros,
  formatSwitchLogLine,
  logsToTsv,
} from "../../diagnosticsViewModel";
import * as styles from "./DiagnosticsPage.styles";

export function DiagnosticsPage() {
  const {
    startDebugCapture,
    stopDebugCapture,
    clearSwitchLogs,
    fetchSwitchLogs,
    fetchFocusSnapshot,
  } = useInputSourceState();
  const [logs, setLogs] = useState<SwitchLogDto[]>([]);
  const [snapshot, setSnapshot] = useState<FocusSnapshotDto | null>(null);
  const [copyError, setCopyError] = useState<string | null>(null);

  useEffect(() => {
    void startDebugCapture();
    return () => {
      void stopDebugCapture();
    };
  }, [startDebugCapture, stopDebugCapture]);

  useEffect(() => {
    let cancelled = false;
    const refresh = async () => {
      const [nextLogs, nextSnapshot] = await Promise.all([fetchSwitchLogs(), fetchFocusSnapshot()]);
      if (!cancelled) {
        setLogs(nextLogs);
        setSnapshot(nextSnapshot);
      }
    };
    void refresh();
    const timer = window.setInterval(() => {
      void refresh();
    }, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [fetchFocusSnapshot, fetchSwitchLogs]);

  const attributes = flattenSnapshotAttributes(snapshot?.elements ?? []);
  const latency = formatLatencyMicros(logs.map((log) => log.processingMicros));

  const copyText = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopyError(null);
    } catch {
      setCopyError("クリップボードへのコピーに失敗しました");
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <Button
          variant="secondary"
          onClick={() => {
            void copyText(logsToTsv(logs));
          }}
        >
          ログをコピー
        </Button>
        <Button
          variant="secondary"
          onClick={() => {
            void clearSwitchLogs().then(() => setLogs([]));
          }}
        >
          ログをクリア
        </Button>
      </div>
      {copyError ? <p className={styles.meta}>{copyError}</p> : null}

      <section className={styles.section}>
        <h2 className={styles.title}>処理時間</h2>
        <p className={styles.meta}>
          p50 {latency.p50} / p95 {latency.p95}
        </p>
      </section>

      <section className={styles.section}>
        <h2 className={styles.title}>フォーカス要素</h2>
        {snapshot ? (
          <p className={styles.meta}>
            {snapshot.app.appId} / generation {snapshot.generation}
          </p>
        ) : (
          <p className={styles.meta}>スナップショットなし</p>
        )}
        {attributes.map((attribute) => (
          <div
            key={`${attribute.depth}-${attribute.attribute}-${attribute.value}`}
            className={styles.row}
          >
            <p className={styles.meta}>
              depth {attribute.depth}: {attribute.attribute} = {attribute.value}
            </p>
            <div className={styles.toolbar}>
              <Button
                variant="secondary"
                onClick={() => {
                  void copyText(attribute.value);
                }}
              >
                コピー
              </Button>
            </div>
          </div>
        ))}
      </section>

      <section className={styles.section}>
        <h2 className={styles.title}>切替ログ</h2>
        <p className={styles.meta}>
          未実行（disabled / snoozed / noRule /
          alreadySelected）も記録します。無視（ignored）は設定のデバッグログ ON 時のみです。
        </p>
        {logs.length === 0 ? <p className={styles.meta}>ログなし</p> : null}
        {logs.map((log) => (
          <p
            key={`${log.timestampUnixMs}-${log.generation}-${log.ruleId}-${log.decision}-${log.processingMicros}`}
            className={styles.meta}
          >
            {formatSwitchLogLine(log)}
          </p>
        ))}
      </section>
    </div>
  );
}
