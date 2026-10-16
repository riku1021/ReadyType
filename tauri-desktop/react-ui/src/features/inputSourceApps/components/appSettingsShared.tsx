import { useEffect, useState } from "react";

import { type InputSourceOption, visibleInputSourceOptions } from "@/features/inputSource";
import { cx } from "@/styled-system/css";
import {
  type AppPresentationView,
  ensureAppPresentationStyleInvalidation,
  loadAppPresentations,
  missingAppPresentationIds,
  peekAppPresentations,
  subscribeAppPresentationCache,
} from "../appPresentations";
import * as styles from "./appSettingsShared.styles";

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

/** アイコンと表示名を取得する（セッションキャッシュあり。未取得 id のみ IPC）。 */
export function useAppPresentations(
  appIds: readonly string[]
): Record<string, AppPresentationView> {
  const key = [...appIds].sort().join("\n");
  const [cacheEpoch, setCacheEpoch] = useState(0);
  const [values, setValues] = useState<Record<string, AppPresentationView>>(() =>
    key ? peekAppPresentations(key.split("\n")) : {}
  );

  useEffect(() => {
    ensureAppPresentationStyleInvalidation();
    return subscribeAppPresentationCache(() => {
      setCacheEpoch((epoch) => epoch + 1);
    });
  }, []);

  useEffect(() => {
    // cacheEpoch はスタイル無効化後の再取得トリガ（effect 本体では読まない）
    void cacheEpoch;
    const ids = key ? key.split("\n") : [];
    if (ids.length === 0) {
      setValues({});
      return;
    }
    setValues(peekAppPresentations(ids));
    if (missingAppPresentationIds(ids).length === 0) {
      return;
    }
    let cancelled = false;
    loadAppPresentations(ids)
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
  }, [key, cacheEpoch]);

  return values;
}

export function LanguageSelect({
  value,
  label,
  options,
  onChange,
  wide = false,
  noneLabel,
}: {
  value: string;
  label: string;
  options: readonly InputSourceOption[];
  onChange: (value: string) => void;
  wide?: boolean;
  /** 指定時、value="" の「設定しない」相当を選択肢に含める（選択中は常に最上段） */
  noneLabel?: string;
}) {
  const visibleOptions = visibleInputSourceOptions(options, value);
  const selected = visibleOptions.find((option) => option.value === value);
  const languagePinned =
    selected ?? (value.length > 0 ? { value, label: value, available: false as const } : null);
  const languageOthers = visibleOptions.filter((option) => option.value !== value);
  const noneOption = noneLabel != null ? { value: "", label: noneLabel } : null;
  const noneSelected = noneOption != null && value.length === 0;
  const pinned = noneSelected ? noneOption : languagePinned;
  const others = [...(noneOption != null && !noneSelected ? [noneOption] : []), ...languageOthers];

  return (
    <select
      className={cx(styles.select, wide ? styles.selectWide : undefined)}
      aria-label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      disabled={visibleOptions.length === 0 && pinned == null}
    >
      {pinned ? (
        <option key={pinned.value === "" ? "__none__" : pinned.value} value={pinned.value}>
          {pinned.label}
        </option>
      ) : null}
      {pinned && others.length > 0 ? <hr /> : null}
      {others.map((option) => (
        <option key={option.value === "" ? "__none__" : option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
