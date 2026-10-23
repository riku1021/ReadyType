import { css, cva } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  maxWidth: "640px",
});

export const card = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  padding: "sm",
  borderRadius: "12px",
  minInlineSize: "0",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
});

export const topRow = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  minInlineSize: "0",
});

export const mainColumn = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minInlineSize: "0",
  flex: "1",
});

export const featureIcon = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  width: "40px",
  height: "40px",
  borderRadius: "10px",
  fontSize: "22px",
  color: "white",
  backgroundColor: "brand.main",
});

export const textStack = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minInlineSize: "0",
  flex: "1",
});

export const title = css({
  margin: "0",
  fontSize: "md",
  fontWeight: "semibold",
  color: "text.main",
  lineHeight: "1.3",
});

export const notice = cva({
  base: {
    display: "flex",
    alignItems: "flex-start",
    gap: "xs",
    margin: "0",
    padding: "sm",
    borderRadius: "10px",
    color: "text.main",
    fontSize: "sm",
    lineHeight: "1.5",
  },
  variants: {
    tone: {
      info: {
        backgroundColor: "status.info.soft",
      },
      error: {
        backgroundColor: "status.error.soft",
      },
    },
  },
  defaultVariants: {
    tone: "info",
  },
});

export const noticeIcon = cva({
  base: {
    fontSize: "md",
    flexShrink: "0",
    marginTop: "0.1em",
  },
  variants: {
    tone: {
      info: {
        color: "status.info.main",
      },
      error: {
        color: "status.error.main",
      },
    },
  },
  defaultVariants: {
    tone: "info",
  },
});

export const divider = css({
  height: "1px",
  margin: "0",
  border: "none",
  backgroundColor: "border.primary",
});

export const privacyStack = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minInlineSize: "0",
});

export const privacyTitle = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
  lineHeight: "1.3",
});

export const privacyText = css({
  margin: "0",
  fontSize: "sm",
  color: "text.sub",
  lineHeight: "1.5",
});

export const actionsColumn = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "flex-end",
  gap: "xs",
  flexShrink: "0",
});

export const statusBadge = cva({
  base: {
    display: "inline-flex",
    alignItems: "center",
    gap: "xs",
    paddingInline: "sm",
    paddingBlock: "2px",
    borderRadius: "999px",
    fontSize: "sm",
    fontWeight: "semibold",
    lineHeight: "1.4",
    whiteSpace: "nowrap",
  },
  variants: {
    tone: {
      success: {
        backgroundColor: "status.success.soft",
        boxShadow: "inset 0 0 0 1px {colors.status.success.softBorder}",
        color: "status.success.main",
      },
      error: {
        backgroundColor: "status.error.soft",
        boxShadow: "inset 0 0 0 1px {colors.status.error.softBorder}",
        color: "status.error.main",
      },
    },
  },
});
