import { atomWithStorage } from "jotai/utils";

import type { GlassLightTone } from "@/styles/recipes/glass.recipe";

/**
 * ライトテーマ時のガラス見え方（比較用）。
 * ダークテーマ側は lightTone によらず同じガラス表現。
 */
export const glassLightToneAtom = atomWithStorage<GlassLightTone>("glassLightTone", "frost");

export const GLASS_LIGHT_TONE_OPTIONS = [
  "白フロスト（縁・影）",
  "ダークガラス",
  "ソリッドカード",
] as const;

export type GlassLightToneLabel = (typeof GLASS_LIGHT_TONE_OPTIONS)[number];

const labelToTone: Record<GlassLightToneLabel, GlassLightTone> = {
  "白フロスト（縁・影）": "frost",
  ダークガラス: "darkTint",
  ソリッドカード: "solid",
};

const toneToLabel: Record<GlassLightTone, GlassLightToneLabel> = {
  frost: "白フロスト（縁・影）",
  darkTint: "ダークガラス",
  solid: "ソリッドカード",
};

export function getGlassLightToneLabel(tone: GlassLightTone): GlassLightToneLabel {
  return toneToLabel[tone];
}

export function glassLightToneFromLabel(label: string): GlassLightTone | undefined {
  if (label in labelToTone) {
    return labelToTone[label as GlassLightToneLabel];
  }
  return undefined;
}
