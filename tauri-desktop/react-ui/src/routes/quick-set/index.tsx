import { createFileRoute } from "@tanstack/react-router";
import { QuickSetPage } from "@/features/quickSet";

export const Route = createFileRoute("/quick-set/")({
  staticData: { title: "quickSet.title" },
  component: QuickSetPage,
});
