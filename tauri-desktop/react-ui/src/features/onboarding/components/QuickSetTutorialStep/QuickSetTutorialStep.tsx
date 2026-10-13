import { useAtomValue } from "jotai";
import { Trans, useTranslation } from "react-i18next";
import { FiSend } from "react-icons/fi";
import { IoKeypadOutline } from "react-icons/io5";
import { MdSettings } from "react-icons/md";

import { inputSourceStateAtom, type LanguageInputPreferences } from "@/features/inputSource";
import {
  acceleratorToChord,
  DEFAULT_QUICK_SET_ACCELERATOR,
  formatShortcutChord,
  type ModifierFlag,
  modifierTextLabel,
  type ShortcutChord,
} from "@/features/settings";
import { getHostOs, type HostOs } from "@/platform";

import * as styles from "./QuickSetTutorialStep.styles";

const MODIFIER_FLAGS: ModifierFlag[] = ["ctrl", "alt", "shift", "meta"];
const ENGLISH_TAG = "en";

type DemoLanguage = {
  tag: string;
  label: string;
  glyph: string;
  selected: boolean;
  tone: "primary" | "english";
};

function shortcutKeyCaps(chord: ShortcutChord, hostOs: HostOs): string[] {
  const caps = MODIFIER_FLAGS.filter((flag) => chord[flag]).map((flag) =>
    modifierTextLabel(flag, hostOs)
  );
  if (chord.key != null && chord.key.length > 0) {
    caps.push(chord.key);
  }
  return caps;
}

function primaryLanguageCode(tag: string): string {
  return tag.split("-")[0]?.toLowerCase() ?? tag.toLowerCase();
}

function isEnglishTag(tag: string): boolean {
  return primaryLanguageCode(tag) === ENGLISH_TAG;
}

function languageGlyph(tag: string): string {
  switch (primaryLanguageCode(tag)) {
    case "ja":
      return "あ";
    case "en":
      return "A";
    case "ko":
      return "가";
    case "zh":
      return "中";
    default: {
      const code = primaryLanguageCode(tag);
      return (code[0] ?? "A").toUpperCase();
    }
  }
}

function languageLabel(tag: string, uiLocale: string): string {
  try {
    const name = new Intl.DisplayNames([uiLocale], { type: "language" }).of(
      primaryLanguageCode(tag)
    );
    if (name != null && name.length > 0) {
      return name;
    }
  } catch {
    // Intl.DisplayNames unavailable or tag unsupported
  }
  return tag;
}

const DEMO_LANGUAGE_LIMIT = 3;

/** OS の入力言語から最大 3 件。英語は必ず含め、残り枠に非英語を入れる。 */
function buildDemoLanguages(
  preferences: LanguageInputPreferences,
  uiLocale: string
): DemoLanguage[] {
  const seen = new Set<string>();
  const nonEnglishTags: string[] = [];
  for (const tag of Object.keys(preferences)) {
    const code = primaryLanguageCode(tag);
    if (seen.has(code) || isEnglishTag(tag)) {
      continue;
    }
    seen.add(code);
    nonEnglishTags.push(tag);
  }

  const otherLimit = DEMO_LANGUAGE_LIMIT - 1;
  const selectedTags = [...nonEnglishTags.slice(0, otherLimit), ENGLISH_TAG];

  return selectedTags.map((tag, index) => {
    const english = isEnglishTag(tag);
    return {
      tag: english ? ENGLISH_TAG : tag,
      label: languageLabel(tag, uiLocale),
      glyph: languageGlyph(tag),
      selected: index === 0,
      tone: english ? "english" : "primary",
    };
  });
}

