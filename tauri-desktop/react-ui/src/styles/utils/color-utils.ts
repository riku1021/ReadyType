/**
 * カラーユーティリティ関数
 * @description 色の調整や変換を行う関数群
 */

/**
 * 16進数カラーコードをRGBに変換
 * @param hex - 16進数カラーコード (#ffffff形式)
 * @returns RGB値の配列 [r, g, b]
 */
function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) {
    throw new Error(`Invalid hex color: ${hex}`);
  }
  return [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)];
}

/**
 * RGBをHSLに変換
 * @param r - 赤 (0-255)
 * @param g - 緑 (0-255)
 * @param b - 青 (0-255)
 * @returns HSL値の配列 [h, s, l]
 */
function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255;
  g /= 255;
  b /= 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max === min) {
    h = s = 0; // achromatic
  } else {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }

  return [h * 360, s * 100, l * 100];
}

/**
 * HSLをRGBに変換
 * @param h - 色相 (0-360)
 * @param s - 彩度 (0-100)
 * @param l - 明度 (0-100)
 * @returns RGB値の配列 [r, g, b]
 */
function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  h /= 360;
  s /= 100;
  l /= 100;

  const hue2rgb = (p: number, q: number, t: number): number => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  let r: number, g: number, b: number;

  if (s === 0) {
    r = g = b = l; // achromatic
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }

  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

/**
 * RGBを16進数カラーコードに変換
 * @param r - 赤 (0-255)
 * @param g - 緑 (0-255)
 * @param b - 青 (0-255)
 * @returns 16進数カラーコード
 */
