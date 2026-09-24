import type { AccessibilityPermission } from "@/features/inputSource";

export type AccessibilityStatusKind = "loading" | "granted" | "denied" | "unsupported" | "error";

export type AccessibilityViewModel = {
  kind: AccessibilityStatusKind;
  headline: string;
  description: string;
  showPromptActions: boolean;
};

export function toAccessibilityViewModel(input: {
  ready: boolean;
  permission: AccessibilityPermission | null;
  initError: string | null;
}): AccessibilityViewModel {
  if (!input.ready) {
    return {
      kind: "loading",
      headline: "状態を確認しています…",
      description: "アクセシビリティ権限の状態を読み込んでいます。",
      showPromptActions: false,
    };
  }

  if (input.initError) {
    return {
      kind: "error",
      headline: "状態を取得できませんでした",
      description: input.initError,
      showPromptActions: false,
    };
  }

  if (input.permission === "unsupported") {
    return {
      kind: "unsupported",
      headline: "このOSでは未対応です",
      description: "アクセシビリティ権限によるフォーカス監視は、現在 macOS のみ対応しています。",
      showPromptActions: false,
    };
  }

  if (input.permission === "granted") {
    return {
      kind: "granted",
      headline: "アクセシビリティは許可されています",
      description:
        "フォーカス監視と入力ソースの自動切替に必要な権限が付与されています。システム設定で取り消した場合は、この画面を開き直すか「状態を再確認」を押してください。",
      showPromptActions: false,
    };
  }

  return {
    kind: "denied",
    headline: "アクセシビリティ権限が必要です",
    description:
      "ReadyType がフォーカス中のアプリを検知するには、システム設定の「プライバシーとセキュリティ > アクセシビリティ」で ReadyType を許可してください。",
    showPromptActions: true,
  };
}
