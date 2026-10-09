import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import type { IconType } from "react-icons";
import { FaArrowDown, FaArrowLeft, FaArrowRight, FaArrowUp } from "react-icons/fa";
import { GoTab } from "react-icons/go";
import { IoBackspaceOutline } from "react-icons/io5";
import { MdKeyboardReturn, MdOutlineReplay, MdSpaceBar } from "react-icons/md";

import { Button } from "@/components/Button";
import { getHostOs } from "@/platform";
import { loadQuickSetShortcut, setQuickSetShortcut } from "./quickSetShortcutApi";
import * as styles from "./ShortcutRecorder.styles";
import { acceleratorToChord, chordToAccelerator } from "./shortcutAccelerator";
import {
  chordFromKeyboardEvent,
  defaultShortcutChord,
  emptyShortcutChord,
  formatShortcutChord,
  isShortcutChordComplete,
  type ShortcutChord,
  shortcutChordsEqual,
} from "./shortcutChord";
import { type ModifierDisplaySlot, modifierDisplaySlots } from "./shortcutModifierDisplay";

const SPECIAL_KEY_ICONS: Record<string, IconType> = {
  Space: MdSpaceBar,
  Backspace: IoBackspaceOutline,
  Enter: MdKeyboardReturn,
  Tab: GoTab,
  ArrowUp: FaArrowUp,
  ArrowDown: FaArrowDown,
  ArrowLeft: FaArrowLeft,
  ArrowRight: FaArrowRight,
};

function modifierPresentation(slot: ModifierDisplaySlot): "symbol" | "label" {
  if (slot.kind === "icon") {
    return "symbol";
  }
  return slot.label.length > 1 ? "label" : "symbol";
}

function ModifierSlotView({ slot, active }: { slot: ModifierDisplaySlot; active: boolean }) {
  const presentation = modifierPresentation(slot);
  const className = styles.modifierGlyph({ active, presentation });
  if (slot.kind === "icon") {
    const Icon = slot.Icon;
    return (
      <span className={className} aria-hidden="true">
        <Icon />
      </span>
    );
  }
  return (
    <span className={className} aria-hidden="true">
      {slot.label}
    </span>
  );
}

function PrimaryKeyView({ keyLabel }: { keyLabel: string }) {
  const Icon = SPECIAL_KEY_ICONS[keyLabel];
  if (Icon != null) {
    return (
      <span className={styles.keyGlyph}>
        <Icon aria-hidden="true" />
      </span>
    );
  }
  return <span className={styles.keyGlyph}>{keyLabel}</span>;
}

export function ShortcutRecorder() {
  const { t } = useTranslation();
  const titleId = useId();
  const hostOs = getHostOs();
  const defaultChord = defaultShortcutChord(hostOs);
  const modifierSlots = modifierDisplaySlots(hostOs);
  const [committed, setCommitted] = useState<ShortcutChord>(defaultChord);
  const [recording, setRecording] = useState(false);
  const [draft, setDraft] = useState<ShortcutChord>(emptyShortcutChord());
  const [pending, setPending] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void loadQuickSetShortcut()
      .then(({ accelerator, registered }) => {
        if (cancelled) {
          return;
        }
        setCommitted(acceleratorToChord(accelerator, hostOs));
        setErrorMessage(registered ? null : t("settings.shortcutRegisterFailed"));
      })
      .catch(() => {
        if (!cancelled) {
          setErrorMessage(t("settings.shortcutLoadFailed"));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [hostOs, t]);

  useEffect(() => {
    if (!recording) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        setRecording(false);
        setDraft(emptyShortcutChord());
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      setDraft((previous) => chordFromKeyboardEvent(event, previous));
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [recording]);

  const chipLabel = formatShortcutChord(committed, hostOs) || t("settings.shortcutUnset");
  const canApply = isShortcutChordComplete(draft) && !pending;
  const canReset = !shortcutChordsEqual(committed, defaultChord) && !pending;

  const persistChord = async (chord: ShortcutChord) => {
    setPending(true);
    setErrorMessage(null);
    try {
      const accelerator = chordToAccelerator(chord, hostOs);
      const saved = await setQuickSetShortcut(accelerator);
      setCommitted(acceleratorToChord(saved, hostOs));
      setRecording(false);
      setDraft(emptyShortcutChord());
    } catch {
      setErrorMessage(t("settings.shortcutSaveFailed"));
    } finally {
      setPending(false);
    }
  };

  const startRecording = () => {
    setDraft(emptyShortcutChord());
    setRecording(true);
    setErrorMessage(null);
  };

  const cancelRecording = () => {
    setRecording(false);
    setDraft(emptyShortcutChord());
  };

  const applyRecording = () => {
    if (!canApply) {
      return;
    }
    void persistChord(draft);
  };

  const resetToDefault = () => {
    void persistChord(defaultChord);
  };

  return (
    <div className={styles.block}>
      <div className={styles.row}>
        <span className={styles.title} id={titleId}>
          {t("settings.recordShortcut")}
        </span>
        <div className={styles.trailing}>
          <button
            type="button"
            className={styles.chordChip}
            aria-labelledby={titleId}
            aria-pressed={recording}
            disabled={pending}
            onClick={startRecording}
          >
            {chipLabel}
          </button>
          <button
            type="button"
            className={styles.iconButton}
            aria-label={t("settings.shortcutResetAria")}
            disabled={!canReset && !recording}
            onClick={resetToDefault}
          >
            <MdOutlineReplay aria-hidden="true" />
          </button>
        </div>
      </div>

      {errorMessage != null ? (
        <p className={styles.errorMessage} role="alert">
          {errorMessage}
        </p>
      ) : null}

      {recording ? (
        <div className={styles.recordingSection}>
          <fieldset className={styles.recorderPanel}>
            <legend className={styles.visuallyHidden}>{t("settings.recordShortcut")}</legend>
            <div className={styles.modifierRow} aria-live="polite">
              <div className={styles.modifierGroup}>
                {modifierSlots.map((slot) => (
                  <ModifierSlotView key={slot.flag} slot={slot} active={draft[slot.flag]} />
                ))}
              </div>
              <span className={styles.keySlot} aria-hidden="true">
                {draft.key != null ? (
                  <PrimaryKeyView keyLabel={draft.key} />
                ) : (
                  <span className={styles.keyDiamond}>
                    <span className={styles.keyDiamondInner}>?</span>
                  </span>
                )}
              </span>
            </div>
          </fieldset>
          <div className={styles.actions}>
            <Button variant="primary" disabled={!canApply} onClick={applyRecording}>
              {t("settings.shortcutApply")}
            </Button>
            <Button variant="neutral" disabled={pending} onClick={cancelRecording}>
              {t("settings.shortcutCancel")}
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
