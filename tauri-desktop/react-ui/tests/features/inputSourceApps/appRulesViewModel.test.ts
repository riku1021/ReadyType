import { describe, expect, it } from "vitest";

import type { RuleDto } from "@/features/inputSource";
import {
  appDisplayName,
  displayRuleLabel,
  groupRulesByApp,
  resolvedInputSourceId,
  ruleTargetFromInputSourceId,
} from "@/features/inputSourceApps/appRulesViewModel";
import type i18n from "@/locales/i18n";

const t = ((key: string) => key) as typeof i18n.t;

const preferences = { en: "en.id", ja: "ja.id" };

function rule(partial: Partial<RuleDto> & Pick<RuleDto, "id" | "appId">): RuleDto {
  return {
    name: partial.name ?? partial.id,
    enabled: partial.enabled ?? true,
    platform: "macOs",
    priority: partial.priority ?? 0,
    conditions: partial.conditions ?? [],
    target: partial.target ?? { kind: "preferredLanguage", language: "en" },
    ...partial,
  };
}

describe("appRulesViewModel", () => {
  it("maps known bundle ids to display names", () => {
    expect(appDisplayName("com.todesktop.230313mzl4w4u92")).toBe("Cursor");
    expect(appDisplayName("com.unknown.app")).toBe("com.unknown.app");
    expect(appDisplayName("com.google.Chrome", "Google Chrome")).toBe("Google Chrome");
    expect(appDisplayName("com.google.Chrome", "  ")).toBe("com.google.Chrome");
  });

  it("uses persisted rule names for contextual rules and i18n for defaults", () => {
    const ax = [
      {
        attribute: "domClassList" as const,
        operator: "contains" as const,
        scope: "focusedAndAncestors" as const,
        value: "marker",
      },
    ];
    expect(
      displayRuleLabel(
        rule({
          id: "preset-cursor-composer",
          name: "Cursor Composer",
          appId: "x",
          conditions: ax,
        }),
        t
      )
    ).toMatchObject({ title: "Cursor Composer", kind: "chat" });
    expect(
      displayRuleLabel(
        rule({
          id: "preset-cursor-editor",
          name: "My Editor",
          appId: "x",
          conditions: ax,
        }),
        t
      ).title
    ).toBe("My Editor");
    expect(
      displayRuleLabel(
        rule({
          id: "preset-cursor-terminal-ja",
          name: "Terminal JA",
          appId: "x",
          conditions: ax,
        }),
        t
      )
    ).toMatchObject({ title: "Terminal JA", kind: "terminal" });
    expect(
      displayRuleLabel(rule({ id: "preset-terminal", appId: "x", conditions: [] }), t).title
    ).toBe("rules.defaultTitle");
  });

  it("groups rules by app and separates default from contextual rules", () => {
    const groups = groupRulesByApp(
      [
        rule({
          id: "preset-cursor-composer",
          name: "Cursor Composer",
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
          target: { kind: "preferredLanguage", language: "ja" },
        }),
        rule({
          id: "preset-cursor-editor",
          name: "Cursor Editor",
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
      ],
      t
    );

    expect(groups.map((group) => group.label)).toEqual(["Cursor", "Terminal"]);
    expect(groups[0]?.contextualRules.map((item) => item.title)).toEqual([
      "Cursor Composer",
      "Cursor Editor",
    ]);
    expect(groups[0]?.defaultRule).toBeNull();
    expect(groups[1]?.defaultRule?.id).toBe("preset-terminal");
    expect(groups[1]?.contextualRules).toEqual([]);
  });

  it("resolves preferred and exact targets to input source ids", () => {
    expect(
      resolvedInputSourceId(
        { kind: "preferredLanguage", language: "en" },
        { en: "com.apple.keylayout.ABC", ja: "ja.id" }
      )
    ).toBe("com.apple.keylayout.ABC");
    expect(resolvedInputSourceId({ kind: "exact", inputSourceId: "zh.id" }, preferences)).toBe(
      "zh.id"
    );
  });

  it("maps selected input source ids back to preferred aliases when possible", () => {
    expect(ruleTargetFromInputSourceId("en.id", preferences)).toEqual({
      kind: "preferredLanguage",
      language: "en",
    });
    expect(ruleTargetFromInputSourceId("ja.id", preferences)).toEqual({
      kind: "preferredLanguage",
      language: "ja",
    });
    expect(ruleTargetFromInputSourceId("zh.id", preferences)).toEqual({
      kind: "exact",
      inputSourceId: "zh.id",
    });
  });
});
