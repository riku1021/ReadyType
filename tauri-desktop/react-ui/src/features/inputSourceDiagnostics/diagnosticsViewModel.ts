import type { AxAttribute, RuleConditionDto, SwitchLogDto } from "@/features/inputSource";

export type SnapshotAttribute = {
  depth: number;
  attribute: AxAttribute;
  value: string;
};

export function toCondition(attribute: SnapshotAttribute): RuleConditionDto {
  return {
    attribute: attribute.attribute,
    operator: attribute.attribute === "domIdentifier" ? "contains" : "equals",
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
    "ruleId",
    "from",
    "to",
    "success",
    "reason",
    "processingMicros",
  ].join("\t");
  const rows = logs.map((log) =>
    [
      log.timestampUnixMs,
      log.appId,
      log.ruleId,
      log.from,
      log.to,
      log.success ? "success" : "failure",
      log.reason,
      log.processingMicros,
    ].join("\t")
  );
  return [header, ...rows].join("\n");
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
