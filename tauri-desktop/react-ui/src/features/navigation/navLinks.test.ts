import { describe, expect, it } from "vitest";

import { getNavLinkByPath, navLinks } from "./navLinks";

describe("getNavLinkByPath", () => {
  it("完全一致のパスを返す", () => {
    expect(getNavLinkByPath("/")?.label).toBe("ホーム");
    expect(getNavLinkByPath("/accessibility")?.label).toBe("アクセシビリティ");
    expect(getNavLinkByPath("/rules")?.label).toBe("ルール");
    expect(getNavLinkByPath("/diagnostics")?.label).toBe("診断");
    expect(getNavLinkByPath("/settings")?.label).toBe("設定");
    expect(getNavLinkByPath("/about")?.label).toBe("アプリについて");
    expect(getNavLinkByPath("/update")?.label).toBe("アップデート");
    expect(getNavLinkByPath("/lab")?.label).toBe("開発確認");
  });

  it("ネストされたパスは最長プレフィックスでマッチする", () => {
    expect(getNavLinkByPath("/settings/theme")?.to).toBe("/settings");
  });

  it("未知のパスは undefined", () => {
    expect(getNavLinkByPath("/unknown")).toBeUndefined();
  });

  it("navLinks はホーム・アクセシビリティ・ルール・診断・設定・アプリについて・アップデート・開発確認を含む", () => {
    expect(navLinks.map((link) => link.to)).toEqual([
      "/",
      "/accessibility",
      "/rules",
      "/diagnostics",
      "/settings",
      "/about",
      "/update",
      "/lab",
    ]);
  });
});
