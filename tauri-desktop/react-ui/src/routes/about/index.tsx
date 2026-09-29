import { createFileRoute } from "@tanstack/react-router";
import { AboutPage } from "@/features/about";

export const Route = createFileRoute("/about/")({
  staticData: { title: "nav.about" },
  component: AboutPage,
});
