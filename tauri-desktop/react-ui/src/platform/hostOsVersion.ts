import { isTauri } from "@tauri-apps/api/core";
import { version as tauriOsVersion } from "@tauri-apps/plugin-os";

import { getHostOs } from "./hostOs";

/** 成功時のみセット。失敗時は `undefined` のまま再取得する。 */
let cachedHostOsVersion: string | null | undefined;

/**
 * ホスト OS のバージョン文字列（例: `27.0.0`）。
 * 取得不能なら `null`。成功した値だけプロセス寿命でキャッシュする。
 */
export function getHostOsVersion(): string | null {
  if (cachedHostOsVersion !== undefined) {
    return cachedHostOsVersion;
  }
  if (!isTauri()) {
    cachedHostOsVersion = null;
    return null;
  }
  try {
    const value = tauriOsVersion().trim();
    if (value.length === 0) {
      return null;
    }
    cachedHostOsVersion = value;
    return cachedHostOsVersion;
  } catch {
    // internals 未注入時など — キャッシュせず次回再試行
    return null;
  }
}

/** `27.0.0` / `27` などからメジャー番号を取る。取れなければ null。 */
export function parseOsMajorVersion(version: string): number | null {
  const match = /^(\d+)/.exec(version.trim());
  if (match == null) {
    return null;
  }
  const major = Number(match[1]);
  return Number.isFinite(major) ? major : null;
}

/** macOS マーケティングバージョンのメジャーが 27 以上か。 */
export function isMacOsDeviceControlPermissionMajor(major: number): boolean {
  return major >= 27;
}

/**
 * macOS 27+ では System Settings の旧「アクセシビリティ」が
 * 「デバイスの制御とデータへのアクセス」に改名されている。
 */
export function usesMacOsDeviceControlPermissionLabel(): boolean {
  if (getHostOs() !== "macOs") {
    return false;
  }
  const version = getHostOsVersion();
  if (version == null) {
    return false;
  }
  const major = parseOsMajorVersion(version);
  return major != null && isMacOsDeviceControlPermissionMajor(major);
}
