import { describe, expect, it } from "vitest";

import type { RuleDto } from "@/features/inputSource";

import {
  appDisplayName,
  displayRuleLabel,
  groupRulesByApp,
  resolvedInputSourceId,
  ruleTargetFromInputSourceId,
  summarizeLanguageTarget,
} from "./appRulesViewModel";

function rule(partial: Partial<RuleDto> & Pick<RuleDto, "id" | "appId">): RuleDto {
  return {
    name: partial.name ?? partial.id,
    enabled: partial.enabled ?? true,
    platform: "macOs",
    priority: partial.priority ?? 0,
    conditions: partial.conditions ?? [],
    target: partial.target ?? { kind: "preferredEnglish" },
    ...partial,
  };
}

describe("appRulesViewModel", () => {
  it("maps known bundle ids to display names", () => {
    expect(appDisplayName("com.todesktop.230313mzl4w4u92")).toBe("Cursor");
    expect(appDisplayName("com.unknown.app")).toBe("com.unknown.app");
  });

  it("labels chat/editor/terminal presets like the settings mock", () => {
    const ax = [
      {
        attribute: "domClassList" as const,
        operator: "contains" as const,
        scope: "focusedAndAncestors" as const,
        value: "marker",
      },
    ];
    expect(
      displayRuleLabel(rule({ id: "preset-cursor-composer", appId: "x", conditions: ax }))
    ).toMatchObject({ title: "Chat（AIチャット欄）", kind: "chat" });
    expect(
      displayRuleLabel(rule({ id: "preset-cursor-editor", appId: "x", conditions: ax })).title
    ).toBe("Editor（コードエディタ）");
    expect(
      displayRuleLabel(rule({ id: "preset-cursor-terminal-ja", appId: "x", conditions: ax })).title
    ).toBe("Terminal（内蔵ターミナル）");
    expect(
      displayRuleLabel(rule({ id: "preset-terminal", appId: "x", conditions: [] })).title
    ).toBe("デフォルト");
  });

  it("groups rules by app and separates default from contextual rules", () => {
    const groups = groupRulesByApp([
      rule({
        id: "preset-cursor-composer",
        appId: "com.todesktop.230313mzl4w4u92",
        priority: 111,
        conditions: [
          {
            attribute: "domClassList",
            operator: "contains",
            scope: "focusedAndAncestors",
            value: "composer-",
          },
        ],
        target: { kind: "preferredJapanese" },
      }),
      rule({
        id: "preset-cursor-editor",
        appId: "com.todesktop.230313mzl4w4u92",
        priority: 130,
        conditions: [
          {
            attribute: "domIdentifier",
            operator: "contains",
            scope: "focusedAndAncestors",
            value: "workbench.parts.editor",
          },
        ],
      }),
      rule({
        id: "preset-terminal",
        appId: "com.apple.Terminal",
        priority: 0,
        conditions: [],
      }),
    ]);

    expect(groups.map((group) => group.label)).toEqual(["Cursor", "Terminal"]);
    expect(groups[0]?.contextualRules.map((item) => item.title)).toEqual([
      "Chat（AIチャット欄）",
      "Editor（コードエディタ）",
    ]);
    expect(groups[0]?.defaultRule).toBeNull();
    expect(groups[1]?.defaultRule?.id).toBe("preset-terminal");
    expect(groups[1]?.contextualRules).toEqual([]);
  });

  it("resolves preferred and exact targets to input source ids", () => {
    expect(summarizeLanguageTarget({ kind: "preferredJapanese" })).toBe("日本語");
    expect(
      resolvedInputSourceId({ kind: "preferredEnglish" }, "com.apple.keylayout.ABC", "ja.id")
    ).toBe("com.apple.keylayout.ABC");
    expect(resolvedInputSourceId({ kind: "exact", inputSourceId: "zh.id" }, "en.id", "ja.id")).toBe(
      "zh.id"
    );
  });

  it("maps selected input source ids back to preferred aliases when possible", () => {
    expect(ruleTargetFromInputSourceId("en.id", "en.id", "ja.id")).toEqual({
      kind: "preferredEnglish",
    });
    expect(ruleTargetFromInputSourceId("ja.id", "en.id", "ja.id")).toEqual({
      kind: "preferredJapanese",
    });
    expect(ruleTargetFromInputSourceId("zh.id", "en.id", "ja.id")).toEqual({
      kind: "exact",
      inputSourceId: "zh.id",
    });
  });
});
