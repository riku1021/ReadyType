import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  minHeight: "0",
  flex: "1",
});

export const detail = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "md",
  padding: "md",
  borderRadius: "12px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  minHeight: "0",
  overflowY: "auto",
});

export const detailHeader = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
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

export const sectionHeader = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
});

export const fieldBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
});

export const fieldLabel = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const fieldHelp = css({
  margin: "0",
  fontSize: "xs",
  color: "text.secondary",
});

export const infoBox = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "xs",
  padding: "sm",
  borderRadius: "10px",
  backgroundColor: "background.main",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
  color: "text.secondary",
  fontSize: "xs",
  lineHeight: "1.5",
});

export const infoIcon = css({
  fontSize: "md",
  flexShrink: "0",
  color: "text.secondary",
  marginTop: "0.1em",
});

export const rulesTable = css({
  height: "auto",
  flex: "none",
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
