import { useRouter } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { type ReactNode, useEffect } from "react";

import { useOnboardingGate } from "@/features/onboarding";
import { NAVIGATE_EVENT } from "@/features/quickSet";
import { RootLayout } from "@/layouts/RootLayout";

type MainShellProps = {
  children: ReactNode;
  /** onboarding など Drawer なしのルート */
  shellLess: boolean;
};

/**
 * メイン窓専用シェル。
 * オンボーディングゲート・トレイ等からの NAVIGATE・RootLayout を担う。
 */
export function MainShell({ children, shellLess }: MainShellProps) {
  const router = useRouter();

  useOnboardingGate();

  useEffect(() => {
    const unlistenPromise = getCurrentWebviewWindow().listen<string>(NAVIGATE_EVENT, (event) => {
      const route = event.payload;
      if (typeof route === "string" && route.startsWith("/")) {
        router.history.push(route);
      }
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [router]);

  if (shellLess) {
    return <>{children}</>;
  }

  return <RootLayout>{children}</RootLayout>;
}
