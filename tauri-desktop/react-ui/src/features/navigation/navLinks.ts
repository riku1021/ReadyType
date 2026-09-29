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
    labelKey: "nav.home",
    icon: Home,
  },
  {
    to: "/accessibility",
    labelKey: "nav.accessibility",
    icon: Accessibility,
  },
  {
    to: "/apps",
    labelKey: "nav.apps",
    icon: Apps,
  },
  {
    to: "/diagnostics",
    labelKey: "nav.diagnostics",
    icon: Diagnostics,
  },
  {
    to: "/settings",
    labelKey: "nav.settings",
    icon: Settings,
  },
  {
    to: "/about",
    labelKey: "nav.about",
    icon: About,
  },
  {
    to: "/update",
    labelKey: "nav.update",
    icon: Update,
  },
  {
    to: "/lab",
    labelKey: "nav.lab",
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
