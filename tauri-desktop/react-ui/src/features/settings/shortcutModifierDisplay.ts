import type { IconType } from "react-icons";
import { FaWindows } from "react-icons/fa";
import { MdViewModule } from "react-icons/md";

import type { HostOs } from "@/platform";

import { type ModifierFlag, modifierTextLabel } from "./shortcutChord";

export type ModifierDisplaySlot =
  | { flag: ModifierFlag; kind: "icon"; Icon: IconType }
  | { flag: ModifierFlag; kind: "text"; label: string };

export function modifierDisplaySlots(hostOs: HostOs): ModifierDisplaySlot[] {
  switch (hostOs) {
    case "macOs":
      return [
        { flag: "ctrl", kind: "text", label: modifierTextLabel("ctrl", hostOs) },
        { flag: "alt", kind: "text", label: modifierTextLabel("alt", hostOs) },
        { flag: "shift", kind: "text", label: modifierTextLabel("shift", hostOs) },
        { flag: "meta", kind: "text", label: modifierTextLabel("meta", hostOs) },
      ];
    case "windows":
      return [
        { flag: "ctrl", kind: "text", label: modifierTextLabel("ctrl", hostOs) },
        { flag: "alt", kind: "text", label: modifierTextLabel("alt", hostOs) },
        { flag: "shift", kind: "text", label: modifierTextLabel("shift", hostOs) },
        { flag: "meta", kind: "icon", Icon: FaWindows },
      ];
    case "linux":
      return [
        { flag: "ctrl", kind: "text", label: modifierTextLabel("ctrl", hostOs) },
        { flag: "alt", kind: "text", label: modifierTextLabel("alt", hostOs) },
        { flag: "shift", kind: "text", label: modifierTextLabel("shift", hostOs) },
        { flag: "meta", kind: "icon", Icon: MdViewModule },
      ];
  }
}
