import i18n from "@/locales/i18n";
import { withConfiguredSource } from "./api";
import type { InputSourceDto } from "./types";

export type InputSourceOption = {
  value: string;
  label: string;
  available: boolean;
};

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
    .sort((left, right) => left.label.localeCompare(right.label, i18n.language));
}

/** 現在値以外の利用不可ソースは選択肢から除外する。 */
export function visibleInputSourceOptions(
  options: readonly InputSourceOption[],
  currentValue: string
): InputSourceOption[] {
  return options.filter((option) => option.available || option.value === currentValue);
}

export function languageDisplayName(languageCode: string): string {
  try {
    const languageNames = new Intl.DisplayNames([i18n.language], { type: "language" });
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
    return i18n.t("common.inputSourceUnavailable", { id: source.id });
  }
  const code = source.languageCode;
  if (!code) {
    return source.name;
  }
  const languageLabel = languageDisplayName(code);
  if ((languageCounts.get(code) ?? 0) > 1) {
    return i18n.t("common.inputSourceWithVariant", {
      language: languageLabel,
      name: source.name,
    });
  }
  return languageLabel;
}
