import { describe, expect, it } from "vitest";

import { toAccessibilityViewModel } from "./accessibilityViewModel";

describe("toAccessibilityViewModel", () => {
  it("loading", () => {
    const view = toAccessibilityViewModel({
      ready: false,
      permission: null,
      initError: null,
    });
    expect(view.kind).toBe("loading");
    expect(view.showPromptActions).toBe(false);
  });

  it("granted", () => {
    const view = toAccessibilityViewModel({
      ready: true,
      permission: "granted",
      initError: null,
    });
    expect(view.kind).toBe("granted");
    expect(view.headline).toContain("許可されています");
    expect(view.showPromptActions).toBe(false);
  });

  it("denied prompts for settings", () => {
    const view = toAccessibilityViewModel({
      ready: true,
      permission: "denied",
      initError: null,
    });
    expect(view.kind).toBe("denied");
    expect(view.showPromptActions).toBe(true);
    expect(view.description).toContain("アクセシビリティ");
  });

  it("unsupported", () => {
    const view = toAccessibilityViewModel({
      ready: true,
      permission: "unsupported",
      initError: null,
    });
    expect(view.kind).toBe("unsupported");
    expect(view.showPromptActions).toBe(false);
  });

  it("error", () => {
    const view = toAccessibilityViewModel({
      ready: true,
      permission: null,
      initError: "boom",
    });
    expect(view.kind).toBe("error");
    expect(view.description).toBe("boom");
  });
});
