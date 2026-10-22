/**
 * LP のダウンロード先。
 * ファイル名は `.github/workflows/desktop-build.yml` が Release に載せる名前と揃える。
 * macOS は Apple Silicon（mac）と Intel（macIntel）で DMG を分ける。
 * ブラウザからは両者を安定して判別できないので、自動選択は Apple Silicon のまま。
 */

export type DesktopOs = "mac" | "macIntel" | "windows" | "linux";

export const desktopReleasePageUrl = "https://github.com/riku1021/ReadyType/releases/latest";

const downloadBase = `${desktopReleasePageUrl}/download`;

export const desktopInstallerFile: Record<DesktopOs, string> = {
  mac: "ReadyType-aarch64.dmg",
  macIntel: "ReadyType-x64.dmg",
  windows: "ReadyType-x64-setup.exe",
  linux: "ReadyType-x64.AppImage",
};

const installerLabel: Record<DesktopOs, string> = {
  mac: "⬇ macOS（Apple Silicon）用をダウンロード",
  macIntel: "⬇ macOS（Intel）用をダウンロード",
  windows: "⬇ Windows 用をダウンロード",
  linux: "⬇ Linux 用をダウンロード",
};

export interface DesktopOsSignals {
  userAgent: string;
  platform: string;
  uaPlatform?: string;
  mobile?: boolean;
  maxTouchPoints?: number;
}

export function desktopInstallerUrl(os: DesktopOs): string {
  return `${downloadBase}/${desktopInstallerFile[os]}`;
}

/** デスクトップ OS。スマホ・タブレット・判別できない環境は null。 */
export function detectDesktopOs(signals: DesktopOsSignals): DesktopOs | null {
  const ua = signals.userAgent.toLowerCase();
  if (signals.mobile) return null;
  if (/android|iphone|ipad|ipod/.test(ua)) return null;
  if (ua.includes("cros")) return null;

  const platform = (signals.uaPlatform ?? signals.platform).toLowerCase();
  const mac = platform.includes("mac") || ua.includes("mac");
  if (mac && (signals.maxTouchPoints ?? 0) > 1) return null;
  if (mac) return "mac";
  if (platform.includes("win") || ua.includes("windows")) return "windows";
  if (platform.includes("linux") || ua.includes("linux")) return "linux";
  return null;
}

function readSignals(): DesktopOsSignals {
  const signals: DesktopOsSignals = {
    userAgent: navigator.userAgent,
    platform: navigator.platform,
    maxTouchPoints: navigator.maxTouchPoints,
  };
  if (!("userAgentData" in navigator)) return signals;
  const data = navigator.userAgentData;
  if (typeof data !== "object" || data === null) return signals;
  if ("platform" in data && typeof data.platform === "string") {
    signals.uaPlatform = data.platform;
  }
  if ("mobile" in data && data.mobile === true) {
    signals.mobile = true;
  }
  return signals;
}

/** 判定できたときだけ、primary の href をその OS のインストーラにする。 */
export function bindDesktopDownload(): void {
  const os = detectDesktopOs(readSignals());
  if (os === null) return;

  const url = desktopInstallerUrl(os);
  const label = installerLabel[os];
  for (const anchor of document.querySelectorAll<HTMLAnchorElement>("[data-download='primary']")) {
    anchor.href = url;
    if (anchor.hasAttribute("data-download-label")) {
      anchor.textContent = label;
    }
  }
}
