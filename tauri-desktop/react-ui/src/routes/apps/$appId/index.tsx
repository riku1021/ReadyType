import { createFileRoute } from "@tanstack/react-router";
import { AppDetailPage } from "@/features/inputSourceApps";

export const Route = createFileRoute("/apps/$appId/")({
  staticData: {
    title: "nav.apps",
    headerBack: { to: "/apps", label: "nav.backToList" },
  },
  component: AppDetailRoute,
});

function AppDetailRoute() {
  const { appId } = Route.useParams();
  return <AppDetailPage appId={appId} />;
}
