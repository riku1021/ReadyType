import { describe, expect, it } from "vitest";

import { mapTauriOsPlatform } from "@/platform";

describe("hostOs", () => {
  it("mapTauriOsPlatform は desktop OS を HostOs に写す", () => {
    expect(mapTauriOsPlatform("macos")).toBe("macOs");
    expect(mapTauriOsPlatform("windows")).toBe("windows");
    expect(mapTauriOsPlatform("linux")).toBe("linux");
  });

  it("mapTauriOsPlatform は desktop 外を null にする", () => {
    expect(mapTauriOsPlatform("ios")).toBeNull();
    expect(mapTauriOsPlatform("android")).toBeNull();
    expect(mapTauriOsPlatform("freebsd")).toBeNull();
  });
});
