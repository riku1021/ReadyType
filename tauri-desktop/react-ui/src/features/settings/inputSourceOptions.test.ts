import { describe, expect, it } from "vitest";

import { buildInputSourceOptions } from "./inputSourceOptions";

describe("buildInputSourceOptions", () => {
  it("keeps a removed configured source visible and unavailable", () => {
    expect(
      buildInputSourceOptions([{ id: "en", name: "ABC", available: true }], "removed")
    ).toContainEqual({
      value: "removed",
      label: "removed（利用不可）",
      available: false,
    });
  });

  it("deduplicates ids and sorts by display name", () => {
    const options = buildInputSourceOptions(
      [
        { id: "b", name: "Bravo", available: true },
        { id: "a", name: "Alpha", available: true },
        { id: "a", name: "Alpha Duplicate", available: true },
      ],
      "a"
    );
    expect(options.map((option) => option.value)).toEqual(["a", "b"]);
  });
});
