import { languageDisplayName } from "@/features/inputSource";
import type { QuickSetLanguageDto, QuickSetSessionDto } from "./types";

export type QuickSetOptionView = {
  language: string;
  label: string;
  sourceName: string;
  isCurrent: boolean;
};

/** Quick Set UI phase (idle → applying → success). */
export type QuickSetUiPhase =
  | { kind: "idle" }
  | { kind: "applying" }
  | { kind: "success"; languageLabel: string };

export function currentLanguageFromTarget(
  target: QuickSetSessionDto["matchedTarget"]
): string | null {
  if (!target) {
    return null;
  }
  if (target.kind === "preferredLanguage") {
    return target.language;
  }
  return null;
}

export function buildQuickSetOptions(session: QuickSetSessionDto): QuickSetOptionView[] {
  const current = currentLanguageFromTarget(session.matchedTarget);
  return session.languages.map((language: QuickSetLanguageDto) => ({
    language: language.language,
    label: languageDisplayName(language.language),
    sourceName: language.preferredSourceName,
    isCurrent: current === language.language,
  }));
}

/** App name only — rule name lives in the editable field. */
export function targetHeading(session: QuickSetSessionDto): string {
  return session.appDisplayName;
}
