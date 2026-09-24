////////////////////////////////////////////
// Z-Index トークン定義
// レイヤー階層を管理するためのz-index値
////////////////////////////////////////////

// z-indexトークン
export const zIndexTokens = {
  modal: { value: 100 },
  floating: { value: 50 },
  header: { value: 40 },
  drawer: { value: 30 },
  footer: { value: 20 },
  front: { value: 10 },
  default: { value: 1 },
  background: { value: -10 },
} as const;

export const zIndexLayers = {
  modal: zIndexTokens.modal.value,
  floating: zIndexTokens.floating.value,
  header: zIndexTokens.header.value,
  drawer: zIndexTokens.drawer.value,
  footer: zIndexTokens.footer.value,
  front: zIndexTokens.front.value,
  default: zIndexTokens.default.value,
  background: zIndexTokens.background.value,
} as const;

export type ZIndexLayerType = keyof typeof zIndexLayers;
