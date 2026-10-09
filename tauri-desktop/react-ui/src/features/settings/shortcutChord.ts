import type { HostOs } from "@/platform";

import { acceleratorToChord, DEFAULT_QUICK_SET_ACCELERATOR } from "./shortcutAccelerator";

/** 記録中／確定済みのショートカット（修飾＋本体キー）。 */
export type ShortcutChord = {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  /** 表示用（例: "A", "Space"）。未入力は null */
  key: string | null;
};

export type ModifierFlag = keyof Pick<ShortcutChord, "ctrl" | "alt" | "shift" | "meta">;

const MODIFIER_ORDER: ModifierFlag[] = ["ctrl", "alt", "shift", "meta"];

/** 既定 chord（正本は `DEFAULT_QUICK_SET_ACCELERATOR`）。 */
export function defaultShortcutChord(hostOs: HostOs): ShortcutChord {
  return acceleratorToChord(DEFAULT_QUICK_SET_ACCELERATOR, hostOs);
}

export function emptyShortcutChord(): ShortcutChord {
  return {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    key: null,
  };
}

/** 修飾キーの key なら対応フラグ。本体キーなら null。 */
export function modifierFlagFromKey(key: string): ModifierFlag | null {
  switch (key) {
    case "Control":
      return "ctrl";
    case "Alt":
      return "alt";
    case "Shift":
      return "shift";
    case "Meta":
    case "OS":
    case "Super":
    case "Hyper":
      return "meta";
    default:
      return null;
  }
}

/** 修飾キーの keydown なら対応フラグ。本体キーなら null。 */
export function modifierFlagFromEvent(event: KeyboardEvent): ModifierFlag | null {
  return modifierFlagFromKey(event.key);
}

/** Caps Lock / Fn はショートカットとして受け付けない。 */
export function isRejectedPrimaryKey(key: string): boolean {
  return key === "CapsLock" || key === "Fn";
}

/** KeyboardEvent から表示用の本体キー文字列を取る。修飾のみ／拒否キーなら null。 */
export function primaryKeyFromEvent(event: KeyboardEvent): string | null {
  if (modifierFlagFromKey(event.key) != null || isRejectedPrimaryKey(event.key)) {
    return null;
  }
  if (event.key === " ") {
    return "Space";
  }
  if (event.key === "Backspace") {
    return "Backspace";
  }
  if (event.key === "Enter") {
    return "Enter";
  }
  if (event.key === "Tab") {
    return "Tab";
  }
  if (event.key === "ArrowUp") {
    return "ArrowUp";
  }
  if (event.key === "ArrowDown") {
    return "ArrowDown";
  }
  if (event.key === "ArrowLeft") {
    return "ArrowLeft";
  }
  if (event.key === "ArrowRight") {
    return "ArrowRight";
  }
  if (event.key === "Escape") {
    return "Esc";
  }
  // Option/Alt 等で event.key が ¬ など別文字になるため、物理キーは code を優先する
  if (event.code.startsWith("Key") && event.code.length === 4) {
    return event.code.slice(3);
  }
  if (event.code.startsWith("Digit") && event.code.length === 6) {
    return event.code.slice(5);
  }
  if (event.key.length === 1) {
    return event.key.toUpperCase();
  }
  return event.key;
}

/**
 * 記録中の chord を更新する。
 * 修飾キーはトグル（再入力で解除）。本体キーは上書き。keydown の repeat は無視。
 */
export function chordFromKeyboardEvent(
  event: KeyboardEvent,
  previous: ShortcutChord
): ShortcutChord {
  if (event.repeat) {
    return previous;
  }
  const flag = modifierFlagFromEvent(event);
  if (flag != null) {
    return { ...previous, [flag]: !previous[flag] };
  }
  const primary = primaryKeyFromEvent(event);
  if (primary == null) {
    return previous;
  }
  return { ...previous, key: primary };
}

/** chip / 記録 UI テキストスロット共通の修飾ラベル。 */
export function modifierTextLabel(flag: ModifierFlag, hostOs: HostOs): string {
  if (hostOs === "macOs") {
    switch (flag) {
      case "ctrl":
        return "⌃";
      case "alt":
        return "⌥";
      case "shift":
        return "⇧";
      case "meta":
        return "⌘";
    }
  }
  switch (flag) {
    case "ctrl":
      return "Ctrl";
    case "alt":
      return "Alt";
    case "shift":
      return "Shift";
    case "meta":
      return hostOs === "windows" ? "Win" : "Super";
  }
}

export function formatShortcutChord(chord: ShortcutChord, hostOs: HostOs = "macOs"): string {
  const mods = MODIFIER_ORDER.filter((flag) => chord[flag]).map((flag) =>
    modifierTextLabel(flag, hostOs)
  );
  const key = chord.key ?? "";
  if (hostOs === "macOs") {
    if (mods.length === 0) {
      return key;
    }
    const modStr = mods.join("");
    if (key.length === 0) {
      return modStr;
    }
    return key.length > 1 ? `${modStr} ${key}` : `${modStr}${key}`;
  }
  if (mods.length === 0) {
    return key;
  }
  if (key.length === 0) {
    return mods.join("+");
  }
  return `${mods.join("+")}+${key}`;
}

export function isShortcutChordComplete(chord: ShortcutChord): boolean {
  return chord.key != null && chord.key.length > 0;
}

export function shortcutChordsEqual(left: ShortcutChord, right: ShortcutChord): boolean {
  return (
    left.ctrl === right.ctrl &&
    left.alt === right.alt &&
    left.shift === right.shift &&
    left.meta === right.meta &&
    left.key === right.key
  );
}
