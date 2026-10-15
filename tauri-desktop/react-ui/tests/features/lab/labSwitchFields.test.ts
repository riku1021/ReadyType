import { describe, expect, it } from "vitest";

import { LAB_SWITCH_FIELDS } from "@/features/lab/labSwitchFields";

describe("LAB_SWITCH_FIELDS", () => {
  it("matches Rust lab identifiers and ja/en alternation (AC-1)", () => {
    expect(LAB_SWITCH_FIELDS).toHaveLength(6);
    for (const [index, field] of LAB_SWITCH_FIELDS.entries()) {
      const ordinal = index + 1;
      expect(field.id).toBe(`readytype-lab-switch-${String(ordinal).padStart(2, "0")}`);
      expect(field.language).toBe(ordinal % 2 === 1 ? "ja" : "en");
    }
  });
});
