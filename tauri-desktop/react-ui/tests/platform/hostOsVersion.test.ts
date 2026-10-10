import { describe, expect, it } from "vitest";

import { isMacOsDeviceControlPermissionMajor, parseOsMajorVersion } from "@/platform";

describe("hostOsVersion", () => {
  it("parseOsMajorVersion は先頭のメジャー番号を返す", () => {
    expect(parseOsMajorVersion("27.0.0")).toBe(27);
    expect(parseOsMajorVersion("15.1")).toBe(15);
    expect(parseOsMajorVersion(" 26 ")).toBe(26);
  });

  it("parseOsMajorVersion は不正値を null にする", () => {
    expect(parseOsMajorVersion("")).toBeNull();
    expect(parseOsMajorVersion("abc")).toBeNull();
  });

  it("isMacOsDeviceControlPermissionMajor は 27 以上で true", () => {
    expect(isMacOsDeviceControlPermissionMajor(26)).toBe(false);
    expect(isMacOsDeviceControlPermissionMajor(27)).toBe(true);
    expect(isMacOsDeviceControlPermissionMajor(28)).toBe(true);
  });
});
