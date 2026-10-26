import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FiArrowLeft, FiArrowRight, FiRefreshCw } from "react-icons/fi";
import { IoShieldCheckmark } from "react-icons/io5";

import { Button } from "@/components/Button";
import { useInputSourceState } from "@/features/inputSource";
import { getHostOs } from "@/platform";

import { PermissionAskStep } from "../PermissionAskStep/PermissionAskStep";
import { PermissionConfirmStep } from "../PermissionConfirmStep/PermissionConfirmStep";
import { QuickSetTutorialStep } from "../QuickSetTutorialStep/QuickSetTutorialStep";
import { WelcomeStep } from "../WelcomeStep/WelcomeStep";
import * as styles from "./OnboardingPage.styles";

const STEP_IDS = ["welcome", "permissionAsk", "permissionConfirm", "quickSet", "tips"] as const;

type StepIndex = 0 | 1 | 2 | 3 | 4;

const STEP_COUNT = STEP_IDS.length;

function Progress({ step, skipPermission }: { step: StepIndex; skipPermission: boolean }) {
  const visibleSteps = skipPermission ? [0, 3, 4] : [0, 1, 2, 3, 4];
  const currentIndex = visibleSteps.indexOf(step);
  return (
    <div className={styles.progressBlock}>
      <div className={styles.progressTrack} aria-hidden="true">
        {visibleSteps.map((stepId, index) => {
          const id = STEP_IDS[stepId];
          const state = index === currentIndex ? "current" : "upcoming";
          return <span key={id} className={styles.progressDot({ state })} />;
        })}
      </div>
    </div>
  );
}

type OnboardingPageProps = {
  /** Lab などからの UI 確認。完了処理をスキップして `/lab` に戻る */
  preview?: boolean;
};

export function OnboardingPage({ preview = false }: OnboardingPageProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    state,
    requestPermission,
    openAccessibilitySettings,
    refreshPermission,
    completeOnboarding,
  } = useInputSourceState();

  const [step, setStep] = useState<StepIndex>(0);
  const [completing, setCompleting] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const skipPermission = getHostOs() === "windows";

  const permission = state?.permission ?? null;
  const granted = permission === "granted";

  useEffect(() => {
    if (step !== 2) {
      return;
    }
    void refreshPermission();

    const onVisible = () => {
      if (document.visibilityState === "visible") {
        void refreshPermission();
      }
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [refreshPermission, step]);

  const goBack = () => {
    setStep((current) => {
      if (skipPermission) {
        return current === 3 ? 0 : current === 4 ? 3 : current;
      }
      return current > 0 ? ((current - 1) as StepIndex) : current;
    });
  };

  const goNext = () => {
    setStep((current) => {
      if (skipPermission) {
        return current === 0 ? 3 : current === 3 ? 4 : current;
      }
      return current < STEP_COUNT - 1 ? ((current + 1) as StepIndex) : current;
    });
  };

  const handleOpenSettings = async () => {
    const permission = await requestPermission();
    if (permission !== "granted") {
      await openAccessibilitySettings();
    }
    setStep(2);
  };

  const handleConfirmPermission = async () => {
    if (granted) {
      setStep(3);
      return;
    }
    setConfirming(true);
    try {
      const next = await refreshPermission();
      if (next === "granted") {
        setStep(3);
      }
    } finally {
      setConfirming(false);
    }
  };

  const finish = async () => {
    if (completing) {
      return;
    }
    setCompleting(true);
    try {
      if (preview) {
        await navigate({ to: "/lab" });
        return;
      }
      await completeOnboarding();
      await navigate({ to: "/apps" });
    } catch {
      setCompleting(false);
    }
  };

  return (
    <div className={styles.page}>
      <Progress step={step} skipPermission={skipPermission} />

      <div className={styles.content}>
        {step === 0 ? <WelcomeStep /> : null}
        {!skipPermission && step === 1 ? <PermissionAskStep /> : null}
        {!skipPermission && step === 2 ? <PermissionConfirmStep granted={granted} /> : null}
        {step === 3 ? <QuickSetTutorialStep /> : null}

        {step === 4 ? <WelcomeStep copy="ready" /> : null}
      </div>

      <footer className={styles.footer}>
        {step === 3 ? (
          <div className={styles.footerInfo}>
            <span className={styles.footerInfoIcon} aria-hidden="true">
              <IoShieldCheckmark />
            </span>
            {t("onboarding.quickSet.info")}
          </div>
        ) : null}

        <div className={styles.footerRow}>
          {step === 0 ? (
            <Button
              variant="neutral"
              onClick={() => {
                void finish();
              }}
              disabled={completing}
              iconLeft={<FiArrowLeft />}
            >
              {t("onboarding.welcome.skip")}
            </Button>
          ) : (
            <Button variant="neutral" onClick={goBack} iconLeft={<FiArrowLeft />}>
              {t("onboarding.back")}
            </Button>
          )}

          {step === 1 ? (
            <div className={styles.footerPrimaryColumn}>
              <Button
                variant="primary"
                fullWidth
                onClick={() => {
                  void handleOpenSettings();
                }}
                iconRight={<FiArrowRight />}
              >
                {t("onboarding.permissionAsk.openSettings")}
              </Button>
              <p className={styles.footerNote}>{t("onboarding.permissionAsk.settingsNote")}</p>
            </div>
          ) : null}

          {step !== 1 ? (
            <div className={styles.footerActions}>
              {step === 0 ? (
                <Button variant="primary" onClick={goNext} iconRight={<FiArrowRight />}>
                  {t("onboarding.next")}
                </Button>
              ) : null}

              {step === 2 ? (
                <>
                  {!granted ? (
                    <Button variant="text" onClick={goNext} disabled={confirming}>
                      {t("onboarding.permissionConfirm.later")}
                    </Button>
                  ) : null}
                  <Button
                    variant="primary"
                    onClick={() => {
                      void handleConfirmPermission();
                    }}
                    disabled={confirming}
                    iconLeft={granted ? undefined : <FiRefreshCw />}
                    iconRight={granted ? <FiArrowRight /> : undefined}
                  >
                    {granted ? t("onboarding.next") : t("onboarding.permissionConfirm.check")}
                  </Button>
                </>
              ) : null}

              {step === 3 ? (
                <Button variant="primary" onClick={goNext} iconRight={<FiArrowRight />}>
                  {t("onboarding.next")}
                </Button>
              ) : null}

              {step === 4 ? (
                <Button
                  variant="primary"
                  onClick={() => {
                    void finish();
                  }}
                  disabled={completing}
                >
                  {t("onboarding.tips.finish")}
                </Button>
              ) : null}
            </div>
          ) : null}
        </div>
      </footer>
    </div>
  );
}
