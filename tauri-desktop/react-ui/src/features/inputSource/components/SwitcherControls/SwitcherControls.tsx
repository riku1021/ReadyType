import { useRef, useState } from "react";
import { MdPowerSettingsNew } from "react-icons/md";

import { SelectionMenu } from "@/components/SelectionMenu";
import { useInputSourceState } from "@/features/inputSource";
import { cx } from "@/styled-system/css";

import * as styles from "./SwitcherControls.styles";

const SNOOZE_LABELS = ["5分間", "15分間", "1時間", "無期限"] as const;

type SnoozeLabel = (typeof SNOOZE_LABELS)[number];

function snoozeMinutes(label: SnoozeLabel): number | null {
  switch (label) {
    case "5分間":
      return 5;
    case "15分間":
      return 15;
    case "1時間":
      return 60;
    case "無期限":
      return null;
  }
}

export function SwitcherControls() {
  const { state, setEnabled, setSnooze, snoozeIndefinitely, resume } = useInputSourceState();
  const playPauseRef = useRef<HTMLButtonElement>(null);
  const [menuOpen, setMenuOpen] = useState(false);

  const unsupported = state?.permission === "unsupported";
  const ready = Boolean(state?.ready);
  const enabled = state?.enabled ?? false;
  const snoozed = state?.snoozedUntilUnixMs != null;
  const controlsDisabled = !ready || unsupported;
  const showPlay = snoozed;

  const onPlayPauseClick = () => {
    if (controlsDisabled || !enabled) {
      return;
    }
    if (snoozed) {
      void resume();
      return;
    }
    setMenuOpen((open) => !open);
  };

  const onSnoozeSelected = (label: string) => {
    if (!(SNOOZE_LABELS as readonly string[]).includes(label)) {
      return;
    }
    const minutes = snoozeMinutes(label as SnoozeLabel);
    if (minutes === null) {
      void snoozeIndefinitely();
      return;
    }
    void setSnooze(minutes);
  };

  return (
    <>
      <button
        ref={playPauseRef}
        type="button"
        className={styles.playPause}
        aria-label={snoozed ? "自動切替を再開" : "一時停止"}
        aria-haspopup={snoozed ? undefined : "listbox"}
        aria-expanded={snoozed ? undefined : menuOpen}
        disabled={controlsDisabled || !enabled}
        onClick={onPlayPauseClick}
      >
        <span className={styles.playPauseGlyph({ playing: showPlay })} aria-hidden>
          <span className={styles.playPauseBar({ side: "start", playing: showPlay })} />
          <span className={styles.playPauseBar({ side: "end", playing: showPlay })} />
        </span>
      </button>
      <SelectionMenu
        isOpen={menuOpen}
        onClose={() => setMenuOpen(false)}
        anchorRef={playPauseRef}
        label="一時停止の長さ"
        options={SNOOZE_LABELS}
        currentSelection=""
        onSelectionChanged={onSnoozeSelected}
      />
      <button
        type="button"
        className={cx(styles.iconButton, enabled ? styles.powerOn : undefined)}
        aria-label="自動切替の電源"
        aria-pressed={enabled}
        disabled={controlsDisabled}
        onClick={() => {
          void setEnabled(!enabled);
        }}
      >
        <MdPowerSettingsNew className={styles.icon} aria-hidden="true" />
      </button>
    </>
  );
}
