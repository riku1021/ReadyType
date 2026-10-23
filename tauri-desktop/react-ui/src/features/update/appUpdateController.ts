import {
  readUpdatePreferences,
  UPDATE_AUTO_CHECK_KEY,
  UPDATE_AUTO_DOWNLOAD_KEY,
  UPDATE_LAST_CHECKED_KEY,
  type UpdatePreferenceStore,
  type UpdatePreferences,
} from "./updatePreferences";

export type UpdateErrorKey =
  | "desktopOnly"
  | "checkFailed"
  | "downloadFailed"
  | "installFailed"
  | "restartFailed";

export type UpdatePhase =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "available"; version: string }
  | { kind: "downloading"; version: string }
  | { kind: "downloaded"; version: string }
  | { kind: "installing"; version: string }
  | { kind: "error"; messageKey: UpdateErrorKey };

export type PendingUpdate = {
  version: string;
  download: () => Promise<void>;
  install: () => Promise<void>;
};

export type UpdateCheckResult = { status: "upToDate" } | ({ status: "available" } & PendingUpdate);

export type UpdateClient = {
  check: () => Promise<UpdateCheckResult>;
  relaunch: () => Promise<void>;
};

export type AppUpdateSnapshot = {
  preferences: UpdatePreferences;
  phase: UpdatePhase;
  installReady: boolean;
};

type CreateAppUpdateControllerOptions = {
  client: UpdateClient;
  storage: UpdatePreferenceStore;
  canCheck: () => boolean;
  now: () => Date;
};

type HeldUpdate = PendingUpdate & { downloaded: boolean };

export type AppUpdateController = {
  subscribe: (listener: () => void) => () => void;
  getSnapshot: () => AppUpdateSnapshot;
  bootstrap: () => void;
  setAutoCheck: (enabled: boolean) => void;
  setAutoDownload: (enabled: boolean) => void;
  checkNow: () => Promise<void>;
  install: () => Promise<void>;
};

export function createAppUpdateController(
  options: CreateAppUpdateControllerOptions
): AppUpdateController {
  const listeners = new Set<() => void>();
  let preferences = readUpdatePreferences(options.storage);
  let phase: UpdatePhase = { kind: "idle" };
  let installReady = false;
  let pending: HeldUpdate | null = null;
  let snapshot: AppUpdateSnapshot = { preferences, phase, installReady };
  let running: Promise<void> | null = null;
  let bootstrapped = false;

  const emit = () => {
    snapshot = { preferences, phase, installReady };
    for (const listener of listeners) {
      listener();
    }
  };

  const publish = (nextPhase: UpdatePhase, nextInstallReady: boolean) => {
    phase = nextPhase;
    installReady = nextInstallReady;
    emit();
  };

  const rememberChecked = () => {
    const lastCheckedAt = options.now().toISOString();
    preferences = { ...preferences, lastCheckedAt };
    options.storage.setItem(UPDATE_LAST_CHECKED_KEY, lastCheckedAt);
  };

  const downloadPending = async (update: HeldUpdate): Promise<boolean> => {
    publish({ kind: "downloading", version: update.version }, false);
    try {
      await update.download();
      update.downloaded = true;
      publish({ kind: "downloaded", version: update.version }, true);
      return true;
    } catch (error) {
      console.error(error);
      publish({ kind: "error", messageKey: "downloadFailed" }, false);
      return false;
    }
  };

  const performCheck = async () => {
    if (!options.canCheck()) {
      publish({ kind: "error", messageKey: "desktopOnly" }, false);
      return;
    }
    publish({ kind: "checking" }, false);
    try {
      const result = await options.client.check();
      rememberChecked();
      if (result.status === "upToDate") {
        pending = null;
        publish({ kind: "upToDate" }, false);
        return;
      }
      const update: HeldUpdate = { ...result, downloaded: false };
      pending = update;
      if (preferences.autoDownload) {
        await downloadPending(update);
        return;
      }
      publish({ kind: "available", version: update.version }, true);
    } catch (error) {
      console.error(error);
      pending = null;
      publish({ kind: "error", messageKey: "checkFailed" }, false);
    }
  };

  const enqueue = (task: () => Promise<void>): Promise<void> => {
    if (running) {
      return running;
    }
    running = task().finally(() => {
      running = null;
    });
    return running;
  };

  return {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    getSnapshot: () => snapshot,
    bootstrap: () => {
      if (bootstrapped) {
        return;
      }
      bootstrapped = true;
      if (!options.canCheck() || !preferences.autoCheck) {
        return;
      }
      void enqueue(performCheck);
    },
    setAutoCheck: (enabled) => {
      preferences = { ...preferences, autoCheck: enabled };
      options.storage.setItem(UPDATE_AUTO_CHECK_KEY, String(enabled));
      emit();
      if (enabled) {
        void enqueue(performCheck);
      }
    },
    setAutoDownload: (enabled) => {
      preferences = { ...preferences, autoDownload: enabled };
      options.storage.setItem(UPDATE_AUTO_DOWNLOAD_KEY, String(enabled));
      emit();
      const update = pending;
      if (enabled && phase.kind === "available" && update && !update.downloaded) {
        void enqueue(async () => {
          await downloadPending(update);
        });
      }
    },
    checkNow: () => enqueue(performCheck),
    install: () =>
      enqueue(async () => {
        const update = pending;
        if (!update) {
          return;
        }
        if (!update.downloaded) {
          const downloaded = await downloadPending(update);
          if (!downloaded) {
            return;
          }
        }
        publish({ kind: "installing", version: update.version }, false);
        try {
          await update.install();
        } catch (error) {
          console.error(error);
          publish({ kind: "error", messageKey: "installFailed" }, update.downloaded);
          return;
        }
        try {
          await options.client.relaunch();
        } catch (error) {
          console.error(error);
          publish({ kind: "error", messageKey: "restartFailed" }, false);
        }
      }),
  };
}
