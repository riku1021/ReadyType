import "@/styles/index.css";

import { createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { useInputSourceBootstrap } from "@/features/inputSource";
import { useRevealMainWindow } from "@/hooks/useRevealMainWindow";
import { useThemeManager } from "@/hooks/useThemeManager";
import { routeTree } from "@/routeTree.gen";

const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }

  interface StaticDataRouteOption {
    title?: string;
    headerBack?: {
      to: string;
      label: string;
    };
  }
}

function App() {
  useThemeManager();
  useRevealMainWindow();
  useInputSourceBootstrap();
  return <RouterProvider router={router} />;
}

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("Root element #root not found");
}

createRoot(rootEl).render(
  <StrictMode>
    <App />
  </StrictMode>
);
