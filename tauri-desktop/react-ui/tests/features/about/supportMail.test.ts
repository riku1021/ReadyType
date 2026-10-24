import { describe, expect, it, vi } from "vitest";
import {
  buildSupportMailDraft,
  hostOsDisplayName,
  openSupportMailDraft,
  SUPPORT_EMAIL,
  SUPPORT_MAIL_SUBJECT,
  type SupportMailEnv,
} from "@/features/about/supportMail";
import en from "@/locales/en/translation.json";
import ja from "@/locales/ja/translation.json";
import zh from "@/locales/zh/translation.json";

const PLACEHOLDERS = ["{{appName}}", "{{version}}", "{{osName}}", "{{osVersion}}", "{{locale}}"];

const env: SupportMailEnv = {
  appName: "ReadyType",
  version: "1.2.3",
  osName: "macOS",
  osVersion: "27.0.0",
  locale: "ja",
};

function fill(template: string, values: Record<string, string>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key: string) => values[key] ?? match);
}

function expectedBody(prompt: string, environment: string): string {
  return `${prompt}\n\n\n${environment}`.replace(/\n/g, "\r\n");
}

describe("buildSupportMailDraft", () => {
  it("uses a stable subject and the feature headings", () => {
    const prompt = ja.about.featureRequestBody;
    const environment = fill(ja.about.feedbackEnvironment, env);
    const draft = buildSupportMailDraft({
      kind: "feature",
      prompt,
      environment,
      clipboardLead: fill(ja.about.feedbackClipboardLead, {
        email: SUPPORT_EMAIL,
        subject: SUPPORT_MAIL_SUBJECT.feature,
      }),
    });

    expect(draft.subject).toBe("[ReadyType][Feature]");
    expect(draft.body).toBe(expectedBody(prompt, environment));
    expect(draft.body).toContain("やりたいこと:");
    expect(draft.body).toContain("困っていること:");
    expect(draft.body).not.toContain("再現手順:");
    expect(draft.body).toContain("ReadyType 1.2.3");
    expect(draft.body).toContain("macOS 27.0.0");
    expect(draft.body).toContain("言語: ja");
    expect(draft.clipboardText).toContain(`宛先: ${SUPPORT_EMAIL}`);
    expect(draft.clipboardText).toContain("件名: [ReadyType][Feature]");
  });

  it("uses the bug headings and keeps a missing version as a dash", () => {
    const prompt = ja.about.bugReportBody;
    const environment = fill(ja.about.feedbackEnvironment, {
      ...env,
      version: "—",
      osVersion: "—",
    });
    const draft = buildSupportMailDraft({
      kind: "bug",
      prompt,
      environment,
      clipboardLead: "宛先: x",
    });

    expect(draft.subject).toBe("[ReadyType][Bug]");
    expect(draft.body).toBe(expectedBody(prompt, environment));
    expect(draft.body).toContain("何が起きたか:");
    expect(draft.body).toContain("再現手順:");
    expect(draft.body).toContain("期待した動作:");
    expect(draft.body).toContain("実際の動作:");
    expect(draft.body).toContain("アプリ: ReadyType —");
    expect(draft.body).toContain("OS: macOS —");
    expect(draft.body).not.toContain("1.2.3");
  });

  it("percent-encodes the mailto query", () => {
    const prompt = en.about.featureRequestBody;
    const environment = fill(en.about.feedbackEnvironment, env);
    const draft = buildSupportMailDraft({
      kind: "feature",
      prompt,
      environment,
      clipboardLead: "To: x",
    });

    expect(draft.mailtoUrl.startsWith(`mailto:${SUPPORT_EMAIL}?`)).toBe(true);
    expect(draft.mailtoUrl).toContain("%20");
    expect(draft.mailtoUrl).toContain("%0D%0A");
    expect(draft.mailtoUrl).not.toContain(" ");
    expect(draft.mailtoUrl).not.toContain("\n");
    expect(draft.mailtoUrl).not.toContain("+");

    const parsed = new URL(draft.mailtoUrl);
    expect(parsed.searchParams.get("subject")).toBe("[ReadyType][Feature]");
    expect(parsed.searchParams.get("body")).toBe(draft.body);
    expect(draft.body.length).toBeLessThan(1500);
  });

  it("keeps environment placeholders in one shared string", () => {
    for (const pack of [ja, en, zh]) {
      expect(pack.about.featureRequestBody).not.toContain("{{");
      expect(pack.about.bugReportBody).not.toContain("{{");
      for (const placeholder of PLACEHOLDERS) {
        expect(pack.about.feedbackEnvironment).toContain(placeholder);
      }
      const featureLength =
        `${pack.about.featureRequestBody}\n\n\n${pack.about.feedbackEnvironment}`.length;
      const bugLength = `${pack.about.bugReportBody}\n\n\n${pack.about.feedbackEnvironment}`.length;
      expect(featureLength).toBeLessThan(1500);
      expect(bugLength).toBeLessThan(1500);
      expect(pack.about.featureRequestBody).not.toBe(pack.about.bugReportBody);
    }
  });
});

describe("hostOsDisplayName", () => {
  it("maps host ids to display names", () => {
    expect(hostOsDisplayName("macOs")).toBe("macOS");
    expect(hostOsDisplayName("windows")).toBe("Windows");
    expect(hostOsDisplayName("linux")).toBe("Linux");
  });
});

describe("openSupportMailDraft", () => {
  it("stays quiet when the mail app opens", async () => {
    const writeClipboard = vi.fn<(text: string) => Promise<void>>();
    const result = await openSupportMailDraft(draft(), {
      openUrl: () => Promise.resolve(),
      writeClipboard,
    });

    expect(result).toBe("opened");
    expect(writeClipboard).not.toHaveBeenCalled();
  });

  it("copies the draft when the mail app cannot open", async () => {
    const writeClipboard = vi.fn<(text: string) => Promise<void>>(() => Promise.resolve());
    const sample = draft();
    const result = await openSupportMailDraft(sample, {
      openUrl: () => Promise.reject(new Error("no handler")),
      writeClipboard,
    });

    expect(result).toBe("copied");
    expect(writeClipboard).toHaveBeenCalledWith(sample.clipboardText);
  });

  it("reports failure when the clipboard copy also fails", async () => {
    const result = await openSupportMailDraft(draft(), {
      openUrl: () => Promise.reject(new Error("no handler")),
      writeClipboard: () => Promise.reject(new Error("denied")),
    });

    expect(result).toBe("failed");
  });
});

function draft() {
  return buildSupportMailDraft({
    kind: "bug",
    prompt: "何が起きたか:",
    environment: "---\nアプリ: ReadyType —",
    clipboardLead: "宛先: readytype.support@gmail.com",
  });
}
