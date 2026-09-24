import { createFileRoute } from "@tanstack/react-router";
import { RulesPage } from "@/features/inputSourceRules";

export const Route = createFileRoute("/rules/")({
  staticData: { title: "ルール" },
  component: RulesPage,
});
