import { describe, expect, it } from "vitest";

import {
  parseLastCheckedAt,
  readUpdatePreferences,
  UPDATE_AUTO_CHECK_KEY,
  UPDATE_AUTO_DOWNLOAD_KEY,
  UPDATE_LAST_CHECKED_KEY,
} from "@/features/update/updatePreferences";

import { memoryStore } from "./memoryStore";

describe("readUpdatePreferences", () => {
  it("defaults both switches on and has no last-checked time", () => {
    expect(readUpdatePreferences(memoryStore())).toEqual({
      autoCheck: true,
      autoDownload: true,
      lastCheckedAt: null,
    });
  });

  it("reads persisted flags", () => {
    const store = memoryStore({
      [UPDATE_AUTO_CHECK_KEY]: "false",
      [UPDATE_AUTO_DOWNLOAD_KEY]: "true",
      [UPDATE_LAST_CHECKED_KEY]: "2026-10-23T03:00:00.000Z",
    });
    expect(readUpdatePreferences(store)).toEqual({
      autoCheck: false,
      autoDownload: true,
      lastCheckedAt: "2026-10-23T03:00:00.000Z",
    });
  });
});

describe("parseLastCheckedAt", () => {
  it("returns null for empty or invalid values", () => {
    expect(parseLastCheckedAt(null)).toBeNull();
    expect(parseLastCheckedAt("")).toBeNull();
    expect(parseLastCheckedAt("not-a-date")).toBeNull();
  });

  it("parses an ISO timestamp", () => {
    expect(parseLastCheckedAt("2026-10-23T03:00:00.000Z")?.toISOString()).toBe(
      "2026-10-23T03:00:00.000Z"
    );
  });
});
