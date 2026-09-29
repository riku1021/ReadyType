import { createFileRoute } from "@tanstack/react-router";
import { DiagnosticsPage } from "@/features/inputSourceDiagnostics";

export const Route = createFileRoute("/diagnostics/")({
  staticData: { title: "nav.diagnostics" },
  component: DiagnosticsPage,
});
