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
  configuredIds: string | readonly string[],
  displayNames: Readonly<Record<string, string>> = {}
): InputSourceOption[] {
  const ids = typeof configuredIds === "string" ? [configuredIds] : configuredIds;
  let merged: InputSourceDto[] = [...sources];
  for (const id of ids) {
    if (id.length === 0) {
      continue;
    }
    merged = withConfiguredSource(merged, id, displayNames[id]);
  }
  const unique = new Map<string, InputSourceDto>();
  for (const source of merged) {
    if (!unique.has(source.id)) {
      unique.set(source.id, source);
    }
  }

  return [...unique.values()]
    .map((source) => ({
      value: source.id,
      label: displayLabelForSource(source),
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

/** 利用可能時は OS LocalizedName。利用不可時は最終既知名（無ければ id）。 */
function displayLabelForSource(source: InputSourceDto): string {
  if (!source.available) {
    const label = source.name.length > 0 ? source.name : source.id;
    return i18n.t("common.inputSourceUnavailable", { label });
  }
  return source.name;
}
