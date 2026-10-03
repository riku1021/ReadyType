import { useRouter } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type ReactNode, useEffect, useState } from "react";
import { NAVIGATE_EVENT } from "@/features/quickSet";
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

  useEffect(() => {
    if (windowLabel !== "quick-set") {
      return;
    }
    void router.navigate({ to: "/quick-set" });
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

  useEffect(() => {
    if (windowLabel === "quick-set") {
      return;
    }
    let unlisten: (() => void) | undefined;
    void listen<string>(NAVIGATE_EVENT, (event) => {
      const route = event.payload;
      if (typeof route === "string" && route.startsWith("/")) {
        router.history.push(route);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [router, windowLabel]);

  if (windowLabel === "quick-set") {
    return <>{children}</>;
  }

  return <RootLayout>{children}</RootLayout>;
}
