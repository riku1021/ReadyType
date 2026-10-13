import { type ReactNode, useEffect } from "react";

type QuickSetShellProps = {
  children: ReactNode;
};

/**
 * quick-set 窓専用シェル。
 * 初期ルートは `main.tsx` の memory history（`/quick-set/`）が正本。
 * セッション反映は QuickSetPage の listen / loadSession が担当。
 */
export function QuickSetShell({ children }: QuickSetShellProps) {
  useEffect(() => {
    const html = document.documentElement;
    html.dataset.window = "quick-set";
    return () => {
      delete html.dataset.window;
    };
  }, []);

  return <>{children}</>;
}
