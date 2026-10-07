import type { TFunction } from "i18next";

import type { AxAttribute, RuleConditionDto, SwitchLogDto } from "@/features/inputSource";

export type SnapshotAttribute = {
  depth: number;
  attribute: AxAttribute;
  value: string;
};

export function switchLogKey(log: SwitchLogDto): string {
  return `${log.timestampUnixMs}-${log.generation}-${log.ruleId}-${log.decision}`;
}

export function toCondition(attribute: SnapshotAttribute): RuleConditionDto {
  return {
    attribute: attribute.attribute,
    operator:
      attribute.attribute === "domIdentifier" || attribute.attribute === "domClassList"
        ? "contains"
        : "equals",
    scope: attribute.depth === 0 ? "focusedOnly" : "focusedAndAncestors",
    value: attribute.value,
  };
}

export function flattenSnapshotAttributes(
  elements: readonly {
    role: string;
    subrole: string;
    roleDescription: string;
    title: string;
    description: string;
    identifier: string;
    domIdentifier: string;
    domClassList: string;
    help: string;
    placeholder: string;
  }[]
): SnapshotAttribute[] {
  const result: SnapshotAttribute[] = [];
  elements.forEach((element, depth) => {
    const entries: Array<[AxAttribute, string]> = [
      ["role", element.role],
      ["subrole", element.subrole],
      ["roleDescription", element.roleDescription],
      ["title", element.title],
      ["description", element.description],
      ["identifier", element.identifier],
      ["domIdentifier", element.domIdentifier],
      ["domClassList", element.domClassList],
      ["help", element.help],
      ["placeholder", element.placeholder],
    ];
    for (const [attribute, value] of entries) {
      if (value.trim()) {
        result.push({ depth, attribute, value });
      }
    }
  });
  return result;
}

export function logsToTsv(logs: readonly SwitchLogDto[]): string {
  const header = [
    "timestampUnixMs",
    "appId",
    "pid",
    "generation",
    "focusKey",
    "ruleId",
    "decision",
    "from",
    "to",
    "success",
    "attempted",
    "reason",
    "detail",
    "processingMicros",
    "pipelineMicros",
    "queueMicros",
    "prepareMicros",
    "handoffMicros",
    "currentMicros",
    "currentCacheHit",
    "selectMicros",
    "finalizeMicros",
    "publishMicros",
    "axMicros",
  ].join("\t");
  const rows = logs.map((log) =>
    [
      log.timestampUnixMs,
      log.appId,
      log.pid,
      log.generation,
      log.focusKey.replace(/\t/g, " ").replace(/\n/g, " "),
      log.ruleId,
      log.decision,
      log.from,
      log.to,
      log.success ? "success" : "failure",
      log.attempted ? "attempted" : "skipped",
      log.reason.replace(/\t/g, " ").replace(/\n/g, " "),
      log.detail.replace(/\t/g, " ").replace(/\n/g, " "),
      log.processingMicros,
      log.pipelineMicros,
      log.queueMicros,
      log.prepareMicros,
      log.handoffMicros,
      log.currentMicros,
      log.currentCacheHit ? "hit" : "miss",
      log.selectMicros,
      log.finalizeMicros,
      log.publishMicros,
      log.axMicros,
    ].join("\t")
  );
  return [header, ...rows].join("\n");
}

function compactTimingSuffix(log: SwitchLogDto): string | null {
  const segments: string[] = [];
  if (log.axMicros > 0) {
    segments.push(`ax=${log.axMicros}`);
  }
  if (log.queueMicros > 0) {
    segments.push(`q=${log.queueMicros}`);
  }
  if (log.prepareMicros > 0) {
    segments.push(`prep=${log.prepareMicros}`);
  }
  if (log.handoffMicros > 0) {
    segments.push(`handoff=${log.handoffMicros}`);
  }
  if (log.currentMicros > 0) {
    segments.push(`cur=${log.currentMicros}${log.currentCacheHit ? "h" : "m"}`);
  }
  if (log.selectMicros > 0) {
    segments.push(`sel=${log.selectMicros}`);
  }
  if (log.publishMicros > 0) {
    segments.push(`pub=${log.publishMicros}`);
  }
  if (segments.length === 0 && log.pipelineMicros === 0 && log.processingMicros === 0) {
    return null;
  }
  if (segments.length === 0) {
    return `pipe=${log.pipelineMicros}`;
  }
  return segments.join(" ");
}

export function formatSwitchLogLine(log: SwitchLogDto, t: TFunction): string {
  const route = log.from || log.to ? `${log.from || "—"} → ${log.to || "—"}` : "—";
  const parts = [
    log.success ? t("diagnostics.logSuccess") : t("diagnostics.logFailure"),
    log.decision,
    log.attempted ? t("diagnostics.logAttempted") : t("diagnostics.logSkipped"),
    log.appId || "app —",
    `gen ${log.generation}`,
  ];
  if (log.focusKey) {
    parts.push(`focusKey ${log.focusKey}`);
  }
  if (log.ruleId) {
    parts.push(log.ruleId);
  }
  if (log.from || log.to) {
    parts.push(route);
  }
  if (log.reason) {
    parts.push(log.reason);
  }
  if (log.detail) {
    parts.push(log.detail);
  }
  const timing = compactTimingSuffix(log);
  if (timing) {
    parts.push(timing);
  }
  return parts.join(" / ");
}

