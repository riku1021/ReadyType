import { createFileRoute } from "@tanstack/react-router";
import { UpdatePage } from "@/features/update";

export const Route = createFileRoute("/update/")({
  staticData: { title: "アップデート" },
  component: UpdatePage,
});
