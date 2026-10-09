import { createFileRoute, redirect } from "@tanstack/react-router";

/** 旧パス互換。表示名は「一般」、正規ルートは `/general`。 */
export const Route = createFileRoute("/settings/")({
  beforeLoad: () => {
    throw redirect({ to: "/general" });
  },
});
