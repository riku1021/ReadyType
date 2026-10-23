import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useSyncExternalStore } from "react";

import {
  APP_UPDATE_EVENT,
  type AppUpdateClientAckDto,
  type AppUpdateSnapshot,
  type AppUpdateStateDto,
  initialAppUpdateSnapshot,
  snapshotFromDto,
} from "./types";
import {
  clearLegacyUpdatePreferences,
  readLegacyUpdatePreferences,
  type UpdatePreferenceStore,
} from "./updatePreferences";

type Listener = () => void;

const listeners = new Set<Listener>();
let snapshot: AppUpdateSnapshot = initialAppUpdateSnapshot();

function emit(next: AppUpdateSnapshot) {
  snapshot = next;
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: Listener) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function getSnapshot() {
  return snapshot;
}

function browserStorage(): UpdatePreferenceStore | null {
  try {
    if (typeof localStorage !== "undefined" && typeof localStorage.getItem === "function") {
      localStorage.getItem("readytype.update.autoCheck");
      return localStorage;
    }
  } catch {
    // Node の localStorage はファイル未指定だと getItem が例外を投げる。
  }
  return null;
}

async function invokeUpdate(
  command:
    | "set_app_update_auto_check"
    | "set_app_update_auto_download"
    | "check_app_update_now"
    | "install_app_update",
  args?: { enabled: boolean }
) {
  const dto = await invoke<AppUpdateStateDto>(command, args);
  emit(snapshotFromDto(dto));
}

export function useAppUpdate() {
  const current = useSyncExternalStore(subscribe, getSnapshot);
  return {
    ...current,
    setAutoCheck: (enabled: boolean) => {
      if (!isTauri()) {
        emit({
          ...current,
          preferences: { ...current.preferences, autoCheck: enabled },
        });
        return;
      }
      void invokeUpdate("set_app_update_auto_check", { enabled }).catch((error) => {
        console.error(error);
      });
    },
    setAutoDownload: (enabled: boolean) => {
      if (!isTauri()) {
        emit({
          ...current,
          preferences: { ...current.preferences, autoDownload: enabled },
        });
        return;
      }
      void invokeUpdate("set_app_update_auto_download", { enabled }).catch((error) => {
        console.error(error);
      });
    },
    checkNow: () => {
      if (!isTauri()) {
        emit({
          ...getSnapshot(),
          phase: { kind: "error", messageKey: "desktopOnly" },
          installReady: false,
        });
        return Promise.resolve();
      }
      return invokeUpdate("check_app_update_now").catch((error) => {
        console.error(error);
      });
    },
    install: () => {
      if (!isTauri()) {
        return Promise.resolve();
      }
      return invokeUpdate("install_app_update").catch((error) => {
        console.error(error);
      });
    },
  };
}

/** メイン窓の起動時に、旧設定を渡してから Rust の確認スケジューラを進める。 */
export function useAppUpdateBootstrap() {
  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    const start = async () => {
      let generation = 0;
      const stop = await listen<AppUpdateStateDto>(APP_UPDATE_EVENT, (event) => {
        generation += 1;
        emit(snapshotFromDto(event.payload));
      });
      if (cancelled) {
        stop();
        return;
      }
      unlisten = stop;
      const generationAtAck = generation;
      const storage = browserStorage();
      const legacy = storage ? readLegacyUpdatePreferences(storage) : null;
      const dto = await invoke<AppUpdateClientAckDto>("ack_app_update_client", { legacy });
      if (dto.clearLegacy) {
        clearLegacyUpdatePreferences();
      }
      if (!cancelled && generation === generationAtAck) {
        emit(snapshotFromDto(dto));
      }
    };

    void start().catch((error) => {
      console.error(error);
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
