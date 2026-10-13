import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { useEffect } from "react";

import { inputSourceStateAtom } from "@/features/inputSource";

import { resolveOnboardingNavigation } from "../onboardingGate";

function readPreviewFlag(search: object): boolean {
  if (!("preview" in search)) {
    return false;
  }
  const value = search.preview;
  return value === true || value === "1" || value === "true";
}

/**
 * 未完了なら `/onboarding` へ、完了済みでオンボーディングに居るなら `/apps` へ誘導する。
 * `?preview=1` のときは完了済みでもオンボーディングに留まる（Lab 確認用）。
 * settings 未取得時は何もしない（ブラウザ単体起動など）。
 *
 * MainShell 専用（QuickSetApp / QuickSetShell ではマウントしない）。
 */
export function useOnboardingGate() {
  const feature = useAtomValue(inputSourceStateAtom);
  const navigate = useNavigate();
  const { pathname, preview } = useRouterState({
    select: (state) => {
      const search = state.location.search;
      const previewFlag =
        typeof search === "object" && search !== null ? readPreviewFlag(search) : false;
      return {
        pathname: state.location.pathname,
        preview: previewFlag,
      };
    },
  });

  useEffect(() => {
    const target = resolveOnboardingNavigation({
      ready: feature.ready,
      onboardingCompleted: feature.settings?.onboardingCompleted ?? null,
      pathname,
      preview,
    });
    if (target == null) {
      return;
    }
    void navigate({ to: target });
  }, [feature.ready, feature.settings, navigate, pathname, preview]);
}
