import { invoke } from "@tauri-apps/api/core";
import { type ChangeEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdPowerSettingsNew } from "react-icons/md";

import { AlertDialog } from "@/components/Dialog";
import { useInputSourceState } from "@/features/inputSource";
import { cx } from "@/styled-system/css";

import * as styles from "./SwitcherControls.styles";

const SNOOZE_OPTION_KEYS = [
  { value: "5", labelKey: "switcher.snooze5m" },
  { value: "15", labelKey: "switcher.snooze15m" },
  { value: "60", labelKey: "switcher.snooze1h" },
  { value: "indefinite", labelKey: "switcher.snoozeIndefinite" },
] as const;

async function quitApp() {
  await invoke("quit_app");
}

export function SwitcherControls() {
  const { t } = useTranslation();
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
            aria-label={t("switcher.resumeAria")}
            disabled={playPauseDisabled}
            onClick={() => {
              void resume();
            }}
          />
        ) : (
          <select
            className={styles.hitTarget}
            aria-label={t("switcher.snoozeDurationAria")}
            defaultValue=""
            disabled={playPauseDisabled}
            onChange={onSnoozeChange}
          >
            <option value="" disabled>
              {t("switcher.snoozeDurationPlaceholder")}
            </option>
            {SNOOZE_OPTION_KEYS.map((option) => (
              <option key={option.value} value={option.value}>
                {t(option.labelKey)}
              </option>
            ))}
          </select>
        )}
      </span>
      <button
        type="button"
        className={styles.iconButton}
        aria-label={t("switcher.quitAria")}
        onClick={() => setQuitOpen(true)}
      >
        <MdPowerSettingsNew className={styles.icon} aria-hidden="true" />
      </button>
      <AlertDialog
        isOpen={quitOpen}
        onClose={() => setQuitOpen(false)}
        title={t("switcher.quitConfirmTitle")}
        text={t("switcher.quitConfirmText")}
        confirmButtonText={t("switcher.quitConfirmButton")}
        onConfirm={() => {
          void quitApp();
        }}
      />
    </>
  );
}
