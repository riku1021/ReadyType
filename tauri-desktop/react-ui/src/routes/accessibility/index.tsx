import { createFileRoute } from "@tanstack/react-router";
import { AccessibilityPage } from "@/features/accessibility";

export const Route = createFileRoute("/accessibility/")({
  staticData: { title: "アクセシビリティ" },
  component: AccessibilityPage,
});
