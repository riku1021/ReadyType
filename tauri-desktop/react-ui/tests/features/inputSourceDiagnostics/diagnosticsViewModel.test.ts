import { describe, expect, it } from "vitest";

import {
  flattenSnapshotAttributes,
  formatLatencyMicros,
  logsToTsv,
  toCondition,
} from "@/features/inputSourceDiagnostics/diagnosticsViewModel";

describe("diagnosticsViewModel", () => {
  it("maps an ancestor DOM identifier to a contains condition", () => {
    expect(
      toCondition({
        depth: 2,
        attribute: "domIdentifier",
        value: "workbench.panel.chat",
      })
    ).toEqual({
      attribute: "domIdentifier",
      operator: "contains",
      scope: "focusedAndAncestors",
      value: "workbench.panel.chat",
    });
  });

  it("maps focused depth to focusedOnly", () => {
    expect(
      toCondition({
        depth: 0,
        attribute: "role",
        value: "AXTextArea",
      }).scope
    ).toBe("focusedOnly");
  });

  it("excludes empty attributes", () => {
    expect(
      flattenSnapshotAttributes([
        {
          role: "AXTextArea",
          subrole: "",
          roleDescription: "",
          title: "",
          description: "",
          identifier: "",
          domIdentifier: "",
          domClassList: "",
          help: "",
          placeholder: "",
        },
      ])
    ).toEqual([{ depth: 0, attribute: "role", value: "AXTextArea" }]);
  });

  it("maps a DOM class list to a contains condition", () => {
    expect(
      toCondition({
        depth: 0,
        attribute: "domClassList",
        value: "composer-bar",
      })
    ).toEqual({
      attribute: "domClassList",
      operator: "contains",
      scope: "focusedOnly",
      value: "composer-bar",
    });
  });

  it("formats TSV logs and latency percentiles", () => {
    const tsv = logsToTsv([
      {
        timestampUnixMs: 1,
        appId: "app",
        pid: 42,
        generation: 3,
        ruleId: "rule",
        decision: "switched",
        from: "en",
        to: "ja",
        success: true,
        attempted: true,
        reason: "",
        detail: "elements=1",
        processingMicros: 10,
        prepareMicros: 1,
        currentMicros: 2,
        selectMicros: 3,
      },
    ]);
    expect(tsv.split("\n")[0]).toContain("processingMicros");
    expect(tsv.split("\n")[0]).toContain("prepareMicros");
    expect(tsv.split("\n")[0]).toContain("currentMicros");
    expect(tsv.split("\n")[0]).toContain("selectMicros");
    expect(tsv.split("\n")[0]).toContain("decision");
    expect(tsv).toContain("success");
    expect(tsv).toContain("attempted");
    expect(formatLatencyMicros([10, 20, 30, 40, 100])).toEqual({
      p50: "30µs",
      p95: "100µs",
    });
  });
});
