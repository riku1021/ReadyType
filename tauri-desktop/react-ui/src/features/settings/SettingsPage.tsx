import { useNavigate } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { useState } from "react";
import { IoPersonCircle } from "react-icons/io5";
import {
  MdBugReport,
  MdCalendarToday,
  MdChatBubble,
  MdCloudOff,
  MdContrast,
  MdInfoOutline,
  MdLanguage,
  MdPalette,
  MdRocketLaunch,
  MdShare,
  MdStars,
} from "react-icons/md";

import { themeAtom } from "@/atoms/theme";
import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";

import * as styles from "./SettingsPage.styles";
import { getThemeLabel, THEME_OPTIONS, themeFromLabel } from "./themeLabels";

const LANGUAGE_OPTIONS = ["日本語", "English", "中文"] as const;
const WEEK_START_OPTIONS = [
  "日曜日",
  "月曜日",
  "火曜日",
  "水曜日",
  "木曜日",
  "金曜日",
  "土曜日",
] as const;

export function SettingsPage() {
  const navigate = useNavigate();
  const [theme, setTheme] = useAtom(themeAtom);

  const [launchAtLoginEnabled, setLaunchAtLoginEnabled] = useState(false);
  const [selectedLanguage, setSelectedLanguage] = useState<string>("日本語");
  const [selectedWeekStart, setSelectedWeekStart] = useState<string>("日曜日");

  return (
    <div className={styles.page}>
      <SettingsSection title="アカウント">
        <SettingItem icon={IoPersonCircle} title="user@example.com" type="navigation" />
        <SettingItem
          icon={MdCloudOff}
          title="バックアップされていません"
          type="navigation"
          danger
        />
      </SettingsSection>

      <SettingsSection title="外観">
        <SettingItem
          icon={MdLanguage}
          title="言語"
          type="selection"
          value={selectedLanguage}
          options={LANGUAGE_OPTIONS}
          onSelectionChanged={setSelectedLanguage}
        />
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
        <SettingItem icon={MdPalette} title="アクセントカラー" type="navigation" />
        <SettingItem
          icon={MdCalendarToday}
          title="週の始まり"
          type="selection"
          value={selectedWeekStart}
          options={WEEK_START_OPTIONS}
          onSelectionChanged={setSelectedWeekStart}
        />
      </SettingsSection>

      <SettingsSection title="フィードバック">
        <SettingItem icon={MdChatBubble} title="機能追加のリクエスト" type="navigation" />
        <SettingItem icon={MdBugReport} title="不具合の報告" type="navigation" />
        <SettingItem icon={MdStars} title="アプリをレビューで応援する" type="action" />
      </SettingsSection>

      <SettingsSection title="その他">
        <SettingItem
          icon={MdRocketLaunch}
          title="ログイン時に起動"
          type="toggle"
          toggleValue={launchAtLoginEnabled}
          onToggleChanged={setLaunchAtLoginEnabled}
        />
        <SettingItem icon={MdShare} title="アプリをシェアする" type="action" />
        <SettingItem
          icon={MdInfoOutline}
          title="アプリについて"
          type="navigation"
          onClick={() => {
            void navigate({ to: "/about" });
          }}
        />
      </SettingsSection>
    </div>
  );
}
