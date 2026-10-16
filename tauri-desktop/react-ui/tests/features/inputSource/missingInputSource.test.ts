import { describe, expect, it } from "vitest";
import {
  anyAppHasMissingInputSource,
  appHasMissingInputSource,
  collectConfiguredInputSourceIds,
  isSourceMissing,
} from "@/features/inputSource/missingInputSource";
import type { InputSourceDto, RuleDto, RuleTarget } from "@/features/inputSource/types";

const sources: InputSourceDto[] = [
  { id: "abc", name: "ABC", languageCode: "en", available: true },
  { id: "kana", name: "Kana", languageCode: "ja", available: true },
  { id: "disabled", name: "Disabled", languageCode: null, available: false },
];

function rule(id: string, appId: string, target: RuleTarget): RuleDto {
  return {
    id,
    name: id,
    enabled: true,
    platform: "macOs",
    appId,
    priority: 0,
    conditions: [],
    target,
  };
}

describe("isSourceMissing", () => {
  it("treats empty id as not missing", () => {
    expect(isSourceMissing("", sources)).toBe(false);
  });

  it("is false for an available source", () => {
    expect(isSourceMissing("abc", sources)).toBe(false);
  });

  it("is true when the id is not in the list", () => {
    expect(isSourceMissing("removed", sources)).toBe(true);
  });

  it("is true when the source is unavailable", () => {
    expect(isSourceMissing("disabled", sources)).toBe(true);
  });
});

describe("collectConfiguredInputSourceIds", () => {
  it("returns unique non-empty ids from rule targets and preferences", () => {
    const rules = [
      rule("r1", "app.a", { kind: "exact", inputSourceId: "abc" }),
      rule("r2", "app.a", { kind: "preferredLanguage", language: "ja" }),
      rule("r3", "app.b", { kind: "preferredLanguage", language: "en" }),
      rule("r4", "app.b", { kind: "exact", inputSourceId: "abc" }),
    ];
    const ids = collectConfiguredInputSourceIds(rules, { ja: "kana", en: null, zh: "pinyin" });
    expect([...ids].sort()).toEqual(["abc", "kana", "pinyin"]);
  });

  it("drops empty ids (unset default language)", () => {
    const rules = [rule("r1", "app.a", { kind: "preferredLanguage", language: "ja" })];
    expect(collectConfiguredInputSourceIds(rules, {})).toEqual([]);
  });
});

describe("appHasMissingInputSource", () => {
  const rules = [
    rule("r1", "app.ok", { kind: "exact", inputSourceId: "abc" }),
    rule("r2", "app.missing", { kind: "exact", inputSourceId: "removed" }),
    rule("r3", "app.pref", { kind: "preferredLanguage", language: "ja" }),
    rule("r4", "app.unset", { kind: "preferredLanguage", language: "zh" }),
  ];
  const preferences = { ja: "disabled", zh: null };

  it("is false when every target of the app is available", () => {
    expect(appHasMissingInputSource("app.ok", rules, preferences, sources)).toBe(false);
  });

  it("is true for an exact target missing from the list", () => {
    expect(appHasMissingInputSource("app.missing", rules, preferences, sources)).toBe(true);
  });

  it("resolves preferredLanguage through preferences", () => {
    expect(appHasMissingInputSource("app.pref", rules, preferences, sources)).toBe(true);
  });

  it("does not treat an unset preferred language as missing", () => {
    expect(appHasMissingInputSource("app.unset", rules, preferences, sources)).toBe(false);
  });

  it("is false for an app without rules", () => {
    expect(appHasMissingInputSource("app.none", rules, preferences, sources)).toBe(false);
  });
});

describe("anyAppHasMissingInputSource", () => {
  it("is true when any app has a missing source", () => {
    const rules = [
      rule("r1", "app.ok", { kind: "exact", inputSourceId: "abc" }),
      rule("r2", "app.missing", { kind: "exact", inputSourceId: "removed" }),
    ];
    expect(anyAppHasMissingInputSource(rules, {}, sources)).toBe(true);
  });

  it("is false when all sources are available", () => {
    const rules = [rule("r1", "app.ok", { kind: "exact", inputSourceId: "abc" })];
    expect(anyAppHasMissingInputSource(rules, {}, sources)).toBe(false);
  });

  it("is false without rules", () => {
    expect(anyAppHasMissingInputSource([], {}, sources)).toBe(false);
  });
});
