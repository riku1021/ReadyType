import { createFileRoute } from "@tanstack/react-router";
import { DiagnosticsPage } from "@/features/inputSourceDiagnostics";

export const Route = createFileRoute("/diagnostics/")({
  staticData: { title: "診断" },
  component: DiagnosticsPage,
});