function rgbToHex(r: number, g: number, b: number): string {
  const toHex = (n: number): string => {
    const hex = Math.round(n).toString(16);
    return hex.length === 1 ? `0${hex}` : hex;
  };
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

/** sRGB チャネル (0–255) → 線形光 */
function srgbChannelToLinear(channel: number): number {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

/** 線形光 → sRGB チャネル (0–255) */
function linearToSrgbChannel(value: number): number {
  const clamped = Math.max(0, Math.min(1, value));
  const srgb = clamped <= 0.0031308 ? 12.92 * clamped : 1.055 * clamped ** (1 / 2.4) - 0.055;
  return Math.round(Math.max(0, Math.min(1, srgb)) * 255);
}

/** sRGB → OKLab（Björn Ottosson） */
function rgbToOklab(r: number, g: number, b: number): [number, number, number] {
  const lr = srgbChannelToLinear(r);
  const lg = srgbChannelToLinear(g);
  const lb = srgbChannelToLinear(b);

  const l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
  const m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
  const s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

  const lRoot = Math.cbrt(l);
  const mRoot = Math.cbrt(m);
  const sRoot = Math.cbrt(s);

  return [
    0.2104542553 * lRoot + 0.793617785 * mRoot - 0.0040720468 * sRoot,
    1.9779984951 * lRoot - 2.428592205 * mRoot + 0.4505937099 * sRoot,
    0.0259040371 * lRoot + 0.7827717662 * mRoot - 0.808675766 * sRoot,
  ];
}

/** OKLab → sRGB */
function oklabToRgb(L: number, a: number, b: number): [number, number, number] {
  const lRoot = L + 0.3963377774 * a + 0.2158037573 * b;
  const mRoot = L - 0.1055613458 * a - 0.0638541728 * b;
  const sRoot = L - 0.0894841775 * a - 1.291485548 * b;

  const l = lRoot ** 3;
  const m = mRoot ** 3;
  const s = sRoot ** 3;

  const lr = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
  const lg = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
  const lb = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s;

  return [linearToSrgbChannel(lr), linearToSrgbChannel(lg), linearToSrgbChannel(lb)];
}

function oklabToOklch(L: number, a: number, b: number): [number, number, number] {
  const chroma = Math.sqrt(a * a + b * b);
  let hue = (Math.atan2(b, a) * 180) / Math.PI;
  if (hue < 0) {
    hue += 360;
  }
  return [L, chroma, hue];
}

function oklchToOklab(L: number, chroma: number, hue: number): [number, number, number] {
  const hueRad = (hue * Math.PI) / 180;
  return [L, chroma * Math.cos(hueRad), chroma * Math.sin(hueRad)];
}

/**
 * インタラクション（hover 等）向けの OKLCH 明度デルタの目安。
 * 知覚的に均一なスペースで L を下げる（HSL darken より色相ずれが少ない）。
 */
export const INTERACTION_OKLCH_LIGHTNESS_DELTA = -0.06;

/**
 * OKLCH の明度だけを増減する（chroma / hue は維持）。
 * @param color - 16進数カラーコード
 * @param delta - OKLCH L の増減（例: hover 向けに -0.06）
 */
export function adjustOklchLightness(
  color: string,
  delta: number = INTERACTION_OKLCH_LIGHTNESS_DELTA
): string {
  const [r, g, b] = hexToRgb(color);
  const [L, chroma, hue] = oklabToOklch(...rgbToOklab(r, g, b));
  const nextL = Math.max(0, Math.min(1, L + delta));
  const [nextR, nextG, nextB] = oklabToRgb(...oklchToOklab(nextL, chroma, hue));
  return rgbToHex(nextR, nextG, nextB);
}

/**
 * 16進色を白と混ぜる（薄い面・枠の生成用）
 * @param color - 16進数カラーコード
 * @param whiteAmount - 白の割合 (0-100)。大きいほど明るい
 */
export function mixWithWhite(color: string, whiteAmount: number): string {
  const [r, g, b] = hexToRgb(color);
  const t = Math.max(0, Math.min(100, whiteAmount)) / 100;
  return rgbToHex(r + (255 - r) * t, g + (255 - g) * t, b + (255 - b) * t);
}

/**
 * 16進色にアルファを付けた rgba 文字列を返す（ダークの薄い面・枠用）
 * @param color - 16進数カラーコード
 * @param alpha - 不透明度 (0-1)
 */
export function withAlpha(color: string, alpha: number): string {
  const [r, g, b] = hexToRgb(color);
  const a = Math.max(0, Math.min(1, alpha));
  const rounded = Math.round(a * 1000) / 1000;
  return `rgba(${r}, ${g}, ${b}, ${rounded})`;
}

export type ColorSurfaceToken = {
  main: { value: string };
  soft: { value: { base: string; _dark: string } };
  softBorder: { value: { base: string; _dark: string } };
};

export type CreateColorSurfaceOptions = {
  /** Light 背景の白混ぜ割合 (0-100)。default 88 */
  softWhiteAmount?: number;
  /** Light 枠の白混ぜ割合 (0-100)。default 70 */
  softBorderWhiteAmount?: number;
  /** Dark 背景の alpha (0-1)。default 0.12 */
  softDarkAlpha?: number;
  /** Dark 枠の alpha (0-1)。default 0.35 */
  softBorderDarkAlpha?: number;
};

/**
 * 任意のベース色から、ライト/ダーク両対応の面・枠トークンを生成する。
 * status / brand など意味色を問わず利用できる。
 *
 * @example
 * createColorSurface("#28a745")
 * // => { main, soft: { base, _dark }, softBorder: { base, _dark } }
 *
 * brand への適用例:
 * soft: createColorSurface(baseColors.brand).soft
 */
export function createColorSurface(
  color: string,
  options: CreateColorSurfaceOptions = {}
): ColorSurfaceToken {
  const {
    softWhiteAmount = 88,
    softBorderWhiteAmount = 70,
    softDarkAlpha = 0.12,
    softBorderDarkAlpha = 0.35,
  } = options;

  return {
    main: { value: color },
    soft: {
      value: {
        base: mixWithWhite(color, softWhiteAmount),
        _dark: withAlpha(color, softDarkAlpha),
      },
    },
    softBorder: {
      value: {
        base: mixWithWhite(color, softBorderWhiteAmount),
        _dark: withAlpha(color, softBorderDarkAlpha),
      },
    },
  };
}

/** createColorSurface の別名（status 用途でも同じ） */
export const createStatusSurface = createColorSurface;

/**
 * 色の明度を調整する
 * @param color - 16進数カラーコード
 * @param amount - 調整量 (-100 ~ 100)
 * @returns 調整後の16進数カラーコード
 */
export function adjustLightness(color: string, amount: number): string {
  const [r, g, b] = hexToRgb(color);
  const [h, s, l] = rgbToHsl(r, g, b);

  // 明度を調整（0-100の範囲に収める）
  const newL = Math.max(0, Math.min(100, l + amount));

  const [newR, newG, newB] = hslToRgb(h, s, newL);
  return rgbToHex(newR, newG, newB);
}

/**
 * 色の彩度を調整する
 * @param color - 16進数カラーコード
 * @param amount - 調整量 (-100 ~ 100)
 * @returns 調整後の16進数カラーコード
 */
export function adjustSaturation(color: string, amount: number): string {
  const [r, g, b] = hexToRgb(color);
  const [h, s, l] = rgbToHsl(r, g, b);

  // 彩度を調整（0-100の範囲に収める）
  const newS = Math.max(0, Math.min(100, s + amount));

  const [newR, newG, newB] = hslToRgb(h, newS, l);
  return rgbToHex(newR, newG, newB);
}

/**
 * 色相を調整する
 * @param color - 16進数カラーコード
 * @param amount - 調整量 (-360 ~ 360)
 * @returns 調整後の16進数カラーコード
 */
export function adjustHue(color: string, amount: number): string {
  const [r, g, b] = hexToRgb(color);
  const [h, s, l] = rgbToHsl(r, g, b);

  // 色相を調整（0-360の範囲で循環）
  let newH = (h + amount) % 360;
  if (newH < 0) newH += 360;

  const [newR, newG, newB] = hslToRgb(newH, s, l);
  return rgbToHex(newR, newG, newB);
}

/**
 * 色のシェード（暗い色）を作成する
 * @param color - 16進数カラーコード
 * @param amount - シェードの強さ (0-100)
 * @returns シェード後の16進数カラーコード
 */
export function createShade(color: string, amount: number): string {
  return adjustLightness(color, -amount);
}

/**
 * 色のティント（明るい色）を作成する
 * @param color - 16進数カラーコード
 * @param amount - ティントの強さ (0-100)
 * @returns ティント後の16進数カラーコード
 */
export function createTint(color: string, amount: number): string {
  return adjustLightness(color, amount);
}
