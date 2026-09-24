/**
 * 予定種別のドメイン色。
 * 汎用の brand / status とは切り離し、種別ごとの専用色として管理する。
 *
 * - 面談: 紫（1:1）
 * - 説明会: オレンジ（案内）
 * - 面接: 青（フォーマル選考）
 *
 * 消費例: `backgroundColor: "eventKind.interview"`
 */
const eventKindBaseColors = {
  meeting: "#7E57C2",
  infoSession: "#FB8C00",
  interview: "#1565C0",
} as const;

export type EventKindColorKey = keyof typeof eventKindBaseColors;

export const EVENT_KIND_COLOR_KEYS = [
  "meeting",
  "infoSession",
  "interview",
] as const satisfies readonly EventKindColorKey[];

/**
 * Panda `semanticTokens.colors.eventKind` に載せるトークン。
 */
export const eventKindColorTokens = {
  meeting: { value: eventKindBaseColors.meeting },
  infoSession: { value: eventKindBaseColors.infoSession },
  interview: { value: eventKindBaseColors.interview },
} as const;
