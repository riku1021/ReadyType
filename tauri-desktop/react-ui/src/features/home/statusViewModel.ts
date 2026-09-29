import type { TFunction } from "i18next";

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

const DECISION_KEYS = {
  ignored: "home.decisionIgnored",
  disabled: "home.decisionDisabled",
  snoozed: "home.decisionSnoozed",
  noRule: "home.decisionNoRule",
  alreadySelected: "home.decisionAlreadySelected",
  switched: "home.decisionSwitched",
  failed: "home.decisionFailed",
} as const satisfies Record<SwitchDecision, string>;

export function toStatusViewModel(
  state: InputSourceViewState | null,
  t: TFunction
): StatusViewModel {
  if (!state?.ready) {
    return {
      kind: "loading",
      headline: t("home.headlineLoading"),
      currentAppLabel: t("common.emDash"),
      matchedRuleLabel: t("common.emDash"),
      currentSourceLabel: t("common.emDash"),
      targetSourceLabel: t("common.emDash"),
      lastDecisionLabel: t("common.emDash"),
    };
  }

  if (state.initError) {
    return {
      kind: "error",
      headline: state.initError,
      currentAppLabel: t("common.emDash"),
      matchedRuleLabel: t("common.emDash"),
      currentSourceLabel: t("common.emDash"),
      targetSourceLabel: t("common.emDash"),
      lastDecisionLabel: t("common.emDash"),
    };
  }

  if (state.permission === "unsupported") {
    return base(state, "unsupported", t("home.headlineUnsupported"), t);
  }

  if (state.permission === "denied" || !state.accessibilityTrusted) {
    return base(state, "permissionRequired", t("home.headlinePermissionRequired"), t);
  }

  if (state.snoozedUntilUnixMs != null) {
    return base(state, "snoozed", t("home.headlineSnoozed"), t);
  }

  if (!state.enabled) {
    return base(state, "disabled", t("home.headlineDisabled"), t);
  }

  return base(state, "running", t("home.headlineRunning"), t);
}

function base(
  state: InputSourceViewState,
  kind: StatusKind,
  headline: string,
  t: TFunction
): StatusViewModel {
  const matchedRule = state.matchedRuleId
    ? (state.rules.find((rule) => rule.id === state.matchedRuleId)?.name ?? state.matchedRuleId)
    : null;

  return {
    kind,
    headline,
    currentAppLabel: state.currentApp?.appId ?? t("common.none"),
    matchedRuleLabel: matchedRule ?? t("common.none"),
    currentSourceLabel: state.currentInputSourceId ?? t("common.unknown"),
    targetSourceLabel: state.targetInputSourceId ?? t("common.none"),
    lastDecisionLabel: state.lastDecision
      ? t(DECISION_KEYS[state.lastDecision])
      : t("home.noSwitchYet"),
  };
}
