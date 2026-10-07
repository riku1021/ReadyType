import { describe, expect, it } from "vitest";

import type { SwitchLogDto } from "@/features/inputSource";
import {
  buildLatencySummaries,
  findLatencyOutliers,
  flattenSnapshotAttributes,
  formatLatencyMicros,
  logsToTsv,
  switchLogKey,
  toCondition,
} from "@/features/inputSourceDiagnostics/diagnosticsViewModel";

function sampleLog(overrides: Partial<SwitchLogDto> = {}): SwitchLogDto {
  return {
    timestampUnixMs: 1,
    appId: "app",
    pid: 42,
    generation: 3,
    focusKey: "AXTextArea||composer",
    ruleId: "rule",
    decision: "switched",
    from: "en",
    to: "ja",
    success: true,
    attempted: true,
    reason: "selected target input source",
    detail: "override=none elements=1 focused[role=AXTextArea]",
    processingMicros: 10,
    pipelineMicros: 100,
    queueMicros: 50,
    prepareMicros: 8,
    handoffMicros: 5,
    currentMicros: 12,
    currentCacheHit: false,
    selectMicros: 30,
    finalizeMicros: 4,
    publishMicros: 100,
    axMicros: 2000,
    ...overrides,
  };
}

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
    const sample = sampleLog();
    const tsv = logsToTsv([sample]);
    expect(tsv.split("\n")[0]).toContain("focusKey");
    expect(tsv.split("\n")[0]).toContain("handoffMicros");
    expect(tsv.split("\n")[0]).toContain("publishMicros");
    expect(tsv.split("\n")[0]).toContain("axMicros");
    expect(tsv.split("\n")[0]).toContain("currentCacheHit");
    expect(tsv).toContain("AXTextArea||composer");
    expect(tsv).toContain("miss");
    expect(formatLatencyMicros([10, 20, 30, 40, 100])).toEqual({
      p50: "30µs",
      p95: "100µs",
      max: "100µs",
    });
    expect(
      buildLatencySummaries([
        sampleLog({
          processingMicros: 10,
          pipelineMicros: 500,
          queueMicros: 100,
          prepareMicros: 8,
          handoffMicros: 40,
          currentMicros: 200000,
          currentCacheHit: false,
          selectMicros: 300,
          finalizeMicros: 12,
          axMicros: 5000,
        }),
        sampleLog({
          timestampUnixMs: 2,
          generation: 2,
          decision: "alreadySelected",
          attempted: false,
          processingMicros: 5,
          pipelineMicros: 80,
          queueMicros: 20,
          prepareMicros: 8,
          handoffMicros: 1,
          currentMicros: 2,
          currentCacheHit: true,
          selectMicros: 0,
          finalizeMicros: 3,
          axMicros: 100,
        }),
        sampleLog({
          timestampUnixMs: 3,
          generation: 3,
          ruleId: "",
          decision: "noRule",
          from: "",
          to: "",
          success: false,
          attempted: false,
          processingMicros: 200,
          pipelineMicros: 60000,
          queueMicros: 59000,
          prepareMicros: 200,
          handoffMicros: 0,
          currentMicros: 0,
          currentCacheHit: false,
          selectMicros: 0,
          finalizeMicros: 200,
          axMicros: 0,
        }),
      ])
    ).toEqual({
      judgmentAll: { p50: "10µs", p95: "200µs", max: "200µs" },
      judgmentAttempted: { p50: "10µs", p95: "10µs", max: "10µs" },
      pipelineAll: { p50: "500µs", p95: "60000µs", max: "60000µs" },
      pipelineAttempted: { p50: "500µs", p95: "500µs", max: "500µs" },
      currentMiss: { p50: "200000µs", p95: "200000µs", max: "200000µs" },
      currentHit: { p50: "2µs", p95: "2µs", max: "2µs" },
      currentCache: { hit: 1, miss: 1 },
      handoff: { p50: "1µs", p95: "40µs", max: "40µs" },
      finalize: { p50: "12µs", p95: "200µs", max: "200µs" },
      publish: { p50: "100µs", p95: "100µs", max: "100µs" },
      ax: { p50: "100µs", p95: "5000µs", max: "5000µs" },
    });

    const slowLog = sampleLog({
      timestampUnixMs: 2,
      generation: 2,
      ruleId: "",
      decision: "noRule",
      from: "",
      to: "",
      success: false,
      attempted: false,
      processingMicros: 200,
      pipelineMicros: 60000,
      queueMicros: 59000,
      prepareMicros: 200,
      handoffMicros: 0,
      currentMicros: 0,
      currentCacheHit: false,
      selectMicros: 0,
      finalizeMicros: 200,
      axMicros: 0,
    });
    const outliers = findLatencyOutliers([
      sampleLog({
        processingMicros: 10,
        pipelineMicros: 500,
        queueMicros: 100,
        prepareMicros: 8,
        handoffMicros: 5,
        currentMicros: 20,
        currentCacheHit: false,
        selectMicros: 300,
        finalizeMicros: 4,
        axMicros: 9000,
      }),
      slowLog,
    ]);
    expect(outliers.highlightKeys.has(switchLogKey(slowLog))).toBe(true);
    expect(outliers.notes.some((note) => note.kind === "pipelineAll" && note.log === slowLog)).toBe(
      true
    );
    expect(outliers.notes.some((note) => note.kind === "select" && note.value === 300)).toBe(true);
    expect(outliers.notes.some((note) => note.kind === "ax" && note.value === 9000)).toBe(true);
  });
});
