export type UpdateErrorKey = "desktopOnly" | "checkFailed" | "downloadFailed" | "installFailed";

export type UpdatePhase =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "available"; version: string }
  | { kind: "downloading"; version: string }
  | { kind: "downloaded"; version: string }
  | { kind: "installing"; version: string }
  | { kind: "error"; messageKey: UpdateErrorKey };

export type UpdatePreferences = {
  autoCheck: boolean;
  autoDownload: boolean;
  lastCheckedAt: string | null;
};

export type AppUpdateSnapshot = {
  preferences: UpdatePreferences;
  phase: UpdatePhase;
  installReady: boolean;
};

export const APP_UPDATE_EVENT = "readytype://app-update";

const PHASE_NAMES = [
  "idle",
  "checking",
  "upToDate",
  "available",
  "downloading",
  "downloaded",
  "installing",
  "checkFailed",
  "downloadFailed",
  "installFailed",
] as const;

type AppUpdatePhaseName = (typeof PHASE_NAMES)[number];

export type AppUpdateStateDto = {
  autoCheck: boolean;
  autoDownload: boolean;
  lastCheckedAtUnixMs: number | null;
  phase: string;
  version: string | null;
  installReady: boolean;
  errorKey: string | null;
};

export type AppUpdateClientAckDto = AppUpdateStateDto & {
  clearLegacy: boolean;
};

export function initialAppUpdateSnapshot(): AppUpdateSnapshot {
  return {
    preferences: {
      autoCheck: true,
      autoDownload: true,
      lastCheckedAt: null,
    },
    phase: { kind: "idle" },
    installReady: false,
  };
}

function isPhaseName(value: string): value is AppUpdatePhaseName {
  return PHASE_NAMES.some((phase) => phase === value);
}

function errorKey(value: string | null): UpdateErrorKey | null {
  switch (value) {
    case "checkFailed":
    case "downloadFailed":
    case "installFailed":
      return value;
    default:
      return null;
  }
}

function lastCheckedAt(unixMs: number | null): string | null {
  if (unixMs === null) {
    return null;
  }
  const date = new Date(unixMs);
  if (Number.isNaN(date.getTime())) {
    return null;
  }
  return date.toISOString();
}

export function snapshotFromDto(dto: AppUpdateStateDto): AppUpdateSnapshot {
  const version = dto.version ?? "";
  const phaseName = isPhaseName(dto.phase) ? dto.phase : "idle";
  const phase: UpdatePhase = (() => {
    switch (phaseName) {
      case "checking":
        return { kind: "checking" };
      case "upToDate":
        return { kind: "upToDate" };
      case "available":
        return { kind: "available", version };
      case "downloading":
        return { kind: "downloading", version };
      case "downloaded":
        return { kind: "downloaded", version };
      case "installing":
        return { kind: "installing", version };
      case "checkFailed":
      case "downloadFailed":
      case "installFailed":
        return { kind: "error", messageKey: errorKey(dto.errorKey) ?? "checkFailed" };
      case "idle":
        return { kind: "idle" };
    }
  })();
  return {
    preferences: {
      autoCheck: dto.autoCheck,
      autoDownload: dto.autoDownload,
      lastCheckedAt: lastCheckedAt(dto.lastCheckedAtUnixMs),
    },
    phase,
    installReady: dto.installReady,
  };
}
