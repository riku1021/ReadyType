import type { TFunction } from "i18next";

import type { AccessibilityPermission } from "@/features/inputSource";

export type AccessibilityStatusKind = "loading" | "granted" | "denied" | "unsupported" | "error";

export type AccessibilityViewModel = {
  kind: AccessibilityStatusKind;
  headline: string;
  description: string;
  showPromptActions: boolean;
};

export function toAccessibilityViewModel(
  input: {
    ready: boolean;
    permission: AccessibilityPermission | null;
    initError: string | null;
  },
  t: TFunction
): AccessibilityViewModel {
  if (!input.ready) {
    return {
      kind: "loading",
      headline: t("accessibility.headlineLoading"),
      description: t("accessibility.descriptionLoading"),
      showPromptActions: false,
    };
  }

  if (input.initError) {
    return {
      kind: "error",
      headline: t("accessibility.headlineError"),
      description: input.initError,
      showPromptActions: false,
    };
  }

  if (input.permission === "unsupported") {
    return {
      kind: "unsupported",
      headline: t("accessibility.headlineUnsupported"),
      description: t("accessibility.descriptionUnsupported"),
      showPromptActions: false,
    };
  }

  if (input.permission === "granted") {
    return {
      kind: "granted",
      headline: t("accessibility.headlineGranted"),
      description: t("accessibility.descriptionGranted"),
      showPromptActions: false,
    };
  }

  return {
    kind: "denied",
    headline: t("accessibility.headlineDenied"),
    description: t("accessibility.descriptionDenied"),
    showPromptActions: true,
  };
}
