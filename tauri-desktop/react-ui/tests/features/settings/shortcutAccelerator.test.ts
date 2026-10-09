import { describe, expect, it } from "vitest";

import {
  acceleratorToChord,
  chordToAccelerator,
  DEFAULT_QUICK_SET_ACCELERATOR,
} from "@/features/settings/shortcutAccelerator";
import { defaultShortcutChord } from "@/features/settings/shortcutChord";

describe("shortcutAccelerator", () => {
  it("既定 chord は DEFAULT_QUICK_SET_ACCELERATOR になる", () => {
    expect(chordToAccelerator(defaultShortcutChord("macOs"), "macOs")).toBe(
      DEFAULT_QUICK_SET_ACCELERATOR
    );
    expect(chordToAccelerator(defaultShortcutChord("windows"), "windows")).toBe(
      DEFAULT_QUICK_SET_ACCELERATOR
    );
  });

  it("CommandOrControl は OS に応じて meta / ctrl に展開する", () => {
    expect(acceleratorToChord(DEFAULT_QUICK_SET_ACCELERATOR, "macOs")).toEqual(
      defaultShortcutChord("macOs")
    );
    expect(acceleratorToChord(DEFAULT_QUICK_SET_ACCELERATOR, "windows")).toEqual(
      defaultShortcutChord("windows")
    );
  });

  it("明示的な Control は mac でも ctrl のまま", () => {
    const chord = acceleratorToChord("Control+Shift+L", "macOs");
    expect(chord.ctrl).toBe(true);
    expect(chord.meta).toBe(false);
    expect(chordToAccelerator(chord, "macOs")).toBe("Control+Shift+L");
  });
});
