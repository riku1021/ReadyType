import type { TFunction } from "i18next";

import type { AxAttribute, RuleConditionDto, SwitchLogDto } from "@/features/inputSource";

export type SnapshotAttribute = {
  depth: number;
  attribute: AxAttribute;
  value: string;
};

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
    "ruleId",
    "decision",
    "from",
    "to",
    "success",
    "attempted",
    "reason",
    "detail",
    "processingMicros",
  ].join("\t");
  const rows = logs.map((log) =>
    [
      log.timestampUnixMs,
      log.appId,
      log.pid,
      log.generation,
      log.ruleId,
      log.decision,
      log.from,
      log.to,
      log.success ? "success" : "failure",
      log.attempted ? "attempted" : "skipped",
      log.reason.replace(/\t/g, " ").replace(/\n/g, " "),
      log.detail.replace(/\t/g, " ").replace(/\n/g, " "),
      log.processingMicros,
    ].join("\t")
  );
  return [header, ...rows].join("\n");
}

export function formatSwitchLogLine(log: SwitchLogDto, t: TFunction): string {
  const route = log.from || log.to ? `${log.from || "—"} → ${log.to || "—"}` : "—";
  const parts = [
    log.success ? t("diagnostics.logSuccess") : t("diagnostics.logFailure"),
    log.decision,
    log.attempted ? t("diagnostics.logAttempted") : t("diagnostics.logSkipped"),
    log.appId,
    `pid ${log.pid}`,
    `gen ${log.generation}`,
    log.ruleId || "rule —",
    route,
    `${log.processingMicros}µs`,
  ];
  if (log.reason) {
    parts.push(log.reason);
  }
  if (log.detail) {
    parts.push(log.detail);
  }
  return parts.join(" / ");
}

export function formatLatencyMicros(values: readonly number[]): {
  p50: string;
  p95: string;
} {
  if (values.length === 0) {
    return { p50: "—", p95: "—" };
  }
  const sorted = [...values].sort((left, right) => left - right);
  const percentile = (ratio: number) => {
    const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * ratio) - 1));
    return `${(sorted[index] ?? 0).toFixed(0)}µs`;
  };
  return { p50: percentile(0.5), p95: percentile(0.95) };
}
