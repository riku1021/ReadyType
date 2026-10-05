import { useRouter } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type ReactNode, useEffect, useState } from "react";
import { NAVIGATE_EVENT, QUICK_SET_EVENT } from "@/features/quickSet";
import { RootLayout } from "@/layouts/RootLayout";

type AppShellProps = {
  children: ReactNode;
};

export function AppShell({ children }: AppShellProps) {
  const [windowLabel, setWindowLabel] = useState<string | null>(null);
  const router = useRouter();

  useEffect(() => {
    try {
      setWindowLabel(getCurrentWindow().label);
    } catch {
      setWindowLabel("main");
    }
  }, []);

  // quick-set: 起動時および毎回の open（QUICK_SET_EVENT）でルートを固定
  useEffect(() => {
    if (windowLabel !== "quick-set") {
      return;
    }
    void router.navigate({ to: "/quick-set" });
    const unlistenPromise = getCurrentWebviewWindow().listen(QUICK_SET_EVENT, () => {
      void router.navigate({ to: "/quick-set" });
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [router, windowLabel]);

  useEffect(() => {
    if (windowLabel !== "quick-set") {
      return;
    }
    const html = document.documentElement;
    html.dataset.window = "quick-set";
    return () => {
      delete html.dataset.window;
    };
  }, [windowLabel]);

  // main: emit_to("main") と対になる webview 専用 listen（公式パターン）
  useEffect(() => {
    if (windowLabel === null || windowLabel === "quick-set") {
      return;
    }
    const unlistenPromise = getCurrentWebviewWindow().listen<string>(NAVIGATE_EVENT, (event) => {
      const route = event.payload;
      if (typeof route === "string" && route.startsWith("/")) {
        router.history.push(route);
      }
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [router, windowLabel]);

  if (windowLabel === "quick-set") {
    return <>{children}</>;
  }

  return <RootLayout>{children}</RootLayout>;
}
