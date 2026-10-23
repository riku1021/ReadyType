import type { UpdatePreferenceStore } from "@/features/update/updatePreferences";

export function memoryStore(initial: Record<string, string> = {}): UpdatePreferenceStore & {
  data: Map<string, string>;
} {
  const data = new Map(Object.entries(initial));
  return {
    data,
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => {
      data.set(key, value);
    },
  };
}
