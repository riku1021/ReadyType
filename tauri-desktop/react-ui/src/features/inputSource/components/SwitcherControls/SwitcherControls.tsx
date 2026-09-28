import { invoke } from "@tauri-apps/api/core";
import { type ChangeEvent, useState } from "react";
import { MdPowerSettingsNew } from "react-icons/md";

import { AlertDialog } from "@/components/Dialog";
import { useInputSourceState } from "@/features/inputSource";
import { cx } from "@/styled-system/css";

import * as styles from "./SwitcherControls.styles";

const SNOOZE_OPTIONS = [
  { value: "5", label: "5分間" },
  { value: "15", label: "15分間" },
  { value: "60", label: "1時間" },
  { value: "indefinite", label: "無期限" },
] as const;

async function quitApp() {
  await invoke("quit_app");
}

export function SwitcherControls() {
  const { state, setSnooze, snoozeIndefinitely, resume } = useInputSourceState();
  const [quitOpen, setQuitOpen] = useState(false);

  const unsupported = state?.permission === "unsupported";
  const ready = Boolean(state?.ready);
  const enabled = state?.enabled ?? false;
  const snoozed = state?.snoozedUntilUnixMs != null;
  const controlsDisabled = !ready || unsupported;
  const playPauseDisabled = controlsDisabled || !enabled;

  const onSnoozeChange = (event: ChangeEvent<HTMLSelectElement>) => {
    const value = event.target.value;
    event.target.value = "";
    if (!value) {
      return;
    }
    if (value === "indefinite") {
      void snoozeIndefinitely();
      return;
    }
    const minutes = Number(value);
    if (!Number.isFinite(minutes) || minutes <= 0) {
      return;
    }
    void setSnooze(minutes);
  };

  return (
    <>
      {/*
        図形（glyph / bars）は常に同一 DOM を維持する。
        snoozed で button / select ごと差し替えると CSS transition が発火しない。
      */}
      <span
        className={cx(styles.playPause, playPauseDisabled ? styles.playPauseDisabled : undefined)}
      >
        <span className={styles.playPauseGlyph({ playing: snoozed })} aria-hidden>
          <span className={styles.playPauseBar({ side: "start", playing: snoozed })} />
          <span className={styles.playPauseBar({ side: "end", playing: snoozed })} />
        </span>
        {snoozed ? (
          <button
            type="button"
            className={styles.hitTarget}
            aria-label="自動切替を再開"
            disabled={playPauseDisabled}
            onClick={() => {
              void resume();
            }}
          />
        ) : (
          <select
            className={styles.hitTarget}
            aria-label="一時停止の長さ"
            defaultValue=""
            disabled={playPauseDisabled}
            onChange={onSnoozeChange}
          >
            <option value="" disabled>
              一時停止の長さ
            </option>
            {SNOOZE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        )}
      </span>
      <button
        type="button"
        className={styles.iconButton}
        aria-label="ReadyTypeを終了"
        onClick={() => setQuitOpen(true)}
      >
        <MdPowerSettingsNew className={styles.icon} aria-hidden="true" />
      </button>
      <AlertDialog
        isOpen={quitOpen}
        onClose={() => setQuitOpen(false)}
        title="ReadyTypeを終了しますか？"
        text="自動切替も含め、アプリを完全に終了します。ウィンドウを閉じるだけの場合は、ウィンドウの閉じるボタンを使ってください。"
        confirmButtonText="終了"
        onConfirm={() => {
          void quitApp();
        }}
      />
    </>
  );
}
