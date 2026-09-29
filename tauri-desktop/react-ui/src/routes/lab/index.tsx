import { createFileRoute } from "@tanstack/react-router";
import { LabPage } from "@/features/lab";

export const Route = createFileRoute("/lab/")({
  staticData: { title: "nav.lab" },
  component: LabPage,
});
