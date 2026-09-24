import { describe, expect, it } from "vitest";

import {
  flattenSnapshotAttributes,
  formatLatencyMicros,
  logsToTsv,
  toCondition,
} from "./diagnosticsViewModel";

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
          help: "",
        },
      ])
    ).toEqual([{ depth: 0, attribute: "role", value: "AXTextArea" }]);
  });

  it("formats TSV logs and latency percentiles", () => {
    const tsv = logsToTsv([
      {
        timestampUnixMs: 1,
        appId: "app",
        ruleId: "rule",
        from: "en",
        to: "ja",
        success: true,
        reason: "",
        processingMicros: 10,
      },
    ]);
    expect(tsv.split("\n")[0]).toContain("processingMicros");
    expect(tsv).toContain("success");
    expect(formatLatencyMicros([10, 20, 30, 40, 100])).toEqual({
      p50: "30µs",
      p95: "100µs",
    });
  });
});
