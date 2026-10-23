import { createFileRoute } from "@tanstack/react-router";
import { MdLanguage } from "react-icons/md";

import { DisplayLanguagePage } from "@/features/settings";

export const Route = createFileRoute("/general/language/")({
  staticData: {
    title: "settings.displayLanguage",
    headerIcon: MdLanguage,
    headerBack: { to: "/general", label: "nav.backToGeneral" },
  },
  component: DisplayLanguagePage,
});
