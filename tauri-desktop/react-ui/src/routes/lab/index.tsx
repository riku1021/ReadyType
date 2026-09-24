import { createFileRoute } from "@tanstack/react-router";
import { LabPage } from "@/features/lab";

export const Route = createFileRoute("/lab/")({
  staticData: { title: "開発確認" },
  component: LabPage,
});
