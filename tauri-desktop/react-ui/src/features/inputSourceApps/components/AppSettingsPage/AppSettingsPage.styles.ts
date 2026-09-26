import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  minHeight: "0",
  flex: "1",
});

export const intro = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
});

export const layout = css({
  display: "grid",
  gridTemplateColumns: "minmax(220px, 280px) minmax(0, 1fr)",
  gap: "sm",
  minHeight: "0",
  flex: "1",
  mdDown: {
    gridTemplateColumns: "1fr",
  },
});

export const appList = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  padding: "sm",
  borderRadius: "12px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  minHeight: "0",
  overflowY: "auto",
});

export const appListHeader = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  marginBottom: "xs",
});

export const sectionTitle = css({
  margin: "0",
  fontSize: "md",
  fontWeight: "semibold",
  color: "text.main",
});

export const appRow = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  width: "100%",
  padding: "xs sm",
  borderRadius: "10px",
  backgroundColor: "transparent",
  _hover: {
    backgroundColor: "background.main",
  },
});

export const appRowSelected = css({
  backgroundColor: "background.main",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
});

export const appRowMain = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minWidth: "0",
  flex: "1",
  padding: "0",
  border: "none",
  backgroundColor: "transparent",
  color: "text.main",
  cursor: "pointer",
  textAlign: "left",
});

export const appName = css({
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

export const detail = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  padding: "sm",
  borderRadius: "12px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  minHeight: "0",
  overflowY: "auto",
});

export const detailHeader = css({
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "sm",
});

export const detailTitleBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minWidth: "0",
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

export const select = css({
  width: "100%",
  maxWidth: "280px",
  paddingInline: "sm",
  paddingBlock: "xs",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.main",
  color: "text.main",
  fontSize: "sm",
});

export const ruleList = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
});

export const ruleRow = css({
  display: "grid",
  gridTemplateColumns: "minmax(0, 1fr) auto auto",
  alignItems: "center",
  gap: "sm",
  padding: "xs sm",
  borderRadius: "10px",
  backgroundColor: "background.main",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
  mdDown: {
    gridTemplateColumns: "1fr",
  },
});

export const ruleText = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minWidth: "0",
});

export const hint = css({
  margin: "0",
  padding: "sm",
  borderRadius: "10px",
  backgroundColor: "background.main",
  color: "text.secondary",
  fontSize: "xs",
  lineHeight: "1.5",
});

export const empty = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
});

export const toolbar = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "xs",
});
