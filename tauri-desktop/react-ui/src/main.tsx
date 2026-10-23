import "@/styles/index.css";
import "@/locales/i18n";

import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { type ReactNode, StrictMode } from "react";
import { createRoot } from "react-dom/client";
import type { IconType } from "react-icons";
import { useInputSourceBootstrap, useInputSourceSources } from "@/features/inputSource";
import { useAppUpdateBootstrap } from "@/features/update";
import { useRevealMainWindow } from "@/hooks/useRevealMainWindow";
import { useThemeManager } from "@/hooks/useThemeManager";
import {
  getHostOs,
  getHostOsVersion,
  isQuickSetWindow,
  resolveWindowLabel,
  type WindowKind,
  WindowKindProvider,
} from "@/platform";
import { routeTree } from "@/routeTree.gen";

// ホスト OS / バージョンは起動時に1回取得してキャッシュ
getHostOs();
getHostOsVersion();

const windowLabel = resolveWindowLabel();
const quickSet = isQuickSetWindow(windowLabel);
const windowKind: WindowKind = quickSet ? "quick-set" : "main";

// quick-set は `/`→`/apps` redirect を踏まず、最初から `/quick-set/` で起動する
const router = createRouter({
  routeTree,
  // 本文は window ではなく main がスクロールする。遷移のたびにその容器を先頭へ戻す。
  scrollToTopSelectors: ["#main-scroll"],
  ...(quickSet ? { history: createMemoryHistory({ initialEntries: ["/quick-set/"] }) } : {}),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }

  interface StaticDataRouteOption {
    title?: string;
    /** 指定時はナビのアイコンより優先する */
    headerIcon?: IconType;
    headerBack?: {
      to: string;
      label: string;
    };
  }
}

function AppTree({ children }: { children: ReactNode }) {
  return <WindowKindProvider kind={windowKind}>{children}</WindowKindProvider>;
}

/** メイン窓: reveal・オンボーディングゲート（AppShell → MainShell）を含む */
function MainApp() {
  useThemeManager();
  useRevealMainWindow();
  useInputSourceBootstrap();
  useInputSourceSources();
  useAppUpdateBootstrap();
  return <RouterProvider router={router} />;
}

/** quick-set 窓: メイン専用副作用を載せない薄いツリー */
function QuickSetApp() {
  useThemeManager();
  useInputSourceBootstrap();
  return <RouterProvider router={router} />;
}

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("Root element #root not found");
}

createRoot(rootEl).render(
  <StrictMode>
    <AppTree>{quickSet ? <QuickSetApp /> : <MainApp />}</AppTree>
  </StrictMode>
);
