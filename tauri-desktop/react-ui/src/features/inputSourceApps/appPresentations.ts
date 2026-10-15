import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { type AppPresentationDto, getAppPresentations } from "@/features/inputSource";

/** Matches rust-native `ICON_STYLE_CHANGED_EVENT`. */
export const APP_ICON_STYLE_CHANGED_EVENT = "app-presentations://icon-style-changed";

/** UI-ready presentation (asset URL already resolved). */
export type AppPresentationView = {
  iconSrc: string | null;
  displayName: string | null;
};

/** Process-lifetime cache of UI-ready presentations. */
const sessionCache = new Map<string, AppPresentationView>();

const cacheListeners = new Set<() => void>();

let styleInvalidationStarted = false;

/** Convert a native icon path (or legacy data URL) into an `<img src>` value. */
export function resolveAppIconSrc(
  icon: string | null | undefined,
  revision?: string | null
): string | null {
  if (icon == null || icon.length === 0) {
    return null;
  }
  if (
    icon.startsWith("data:") ||
    icon.startsWith("blob:") ||
    icon.startsWith("asset:") ||
    icon.startsWith("http://") ||
    icon.startsWith("https://")
  ) {
    return icon;
  }
  const src = convertFileSrc(icon);
  if (revision == null || revision.length === 0) {
    return src;
  }
  const separator = src.includes("?") ? "&" : "?";
  return `${src}${separator}v=${encodeURIComponent(revision)}`;
}

function toUiPresentation(raw: AppPresentationDto): AppPresentationView {
  return {
    iconSrc: resolveAppIconSrc(raw.icon, raw.iconRevision),
    displayName: raw.displayName,
  };
}

export function peekAppPresentations(
  appIds: readonly string[]
): Record<string, AppPresentationView> {
  const out: Record<string, AppPresentationView> = {};
  for (const id of appIds) {
    const cached = sessionCache.get(id);
    if (cached != null) {
      out[id] = cached;
    }
  }
  return out;
}

export function missingAppPresentationIds(appIds: readonly string[]): string[] {
  return [...new Set(appIds.filter((id) => id.length > 0 && !sessionCache.has(id)))];
}

/** Fetch only uncached ids, merge into the session cache, return presentations for `appIds`. */
export async function loadAppPresentations(
  appIds: readonly string[]
): Promise<Record<string, AppPresentationView>> {
  const unique = [...new Set(appIds.filter((id) => id.length > 0))];
  const missing = missingAppPresentationIds(unique);
  if (missing.length === 0) {
    return peekAppPresentations(unique);
  }
  const fetched = await getAppPresentations(missing);
  for (const id of missing) {
    const raw = fetched[id] ?? { icon: null, iconRevision: null, displayName: null };
    sessionCache.set(id, toUiPresentation(raw));
  }
  return peekAppPresentations(unique);
}

/** Subscribe to session-cache clears (style invalidation). */
export function subscribeAppPresentationCache(listener: () => void): () => void {
  cacheListeners.add(listener);
  return () => {
    cacheListeners.delete(listener);
  };
}

/** Clears the module session cache and notifies subscribers to refetch. */
export function clearAppPresentationSessionCache(): void {
  sessionCache.clear();
  for (const listener of cacheListeners) {
    listener();
  }
}

/** Start listening for native icon-style changes (idempotent). */
export function ensureAppPresentationStyleInvalidation(): void {
  if (styleInvalidationStarted || !isTauri()) {
    return;
  }
  styleInvalidationStarted = true;
  void listen(APP_ICON_STYLE_CHANGED_EVENT, () => {
    clearAppPresentationSessionCache();
  });
}

/** Test helper — clears the module session cache without notifying. */
export function resetAppPresentationSessionCacheForTests(): void {
  sessionCache.clear();
  styleInvalidationStarted = false;
  cacheListeners.clear();
}
