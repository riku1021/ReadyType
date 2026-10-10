import type { TFunction } from "i18next";

import type { AccessibilityPermission } from "@/features/inputSource";
import { focusPermissionPaneLabel } from "@/platform";

export type AccessibilityStatusKind = "loading" | "granted" | "denied" | "unsupported" | "error";

export type AccessibilityCardTone = "default" | "alert";

export type AccessibilityViewModel = {
  kind: AccessibilityStatusKind;
  /** System Settings 上の権限ペイン名（見出し用） */
  permissionPane: string;
  cardTone: AccessibilityCardTone;
  showGrantedBadge: boolean;
  showRequiredStatus: boolean;
  showOpenSettings: boolean;
  /** 未許可時は TCC 登録のあと設定を開く */
  requestBeforeOpenSettings: boolean;
  openSettingsVariant: "primary" | "neutral";
  showPrivacy: boolean;
  /** granted 以外で本文に出すメッセージ。用途説明を使う場合は null */
  detailMessage: string | null;
};

function resolveKind(input: {
  ready: boolean;
  permission: AccessibilityPermission | null;
  initError: string | null;
}): AccessibilityStatusKind {
  if (!input.ready) {
    return "loading";
  }
  if (input.initError) {
    return "error";
  }
  if (input.permission === "unsupported") {
    return "unsupported";
  }
  if (input.permission === "granted") {
    return "granted";
  }
  return "denied";
}

function detailMessageFor(
  kind: AccessibilityStatusKind,
  pane: string,
  initError: string | null,
  t: TFunction
): string | null {
  switch (kind) {
    case "loading":
      return t("accessibility.descriptionLoading", { pane });
    case "error":
      return initError;
    case "unsupported":
      return t("accessibility.descriptionUnsupported", { pane });
    case "denied":
      return t("accessibility.deniedGuidance", { pane });
    case "granted":
      return null;
  }
}

export function toAccessibilityViewModel(
  input: {
    ready: boolean;
    permission: AccessibilityPermission | null;
    initError: string | null;
  },
  t: TFunction
): AccessibilityViewModel {
  const permissionPane = focusPermissionPaneLabel(t);
  const kind = resolveKind(input);
  const isDenied = kind === "denied";
  const isGranted = kind === "granted";

  return {
    kind,
    permissionPane,
    cardTone: isDenied ? "alert" : "default",
    showGrantedBadge: isGranted,
    showRequiredStatus: isDenied,
    showOpenSettings: isGranted || isDenied,
    requestBeforeOpenSettings: isDenied,
    openSettingsVariant: isDenied ? "primary" : "neutral",
    showPrivacy: isGranted || isDenied,
    detailMessage: detailMessageFor(kind, permissionPane, input.initError, t),
  };
}