export function QuickSetTutorialStep() {
  const { t, i18n } = useTranslation();
  const feature = useAtomValue(inputSourceStateAtom);
  const hostOs = getHostOs();
  const accelerator = feature.settings?.quickSetShortcut ?? DEFAULT_QUICK_SET_ACCELERATOR;
  const chord = acceleratorToChord(accelerator, hostOs);
  const shortcutLabel = formatShortcutChord(chord, hostOs);
  const keyCaps = shortcutKeyCaps(chord, hostOs);
  const demoLanguages = buildDemoLanguages(
    feature.settings?.languagePreferences ?? {},
    i18n.resolvedLanguage ?? i18n.language
  );
  const primaryGlyph = demoLanguages[0]?.glyph ?? "A";

  return (
    <div className={styles.root}>
      <h1 className={styles.title}>
        <Trans
          i18nKey="onboarding.quickSet.heading"
          components={{ accent: <span className={styles.titleAccent} /> }}
        />
      </h1>
      <p className={styles.lead}>
        <Trans
          i18nKey="onboarding.quickSet.lead"
          values={{ shortcut: shortcutLabel }}
          components={{ chip: <span className={styles.chip} /> }}
        />
      </p>

      <section className={styles.demo} aria-hidden="true">
        <div className={styles.chat}>
          <div className={styles.chatTraffic}>
            <span className={styles.traffic({ tone: "red", size: "sm" })} />
            <span className={styles.traffic({ tone: "yellow", size: "sm" })} />
            <span className={styles.traffic({ tone: "green", size: "sm" })} />
          </div>
          <div className={styles.chatInner}>
            <div className={styles.chatTitle}>
              <span className={styles.chatBubbleIcon} />
              {t("onboarding.quickSet.demoChatTitle")}
            </div>
            <div className={styles.chatField}>
              {t("onboarding.quickSet.demoPlaceholder")}
              <span className={styles.caret} />
            </div>
          </div>
        </div>

        <div className={styles.keys}>
          <span className={styles.keysLead} />
          {keyCaps.map((cap) => (
            <span key={cap} className={styles.keyCap}>
              {cap}
            </span>
          ))}
          <span className={styles.keysChev} />
        </div>

        <div className={styles.panel}>
          <div className={styles.panelHeader}>
            <img className={styles.panelLogo} src="/pngs/logo.png" alt="" width={16} />
            {t("about.appName")}
            <span className={styles.panelGear}>
              <MdSettings />
            </span>
          </div>
          <div className={styles.fieldLabel}>{t("onboarding.quickSet.panelNameLabel")}</div>
          <div className={styles.fieldInput}>{t("onboarding.quickSet.panelNameValue")}</div>
          <div className={styles.fieldLabel}>{t("onboarding.quickSet.panelSourceLabel")}</div>
          {demoLanguages.map((language) => (
            <div key={language.tag} className={styles.option}>
              <span className={styles.radio({ on: language.selected })} />
              <span className={styles.sourceGlyph({ tone: language.tone })}>{language.glyph}</span>
              {language.label}
            </div>
          ))}
        </div>
      </section>

      <section className={styles.steps}>
        <div className={styles.step}>
          <div className={styles.stepTop}>
            <span className={styles.stepNum}>1</span>
            <span className={styles.stepIcon}>
              <FiSend />
            </span>
          </div>
          <h3 className={styles.stepTitle}>{t("onboarding.quickSet.step1Title")}</h3>
          <p className={styles.stepBody}>{t("onboarding.quickSet.step1Body")}</p>
        </div>
        <span className={styles.stepSep} aria-hidden="true" />
        <div className={styles.step}>
          <div className={styles.stepTop}>
            <span className={styles.stepNum}>2</span>
            <span className={styles.stepIcon}>
              <IoKeypadOutline />
            </span>
          </div>
          <h3 className={styles.stepTitle}>
            {t("onboarding.quickSet.step2Title", { shortcut: shortcutLabel })}
          </h3>
          <p className={styles.stepBody}>{t("onboarding.quickSet.step2Body")}</p>
        </div>
        <span className={styles.stepSep} aria-hidden="true" />
        <div className={styles.step}>
          <div className={styles.stepTop}>
            <span className={styles.stepNum}>3</span>
            <span className={styles.stepIcon}>{primaryGlyph}</span>
          </div>
          <h3 className={styles.stepTitle}>{t("onboarding.quickSet.step3Title")}</h3>
          <p className={styles.stepBody}>{t("onboarding.quickSet.step3Body")}</p>
        </div>
      </section>
    </div>
  );
}
