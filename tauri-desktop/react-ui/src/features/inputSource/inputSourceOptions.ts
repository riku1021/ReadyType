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

/** OS の LocalizedName（`source.name`）をそのまま使う。メニューバー表示に合わせる。 */
function displayLabelForSource(source: InputSourceDto): string {
  if (!source.available) {
    return i18n.t("common.inputSourceUnavailable", { id: source.id });
  }
  return source.name;
}
