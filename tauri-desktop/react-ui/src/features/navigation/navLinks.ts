import {
  IoInformationCircle as About,
  IoShieldCheckmark as Accessibility,
  IoApps as Apps,
  IoPulse as Diagnostics,
  IoHome as Home,
  IoFlask as Lab,
  IoSettingsSharp as Settings,
  IoSync as Update,
} from "react-icons/io5";

import type { NavLink } from "./types";

/**
 * アプリケーション全体で使用するナビゲーションリンクの定義
 */
export const navLinks: NavLink[] = [
  {
    to: "/",
    label: "ホーム",
    icon: Home,
  },
  {
    to: "/accessibility",
    label: "アクセシビリティ",
    icon: Accessibility,
  },
  {
    to: "/apps",
    label: "アプリごとの設定",
    icon: Apps,
  },
  {
    to: "/diagnostics",
    label: "診断",
    icon: Diagnostics,
  },
  {
    to: "/settings",
    label: "設定",
    icon: Settings,
  },
  {
    to: "/about",
    label: "アプリについて",
    icon: About,
  },
  {
    to: "/update",
    label: "アップデート",
    icon: Update,
  },
  {
    to: "/lab",
    label: "開発確認",
    icon: Lab,
  },
];

/**
 * パス名から対応するナビゲーションリンクを取得する。
 * 完全一致を優先し、ネストされたルートは最長のプレフィックスでマッチする。
 */
export function getNavLinkByPath(pathname: string): NavLink | undefined {
  const exactMatch = navLinks.find((link) => link.to === pathname);
  if (exactMatch) {
    return exactMatch;
  }

  return navLinks
    .filter((link) => link.to !== "/" && pathname.startsWith(`${link.to}/`))
    .sort((a, b) => b.to.length - a.to.length)[0];
}
