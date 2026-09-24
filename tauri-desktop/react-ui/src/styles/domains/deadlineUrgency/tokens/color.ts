/**
 * 締切緊急度（3日以内）のドメイン色。
 * 汎用の brand / status とは切り離して管理する。
 *
 * 方針（既存 UI を維持）:
 * - 明日: アクセント赤（最緊急）
 * - あと2日: 警告アンバー
 * - あと3日: プライマリ青（やや余裕）
 *
 * 消費例: `borderLeftColor: "deadlineUrgency.tomorrow"`
 */
const deadlineUrgencyBaseColors = {
  tomorrow: "#f8313e",
  inTwoDays: "#ffc107",
  inThreeDays: "#007bff",
} as const;

export type DeadlineUrgencyColorKey = keyof typeof deadlineUrgencyBaseColors;

export const DEADLINE_URGENCY_COLOR_KEYS = [
  "tomorrow",
  "inTwoDays",
  "inThreeDays",
] as const satisfies readonly DeadlineUrgencyColorKey[];

/**
 * Panda `semanticTokens.colors.deadlineUrgency` に載せるトークン。
 */
export const deadlineUrgencyColorTokens = {
  tomorrow: { value: deadlineUrgencyBaseColors.tomorrow },
  inTwoDays: { value: deadlineUrgencyBaseColors.inTwoDays },
  inThreeDays: { value: deadlineUrgencyBaseColors.inThreeDays },
} as const;
