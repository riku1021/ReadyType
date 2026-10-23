import { createFileRoute } from "@tanstack/react-router";
import { MdAppRegistration } from "react-icons/md";

import { AppDetailPage } from "@/features/inputSourceApps";

export const Route = createFileRoute("/apps/$appId/")({
  staticData: {
    title: "apps.detailTitle",
    headerIcon: MdAppRegistration,
    headerBack: { to: "/apps", label: "nav.backToList" },
  },
  component: AppDetailRoute,
});

function AppDetailRoute() {
  const { appId } = Route.useParams();
  return <AppDetailPage appId={appId} />;
}
