import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

import type { UpdateClient } from "./appUpdateController";

export const tauriUpdateClient: UpdateClient = {
  async check() {
    const update = await check();
    if (!update) {
      return { status: "upToDate" };
    }
    return {
      status: "available",
      version: update.version,
      download: () => update.download(),
      install: () => update.install(),
    };
  },
  relaunch: () => relaunch(),
};
