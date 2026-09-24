import { useNavigate } from "@tanstack/react-router";
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
  logsToTsv,
  type SnapshotAttribute,
  toCondition,
} from "../../diagnosticsViewModel";
import * as styles from "./DiagnosticsPage.styles";

export function DiagnosticsPage() {
  const navigate = useNavigate();
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

  const applyAsCondition = (attribute: SnapshotAttribute) => {
    void navigate({
      to: "/rules",
      state: { conditionPrefill: toCondition(attribute) },
    });
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
              <Button
                variant="secondary"
                onClick={() => {
                  applyAsCondition(attribute);
                }}
              >
                条件に使用
              </Button>
            </div>
          </div>
        ))}
      </section>

      <section className={styles.section}>
        <h2 className={styles.title}>切替ログ</h2>
        {logs.length === 0 ? <p className={styles.meta}>ログなし</p> : null}
        {logs.map((log) => (
          <p
            key={`${log.timestampUnixMs}-${log.ruleId}-${log.processingMicros}`}
            className={styles.meta}
          >
            {log.success ? "成功" : "失敗"} / {log.appId} / {log.ruleId} / {log.from} → {log.to} /{" "}
            {log.processingMicros}µs
            {log.reason ? ` / ${log.reason}` : ""}
          </p>
        ))}
      </section>
    </div>
  );
}
