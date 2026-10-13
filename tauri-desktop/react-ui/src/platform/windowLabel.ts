import { getCurrentWindow } from "@tauri-apps/api/window";

const MAIN_WINDOW_LABEL = "main";
const QUICK_SET_WINDOW_LABEL = "quick-set";

/**
 * 現在の WebView の窓ラベル。
 * Tauri 外（ブラウザ単体）は main 扱い。
 */
export function resolveWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return MAIN_WINDOW_LABEL;
  }
}

export function isQuickSetWindow(label: string = resolveWindowLabel()): boolean {
  return label === QUICK_SET_WINDOW_LABEL;
}
