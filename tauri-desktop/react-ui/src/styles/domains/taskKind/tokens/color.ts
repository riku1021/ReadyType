/**
 * タスク種別のドメイン色。
 * 予定色（eventKind）とは切り離し、作業種別ごとの専用色として管理する。
 *
 * - エントリー: 青（開始・応募）
 * - ES作成: 赤寄り（提出・締切感）
 * - 適性検査: 紫（評価・測定）
 * - Webテスト: アンバー（試験）
 * - コーディングテスト: ティール（技術課題）
 *
 * 消費例: `borderLeftColor: "taskKind.webTest"`
 */
const taskKindBaseColors = {
  entry: "#1565C0",
  esDraft: "#E53935",
  aptitude: "#7E57C2",
  webTest: "#F9A825",
  codingTest: "#00897B",
} as const;

export type TaskKindColorKey = keyof typeof taskKindBaseColors;

export const TASK_KIND_COLOR_KEYS = [
  "entry",
  "esDraft",
  "aptitude",
  "webTest",
  "codingTest",
] as const satisfies readonly TaskKindColorKey[];

/**
 * Panda `semanticTokens.colors.taskKind` に載せるトークン。
 */
export const taskKindColorTokens = {
  entry: { value: taskKindBaseColors.entry },
  esDraft: { value: taskKindBaseColors.esDraft },
  aptitude: { value: taskKindBaseColors.aptitude },
  webTest: { value: taskKindBaseColors.webTest },
  codingTest: { value: taskKindBaseColors.codingTest },
} as const;
