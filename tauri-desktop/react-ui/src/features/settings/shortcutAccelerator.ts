import type { HostOs } from "@/platform";

import type { ShortcutChord } from "./shortcutChord";

/** Rust `DEFAULT_QUICK_SET_SHORTCUT` と同値。既定の正本（accelerator 文字列）。 */
export const DEFAULT_QUICK_SET_ACCELERATOR = "CommandOrControl+Shift+L";

/**
 * UI の chord を Tauri accelerator 文字列にする。
 * mac の meta / Win·Linux の ctrl だけなら `CommandOrControl` に畳む。
 */
export function chordToAccelerator(chord: ShortcutChord, hostOs: HostOs): string {
  if (chord.key == null || chord.key.length === 0) {
    throw new Error("shortcut key is required");
  }

  const parts: string[] = [];
  const primaryOnlyMeta = hostOs === "macOs" && chord.meta && !chord.ctrl;
  const primaryOnlyCtrl = hostOs !== "macOs" && chord.ctrl && !chord.meta;

  if (primaryOnlyMeta || primaryOnlyCtrl) {
    parts.push("CommandOrControl");
  } else {
    if (chord.ctrl) {
      parts.push("Control");
    }
    if (chord.meta) {
      parts.push(hostOs === "macOs" ? "Command" : "Super");
    }
  }
  if (chord.alt) {
    parts.push("Alt");
  }
  if (chord.shift) {
    parts.push("Shift");
  }
  if (parts.length === 0) {
    throw new Error("shortcut requires a modifier");
  }
  parts.push(chord.key);
  return parts.join("+");
}

/** Tauri accelerator を UI の chord に展開する。 */
export function acceleratorToChord(accelerator: string, hostOs: HostOs): ShortcutChord {
  const chord: ShortcutChord = {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    key: null,
  };
  const parts = accelerator
    .split("+")
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  if (parts.length === 0) {
    return chord;
  }

  const key = parts[parts.length - 1] ?? "";
  chord.key = key.length === 1 ? key.toUpperCase() : key;

  for (const part of parts.slice(0, -1)) {
    switch (part.toLowerCase()) {
      case "commandorcontrol":
      case "cmdorctrl":
      case "cmdorcontrol":
        if (hostOs === "macOs") {
          chord.meta = true;
        } else {
          chord.ctrl = true;
        }
        break;
      case "control":
      case "ctrl":
        chord.ctrl = true;
        break;
      case "alt":
      case "option":
        chord.alt = true;
        break;
      case "shift":
        chord.shift = true;
        break;
      case "super":
      case "win":
      case "meta":
      case "command":
      case "cmd":
        chord.meta = true;
        break;
      default:
        break;
    }
  }
  return chord;
}
