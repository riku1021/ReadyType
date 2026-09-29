import { describe, expect, it } from "vitest";

import {
  adjustOklchLightness,
  createColorSurface,
  INTERACTION_OKLCH_LIGHTNESS_DELTA,
  mixWithWhite,
  withAlpha,
} from "@/styles/utils/color-utils";

describe("mixWithWhite", () => {
  it("白 100% は白になる", () => {
    expect(mixWithWhite("#28a745", 100)).toBe("#ffffff");
  });

  it("白 0% は元色のまま", () => {
    expect(mixWithWhite("#28a745", 0)).toBe("#28a745");
  });
});

describe("withAlpha", () => {
  it("rgba 文字列を返す", () => {
    expect(withAlpha("#28a745", 0.12)).toBe("rgba(40, 167, 69, 0.12)");
  });
});

describe("createColorSurface", () => {
  it("任意色から main / soft / softBorder を生成する", () => {
    const surface = createColorSurface("#007bff");

    expect(surface.main.value).toBe("#007bff");
    expect(surface.soft.value.base).toBe(mixWithWhite("#007bff", 88));
    expect(surface.soft.value._dark).toBe("rgba(0, 123, 255, 0.12)");
    expect(surface.softBorder.value.base).toBe(mixWithWhite("#007bff", 70));
    expect(surface.softBorder.value._dark).toBe("rgba(0, 123, 255, 0.35)");
  });

  it("status 以外の色でも同じ形で生成できる", () => {
    const brand = createColorSurface("#04ddb3");
    const accent = createColorSurface("#f8313e");

    expect(brand.main.value).toBe("#04ddb3");
    expect(accent.soft.value._dark).toMatch(/^rgba\(/);
    expect(accent.softBorder.value.base).not.toBe(accent.main.value);
  });
});

describe("adjustOklchLightness", () => {
  it("delta 0 は元色のまま", () => {
    expect(adjustOklchLightness("#007bff", 0)).toBe("#007bff");
  });

  it("負の delta で暗くなる（hover 既定）", () => {
    const hovered = adjustOklchLightness("#007bff", INTERACTION_OKLCH_LIGHTNESS_DELTA);
    expect(hovered).not.toBe("#007bff");
    expect(hovered).toMatch(/^#[0-9a-f]{6}$/i);

    const sum = (hex: string) => {
      const value = hex.replace("#", "");
      return (
        Number.parseInt(value.slice(0, 2), 16) +
        Number.parseInt(value.slice(2, 4), 16) +
        Number.parseInt(value.slice(4, 6), 16)
      );
    };
    expect(sum(hovered)).toBeLessThan(sum("#007bff"));
  });

  it("正の delta で明るくなる", () => {
    const lighter = adjustOklchLightness("#007bff", 0.06);
    const sum = (hex: string) => {
      const value = hex.replace("#", "");
      return (
        Number.parseInt(value.slice(0, 2), 16) +
        Number.parseInt(value.slice(2, 4), 16) +
        Number.parseInt(value.slice(4, 6), 16)
      );
    };
    expect(sum(lighter)).toBeGreaterThan(sum("#007bff"));
  });
});
