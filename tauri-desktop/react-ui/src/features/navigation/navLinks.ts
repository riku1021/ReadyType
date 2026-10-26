import {
  IoShieldCheckmark as Accessibility,
  IoApps as Apps,
  IoFlask as Lab,
  IoSettingsSharp as Settings,
  IoSync as Update,
} from "react-icons/io5";
import { MdInfoOutline as About } from "react-icons/md";

import { getHostOs } from "@/platform/hostOs";

import type { NavLink } from "./types";

/**
 * アプリケーション全体で使用するナビゲーションリンクの定義
 */
const allNavLinks: NavLink[] = [
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
    to: "/general",
    labelKey: "nav.general",
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

const hostOs = getHostOs();
export const navLinks: NavLink[] = allNavLinks.filter(
  (link) => !(hostOs === "windows" && link.to === "/accessibility")
);

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
    .filter((link) => pathname.startsWith(`${link.to}/`))
    .sort((a, b) => b.to.length - a.to.length)[0];
}
