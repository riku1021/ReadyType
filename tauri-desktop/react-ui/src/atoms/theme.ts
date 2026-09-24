import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";

export type Theme = "light" | "dark" | "system";

export const themeAtom = atomWithStorage<Theme>("theme", "system");

/** テーマの判定 */
export const isDarkThemeAtom = atom((get) => {
  const theme = get(themeAtom);
  return (
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)
  );
});

/** テーマの切替え */
export const toggleThemeAtom = atom(
  (get) => get(themeAtom),
  (get, set) => {
    const current = get(themeAtom);
    let next: Theme;
    if (current === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      next = prefersDark ? "light" : "dark";
    } else {
      next = current === "light" ? "dark" : "light";
    }

    set(themeAtom, next);
  }
);
