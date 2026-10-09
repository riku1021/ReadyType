import { describe, expect, it } from "vitest";

import {
  chordFromKeyboardEvent,
  defaultShortcutChord,
  emptyShortcutChord,
  formatShortcutChord,
  isShortcutChordComplete,
  primaryKeyFromEvent,
} from "@/features/settings/shortcutChord";

function fakeKeyEvent(
  partial: Partial<KeyboardEvent> & { key: string; repeat?: boolean }
): KeyboardEvent {
  return {
    key: partial.key,
    code: partial.code ?? "",
    repeat: partial.repeat ?? false,
    ctrlKey: partial.ctrlKey ?? false,
    altKey: partial.altKey ?? false,
    shiftKey: partial.shiftKey ?? false,
    metaKey: partial.metaKey ?? false,
    getModifierState: () => false,
  } as unknown as KeyboardEvent;
}

describe("shortcutChord", () => {
  it("defaultShortcutChord は CommandOrControl+Shift+L", () => {
    expect(defaultShortcutChord("macOs")).toEqual({
      ctrl: false,
      alt: false,
      shift: true,
      meta: true,
      key: "L",
    });
    expect(defaultShortcutChord("windows")).toEqual({
      ctrl: true,
      alt: false,
      shift: true,
      meta: false,
      key: "L",
    });
    expect(defaultShortcutChord("linux")).toEqual({
      ctrl: true,
      alt: false,
      shift: true,
      meta: false,
      key: "L",
    });
  });

  it("formatShortcutChord は macOS で修飾記号とキーを連結する", () => {
    expect(
      formatShortcutChord(
        {
          ctrl: false,
          alt: false,
          shift: true,
          meta: true,
          key: "Space",
        },
        "macOs"
      )
    ).toBe("⇧⌘ Space");
  });

  it("formatShortcutChord は Windows でテキスト修飾と Win を連結する", () => {
    expect(
      formatShortcutChord(
        {
          ctrl: true,
          alt: false,
          shift: true,
          meta: true,
          key: "L",
        },
        "windows"
      )
    ).toBe("Ctrl+Shift+Win+L");
  });

  it("formatShortcutChord は Linux で Super を含めて連結する", () => {
    expect(
      formatShortcutChord(
        {
          ctrl: false,
          alt: true,
          shift: false,
          meta: true,
          key: "Space",
        },
        "linux"
      )
    ).toBe("Alt+Super+Space");
  });

  it("primaryKeyFromEvent は特殊キーと英字を正規化する", () => {
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: " " }))).toBe("Space");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "Backspace" }))).toBe("Backspace");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "Enter" }))).toBe("Enter");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "Tab" }))).toBe("Tab");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "ArrowUp" }))).toBe("ArrowUp");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "ArrowDown" }))).toBe("ArrowDown");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "ArrowLeft" }))).toBe("ArrowLeft");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "ArrowRight" }))).toBe("ArrowRight");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "a", code: "KeyA" }))).toBe("A");
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "Meta" }))).toBeNull();
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "CapsLock" }))).toBeNull();
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "Fn" }))).toBeNull();
  });

  it("primaryKeyFromEvent は Option+L でも物理キー L を返す", () => {
    expect(primaryKeyFromEvent(fakeKeyEvent({ key: "¬", code: "KeyL", altKey: true }))).toBe("L");
  });

  it("Caps Lock / Fn では chord の本体キーが変わらない", () => {
    const base = {
      ...emptyShortcutChord(),
      meta: true,
      key: "A" as string | null,
    };
    const afterCaps = chordFromKeyboardEvent(fakeKeyEvent({ key: "CapsLock" }), base);
    expect(afterCaps.key).toBe("A");
    expect(afterCaps.meta).toBe(true);

    const afterFn = chordFromKeyboardEvent(fakeKeyEvent({ key: "Fn" }), base);
    expect(afterFn.key).toBe("A");
    expect(afterFn.meta).toBe(true);
  });

  it("chordFromKeyboardEvent は修飾キーをトグルする", () => {
    const empty = emptyShortcutChord();
    const withMeta = chordFromKeyboardEvent(fakeKeyEvent({ key: "Meta" }), empty);
    expect(withMeta.meta).toBe(true);
    expect(withMeta.key).toBeNull();

    const toggledOff = chordFromKeyboardEvent(fakeKeyEvent({ key: "Meta" }), withMeta);
    expect(toggledOff.meta).toBe(false);

    const withShift = chordFromKeyboardEvent(fakeKeyEvent({ key: "Shift" }), withMeta);
    expect(withShift.meta).toBe(true);
    expect(withShift.shift).toBe(true);
  });

  it("chordFromKeyboardEvent は本体キーを上書きし修飾は維持する", () => {
    const base = {
      ...emptyShortcutChord(),
      meta: true,
      shift: true,
      key: "A" as string | null,
    };
    const next = chordFromKeyboardEvent(fakeKeyEvent({ key: "l" }), base);
    expect(next.meta).toBe(true);
    expect(next.shift).toBe(true);
    expect(next.key).toBe("L");
  });

  it("chordFromKeyboardEvent は key repeat を無視する", () => {
    const withMeta = chordFromKeyboardEvent(fakeKeyEvent({ key: "Meta" }), emptyShortcutChord());
    const repeated = chordFromKeyboardEvent(fakeKeyEvent({ key: "Meta", repeat: true }), withMeta);
    expect(repeated.meta).toBe(true);
  });

  it("isShortcutChordComplete は本体キー必須", () => {
    expect(
      isShortcutChordComplete({
        ctrl: false,
        alt: false,
        shift: true,
        meta: true,
        key: null,
      })
    ).toBe(false);
    expect(
      isShortcutChordComplete({
        ctrl: false,
        alt: false,
        shift: true,
        meta: true,
        key: "L",
      })
    ).toBe(true);
  });
});
