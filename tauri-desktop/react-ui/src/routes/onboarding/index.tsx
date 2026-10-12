import { createFileRoute } from "@tanstack/react-router";
import { OnboardingPage } from "@/features/onboarding";

type OnboardingSearch = {
  preview?: boolean;
};

export const Route = createFileRoute("/onboarding/")({
  validateSearch: (search: { preview?: boolean | string }): OnboardingSearch => ({
    preview: search.preview === true || search.preview === "1" || search.preview === "true",
  }),
  staticData: { title: "onboarding.title" },
  component: OnboardingRoutePage,
});

function OnboardingRoutePage() {
  const { preview } = Route.useSearch();
  return <OnboardingPage preview={preview === true} />;
}
