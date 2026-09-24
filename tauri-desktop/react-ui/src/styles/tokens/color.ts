import { adjustLightness, adjustOklchLightness, createColorSurface } from "../utils/color-utils";

/**
 * 色のトークン定義
 * @description PandaCSSのトークンシステムを使用したカラー定義
 */

// 基本カラー値の定義
const baseColors = {
  /** 旧オレンジ案: "#ff6800" */
  brand: "#007bff",
  status: {
    success: "#28a745",
    warning: "#ffc107",
    /** 失敗・バリデーション。注意喚起（attention）と同じ赤を共有するが役割は分ける */
    error: "#f8313e",
    /** 緊急・今日・推奨など。error ではない強調 */
    attention: "#f8313e",
  },
  neutral: {
    white: "#ffffff",
    black: "#000000",
  },
} as const;

/**
 * PandaCSSのカラートークン定義
 * @description semantic.colorsに定義するカラートークン
 */
export const colorTokens = {
  // ブランドカラー（単一系統。secondary 廃止後は brand.* で直接参照）
  brand: {
    main: { value: baseColors.brand },
    light: { value: adjustLightness(baseColors.brand, 48.5) },
    /** インタラクション用。OKLCH で明度だけ下げた派生色 */
    hover: { value: adjustOklchLightness(baseColors.brand) },
    soft: createColorSurface(baseColors.brand).soft,
    softBorder: createColorSurface(baseColors.brand).softBorder,
  },

  // 状態カラー（任意ベース色から soft / softBorder を生成）
  status: {
    success: createColorSurface(baseColors.status.success),
    warning: createColorSurface(baseColors.status.warning),
    error: createColorSurface(baseColors.status.error),
    attention: createColorSurface(baseColors.status.attention),
  },

  // 基本カラー
  white: { value: baseColors.neutral.white },
  black: { value: baseColors.neutral.black },

  // テキストカラー（セマンティックトークン）
  text: {
    main: {
      value: {
        base: "#444444",
        _dark: "#e0e0e0",
      },
    },
    sub: {
      value: {
        base: "#666666",
        _dark: "#e5e5e5",
      },
    },
    placeholder: {
      value: {
        base: "#a9a9a9",
        _dark: "#999999",
      },
    },
    disabled: {
      value: {
        base: "#333333",
        _dark: "#666666",
      },
    },
  },

  // 背景カラー（セマンティックトークン）
  background: {
    main: {
      value: {
        base: "#EAEAEA",
        _dark: "#222222",
      },
    },
    sub: {
      value: {
        base: baseColors.neutral.white,
        _dark: "#282828",
      },
    },
    hover: {
      value: {
        base: "rgba(0, 0, 0, 0.05)",
        _dark: "rgba(255, 255, 255, 0.1)",
      },
    },
    overlay: {
      value: {
        base: "rgba(0, 0, 0, 0.5)",
        _dark: "rgba(0, 0, 0, 0.7)",
      },
    },
  },

  // ボーダーカラー（セマンティックトークン）
  border: {
    primary: {
      value: {
        base: "#eeeeee",
        _dark: "#333333",
      },
    },
    secondary: {
      value: {
        base: "#f0f0f0",
        _dark: "#2a2a2a",
      },
    },
    focus: {
      value: baseColors.brand,
    },
  },

  // シャドウカラー（セマンティックトークン）
  shadow: {
    primary: {
      value: {
        base: "rgba(0, 0, 0, 0.1)",
        _dark: "rgba(0, 0, 0, 0.3)",
      },
    },
    secondary: {
      value: {
        base: "rgba(0, 0, 0, 0.05)",
        _dark: "rgba(0, 0, 0, 0.2)",
      },
    },
  },

  // アイコンカラー（セマンティックトークン）
  icon: {
    primary: {
      value: {
        base: "#444444",
        _dark: "#f5f5f5",
      },
    },
    secondary: {
      value: {
        base: "#666666",
        _dark: "#e0e0e0",
      },
    },
    disabled: {
      value: {
        base: "#999999",
        _dark: "#666666",
      },
    },
  },
} as const;

// 型定義
export type ColorTokens = typeof colorTokens;
