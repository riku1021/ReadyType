import { describe, expect, it } from "vitest";

import {
  parseLastCheckedAt,
  readLegacyUpdatePreferences,
  UPDATE_AUTO_CHECK_KEY,
  UPDATE_LAST_CHECKED_KEY,
} from "@/features/update/updatePreferences";

import { memoryStore } from "./memoryStore";

describe("readLegacyUpdatePreferences", () => {
  it("returns null when nothing was stored", () => {
    expect(readLegacyUpdatePreferences(memoryStore())).toBeNull();
  });

  it("keeps an explicit off switch and the last successful check", () => {
    const store = memoryStore({
      [UPDATE_AUTO_CHECK_KEY]: "false",
      [UPDATE_LAST_CHECKED_KEY]: "2026-10-23T03:00:00.000Z",
    });
    expect(readLegacyUpdatePreferences(store)).toEqual({
      autoCheck: false,
      autoDownload: null,
      lastCheckedAtUnixMs: Date.parse("2026-10-23T03:00:00.000Z"),
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
