import { describe, expect, it } from "vitest";

import { pageSlideBackToListTypes, pageSlideTypes } from "@/features/navigation/pageSlide";
import { pageSlideBackType, pageSlideForwardType } from "@/styles/bases/pageSlide";

describe("pageSlideTypes", () => {
  it("進むときは右から入り、戻るときは左から入る", () => {
    expect(pageSlideTypes("forward", false)).toEqual([pageSlideForwardType]);
    expect(pageSlideTypes("back", false)).toEqual([pageSlideBackType]);
  });

  it("動きを減らす設定ではスライドしない", () => {
    expect(pageSlideTypes("forward", true)).toBe(false);
    expect(pageSlideTypes("back", true)).toBe(false);
  });
});

describe("pageSlideBackToListTypes", () => {
  it("詳細からその一覧へ戻るときは左から入る", () => {
    expect(pageSlideBackToListTypes("/apps/com.apple.Terminal", "/apps", "/apps", false)).toEqual([
      pageSlideBackType,
    ]);
  });

  it("一覧の詳細以外からその一覧へ移るときはスライドしない", () => {
    expect(pageSlideBackToListTypes("/settings", "/apps", "/apps", false)).toBe(false);
    expect(pageSlideBackToListTypes("/apps", "/apps", "/apps", false)).toBe(false);
    expect(pageSlideBackToListTypes("/apps/Cursor", "/general", "/apps", false)).toBe(false);
    expect(pageSlideBackToListTypes("/apps/Cursor/extra", "/apps", "/apps", false)).toBe(false);
    expect(pageSlideBackToListTypes(undefined, "/apps", "/apps", false)).toBe(false);
  });

  it("動きを減らす設定ではスライドしない", () => {
    expect(pageSlideBackToListTypes("/apps/Cursor", "/apps", "/apps", true)).toBe(false);
  });
});
