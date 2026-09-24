import { cva } from "@/styled-system/css";

/**
 * ガラス（glassmorphism）Material。
 *
 * 見た目の共通部分だけを担い、形状・操作・レイアウトは呼び出し側が css を合成して足す。
 * 既存の `styles/objects/surface.ts`（面コンテキスト）と同じく「見た目レイヤ」の位置づけ。
 *
 * lightTone（ライトテーマのみ）:
 *  - frost … 白フロスト＋縁・影強化（背後の UI がぼかされて見える前提）
 *  - darkTint … ライト背景向けダークガラス
 *  - solid … ライトではソリッドカード、ガラスはダーク専用
 *
 * ダークテーマ側の塗り・影は lightTone によらず共通。
 *
 * @see https://codepen.io/vladracoare/pen/wBKjLVV
 * @see https://csstools.io/blog/glassmorphism-guide
 * @see https://pixcode.io/en/blog/css-glassmorphism-2025/
 */

const IRIDESCENT_GRADIENT =
  "linear-gradient(90deg, #ebe8f5 0%, #eec3ff 30.34%, #ffdfcd 51.92%, #d1ffd1 75.48%, #7fcffb 100%)";

/** ダークテーマ共通のガラス塗り（lightTone で変えず維持） */
const DARK_BACKGROUND =
  "linear-gradient(180deg, rgba(52, 52, 58, 0.78) 0%, rgba(34, 34, 38, 0.78) 57.59%, rgba(44, 44, 50, 0.78) 100%)";

const DARK_BOX_SHADOW = `
  inset 0 -3px 4px {colors.glass.inset},
  inset 0 -27px 12px {colors.glass.highlight},
  inset 1px 1px 1px {colors.glass.highlight},
  inset -1px -1px 1px {colors.glass.highlight},
  inset 0 4px 4px {colors.glass.highlight},
  inset 0 4px 8px {colors.glass.highlight},
  0 4px 12px {colors.glass.drop}
`;

export const glassRecipe = cva({
  base: {
    // position は呼び出し側が指定する（fixed / absolute / relative）
    isolation: "isolate",
    backdropFilter: "blur(10px)",
  },

  variants: {
    radius: {
      card: {
        borderRadius: "24px",
      },
      button: {
        borderRadius: "9999px",
      },
      menu: {
        borderRadius: "12px",
      },
    },

    intensity: {
      subtle: {
        backdropFilter: "blur(6px)",
      },
      default: {
        backdropFilter: "blur(10px)",
      },
      strong: {
        backdropFilter: "blur(14px)",
      },
    },

    /**
     * ライトテーマ向けの見え方。ダーク側は常に同じガラス。
     */
    lightTone: {
      /** 白フロスト＋縁・影。背後に UI があると blur が効く */
      frost: {
        background: {
          base: "linear-gradient(180deg, rgba(255, 255, 255, 0.42) 0%, rgba(248, 250, 252, 0.32) 100%)",
          _dark: DARK_BACKGROUND,
        },
        boxShadow: {
          base: `
            inset 0 0 0 1px rgba(15, 23, 42, 0.10),
            inset 0 1px 0 rgba(255, 255, 255, 0.85),
            0 8px 28px rgba(15, 23, 42, 0.14)
          `,
          _dark: DARK_BOX_SHADOW,
        },
        backdropFilter: {
          base: "blur(14px) saturate(140%)",
          _dark: "blur(10px)",
        },
      },

      /** ライト背景向けダークガラス（コントラスト重視） */
      darkTint: {
        background: {
          base: "linear-gradient(180deg, rgba(70, 74, 84, 0.42) 0%, rgba(55, 58, 68, 0.36) 100%)",
          _dark: DARK_BACKGROUND,
        },
        boxShadow: {
          base: `
            inset 0 0 0 1px rgba(255, 255, 255, 0.18),
            inset 0 1px 0 rgba(255, 255, 255, 0.28),
            0 8px 28px rgba(15, 23, 42, 0.16)
          `,
          _dark: DARK_BOX_SHADOW,
        },
        backdropFilter: {
          base: "blur(16px) saturate(150%)",
          _dark: "blur(10px)",
        },
        color: {
          base: "white",
          _dark: "inherit",
        },
      },

      /** ライトはソリッドカード、ガラスはダーク専用 */
      solid: {
        background: {
          base: "{colors.background.sub}",
          _dark: DARK_BACKGROUND,
        },
        boxShadow: {
          base: `
            inset 0 0 0 1px {colors.border.primary},
            0 4px 16px {colors.shadow.primary}
          `,
          _dark: DARK_BOX_SHADOW,
        },
        backdropFilter: {
          base: "none",
          _dark: "blur(10px)",
        },
      },
    },

    /**
     * 虹色の屈折・外側 glow。
     * CodePen の Button 表現向け。Popover / Menu では off にする。
     */
    sheen: {
      on: {
        _before: {
          content: '""',
          position: "absolute",
          left: "0",
          bottom: "0",
          width: "100%",
          height: "33.33%",
          background: IRIDESCENT_GRADIENT,
          boxShadow: "inset 0 -9px 8px {colors.glass.highlight}",
          filter: "blur(8px)",
          transform: "rotate(-15deg)",
          pointerEvents: "none",
          zIndex: "-1",
          borderRadius: "inherit",
        },
        _after: {
          content: '""',
          position: "absolute",
          left: "-8px",
          bottom: "-8px",
          width: "100%",
          height: "33.33%",
          background: IRIDESCENT_GRADIENT,
          opacity: 0.7,
          filter: "blur(8px)",
          transform: "rotate(-15deg) scaleX(0.8) scaleY(1.2)",
          transformOrigin: "top",
          pointerEvents: "none",
          zIndex: "-2",
        },
      },
      off: {
        _before: {
          content: "none",
          display: "none",
        },
        _after: {
          content: "none",
          display: "none",
        },
      },
    },
  },

  compoundVariants: [
    {
      sheen: "on",
      intensity: "subtle",
      css: {
        _before: { opacity: 0.45 },
        _after: { opacity: 0.35 },
      },
    },
    {
      sheen: "on",
      intensity: "strong",
      css: {
        _before: { opacity: 1, filter: "blur(10px)" },
        _after: { opacity: 0.9, filter: "blur(10px)" },
      },
    },
  ],

  defaultVariants: {
    radius: "card",
    intensity: "default",
    sheen: "on",
    lightTone: "frost",
  },
});

export type GlassLightTone = "frost" | "darkTint" | "solid";
