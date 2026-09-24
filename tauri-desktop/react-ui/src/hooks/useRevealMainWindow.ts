import { useEffect } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  clearBootstrapBackground,
  resolveStoredTheme,
} from "@/theme/bootstrapTheme";

/** CSS / ネイティブ背景が落ち着くまで待つ（非表示時は rAF が使えない） */
const SETTLE_MS = 50;

function readResolvedTheme(): "light" | "dark" {
  const attr = document.documentElement.getAttribute("data-theme");
  if (attr === "light" || attr === "dark") {
    return attr;
  }
  return resolveStoredTheme();
}

/**
 * 起動時は `visible: false` のため、ネイティブ背景の準備後に表示する。
 * `data-theme` は `index.html` のインライン script が先行適用済み。
 *
 * 注意: 非表示ウィンドウでは `requestAnimationFrame` が発火しない／遅延するため使わない。
 */
export function useRevealMainWindow() {
  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let cancelled = false;

    const reveal = async () => {
      const theme = readResolvedTheme();

      try {
        await invoke("sync_window_chrome_theme", { theme });
      } catch {
        // ブラウザ単体起動など
      }

      await new Promise<void>((resolve) => {
        window.setTimeout(resolve, SETTLE_MS);
      });

      if (cancelled) {
        return;
      }

      try {
        await getCurrentWindow().show();
      } catch (error: unknown) {
        console.error("Failed to show main window", error);
        return;
      }

      if (cancelled) {
        return;
      }

      clearBootstrapBackground();
    };

    void reveal();

    return () => {
      cancelled = true;
    };
  }, []);
}
