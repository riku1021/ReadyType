import { createFileRoute } from "@tanstack/react-router";
import { AppDetailPage } from "@/features/inputSourceApps";

export const Route = createFileRoute("/apps/$appId/")({
  staticData: {
    title: "アプリごとの設定",
    headerBack: { to: "/apps", label: "一覧に戻る" },
  },
  component: AppDetailRoute,
});

function AppDetailRoute() {
  const { appId } = Route.useParams();
  return <AppDetailPage appId={appId} />;
}
