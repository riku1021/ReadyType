import { useEffect, useState } from "react";
import {
  IoApps,
  IoChatbubbleEllipses,
  IoCreate,
  IoDocumentText,
  IoOptions,
  IoTerminal,
} from "react-icons/io5";

import { getAppIcons, type InputSourceOption } from "@/features/inputSource";
import { cx } from "@/styled-system/css";

import type { RuleKind } from "../appRulesViewModel";
import * as styles from "./appSettingsShared.styles";

const RULE_ICONS: Record<RuleKind, typeof IoChatbubbleEllipses> = {
  chat: IoChatbubbleEllipses,
  editor: IoDocumentText,
  terminal: IoTerminal,
  rename: IoCreate,
  default: IoApps,
  other: IoOptions,
};

export function RuleKindIcon({ kind }: { kind: RuleKind }) {
  const Icon = RULE_ICONS[kind];
  return (
    <span className={styles.ruleIconBox} aria-hidden="true">
      <Icon />
    </span>
  );
}

export function AppIconView({
  src,
  label,
  large,
}: {
  src?: string | null;
  label: string;
  large?: boolean;
}) {
  if (src) {
    return <img className={large ? styles.appIconLarge : styles.appIcon} src={src} alt="" />;
  }
  return (
    <span
      className={large ? styles.appIconFallbackLarge : styles.appIconFallback}
      aria-hidden="true"
    >
      {label.slice(0, 1)}
    </span>
  );
}

export function useAppIcons(appIds: readonly string[]): Record<string, string | null> {
  const [icons, setIcons] = useState<Record<string, string | null>>({});
  const key = [...appIds].sort().join("\n");

  useEffect(() => {
    const ids = key ? key.split("\n") : [];
    if (ids.length === 0) {
      return;
    }
    let cancelled = false;
    getAppIcons(ids)
      .then((result) => {
        if (!cancelled) {
          setIcons(result);
        }
      })
      .catch(() => {
        // ブラウザ単体起動やネイティブ未接続では取得できない
      });
    return () => {
      cancelled = true;
    };
  }, [key]);

  return icons;
}

export function LanguageSelect({
  value,
  label,
  options,
  onChange,
  wide = false,
}: {
  value: string;
  label: string;
  options: readonly InputSourceOption[];
  onChange: (value: string) => void;
  wide?: boolean;
}) {
  const hasOption = options.some((option) => option.value === value);
  return (
    <select
      className={cx(styles.select, wide ? styles.selectWide : undefined)}
      aria-label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      disabled={options.length === 0}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
      {hasOption || value.length === 0 ? null : <option value={value}>{value}</option>}
    </select>
  );
}
