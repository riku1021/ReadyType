import { describe, expect, it, vi } from "vitest";

import {
  createAppUpdateController,
  type PendingUpdate,
  type UpdateClient,
} from "@/features/update/appUpdateController";
import {
  UPDATE_AUTO_CHECK_KEY,
  UPDATE_AUTO_DOWNLOAD_KEY,
  UPDATE_LAST_CHECKED_KEY,
} from "@/features/update/updatePreferences";

import { memoryStore } from "./memoryStore";

const now = new Date("2026-10-23T03:00:00.000Z");

function pendingUpdate(version = "0.2.0"): PendingUpdate & {
  downloads: number;
  installs: number;
} {
  const update = {
    version,
    downloads: 0,
    installs: 0,
    download: vi.fn(async () => {
      update.downloads += 1;
    }),
    install: vi.fn(async () => {
      update.installs += 1;
    }),
  };
  return update;
}

function createHarness(options?: {
  storage?: ReturnType<typeof memoryStore>;
  canCheck?: boolean;
  check?: UpdateClient["check"];
}) {
  const storage = options?.storage ?? memoryStore();
  const relaunch = vi.fn(async () => undefined);
  const check = options?.check ?? (async () => ({ status: "upToDate" as const }));
  const controller = createAppUpdateController({
    client: { check, relaunch },
    storage,
    canCheck: () => options?.canCheck ?? true,
    now: () => now,
  });
  return { controller, storage, relaunch };
}

describe("createAppUpdateController", () => {
  it("checks and downloads on startup when both switches are on", async () => {
    const update = pendingUpdate();
    const { controller, storage } = createHarness({
      check: async () => ({ status: "available", ...update }),
    });

    controller.bootstrap();
    await vi.waitFor(() => {
      expect(controller.getSnapshot().phase).toEqual({ kind: "downloaded", version: "0.2.0" });
    });

    expect(update.downloads).toBe(1);
    expect(update.installs).toBe(0);
    expect(storage.data.get(UPDATE_LAST_CHECKED_KEY)).toBe(now.toISOString());
    expect(controller.getSnapshot().installReady).toBe(true);
  });

  it("does not check on startup when automatic checks are off", async () => {
    const check = vi.fn(async () => ({ status: "upToDate" as const }));
    const { controller } = createHarness({
      storage: memoryStore({ [UPDATE_AUTO_CHECK_KEY]: "false" }),
      check,
    });

    controller.bootstrap();
    controller.bootstrap();

    expect(check).not.toHaveBeenCalled();
    expect(controller.getSnapshot().phase).toEqual({ kind: "idle" });
  });

  it("checks without downloading when automatic download is off", async () => {
    const update = pendingUpdate();
    const { controller } = createHarness({
      storage: memoryStore({ [UPDATE_AUTO_DOWNLOAD_KEY]: "false" }),
      check: async () => ({ status: "available", ...update }),
    });

    await controller.checkNow();

    expect(controller.getSnapshot().phase).toEqual({ kind: "available", version: "0.2.0" });
    expect(update.downloads).toBe(0);
    expect(controller.getSnapshot().installReady).toBe(true);
  });

  it("records an up-to-date check", async () => {
    const { controller, storage } = createHarness();

    await controller.checkNow();

    expect(controller.getSnapshot().phase).toEqual({ kind: "upToDate" });
    expect(storage.data.get(UPDATE_LAST_CHECKED_KEY)).toBe(now.toISOString());
  });

  it("keeps the previous check time when the check fails", async () => {
    const storage = memoryStore({ [UPDATE_LAST_CHECKED_KEY]: "2026-01-01T00:00:00.000Z" });
    const { controller } = createHarness({
      storage,
      check: async () => {
        throw new Error("offline");
      },
    });

    await controller.checkNow();

    expect(controller.getSnapshot().phase).toEqual({ kind: "error", messageKey: "checkFailed" });
    expect(storage.data.get(UPDATE_LAST_CHECKED_KEY)).toBe("2026-01-01T00:00:00.000Z");
  });

  it("reports that checks are desktop-only outside the app", async () => {
    const check = vi.fn(async () => ({ status: "upToDate" as const }));
    const { controller } = createHarness({ canCheck: false, check });

    await controller.checkNow();

    expect(check).not.toHaveBeenCalled();
    expect(controller.getSnapshot().phase).toEqual({ kind: "error", messageKey: "desktopOnly" });
  });

  it("downloads once, installs, and relaunches", async () => {
    const update = pendingUpdate();
    const { controller, relaunch } = createHarness({
      storage: memoryStore({ [UPDATE_AUTO_DOWNLOAD_KEY]: "false" }),
      check: async () => ({ status: "available", ...update }),
    });

    await controller.checkNow();
    await controller.install();

    expect(update.downloads).toBe(1);
    expect(update.installs).toBe(1);
    expect(relaunch).toHaveBeenCalledOnce();
  });

  it("starts a download when automatic download is turned on for an available update", async () => {
    const update = pendingUpdate();
    const { controller } = createHarness({
      storage: memoryStore({ [UPDATE_AUTO_DOWNLOAD_KEY]: "false" }),
      check: async () => ({ status: "available", ...update }),
    });

    await controller.checkNow();
    controller.setAutoDownload(true);
    await vi.waitFor(() => {
      expect(controller.getSnapshot().phase).toEqual({ kind: "downloaded", version: "0.2.0" });
    });

    expect(update.downloads).toBe(1);
    expect(controller.getSnapshot().preferences.autoDownload).toBe(true);
  });
});
