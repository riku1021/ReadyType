import { createFileRoute } from "@tanstack/react-router";
import { SettingsPage } from "@/features/settings";

export const Route = createFileRoute("/settings/")({
  staticData: { title: "設定" },
  component: SettingsPage,
});
