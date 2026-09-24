import type { InputSourceViewState, SwitchDecision } from "@/features/inputSource";

export type StatusKind =
  | "loading"
  | "error"
  | "unsupported"
  | "permissionRequired"
  | "snoozed"
  | "disabled"
  | "running";

export type StatusViewModel = {
  kind: StatusKind;
  headline: string;
  currentAppLabel: string;
  matchedRuleLabel: string;
  currentSourceLabel: string;
  targetSourceLabel: string;
  lastDecisionLabel: string;
};

const DECISION_LABELS: Record<SwitchDecision, string> = {
  ignored: "無視",
  disabled: "無効のため未切替",
  snoozed: "一時停止中のため未切替",
  noRule: "一致ルールなし",
  alreadySelected: "すでに切替済み",
  switched: "切替成功",
};

export function toStatusViewModel(state: InputSourceViewState | null): StatusViewModel {
  if (!state?.ready) {
    return {
      kind: "loading",
      headline: "状態を読み込み中…",
      currentAppLabel: "—",
      matchedRuleLabel: "—",
      currentSourceLabel: "—",
      targetSourceLabel: "—",
      lastDecisionLabel: "—",
    };
  }

  if (state.initError) {
    return {
      kind: "error",
      headline: state.initError,
      currentAppLabel: "—",
      matchedRuleLabel: "—",
      currentSourceLabel: "—",
      targetSourceLabel: "—",
      lastDecisionLabel: "—",
    };
  }

  if (state.permission === "unsupported") {
    return base(state, "unsupported", "このOSでは未対応です");
  }

  if (state.permission === "denied" || !state.accessibilityTrusted) {
    return base(state, "permissionRequired", "アクセシビリティ権限が必要です");
  }

  if (state.snoozedUntilUnixMs != null) {
    return base(state, "snoozed", "一時停止中");
  }

  if (!state.enabled) {
    return base(state, "disabled", "自動切替はOFFです");
  }

  return base(state, "running", "自動切替は有効です");
}

function base(state: InputSourceViewState, kind: StatusKind, headline: string): StatusViewModel {
  const matchedRule = state.matchedRuleId
    ? (state.rules.find((rule) => rule.id === state.matchedRuleId)?.name ?? state.matchedRuleId)
    : null;

  return {
    kind,
    headline,
    currentAppLabel: state.currentApp?.appId ?? "なし",
    matchedRuleLabel: matchedRule ?? "なし",
    currentSourceLabel: state.currentInputSourceId ?? "不明",
    targetSourceLabel: state.targetInputSourceId ?? "なし",
    lastDecisionLabel: state.lastDecision
      ? DECISION_LABELS[state.lastDecision]
      : "まだ切替していません",
  };
}
