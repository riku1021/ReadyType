export const UPDATE_AUTO_CHECK_KEY = "readytype.update.autoCheck";
export const UPDATE_AUTO_DOWNLOAD_KEY = "readytype.update.autoDownload";
export const UPDATE_LAST_CHECKED_KEY = "readytype.update.lastCheckedAt";

export type UpdatePreferenceStore = {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
};

export type UpdatePreferences = {
  autoCheck: boolean;
  autoDownload: boolean;
  lastCheckedAt: string | null;
};

function readFlag(store: UpdatePreferenceStore, key: string, fallback: boolean): boolean {
  const raw = store.getItem(key);
  if (raw === "true") {
    return true;
  }
  if (raw === "false") {
    return false;
  }
  return fallback;
}

export function readUpdatePreferences(store: UpdatePreferenceStore): UpdatePreferences {
  const lastCheckedAt = store.getItem(UPDATE_LAST_CHECKED_KEY);
  return {
    autoCheck: readFlag(store, UPDATE_AUTO_CHECK_KEY, true),
    autoDownload: readFlag(store, UPDATE_AUTO_DOWNLOAD_KEY, true),
    lastCheckedAt: lastCheckedAt && lastCheckedAt.length > 0 ? lastCheckedAt : null,
  };
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
