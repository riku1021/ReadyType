import type { HostOs } from "@/platform/hostOs";

/** サポート宛先。翻訳文には置かない。 */
export const SUPPORT_EMAIL = "readytype.support@gmail.com";

/** 受信トレイのフィルタ用。UI 言語では変えない。 */
export const SUPPORT_MAIL_SUBJECT = {
  feature: "[ReadyType][Feature]",
  bug: "[ReadyType][Bug]",
} as const;

export type SupportMailKind = keyof typeof SUPPORT_MAIL_SUBJECT;

export type SupportMailEnv = {
  appName: string;
  version: string;
  osName: string;
  osVersion: string;
  locale: string;
};

export type SupportMailDraft = {
  to: string;
  subject: string;
  body: string;
  mailtoUrl: string;
  clipboardText: string;
};

const BODY_KEY = {
  feature: "about.featureRequestBody",
  bug: "about.bugReportBody",
} as const;

export type SupportMailBodyKey = (typeof BODY_KEY)[SupportMailKind];

export function supportMailBodyKey(kind: SupportMailKind): SupportMailBodyKey {
  return BODY_KEY[kind];
}

export function hostOsDisplayName(os: HostOs): string {
  switch (os) {
    case "macOs":
      return "macOS";
    case "windows":
      return "Windows";
    case "linux":
      return "Linux";
  }
}

/** 翻訳文の LF を mailto 用の CRLF にする。 */
function toCrlf(text: string): string {
  return text.replace(/\r\n/g, "\n").replace(/\n/g, "\r\n");
}

export function buildSupportMailDraft(input: {
  kind: SupportMailKind;
  prompt: string;
  environment: string;
  clipboardLead: string;
}): SupportMailDraft {
  const subject = SUPPORT_MAIL_SUBJECT[input.kind];
  const body = toCrlf(`${input.prompt}\n\n\n${input.environment}`);
  const mailtoUrl = `mailto:${SUPPORT_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
  const clipboardText = toCrlf(`${input.clipboardLead}\n\n${body}`);
  return {
    to: SUPPORT_EMAIL,
    subject,
    body,
    mailtoUrl,
    clipboardText,
  };
}

export async function openSupportMailDraft(
  draft: SupportMailDraft,
  deps: {
    openUrl: (url: string) => Promise<void>;
    writeClipboard: (text: string) => Promise<void>;
  }
): Promise<"opened" | "copied" | "failed"> {
  try {
    await deps.openUrl(draft.mailtoUrl);
    return "opened";
  } catch {
    try {
      await deps.writeClipboard(draft.clipboardText);
      return "copied";
    } catch {
      return "failed";
    }
  }
}
