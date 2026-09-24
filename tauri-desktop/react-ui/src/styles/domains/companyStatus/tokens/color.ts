import { createColorSurface } from "../../../utils/color-utils";

/**
 * 企業の選考ステータスのドメイン色。
 * 汎用の brand / status とは切り離して管理する。
 *
 * 方針（ATS・ステータスUI慣例）:
 * - 選考中: 青（パイプライン進行）
 * - 募集中: 黄（オープン・注目。選考中の青と明確に分離）
 * - 内定: 緑（成功）
 * - 不採用: ローズ（ネガティブ結果）
 * - 終了: グレー（アーカイブ・ニュートラル）
 *
 * CompanyStatusBadge（Fill 型）用に main / soft / softBorder を生成する。
 * 消費例: `color: "companyStatus.offer.main"`
 */
const companyStatusBaseColors = {
  inProcess: "#1565C0",
  recruiting: "#FBC02D",
  offer: "#2E7D32",
  rejected: "#E91E63",
  closed: "#78909C",
} as const;

export type CompanyStatusColorKey = keyof typeof companyStatusBaseColors;

export const COMPANY_STATUS_COLOR_KEYS = [
  "inProcess",
  "recruiting",
  "offer",
  "rejected",
  "closed",
] as const satisfies readonly CompanyStatusColorKey[];

/**
 * Panda `semanticTokens.colors.companyStatus` に載せるトークン。
 */
export const companyStatusColorTokens = {
  inProcess: createColorSurface(companyStatusBaseColors.inProcess),
  recruiting: createColorSurface(companyStatusBaseColors.recruiting),
  offer: createColorSurface(companyStatusBaseColors.offer),
  rejected: createColorSurface(companyStatusBaseColors.rejected),
  closed: createColorSurface(companyStatusBaseColors.closed),
} as const;
