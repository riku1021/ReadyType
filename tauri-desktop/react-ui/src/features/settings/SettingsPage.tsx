import { useNavigate } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { useEffect, useMemo, useState } from "react";
import { MdBugReport, MdContrast, MdInfoOutline, MdKeyboard, MdRocketLaunch } from "react-icons/md";

import { themeAtom } from "@/atoms/theme";
import { AlertDialog } from "@/components/Dialog";
import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";
import { type InputSourceDto, useInputSourceState } from "@/features/inputSource";

import { buildInputSourceOptions } from "./inputSourceOptions";
import * as styles from "./SettingsPage.styles";
import { getThemeLabel, THEME_OPTIONS, themeFromLabel } from "./themeLabels";

export function SettingsPage() {
  const navigate = useNavigate();
  const [theme, setTheme] = useAtom(themeAtom);
  const {
    state,
    fetchInputSources,
    updateInputSourcePreferences,
    setDebugLogging,
    fetchAutostartEnabled,
    setAutostartEnabled,
    resetDefaultRules,
  } = useInputSourceState();

  const [sources, setSources] = useState<InputSourceDto[]>([]);
  const [autostartEnabled, setAutostartEnabledState] = useState(false);
  const [pending, setPending] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const [nextSources, nextAutostart] = await Promise.all([
          fetchInputSources(true),
          fetchAutostartEnabled(),
        ]);
        if (!cancelled) {
          setSources(nextSources);
          setAutostartEnabledState(nextAutostart);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : "設定の読み込みに失敗しました");
        }
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [fetchAutostartEnabled, fetchInputSources]);

  const englishOptions = useMemo(
    () =>
      buildInputSourceOptions(
        sources,
        state?.settingsPreferredEnglish ?? "com.apple.keylayout.ABC"
      ),
    [sources, state?.settingsPreferredEnglish]
  );
  const japaneseOptions = useMemo(
    () =>
      buildInputSourceOptions(
        sources,
        state?.settingsPreferredJapanese ?? "com.google.inputmethod.Japanese.base"
      ),
    [sources, state?.settingsPreferredJapanese]
  );

  const selectedEnglishLabel =
    englishOptions.find((option) => option.value === state?.settingsPreferredEnglish)?.label ??
    state?.settingsPreferredEnglish ??
    "未設定";
  const selectedJapaneseLabel =
    japaneseOptions.find((option) => option.value === state?.settingsPreferredJapanese)?.label ??
    state?.settingsPreferredJapanese ??
    "未設定";

  const selectPreferredEnglish = (label: string) => {
    const option = englishOptions.find((item) => item.label === label);
    if (!option || !state?.settingsPreferredJapanese) {
      return;
    }
    setPending(true);
    void updateInputSourcePreferences(option.value, state.settingsPreferredJapanese)
      .catch((updateError) => {
        setError(updateError instanceof Error ? updateError.message : "保存に失敗しました");
      })
      .finally(() => setPending(false));
  };

  const selectPreferredJapanese = (label: string) => {
    const option = japaneseOptions.find((item) => item.label === label);
    if (!option || !state?.settingsPreferredEnglish) {
      return;
    }
    setPending(true);
    void updateInputSourcePreferences(state.settingsPreferredEnglish, option.value)
      .catch((updateError) => {
        setError(updateError instanceof Error ? updateError.message : "保存に失敗しました");
      })
      .finally(() => setPending(false));
  };

  return (
    <div className={styles.page}>
      <SettingsSection title="入力ソース">
        <SettingItem
          icon={MdKeyboard}
          title="英数入力ソース"
          type="selection"
          value={selectedEnglishLabel}
          options={englishOptions.map((option) => option.label)}
          onSelectionChanged={selectPreferredEnglish}
        />
        <SettingItem
          icon={MdKeyboard}
          title="日本語入力ソース"
          type="selection"
          value={selectedJapaneseLabel}
          options={japaneseOptions.map((option) => option.label)}
          onSelectionChanged={selectPreferredJapanese}
        />
        <SettingItem
          icon={MdBugReport}
          title="デバッグログ"
          type="toggle"
          toggleValue={state?.debugLogging ?? false}
          onToggleChanged={(enabled) => {
            setPending(true);
            void setDebugLogging(enabled)
              .catch((updateError) => {
                setError(updateError instanceof Error ? updateError.message : "保存に失敗しました");
              })
              .finally(() => setPending(false));
          }}
        />
      </SettingsSection>

      <SettingsSection title="起動">
        <SettingItem
          icon={MdRocketLaunch}
          title="ログイン時に起動"
          type="toggle"
          toggleValue={autostartEnabled}
          onToggleChanged={(enabled) => {
            setPending(true);
            void setAutostartEnabled(enabled)
              .then(setAutostartEnabledState)
              .catch((updateError) => {
                setError(updateError instanceof Error ? updateError.message : "保存に失敗しました");
              })
              .finally(() => setPending(false));
          }}
        />
      </SettingsSection>

      <SettingsSection title="外観">
        <SettingItem
          icon={MdContrast}
          title="テーマ"
          type="selection"
          value={getThemeLabel(theme)}
          options={THEME_OPTIONS}
          onSelectionChanged={(label) => {
            const nextTheme = themeFromLabel(label);
            if (nextTheme) {
              setTheme(nextTheme);
            }
          }}
        />
      </SettingsSection>

      <SettingsSection title="その他">
        <SettingItem
          icon={MdInfoOutline}
          title="アプリについて"
          type="navigation"
          onClick={() => {
            void navigate({ to: "/about" });
          }}
        />
        <SettingItem
          icon={MdBugReport}
          title="既定設定へリセット"
          type="action"
          danger
          onClick={() => setResetOpen(true)}
        />
      </SettingsSection>

      {error ? <p>{error}</p> : null}
      {pending ? <p>保存中…</p> : null}

      <AlertDialog
        isOpen={resetOpen}
        onClose={() => setResetOpen(false)}
        title="既定設定へ戻しますか？"
        text="ルール一覧を既定プリセットへ戻します。"
        confirmButtonText="リセット"
        onConfirm={() => {
          void resetDefaultRules().catch((resetError) => {
            setError(resetError instanceof Error ? resetError.message : "リセットに失敗しました");
          });
        }}
      />
    </div>
  );
}
