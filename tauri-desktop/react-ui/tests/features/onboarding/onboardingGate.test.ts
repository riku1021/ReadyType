import { describe, expect, it } from "vitest";

import { resolveOnboardingNavigation } from "@/features/onboarding";

describe("resolveOnboardingNavigation", () => {
  it("does nothing until ready with settings", () => {
    expect(
      resolveOnboardingNavigation({
        ready: false,
        onboardingCompleted: false,
        pathname: "/apps",
      })
    ).toBeNull();
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: null,
        pathname: "/apps",
      })
    ).toBeNull();
  });

  it("sends incomplete users to onboarding", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: false,
        pathname: "/apps",
      })
    ).toBe("/onboarding");
  });

  it("keeps incomplete users on onboarding", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: false,
        pathname: "/onboarding",
      })
    ).toBeNull();
  });

  it("sends completed users away from onboarding", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: true,
        pathname: "/onboarding/",
      })
    ).toBe("/apps");
  });

  it("keeps completed users on onboarding in preview mode", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: true,
        pathname: "/onboarding",
        preview: true,
      })
    ).toBeNull();
  });

  it("leaves completed users on normal routes", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: true,
        pathname: "/apps",
      })
    ).toBeNull();
  });

  it("does not redirect quick-set routes even when onboarding is incomplete", () => {
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: false,
        pathname: "/quick-set",
      })
    ).toBeNull();
    expect(
      resolveOnboardingNavigation({
        ready: true,
        onboardingCompleted: false,
        pathname: "/quick-set/",
      })
    ).toBeNull();
  });
});
