import type { ReactNode } from "react";
import { Drawer } from "@/layouts/Drawer";
import { Header } from "@/layouts/Header";
import { contentColumn, mainContent, rootLayout } from "@/layouts/rootLayout.styles";

type RootLayoutProps = {
  children: ReactNode;
};

export function RootLayout({ children }: RootLayoutProps) {
  return (
    <div className={rootLayout}>
      <Header />
      <Drawer />
      <main id="main-scroll" className={mainContent}>
        <div className={contentColumn}>{children}</div>
      </main>
    </div>
  );
}
