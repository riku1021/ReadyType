/**
 * Z-Index使用例
 * 各レイヤーの使用方法とベストプラクティスを示す
 */

import { css } from "@/styled-system/css";
import { type ZIndexLayerType, zIndexLayers } from "@/styles/tokens/zindex";

// 直接値を参照する方法
export const headerStyle = css({
  position: "sticky",
  top: 0,
  zIndex: zIndexLayers.header,
  backgroundColor: "white",
  borderBottom: "1px solid #eee",
});

/**
 * 動的なz-index管理の例
 */

// z-indexを動的に設定するヘルパー関数
export const createLayeredStyle = (baseLayer: ZIndexLayerType, offset: number = 0) => {
  return {
    zIndex: zIndexLayers[baseLayer] + offset,
  };
};

// 使用例: モーダルの中でさらに階層を作る
export const nestedModalStyle = css({
  position: "fixed",
  top: "10%",
  left: "10%",
  width: "80%",
  height: "80%",
  backgroundColor: "white",
  borderRadius: "8px",
  ...createLayeredStyle("modal", 1), // 101
});
