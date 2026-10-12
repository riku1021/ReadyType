export type OnboardingNavTarget = "/onboarding" | "/apps";

type ResolveArgs = {
  ready: boolean;
  /** null = settings 未取得 */
  onboardingCompleted: boolean | null;
  pathname: string;
  /** Lab などからの UI 確認用。完了済みでも `/onboarding` に留まれる */
  preview?: boolean;
};

function isOnboardingPath(pathname: string): boolean {
  return pathname === "/onboarding" || pathname.startsWith("/onboarding/");
}

function isQuickSetPath(pathname: string): boolean {
  return pathname === "/quick-set" || pathname.startsWith("/quick-set/");
}

/** 起動ゲートの遷移先。不要なら null。 */
export function resolveOnboardingNavigation({
  ready,
  onboardingCompleted,
  pathname,
  preview = false,
}: ResolveArgs): OnboardingNavTarget | null {
  if (!ready || onboardingCompleted == null) {
    return null;
  }

  // Quick Set 窓専用ルート。メインのオンボーディング誘導対象外。
  if (isQuickSetPath(pathname)) {
    return null;
  }

  const onOnboarding = isOnboardingPath(pathname);

  if (!onboardingCompleted && !onOnboarding) {
    return "/onboarding";
  }

  if (onboardingCompleted && onOnboarding && !preview) {
    return "/apps";
  }

  return null;
}
