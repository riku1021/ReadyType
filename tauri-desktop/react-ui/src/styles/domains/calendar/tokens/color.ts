/**
 * カレンダー曜日のドメイン色。
 * 汎用の brand / status とは切り離し、カレンダー UI 専用として管理する。
 *
 * - 土曜: プライマリ青
 * - 日曜: 注意喚起と同じ赤（意味は曜日表示）
 *
 * 消費例: `color: "calendar.saturday"`
 */
const calendarBaseColors = {
  saturday: "#007bff",
  sunday: "#f8313e",
} as const;

export type CalendarColorKey = keyof typeof calendarBaseColors;

export const CALENDAR_COLOR_KEYS = [
  "saturday",
  "sunday",
] as const satisfies readonly CalendarColorKey[];

/**
 * Panda `semanticTokens.colors.calendar` に載せるトークン。
 */
export const calendarColorTokens = {
  saturday: { value: calendarBaseColors.saturday },
  sunday: { value: calendarBaseColors.sunday },
} as const;
