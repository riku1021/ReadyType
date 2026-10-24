import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

/** 固定ヘッダー + スクロール本体 + 任意フッターのパネル殻 */
export const panel = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "0",
  minWidth: "0",
  minHeight: "0",
  flex: "1",
  height: "100%",
  overflow: "hidden",
  padding: "0",
  // SettingsSection.card と同じ枠線＋影＋角丸。枠を border にすると影の角が四角になる
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  borderRadius: "12px",
});

export const tableHeader = css({
  flexShrink: "0",
  width: "100%",
});

export const tableWrap = css({
  flex: "1",
  minHeight: "0",
  overflow: "auto",
  width: "100%",
  scrollbarGutter: "stable",
});

export const table = css({
  width: "100%",
  borderCollapse: "collapse",
  tableLayout: "fixed",
});

export const empty = css({
  margin: "0",
  padding: "sm",
  fontSize: "sm",
  color: "text.sub",
});

/** 最終行の下線はカード下端に余白として残る。項目間だけ線を引く */
export const dataRow = css({
  _last: {
    borderBottomWidth: "0",
  },
});

/** 行選択できる表。ホバー色は SettingItem に揃える */
export const interactiveRow = "group";

export const interactiveCell = css({
  transitionProperty: "background-color",
  transitionDuration: "150ms",
  transitionTimingFunction: "ease",
  _groupHover: {
    backgroundColor: "background.hover",
  },
  "tr.group[aria-selected=true] &": {
    _first: {
      boxShadow: "inset 3px 0 0 {colors.brand.main}",
    },
  },
});
