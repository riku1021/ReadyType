import { useAtomValue } from "jotai";
import { Trans, useTranslation } from "react-i18next";
import { FiSearch } from "react-icons/fi";
import { IoChatbubble } from "react-icons/io5";
import { MdBolt, MdDesktopWindows, MdSettings } from "react-icons/md";

import { inputSourceStateAtom } from "@/features/inputSource";
import {
  acceleratorToChord,
  DEFAULT_QUICK_SET_ACCELERATOR,
  type ModifierFlag,
  modifierTextLabel,
  type ShortcutChord,
} from "@/features/settings";
import { getHostOs, type HostOs } from "@/platform";

import * as styles from "./WelcomeStep.styles";

type WelcomeStepProps = {
  /** welcome: 初回紹介 / ready: オンボーディング完了前の確認 */
  copy?: "welcome" | "ready";
};

const MODIFIER_FLAGS: ModifierFlag[] = ["ctrl", "alt", "shift", "meta"];

function shortcutKeyCaps(chord: ShortcutChord, hostOs: HostOs): string[] {
  const caps = MODIFIER_FLAGS.filter((flag) => chord[flag]).map((flag) =>
    modifierTextLabel(flag, hostOs)
  );
  if (chord.key != null && chord.key.length > 0) {
    caps.push(chord.key);
  }
  return caps;
}

export function WelcomeStep({ copy = "welcome" }: WelcomeStepProps) {
  const { t } = useTranslation();
  const feature = useAtomValue(inputSourceStateAtom);
  const hostOs = getHostOs();
  const accelerator = feature.settings?.quickSetShortcut ?? DEFAULT_QUICK_SET_ACCELERATOR;
  const chord = acceleratorToChord(accelerator, hostOs);
  const keyCaps = shortcutKeyCaps(chord, hostOs);
  const headingKey = copy === "ready" ? "onboarding.tips.heading" : "onboarding.welcome.heading";
  const descriptionKey =
    copy === "ready" ? "onboarding.tips.description" : "onboarding.welcome.description";

  return (
    <div className={styles.root}>
      <img className={styles.logo} src="/pngs/logo.png" alt="" width={64} />

      <h1 className={styles.title}>
        <Trans
          i18nKey={headingKey}
          values={{ appName: t("about.appName") }}
          components={{ accent: <span className={styles.titleAccent} /> }}
        />
      </h1>
      <p className={styles.lead}>{t(descriptionKey)}</p>

      <section className={styles.demo} aria-hidden="true">
        <div className={styles.demoCard({ side: "message" })}>
          <div className={styles.appRow}>
            <span className={styles.messageAppIcon}>
              <IoChatbubble />
            </span>
            {t("onboarding.welcome.demoMessageApp")}
          </div>
          <div className={styles.messageField}>
            <span className={styles.messageBubble}>{t("onboarding.welcome.demoMessageText")}</span>
            <span className={styles.caret} />
          </div>
          <div className={styles.pill({ tone: "japanese" })}>
            <span className={styles.pillGlyph({ tone: "japanese" })}>
              {t("onboarding.welcome.demoJaGlyph")}
            </span>
            {t("onboarding.welcome.demoJaPill")}
          </div>
        </div>

        <div className={styles.hub}>
          <img className={styles.hubLogo} src="/pngs/logo.png" alt="" width={44} />
          <svg
            className={styles.hubCycle}
            viewBox="0 0 170 170"
            fill="none"
            stroke="url(#onboardingWelcomeCycle)"
            strokeWidth="4"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
            focusable="false"
          >
            <defs>
              <linearGradient id="onboardingWelcomeCycle" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0" stopColor="#9b7bff" />
                <stop offset="1" stopColor="#4a8cff" />
              </linearGradient>
            </defs>
            <path d="M24 34 A80 80 0 0 1 146 34" />
            <path d="M144 20 L146 34 L133 29" />
            <path d="M146 136 A80 80 0 0 1 24 136" />
            <path d="M26 150 L24 136 L37 141" />
          </svg>
        </div>

        <div className={styles.browserWin}>
          <div className={styles.browserTraffic}>
            <span className={styles.traffic({ tone: "red", size: "sm" })} />
            <span className={styles.traffic({ tone: "yellow", size: "sm" })} />
            <span className={styles.traffic({ tone: "green", size: "sm" })} />
          </div>
          <div className={styles.browserInner}>
            <div className={styles.searchBar}>
              <span className={styles.googleMark}>G</span>
              {t("about.appName")}
              <span className={styles.caret} />
              <span className={styles.searchMag}>
                <FiSearch />
              </span>
            </div>
          </div>
          <div className={styles.pill({ tone: "english" })}>
            <span className={styles.pillGlyph({ tone: "english" })}>
              {t("onboarding.welcome.demoEnGlyph")}
            </span>
            {t("onboarding.welcome.demoEnPill")}
          </div>
        </div>
      </section>

      {copy === "welcome" ? (
        <section className={styles.features}>
          <div className={styles.feature({ divided: false })}>
            <span className={styles.featureIcon({ tone: "auto" })} aria-hidden="true">
              <MdBolt />
            </span>
            <span>{t("onboarding.welcome.featureAuto")}</span>
          </div>
          <div className={styles.feature({ divided: true })}>
            <span className={styles.featureIcon({ tone: "apps" })} aria-hidden="true">
              <MdDesktopWindows />
            </span>
            <span>{t("onboarding.welcome.featureApps")}</span>
          </div>
          <div className={styles.feature({ divided: true })}>
            <span className={styles.featureIcon({ tone: "setup" })} aria-hidden="true">
              <MdSettings />
            </span>
            <span>{t("onboarding.welcome.featureSetup")}</span>
          </div>
        </section>
      ) : (
        <section className={styles.shortcutTip}>
          <div className={styles.shortcutTipKeys} aria-hidden="true">
            {keyCaps.map((cap) => (
              <span key={cap} className={styles.shortcutTipKeyCap}>
                {cap}
              </span>
            ))}
          </div>
          <div className={styles.shortcutTipDivider} aria-hidden="true" />
          <div className={styles.shortcutTipCopy}>
            <p className={styles.shortcutTipTitle}>
              <Trans
                i18nKey="onboarding.tips.shortcutTipTitle"
                components={{ accent: <span className={styles.shortcutTipTitleAccent} /> }}
              />
            </p>
            <p className={styles.shortcutTipBody}>{t("onboarding.tips.shortcutTipBody")}</p>
          </div>
        </section>
      )}
    </div>
  );
}
