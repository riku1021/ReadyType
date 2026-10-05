import { useEffect, useState } from "react";
import {
  IoApps,
  IoChatbubbleEllipses,
  IoCreate,
  IoDocumentText,
  IoOptions,
  IoTerminal,
} from "react-icons/io5";

import {
  type AppPresentationDto,
  getAppPresentations,
  type InputSourceOption,
  visibleInputSourceOptions,
} from "@/features/inputSource";
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

/** アイコンと表示名を 1 IPC で取得する。 */
export function useAppPresentations(appIds: readonly string[]): Record<string, AppPresentationDto> {
  const [values, setValues] = useState<Record<string, AppPresentationDto>>({});
  const key = [...appIds].sort().join("\n");

  useEffect(() => {
    const ids = key ? key.split("\n") : [];
    if (ids.length === 0) {
      return;
    }
    let cancelled = false;
    getAppPresentations(ids)
      .then((result) => {
        if (!cancelled) {
          setValues(result);
        }
      })
      .catch(() => {
        // ブラウザ単体起動やネイティブ未接続では取得できない
      });
    return () => {
      cancelled = true;
    };
  }, [key]);

  return values;
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
  const visibleOptions = visibleInputSourceOptions(options, value);
  const selected = visibleOptions.find((option) => option.value === value);
  const pinned =
    selected ?? (value.length > 0 ? { value, label: value, available: false as const } : null);
  const others = visibleOptions.filter((option) => option.value !== value);

  return (
    <select
      className={cx(styles.select, wide ? styles.selectWide : undefined)}
      aria-label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      disabled={visibleOptions.length === 0 && !pinned}
    >
      {pinned ? (
        <option key={pinned.value} value={pinned.value}>
          {pinned.label}
        </option>
      ) : null}
      {pinned && others.length > 0 ? <hr /> : null}
      {others.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
