import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

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
  const { t } = useTranslation();
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
      setCopyError(t("diagnostics.copyFailed"));
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
          {t("diagnostics.copyLogs")}
        </Button>
        <Button
          variant="secondary"
          onClick={() => {
            void clearSwitchLogs().then(() => setLogs([]));
          }}
        >
          {t("diagnostics.clearLogs")}
        </Button>
      </div>
      {copyError ? <p className={styles.meta}>{copyError}</p> : null}

      <section className={styles.section}>
        <h2 className={styles.title}>{t("diagnostics.sectionLatency")}</h2>
        <p className={styles.meta}>
          p50 {latency.p50} / p95 {latency.p95}
        </p>
      </section>

      <section className={styles.section}>
        <h2 className={styles.title}>{t("diagnostics.sectionFocus")}</h2>
        {snapshot ? (
          <p className={styles.meta}>
            {snapshot.app.appId} / generation {snapshot.generation}
          </p>
        ) : (
          <p className={styles.meta}>{t("diagnostics.noSnapshot")}</p>
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
                {t("diagnostics.copy")}
              </Button>
            </div>
          </div>
        ))}
      </section>

      <section className={styles.section}>
        <h2 className={styles.title}>{t("diagnostics.sectionSwitchLogs")}</h2>
        <p className={styles.meta}>{t("diagnostics.switchLogsHelp")}</p>
        {logs.length === 0 ? <p className={styles.meta}>{t("diagnostics.noLogs")}</p> : null}
        {logs.map((log) => (
          <p
            key={`${log.timestampUnixMs}-${log.generation}-${log.ruleId}-${log.decision}-${log.processingMicros}`}
            className={styles.meta}
          >
            {formatSwitchLogLine(log, t)}
          </p>
        ))}
      </section>
    </div>
  );
}
