export const navItems = [
  { href: "/#problem", id: "problem", label: "お悩み" },
  { href: "/#solution", id: "solution", label: "解決" },
  { href: "/#video", id: "video", label: "動画" },
  { href: "/#how", id: "how", label: "使い方" },
  { href: "/#features", id: "features", label: "特長" },
  { href: "/#review", id: "review", label: "Review" },
  { href: "/#faq", id: "faq", label: "FAQ" },
] as const;

export const sideNavItems = [
  ...navItems,
  { href: "/#download", id: "download", label: "ダウンロード" },
] as const;

export const apps = [
  { name: "VS Code", mark: "V", tone: "vscode" },
  { name: "Cursor", mark: "C", tone: "cursor" },
  { name: "Terminal", mark: ">_", tone: "terminal" },
  { name: "Slack", mark: "S", tone: "slack" },
  { name: "Notion", mark: "N", tone: "notion" },
  { name: "Chrome", mark: "G", tone: "chrome" },
  { name: "Discord", mark: "D", tone: "discord" },
] as const;

export type AppTone = (typeof apps)[number]["tone"];

export const footerProduct = [
  { href: "/#solution", label: "解決" },
  { href: "/#video", label: "デモ動画" },
  { href: "/#how", label: "使い方" },
  { href: "/#features", label: "特長" },
] as const;

export const footerSupport = [
  { href: "/#review", label: "Review" },
  { href: "/#faq", label: "FAQ" },
  { href: "/#download", label: "ダウンロード" },
] as const;

export const faqs = [
  { q: "質問をここに入力", a: "回答をここに入力" },
  { q: "質問をここに入力", a: "回答をここに入力" },
  { q: "質問をここに入力", a: "回答をここに入力" },
  { q: "質問をここに入力", a: "回答をここに入力" },
] as const;

export const reviews = [
  "レビュー 1\n（ここに内容を追加）",
  "レビュー 2\n（ここに内容を追加）",
  "レビュー 3\n（ここに内容を追加）",
] as const;
