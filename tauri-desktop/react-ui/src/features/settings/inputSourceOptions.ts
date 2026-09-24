import type { InputSourceDto } from "@/features/inputSource";
import { withConfiguredSource } from "@/features/inputSource";

export type InputSourceOption = {
  value: string;
  label: string;
  available: boolean;
};

export function buildInputSourceOptions(
  sources: readonly InputSourceDto[],
  configuredId: string
): InputSourceOption[] {
  const merged = withConfiguredSource(sources, configuredId);
  const unique = new Map<string, InputSourceDto>();
  for (const source of merged) {
    if (!unique.has(source.id)) {
      unique.set(source.id, source);
    }
  }
  return [...unique.values()]
    .sort((left, right) => left.name.localeCompare(right.name, "ja"))
    .map((source) => ({
      value: source.id,
      label: source.available ? `${source.name}（${source.id}）` : `${source.id}（利用不可）`,
      available: source.available,
    }));
}
