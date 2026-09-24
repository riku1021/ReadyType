import { createRootRoute, Outlet } from "@tanstack/react-router";
import { NotFoundPage } from "@/features/notFound";
import { RootLayout } from "@/layouts/RootLayout";

export const Route = createRootRoute({
  component: () => (
    <RootLayout>
      <Outlet />
    </RootLayout>
  ),
  notFoundComponent: NotFoundPage,
});
