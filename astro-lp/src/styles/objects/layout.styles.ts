import { css } from "styled-system/css";

export const wrap = css({
  maxWidth: "wrap",
  mx: "auto",
  px: "gutter",
});

export const section = css({
  position: "relative",
  py: "section",
  _bp900: { py: "sectionCompact" },
});

export const sectionFlush = css({
  position: "relative",
  pt: "0",
  pb: "section",
  _bp900: { pb: "sectionCompact" },
});