export function formatLatencyMicros(values: readonly number[]): {
  p50: string;
  p95: string;
  max: string;
} {
  if (values.length === 0) {
    return { p50: "—", p95: "—", max: "—" };
  }
  const sorted = [...values].sort((left, right) => left - right);
  const percentile = (ratio: number) => {
    const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * ratio) - 1));
    return `${(sorted[index] ?? 0).toFixed(0)}µs`;
  };
  const max = sorted[sorted.length - 1] ?? 0;
  return { p50: percentile(0.5), p95: percentile(0.95), max: `${max.toFixed(0)}µs` };
}

export type LatencySummary = ReturnType<typeof formatLatencyMicros>;

export function buildLatencySummaries(logs: readonly SwitchLogDto[]): {
  judgmentAll: LatencySummary;
  judgmentAttempted: LatencySummary;
  pipelineAll: LatencySummary;
  pipelineAttempted: LatencySummary;
  currentMiss: LatencySummary;
  currentHit: LatencySummary;
  currentCache: { hit: number; miss: number };
  handoff: LatencySummary;
  finalize: LatencySummary;
  publish: LatencySummary;
  ax: LatencySummary;
} {
  const attempted = logs.filter((log) => log.attempted);
  // switch_input_source を通った行（current 取得あり）。
  const gatewayLookups = logs.filter(
    (log) => log.currentCacheHit || log.currentMicros > 0 || log.selectMicros > 0
  );
  const misses = gatewayLookups.filter((log) => !log.currentCacheHit);
  const hits = gatewayLookups.filter((log) => log.currentCacheHit);
  return {
    judgmentAll: formatLatencyMicros(logs.map((log) => log.processingMicros)),
    judgmentAttempted: formatLatencyMicros(attempted.map((log) => log.processingMicros)),
    pipelineAll: formatLatencyMicros(logs.map((log) => log.pipelineMicros)),
    pipelineAttempted: formatLatencyMicros(attempted.map((log) => log.pipelineMicros)),
    currentMiss: formatLatencyMicros(misses.map((log) => log.currentMicros)),
    currentHit: formatLatencyMicros(hits.map((log) => log.currentMicros)),
    currentCache: { hit: hits.length, miss: misses.length },
    handoff: formatLatencyMicros(logs.map((log) => log.handoffMicros)),
    finalize: formatLatencyMicros(logs.map((log) => log.finalizeMicros)),
    publish: formatLatencyMicros(
      logs.filter((log) => log.publishMicros > 0).map((log) => log.publishMicros)
    ),
    ax: formatLatencyMicros(logs.filter((log) => log.axMicros > 0).map((log) => log.axMicros)),
  };
}

export type LatencyOutlierKind =
  | "judgmentAll"
  | "pipelineAll"
  | "pipelineAttempted"
  | "queue"
  | "handoff"
  | "current"
  | "select"
  | "finalize"
  | "publish"
  | "ax";

export type LatencyOutlierNote = {
  kind: LatencyOutlierKind;
  log: SwitchLogDto;
  value: number;
};

function maxLogs(
  logs: readonly SwitchLogDto[],
  pick: (log: SwitchLogDto) => number
): SwitchLogDto[] {
  if (logs.length === 0) {
    return [];
  }
  const peak = Math.max(...logs.map(pick));
  return logs.filter((log) => pick(log) === peak);
}

function formatMicros(value: number): string {
  return `${value.toFixed(0)}µs`;
}

export function findLatencyOutliers(logs: readonly SwitchLogDto[]): {
  highlightKeys: ReadonlySet<string>;
  notes: LatencyOutlierNote[];
} {
  const attempted = logs.filter((log) => log.attempted);
  const withSelect = logs.filter((log) => log.selectMicros > 0);
  const withAx = logs.filter((log) => log.axMicros > 0);
  const specs: Array<{
    kind: LatencyOutlierKind;
    candidates: readonly SwitchLogDto[];
    pick: (log: SwitchLogDto) => number;
  }> = [
    { kind: "judgmentAll", candidates: logs, pick: (log) => log.processingMicros },
    { kind: "pipelineAll", candidates: logs, pick: (log) => log.pipelineMicros },
    {
      kind: "pipelineAttempted",
      candidates: attempted,
      pick: (log) => log.pipelineMicros,
    },
    { kind: "queue", candidates: logs, pick: (log) => log.queueMicros },
    {
      kind: "handoff",
      candidates: logs.filter((log) => log.handoffMicros > 0),
      pick: (log) => log.handoffMicros,
    },
    {
      kind: "current",
      candidates: logs.filter((log) => !log.currentCacheHit && log.currentMicros > 0),
      pick: (log) => log.currentMicros,
    },
    { kind: "select", candidates: withSelect, pick: (log) => log.selectMicros },
    {
      kind: "finalize",
      candidates: logs.filter((log) => log.finalizeMicros > 0),
      pick: (log) => log.finalizeMicros,
    },
    {
      kind: "publish",
      candidates: logs.filter((log) => log.publishMicros > 0),
      pick: (log) => log.publishMicros,
    },
    { kind: "ax", candidates: withAx, pick: (log) => log.axMicros },
  ];

  const highlightKeys = new Set<string>();
  const notes: LatencyOutlierNote[] = [];

  for (const spec of specs) {
    const peaks = maxLogs(spec.candidates, spec.pick);
    for (const log of peaks) {
      highlightKeys.add(switchLogKey(log));
      notes.push({ kind: spec.kind, log, value: spec.pick(log) });
    }
  }

  return { highlightKeys, notes };
}

export function formatLatencyOutlierNote(note: LatencyOutlierNote, t: TFunction): string {
  const ruleId = note.log.ruleId || "—";
  return t(`diagnostics.latencyMax.${note.kind}`, {
    generation: note.log.generation,
    ruleId,
    decision: note.log.decision,
    value: formatMicros(note.value),
  });
}
