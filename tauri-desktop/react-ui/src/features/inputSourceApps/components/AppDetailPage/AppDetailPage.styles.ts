import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  minHeight: "0",
  flex: "1",
});

export const detailHeader = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  flexShrink: "0",
  paddingInline: "sm",
  paddingBlock: "sm",
});

/** カード内のサブセクション（デフォルト入力言語など） */
export const cardSection = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  paddingInline: "sm",
  paddingBlock: "sm",
});

export const detailIdentity = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minWidth: "0",
});

export const detailTitleBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "4px",
  minWidth: "0",
});

export const detailName = css({
  margin: "0",
  fontSize: "lg",
  fontWeight: "semibold",
  color: "text.main",
});

export const detailPath = css({
  margin: "0",
  fontSize: "xs",
  color: "text.secondary",
  wordBreak: "break-all",
});

export const fieldBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  flexShrink: "0",
});

export const fieldLabel = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const fieldHelp = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "xs",
  margin: "0",
  padding: "sm",
  borderRadius: "10px",
  backgroundColor: "status.info.soft",
  color: "text.main",
  fontSize: "sm",
  lineHeight: "1.5",
});

export const fieldHelpIcon = css({
  fontSize: "md",
  flexShrink: "0",
  color: "status.info.main",
  marginTop: "0.1em",
});

export const th = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "left",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const thRule = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineStart: "sm",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "left",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const thActions = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineEnd: "sm",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "right",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const row = css({
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
});

export const td = css({
  paddingBlock: "sm",
  paddingInline: "xs",
  verticalAlign: "middle",
  minWidth: "0",
  backgroundColor: "transparent",
});

export const tdRule = css({
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineStart: "sm",
  verticalAlign: "middle",
  minWidth: "0",
  backgroundColor: "transparent",
});

export const tdActions = css({
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineEnd: "sm",
  verticalAlign: "middle",
  minWidth: "0",
  backgroundColor: "transparent",
  textAlign: "right",
});

export const ruleCell = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minWidth: "0",
});

export const ruleText = css({
  display: "flex",
  flexDirection: "column",
  gap: "4px",
  minWidth: "0",
});

export const ruleTitle = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const appMeta = css({
  margin: "0",
  fontSize: "xs",
  color: "text.secondary",
  wordBreak: "break-all",
});

export const iconButton = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "32px",
  height: "32px",
  borderRadius: "8px",
  border: "none",
  backgroundColor: "transparent",
  color: "text.secondary",
  fontSize: "lg",
  cursor: "pointer",
  _hover: {
    backgroundColor: "background.sub",
    color: "text.main",
  },
  _disabled: {
    opacity: 0.5,
    cursor: "not-allowed",
  },
});

export const empty = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
});
