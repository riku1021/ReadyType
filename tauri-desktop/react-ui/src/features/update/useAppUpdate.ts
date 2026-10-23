import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useSyncExternalStore } from "react";

import { createAppUpdateController } from "./appUpdateController";
import { tauriUpdateClient } from "./tauriUpdateClient";
import { UPDATE_AUTO_CHECK_KEY, type UpdatePreferenceStore } from "./updatePreferences";

function preferenceStorage(): UpdatePreferenceStore {
  try {
    if (typeof localStorage !== "undefined" && typeof localStorage.getItem === "function") {
      localStorage.getItem(UPDATE_AUTO_CHECK_KEY);
      return localStorage;
    }
  } catch {
    // Node の localStorage はファイル未指定だと getItem が例外を投げる。
  }
  const memory = new Map<string, string>();
  return {
    getItem: (key) => memory.get(key) ?? null,
    setItem: (key, value) => {
      memory.set(key, value);
    },
  };
}

const controller = createAppUpdateController({
  client: tauriUpdateClient,
  storage: preferenceStorage(),
  canCheck: isTauri,
  now: () => new Date(),
});

export function useAppUpdate() {
  const snapshot = useSyncExternalStore(controller.subscribe, controller.getSnapshot);
  return {
    ...snapshot,
    setAutoCheck: controller.setAutoCheck,
    setAutoDownload: controller.setAutoDownload,
    checkNow: controller.checkNow,
    install: controller.install,
  };
}

/** メイン窓の起動時に、自動確認がオンなら一度だけ確認する。 */
export function useAppUpdateBootstrap() {
  useEffect(() => {
    controller.bootstrap();
  }, []);
}
