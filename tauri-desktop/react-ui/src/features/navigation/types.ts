import type { IconType } from "react-icons";

export type NavLabelKey =
  | "nav.accessibility"
  | "nav.apps"
  | "nav.general"
  | "nav.about"
  | "nav.update"
  | "nav.lab";

export interface NavLink {
  /** リンク先のパス */
  to: string;
  /** リンクのラベル（i18n キー） */
  labelKey: NavLabelKey;
  /** リンクのアイコン */
  icon: IconType;
}
