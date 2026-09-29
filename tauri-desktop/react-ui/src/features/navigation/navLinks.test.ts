import { describe, expect, it } from "vitest";

import { getNavLinkByPath, navLinks } from "./navLinks";

describe("getNavLinkByPath", () => {
  it("完全一致のパスを返す", () => {
    expect(getNavLinkByPath("/")?.labelKey).toBe("nav.home");
    expect(getNavLinkByPath("/accessibility")?.labelKey).toBe("nav.accessibility");
    expect(getNavLinkByPath("/apps")?.labelKey).toBe("nav.apps");
    expect(getNavLinkByPath("/diagnostics")?.labelKey).toBe("nav.diagnostics");
    expect(getNavLinkByPath("/settings")?.labelKey).toBe("nav.settings");
    expect(getNavLinkByPath("/about")?.labelKey).toBe("nav.about");
    expect(getNavLinkByPath("/update")?.labelKey).toBe("nav.update");
    expect(getNavLinkByPath("/lab")?.labelKey).toBe("nav.lab");
  });

  it("ネストされたパスは最長プレフィックスでマッチする", () => {
    expect(getNavLinkByPath("/settings/theme")?.to).toBe("/settings");
    expect(getNavLinkByPath("/apps/com.apple.Terminal")?.to).toBe("/apps");
  });

  it("未知のパスは undefined", () => {
    expect(getNavLinkByPath("/unknown")).toBeUndefined();
  });

  it("navLinks はホーム・アクセシビリティ・アプリごとの設定・診断・設定・アプリについて・アップデート・開発確認を含む", () => {
    expect(navLinks.map((link) => link.to)).toEqual([
      "/",
      "/accessibility",
      "/apps",
      "/diagnostics",
      "/settings",
      "/about",
      "/update",
      "/lab",
    ]);
    expect(navLinks.map((link) => link.labelKey)).toEqual([
      "nav.home",
      "nav.accessibility",
      "nav.apps",
      "nav.diagnostics",
      "nav.settings",
      "nav.about",
      "nav.update",
      "nav.lab",
    ]);
  });
});
