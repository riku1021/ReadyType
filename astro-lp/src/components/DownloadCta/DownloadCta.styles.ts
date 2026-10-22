import { css } from "styled-system/css";

export const band = css({
  bgGradient: "brand",
  borderRadius: "clamp(28px, 4vw, 44px)",
  color: "white",
  textAlign: "center",
  py: "clamp(48px, 8vw, 76px)",
  px: "gutter",
  position: "relative",
  overflow: "hidden",
  containerType: "inline-size",
  "&::before, &::after": {
    content: '""',
    position: "absolute",
    borderRadius: "50%",
    bg: "rgba(255, 255, 255, 0.12)",
  },
  "&::before": { width: "300px", height: "300px", left: "-90px", top: "-100px" },
  "&::after": { width: "240px", height: "240px", right: "-70px", bottom: "-90px" },
  "& > *": { position: "relative", zIndex: "1" },
});

export const key = css({
  // .band > * の position: relative より詳細度を上げる。
  // example.html の .kc { position: absolute !important } と同じ役割。
  "&&": { position: "absolute" },
  width: "68px",
  height: "68px",
  borderRadius: "20px",
  bg: "rgba(255, 255, 255, 0.18)",
  backdropFilter: "blur(6px)",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "rgba(255, 255, 255, 0.45)",
  display: "grid",
  placeItems: "center",
  fontFamily: "latin",
  fontWeight: "black",
  fontSize: "1.75rem",
  boxShadow: "key",
  animation: "bob 5s ease-in-out infinite",
  zIndex: "1",
  _bandTight: { display: "none" },
});

export const keyPlace = [
  css({ left: "8%", top: "18%" }),
  css({ right: "9%", top: "14%", animationDelay: "-1.5s" }),
  css({ left: "14%", bottom: "14%", animationDelay: "-3s" }),
  css({ right: "15%", bottom: "16%", animationDelay: "-0.8s" }),
];

export const kicker = css({
  display: "inline-block",
  bg: "rgba(255, 255, 255, 0.2)",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "rgba(255, 255, 255, 0.4)",
  px: "18px",
  py: "6px",
  borderRadius: "pill",
  fontWeight: "black",
  mb: "16px",
  fontSize: "0.9375rem",
});

export const title = css({
  color: "white",
  textAlign: "center",
  fontSize: "clamp(1.75rem, 4.2vw, 2.625rem)",
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  mb: "14px",
});

export const copy = css({ mb: "30px", opacity: "0.92" });

export const otherOs = css({
  display: "flex",
  justifyContent: "center",
  flexWrap: "wrap",
  gap: "8px 20px",
  mt: "18px",
  fontSize: "0.875rem",
  fontWeight: "black",
});

export const otherOsLink = css({
  color: "white",
  textDecoration: "underline",
  textUnderlineOffset: "3px",
});
