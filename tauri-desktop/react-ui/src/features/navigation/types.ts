import type { IconType } from "react-icons";

export interface NavLink {
  /** リンク先のパス */
  to: string;
  /** リンクのラベル */
  label: string;
  /** リンクのアイコン */
  icon: IconType;
}
