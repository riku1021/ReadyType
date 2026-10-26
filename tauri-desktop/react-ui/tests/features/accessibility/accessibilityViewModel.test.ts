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
    expect(view.noticeTone).toBe("info");
    expect(view.showOpenSettings).toBe(false);
    expect(view.requestBeforeOpenSettings).toBe(false);
    expect(view.showPrivacy).toBe(false);
    expect(view.detailMessage).toBe("accessibility.descriptionLoading");
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
    expect(view.noticeTone).toBe("info");
    expect(view.showGrantedBadge).toBe(true);
    expect(view.showRequiredStatus).toBe(false);
    expect(view.showOpenSettings).toBe(true);
    expect(view.requestBeforeOpenSettings).toBe(false);
    expect(view.openSettingsVariant).toBe("neutral");
    expect(view.showPrivacy).toBe(true);
    expect(view.detailMessage).toBeNull();
  });

  it("Windows reports that no separate accessibility permission is needed", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: "granted",
        initError: null,
      },
      t,
      "windows"
    );

    expect(view.showNotRequiredBadge).toBe(true);
    expect(view.showGrantedBadge).toBe(false);
    expect(view.showOpenSettings).toBe(false);
    expect(view.detailMessage).toBeNull();
  });

  it("denied prompts for settings after TCC registration", () => {
    const view = toAccessibilityViewModel(
      {
        ready: true,
        permission: "denied",
        initError: null,
      },
      t
    );
    expect(view.kind).toBe("denied");
    expect(view.noticeTone).toBe("error");
    expect(view.showRequiredStatus).toBe(true);
    expect(view.showGrantedBadge).toBe(false);
    expect(view.showOpenSettings).toBe(true);
    expect(view.requestBeforeOpenSettings).toBe(true);
    expect(view.openSettingsVariant).toBe("primary");
    expect(view.showPrivacy).toBe(true);
    expect(view.detailMessage).toBe("accessibility.deniedGuidance");
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
    expect(view.noticeTone).toBe("info");
    expect(view.showOpenSettings).toBe(false);
    expect(view.detailMessage).toBe("accessibility.descriptionUnsupported");
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
    expect(view.noticeTone).toBe("error");
    expect(view.detailMessage).toBe("boom");
    expect(view.showOpenSettings).toBe(false);
  });
});
