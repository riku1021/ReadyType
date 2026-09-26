import { createFileRoute } from "@tanstack/react-router";
import { AppSettingsPage } from "@/features/inputSourceApps";

export const Route = createFileRoute("/apps/")({
  staticData: { title: "アプリごとの設定" },
  component: AppSettingsPage,
});
