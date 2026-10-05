import { buildInputSourceOptions, visibleInputSourceOptions } from "@/features/inputSource";
import { resolvedInputSourceId } from "@/features/inputSourceApps";

import type { QuickSetSessionDto } from "./types";

export type QuickSetOptionView = {
  sourceId: string;
  label: string;
  isCurrent: boolean;
};

/** Quick Set UI phase (idle → applying → success). */
export type QuickSetUiPhase =
  | { kind: "idle" }
  | { kind: "applying" }
  | { kind: "success"; languageLabel: string };

/** Same option set as app-settings LanguageSelect. */
export function buildQuickSetOptions(session: QuickSetSessionDto): QuickSetOptionView[] {
  const preferences = session.languagePreferences;
  const current = session.matchedTarget
    ? resolvedInputSourceId(session.matchedTarget, preferences)
    : "";
  const configuredIds = [
    ...Object.values(preferences).filter(
      (id): id is string => typeof id === "string" && id.length > 0
    ),
    current,
  ];
  const options = visibleInputSourceOptions(
    buildInputSourceOptions(session.sources, configuredIds),
    current
  );
  return options.map((option) => ({
    sourceId: option.value,
    label: option.label,
    isCurrent: current.length > 0 && option.value === current,
  }));
}

/** App name only — rule name lives in the editable field. */
export function targetHeading(session: QuickSetSessionDto): string {
  return session.appDisplayName;
}
