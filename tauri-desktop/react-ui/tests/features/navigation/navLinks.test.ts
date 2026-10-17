import { describe, expect, it } from "vitest";

import { getNavLinkByPath, navLinks } from "@/features/navigation/navLinks";

describe("getNavLinkByPath", () => {
  it("完全一致のパスを返す", () => {
    expect(getNavLinkByPath("/accessibility")?.labelKey).toBe("nav.accessibility");
    expect(getNavLinkByPath("/apps")?.labelKey).toBe("nav.apps");
    expect(getNavLinkByPath("/general")?.labelKey).toBe("nav.general");
    expect(getNavLinkByPath("/about")?.labelKey).toBe("nav.about");
    expect(getNavLinkByPath("/update")?.labelKey).toBe("nav.update");
    expect(getNavLinkByPath("/lab")?.labelKey).toBe("nav.lab");
  });

  it("ネストされたパスは最長プレフィックスでマッチする", () => {
    expect(getNavLinkByPath("/general/theme")?.to).toBe("/general");
    expect(getNavLinkByPath("/apps/com.apple.Terminal")?.to).toBe("/apps");
  });

  it("未知のパスは undefined", () => {
    expect(getNavLinkByPath("/unknown")).toBeUndefined();
    expect(getNavLinkByPath("/")).toBeUndefined();
  });

  it("navLinks はアクセシビリティ・アプリごとの設定・一般・アプリについて・アップデート・開発確認を含む", () => {
    expect(navLinks.map((link) => link.to)).toEqual([
      "/accessibility",
      "/apps",
      "/general",
      "/about",
      "/update",
      "/lab",
    ]);
    expect(navLinks.map((link) => link.labelKey)).toEqual([
      "nav.accessibility",
      "nav.apps",
      "nav.general",
      "nav.about",
      "nav.update",
      "nav.lab",
    ]);
  });
});
