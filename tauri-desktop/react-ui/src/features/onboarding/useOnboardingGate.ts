import { useNavigate, useRouterState } from "@tanstack/react-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAtomValue } from "jotai";
import { useEffect } from "react";

import { inputSourceStateAtom } from "@/features/inputSource";

import { resolveOnboardingNavigation } from "./onboardingGate";

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
 * メイン窓専用（`useRevealMainWindow` と同様）。quick-set などサブ窓では動かない。
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
    try {
      if (getCurrentWindow().label !== "main") {
        return;
      }
    } catch {
      // Tauri 外（ブラウザ）ではゲートを適用する
    }

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
