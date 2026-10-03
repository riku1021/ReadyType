import { createRootRoute, Outlet } from "@tanstack/react-router";
import { NotFoundPage } from "@/features/notFound";
import { AppShell } from "@/layouts/AppShell";

export const Route = createRootRoute({
  component: () => (
    <AppShell>
      <Outlet />
    </AppShell>
  ),
  notFoundComponent: NotFoundPage,
});
