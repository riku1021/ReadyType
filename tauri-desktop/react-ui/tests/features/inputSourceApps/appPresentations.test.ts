import { convertFileSrc } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import {
  clearAppPresentationSessionCache,
  loadAppPresentations,
  missingAppPresentationIds,
  resetAppPresentationSessionCacheForTests,
  resolveAppIconSrc,
  subscribeAppPresentationCache,
} from "@/features/inputSourceApps/appPresentations";

vi.mock("@tauri-apps/api/core", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/core")>();
  return {
    ...actual,
    convertFileSrc: vi.fn((path: string) => `asset://localhost/${encodeURIComponent(path)}`),
    isTauri: () => false,
  };
});

beforeAll(() => {
  Object.defineProperty(globalThis, "window", {
    value: globalThis,
    configurable: true,
  });
});

describe("resolveAppIconSrc", () => {
  it("passes through data URLs and appends revision on filesystem paths", () => {
    expect(resolveAppIconSrc("data:image/png;base64,AA==")).toBe("data:image/png;base64,AA==");
    expect(resolveAppIconSrc("/tmp/com.example.App.png", "10:default")).toBe(
      `${convertFileSrc("/tmp/com.example.App.png")}?v=${encodeURIComponent("10:default")}`
    );
    expect(resolveAppIconSrc(null)).toBeNull();
  });
});

describe("loadAppPresentations", () => {
  afterEach(() => {
    clearMocks();
    resetAppPresentationSessionCacheForTests();
    vi.clearAllMocks();
  });

  it("fetches only missing ids and reuses the session cache", async () => {
    const calls: string[][] = [];
    mockIPC((command, args) => {
      if (command === "get_app_presentations") {
        const bundleIds = (args as { bundleIds: string[] }).bundleIds;
        calls.push(bundleIds);
        return Object.fromEntries(
          bundleIds.map((id) => [
            id,
            {
              icon: `/cache/${id}.png`,
              iconRevision: `1:${id}`,
              displayName: id.split(".").slice(-1)[0] ?? id,
            },
          ])
        );
      }
      return null;
    });

    const first = await loadAppPresentations(["com.a.App", "com.b.App"]);
    expect(calls).toEqual([["com.a.App", "com.b.App"]]);
    expect(first["com.a.App"]?.iconSrc).toBe(
      `${convertFileSrc("/cache/com.a.App.png")}?v=${encodeURIComponent("1:com.a.App")}`
    );

    const second = await loadAppPresentations(["com.b.App", "com.c.App"]);
    expect(calls).toEqual([["com.a.App", "com.b.App"], ["com.c.App"]]);
    expect(second["com.b.App"]?.displayName).toBe("App");
    expect(second["com.c.App"]?.iconSrc).toBe(
      `${convertFileSrc("/cache/com.c.App.png")}?v=${encodeURIComponent("1:com.c.App")}`
    );
    expect(missingAppPresentationIds(["com.a.App", "com.b.App"])).toEqual([]);
  });

  it("notifies subscribers when the session cache is cleared", () => {
    const seen: number[] = [];
    const unsubscribe = subscribeAppPresentationCache(() => {
      seen.push(1);
    });
    clearAppPresentationSessionCache();
    expect(seen).toEqual([1]);
    unsubscribe();
  });
});
