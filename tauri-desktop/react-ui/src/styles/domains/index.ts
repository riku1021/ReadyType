import { calendarColorTokens } from "./calendar";
import { companyStatusColorTokens } from "./companyStatus";
import { deadlineUrgencyColorTokens } from "./deadlineUrgency";
import { eventKindColorTokens } from "./eventKind";
import { taskKindColorTokens } from "./taskKind";

/**
 * アプリ固有の意味色をまとめた登録表。
 * ドメインを増やすときはここに 1 エントリ足す（baseTheme は触らない）。
 */
export const domainColorTokens = {
  eventKind: eventKindColorTokens,
  taskKind: taskKindColorTokens,
  companyStatus: companyStatusColorTokens,
  deadlineUrgency: deadlineUrgencyColorTokens,
  calendar: calendarColorTokens,
} as const;
