import { useRouterState } from "@tanstack/react-router";
import type { ReactNode } from "react";

import { MainShell } from "@/layouts/MainShell";
import { QuickSetShell } from "@/layouts/QuickSetShell";
import { useWindowKind } from "@/platform";

type AppShellProps = {
  children: ReactNode;
};

function isShellLessPath(pathname: string): boolean {
  return pathname === "/onboarding" || pathname.startsWith("/onboarding/");
}

/**
 * `WindowKindProvider`（main.tsx）の kind に従ってシェルを選ぶ。
 * 窓ラベルの再取得はしない。
 */
export function AppShell({ children }: AppShellProps) {
  const kind = useWindowKind();
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  if (kind === "quick-set") {
    return <QuickSetShell>{children}</QuickSetShell>;
  }

  return <MainShell shellLess={isShellLessPath(pathname)}>{children}</MainShell>;
}
