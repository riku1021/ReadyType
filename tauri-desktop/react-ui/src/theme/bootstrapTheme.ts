export type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "theme";

/**
 * jotai `atomWithStorage("theme")` と同じキーから解決テーマを読む。
 * JSON 文字列（例: `"system"`）を想定。
 */
export function resolveStoredTheme(): ResolvedTheme {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    const stored = raw === null ? "system" : (JSON.parse(raw) as string);
    if (stored === "dark") {
      return "dark";
    }
    if (stored === "light") {
      return "light";
    }
  } catch {
    // localStorage / JSON 不正時は system 扱い
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

/** `index.html` のインライン背景を外し、Panda の token に任せる */
export function clearBootstrapBackground() {
  document.documentElement.style.backgroundColor = "";
  document.body.style.backgroundColor = "";
}
