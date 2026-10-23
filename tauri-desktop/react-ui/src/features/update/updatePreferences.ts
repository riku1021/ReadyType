export const UPDATE_AUTO_CHECK_KEY = "readytype.update.autoCheck";
export const UPDATE_AUTO_DOWNLOAD_KEY = "readytype.update.autoDownload";
export const UPDATE_LAST_CHECKED_KEY = "readytype.update.lastCheckedAt";

export type UpdatePreferenceStore = {
  getItem: (key: string) => string | null;
};

export type LegacyUpdatePreferences = {
  autoCheck: boolean | null;
  autoDownload: boolean | null;
  lastCheckedAtUnixMs: number | null;
};

function flagOrNull(raw: string | null): boolean | null {
  if (raw === "true") {
    return true;
  }
  if (raw === "false") {
    return false;
  }
  return null;
}

/** 旧画面が localStorage に書いた設定。Rust が保存するまで渡す。 */
export function readLegacyUpdatePreferences(
  store: UpdatePreferenceStore
): LegacyUpdatePreferences | null {
  const autoCheck = store.getItem(UPDATE_AUTO_CHECK_KEY);
  const autoDownload = store.getItem(UPDATE_AUTO_DOWNLOAD_KEY);
  const lastCheckedAt = store.getItem(UPDATE_LAST_CHECKED_KEY);
  if (autoCheck === null && autoDownload === null && lastCheckedAt === null) {
    return null;
  }
  return {
    autoCheck: flagOrNull(autoCheck),
    autoDownload: flagOrNull(autoDownload),
    lastCheckedAtUnixMs: parseLastCheckedAt(lastCheckedAt)?.getTime() ?? null,
  };
}

const LEGACY_KEYS = [UPDATE_AUTO_CHECK_KEY, UPDATE_AUTO_DOWNLOAD_KEY, UPDATE_LAST_CHECKED_KEY];

export function clearLegacyUpdatePreferences(): void {
  if (typeof localStorage === "undefined" || typeof localStorage.removeItem !== "function") {
    return;
  }
  for (const key of LEGACY_KEYS) {
    localStorage.removeItem(key);
  }
}

export function parseLastCheckedAt(value: string | null): Date | null {
  if (value === null || value.length === 0) {
    return null;
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return null;
  }
  return date;
}
