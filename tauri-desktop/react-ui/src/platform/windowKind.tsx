import { createContext, type ReactNode, useContext } from "react";

export type WindowKind = "main" | "quick-set";

const WindowKindContext = createContext<WindowKind>("main");

/** main.tsx の MainApp / QuickSetApp で一度だけ設定する。AppShell などは再判定しない。 */
export function WindowKindProvider({ kind, children }: { kind: WindowKind; children: ReactNode }) {
  return <WindowKindContext.Provider value={kind}>{children}</WindowKindContext.Provider>;
}

export function useWindowKind(): WindowKind {
  return useContext(WindowKindContext);
}
