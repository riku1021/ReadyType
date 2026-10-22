import { css } from "styled-system/css";

export const band = css({
  bgGradient: "brand",
  borderRadius: "44px",
  color: "white",
  textAlign: "center",
  py: "76px",
  px: "24px",
  position: "relative",
  overflow: "hidden",
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
  position: "absolute",
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
  fontSize: "28px",
  boxShadow: "key",
  animation: "bob 5s ease-in-out infinite",
  zIndex: "1",
  _bp900: { display: "none" },
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
  fontSize: "15px",
});

export const title = css({
  color: "white",
  textAlign: "center",
  fontSize: "clamp(28px, 4.2vw, 42px)",
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  mb: "14px",
});

export const copy = css({ mb: "30px", opacity: "0.92" });
