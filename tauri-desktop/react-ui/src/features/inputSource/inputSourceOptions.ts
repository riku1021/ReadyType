import { withConfiguredSource } from "./api";
import type { InputSourceDto } from "./types";

export type InputSourceOption = {
  value: string;
  label: string;
  available: boolean;
};

const languageNames = new Intl.DisplayNames(["ja"], { type: "language" });

export function buildInputSourceOptions(
  sources: readonly InputSourceDto[],
  configuredIds: string | readonly string[]
): InputSourceOption[] {
  const ids = typeof configuredIds === "string" ? [configuredIds] : configuredIds;
  let merged: InputSourceDto[] = [...sources];
  for (const id of ids) {
    if (id.length === 0) {
      continue;
    }
    merged = withConfiguredSource(merged, id);
  }
  const unique = new Map<string, InputSourceDto>();
  for (const source of merged) {
    if (!unique.has(source.id)) {
      unique.set(source.id, source);
    }
  }
  const uniqueSources = [...unique.values()];
  const languageCounts = new Map<string, number>();
  for (const source of uniqueSources) {
    const code = source.languageCode;
    if (!code) {
      continue;
    }
    languageCounts.set(code, (languageCounts.get(code) ?? 0) + 1);
  }

  return uniqueSources
    .map((source) => ({
      value: source.id,
      label: displayLabelForSource(source, languageCounts),
      available: source.available,
    }))
    .sort((left, right) => left.label.localeCompare(right.label, "ja"));
}

export function languageDisplayName(languageCode: string): string {
  try {
    return languageNames.of(languageCode) ?? languageCode;
  } catch {
    return languageCode;
  }
}

function displayLabelForSource(
  source: InputSourceDto,
  languageCounts: ReadonlyMap<string, number>
): string {
  if (!source.available) {
    return `${source.id}（利用不可）`;
  }
  const code = source.languageCode;
  if (!code) {
    return source.name;
  }
  const languageLabel = languageDisplayName(code);
  if ((languageCounts.get(code) ?? 0) > 1) {
    return `${languageLabel}（${source.name}）`;
  }
  return languageLabel;
}
