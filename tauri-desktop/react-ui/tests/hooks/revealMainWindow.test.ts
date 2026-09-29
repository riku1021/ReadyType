import { describe, expect, it } from "vitest";
import { shouldRevealMainWindow } from "@/hooks/revealMainWindow";

describe("shouldRevealMainWindow", () => {
  it("shows the window for a manual launch", () => {
    expect(shouldRevealMainWindow(["ReadyType"])).toBe(true);
  });

  it("keeps the window hidden for login launch", () => {
    expect(shouldRevealMainWindow(["ReadyType", "--autostart"])).toBe(false);
  });
});
