import { invoke } from "@tauri-apps/api/core";
import { useAtomValue } from "jotai";
import { useEffect } from "react";
import { themeAtom } from "@/atoms/theme";
import { useWindowKind } from "@/platform";

async function syncNativeWindowChrome(resolved: "light" | "dark") {
  try {
    await invoke("sync_window_chrome_theme", { theme: resolved });
  } catch {
    // ブラウザ単体起動など、ネイティブ未接続時は無視
  }
}

/** ウィンドウ表示・テーマ反映後にだけトランジションを付け、起動時の色アニメを避ける */
const ENABLE_TRANSITIONS_MS = 150;

/**
 * `data-theme` を html 要素へ同期し、Panda CSS の light/dark 条件を有効化する。
 * メイン窓では macOS の Transparent タイトルバー背景も `background.main` に揃える。
 * quick-set 窓は透過 NSWindow のため native chrome sync は行わない。
 *
 * 起動時のネイティブ同期は `useRevealMainWindow` が await する。
 * ここでは値が変わったときだけ同期し、二重 invoke を避ける。
 */
export const useThemeManager = () => {
  const theme = useAtomValue(themeAtom);
  const windowKind = useWindowKind();

  useEffect(() => {
    const root = document.documentElement;
    const syncChrome = windowKind === "main";

    const applyTheme = (value: "light" | "dark") => {
      const previous = root.getAttribute("data-theme");
      root.setAttribute("data-theme", value);
      if (syncChrome && previous !== value) {
        void syncNativeWindowChrome(value);
      }
    };

    const transitionsTimer = window.setTimeout(() => {
      root.classList.add("enable-transitions");
    }, ENABLE_TRANSITIONS_MS);

    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      applyTheme(media.matches ? "dark" : "light");

      const onChange = (event: MediaQueryListEvent) => {
        applyTheme(event.matches ? "dark" : "light");
      };
      media.addEventListener("change", onChange);
      return () => {
        media.removeEventListener("change", onChange);
        window.clearTimeout(transitionsTimer);
      };
    }

    applyTheme(theme);
    return () => window.clearTimeout(transitionsTimer);
  }, [theme, windowKind]);

  return { theme };
};
