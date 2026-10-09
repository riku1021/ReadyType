import { createFileRoute } from "@tanstack/react-router";

import { SettingsPage } from "@/features/settings";

export const Route = createFileRoute("/general/")({
  staticData: { title: "nav.general" },
  component: SettingsPage,
});
