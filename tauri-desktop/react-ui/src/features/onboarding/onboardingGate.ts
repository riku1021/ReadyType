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

/** 起動ゲートの遷移先。不要なら null。MainShell 専用。 */
export function resolveOnboardingNavigation({
  ready,
  onboardingCompleted,
  pathname,
  preview = false,
}: ResolveArgs): OnboardingNavTarget | null {
  if (!ready || onboardingCompleted == null) {
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
