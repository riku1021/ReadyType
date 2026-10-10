import { css, cva } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  maxWidth: "640px",
});

export const card = cva({
  base: css.raw(surfaceSub, {
    display: "flex",
    flexDirection: "column",
    gap: "0",
    padding: "sm",
    borderRadius: "12px",
    minInlineSize: "0",
    boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  }),
  variants: {
    tone: {
      default: {},
      alert: {
        backgroundColor: "status.error.soft",
        boxShadow:
          "inset 0 0 0 1px {colors.status.error.softBorder}, 0 2px 8px {colors.shadow.primary}",
      },
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

export const topRow = css({
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "sm",
  minInlineSize: "0",
});

export const mainColumn = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "sm",
  minInlineSize: "0",
  flex: "1",
});

export const featureIcon = cva({
  base: {
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
  },
  variants: {
    tone: {
      default: {},
      alert: {
        backgroundColor: "status.error.main",
      },
    },
  },
  defaultVariants: {
    tone: "default",
  },
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

export const requiredStatus = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "xs",
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "status.error.main",
  lineHeight: "1.3",
});

export const description = css({
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

export const grantedBadge = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "xs",
  paddingInline: "sm",
  paddingBlock: "2px",
  borderRadius: "999px",
  backgroundColor: "status.success.soft",
  boxShadow: "inset 0 0 0 1px {colors.status.success.softBorder}",
  color: "status.success.main",
  fontSize: "sm",
  fontWeight: "semibold",
  lineHeight: "1.4",
  whiteSpace: "nowrap",
});

export const divider = css({
  height: "1px",
  marginBlock: "sm",
  marginInline: "0",
  border: "none",
  backgroundColor: "border.primary",
});

export const privacyRow = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "xs",
  minInlineSize: "0",
});

export const privacyIcon = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  marginTop: "2px",
  color: "icon.secondary",
  fontSize: "md",
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
