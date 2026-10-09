import type { IconType } from "react-icons";

export type NavLabelKey =
  | "nav.home"
  | "nav.accessibility"
  | "nav.apps"
  | "nav.diagnostics"
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
