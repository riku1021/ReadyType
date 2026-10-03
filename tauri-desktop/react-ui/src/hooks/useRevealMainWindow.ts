import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { clearBootstrapBackground, resolveStoredTheme } from "@/theme/bootstrapTheme";

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
      try {
        if (getCurrentWindow().label !== "main") {
          clearBootstrapBackground();
          return;
        }
      } catch {
        // browser / non-tauri
      }

      const theme = readResolvedTheme();

      try {
        await invoke("sync_window_chrome_theme", { theme });
      } catch {
        // ブラウザ単体起動など
      }

      let shouldShow = true;
      try {
        shouldShow = await invoke<boolean>("should_show_main_window_on_launch");
      } catch (error) {
        console.error("Failed to resolve launch visibility; showing window", error);
      }
      if (!shouldShow || cancelled) {
        clearBootstrapBackground();
        return;
      }

      await new Promise<void>((resolve) => {
        window.setTimeout(resolve, SETTLE_MS);
      });

      if (cancelled) {
        return;
      }

      try {
        await getCurrentWindow().show();
      } catch (error) {
        console.error("Failed to show main window", error);
        return;
      }

      try {
        await getCurrentWindow().setFocus();
      } catch (error) {
        console.error("Failed to focus main window", error);
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
