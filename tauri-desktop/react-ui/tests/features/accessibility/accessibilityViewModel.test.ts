import { describe, expect, it } from "vitest";
import { toAccessibilityViewModel } from "@/features/accessibility/accessibilityViewModel";
import type i18n from "@/locales/i18n";

const t = ((key: string) => key) as typeof i18n.t;

describe("toAccessibilityViewModel", () => {
  it("loading", () => {
    const view = toAccessibilityViewModel(
      {
        ready: false,
        permission: null,
        initError: null,
      },
      t
    );
    expect(view.kind).toBe("loading");
    expect(view.headline).toBe("accessibility.headlineLoading");
    expect(view.showPromptActions).toBe(false);
  });

  it("granted", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: "granted",
        initError: null,
      },
      t
    );
    expect(view.kind).toBe("granted");
    expect(view.headline).toBe("accessibility.headlineGranted");
    expect(view.showPromptActions).toBe(false);
  });

  it("denied prompts for settings", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: "denied",
        initError: null,
      },
      t
    );
    expect(view.kind).toBe("denied");
    expect(view.showPromptActions).toBe(true);
    expect(view.description).toBe("accessibility.descriptionDenied");
  });

  it("unsupported", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: "unsupported",
        initError: null,
      },
      t
    );
    expect(view.kind).toBe("unsupported");
    expect(view.showPromptActions).toBe(false);
  });

  it("error", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: null,
        initError: "boom",
      },
      t
    );
    expect(view.kind).toBe("error");
    expect(view.headline).toBe("accessibility.headlineError");
    expect(view.description).toBe("boom");
  });
});
