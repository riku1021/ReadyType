import { isTauri } from "@tauri-apps/api/core";
import { platform as tauriPlatform } from "@tauri-apps/plugin-os";

/**
 * このデスクトップアプリが動いているホスト OS。
 * 入力ソース規則の `Platform`（ルール／アプリの対象 OS）とは別概念。
 */
export type HostOs = "macOs" | "windows" | "linux";

type TauriOsPlatform = ReturnType<typeof tauriPlatform>;

/**
 * Tauri `plugin-os` の `platform()` 値を `HostOs` に写す。
 * デスクトップ対象外（ios / android / BSD 等）は `null`。
 */
export function mapTauriOsPlatform(value: TauriOsPlatform | string): HostOs | null {
  switch (value) {
    case "macos":
      return "macOs";
    case "windows":
      return "windows";
    case "linux":
      return "linux";
    default:
      return null;
  }
}

/** Vite 単体など plugin-os が無いときの推定。 */
function detectFromNavigator(): HostOs {
  if (typeof navigator === "undefined") {
    return "macOs";
  }
  const platform = navigator.platform.toLowerCase();
  const userAgent = navigator.userAgent.toLowerCase();
  if (platform.includes("mac") || userAgent.includes("mac")) {
    return "macOs";
  }
  if (platform.includes("win") || userAgent.includes("windows")) {
    return "windows";
  }
  if (platform.includes("linux") || userAgent.includes("linux")) {
    return "linux";
  }
  return "macOs";
}

let cachedHostOs: HostOs | null = null;

/**
 * ホスト OS を解決する（プロセス寿命でキャッシュ）。
 * 正本は `@tauri-apps/plugin-os` の `platform()`。未対応値や非 Tauri は navigator 推定。
 */
export function getHostOs(): HostOs {
  if (cachedHostOs != null) {
    return cachedHostOs;
  }
  if (isTauri()) {
    try {
      const mapped = mapTauriOsPlatform(tauriPlatform());
      if (mapped != null) {
        cachedHostOs = mapped;
        return cachedHostOs;
      }
    } catch {
      // internals 未注入時などは navigator へ
    }
  }
  cachedHostOs = detectFromNavigator();
  return cachedHostOs;
}
