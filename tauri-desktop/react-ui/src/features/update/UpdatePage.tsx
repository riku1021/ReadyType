import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { SettingItem } from "@/components/SettingItem";
import { SettingsSection } from "@/components/SettingsSection";

import * as styles from "./UpdatePage.styles";

// TODO(update): UI のみ。自動確認／自動ダウンロードの永続化、最終確認時刻の保存、
// 実 updater（チェック・ダウンロード・適用）との連携は未実装。

function formatLastChecked(date: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
}

export function UpdatePage() {
  const { t, i18n } = useTranslation();
  // TODO(update): 設定・最終確認はローカル state のみ（再起動で消える）
  const [autoCheck, setAutoCheck] = useState(true);
  const [autoDownload, setAutoDownload] = useState(true);
  const [lastCheckedAt, setLastCheckedAt] = useState<Date | null>(null);
  const [checking, setChecking] = useState(false);

  const lastCheckedLabel = useMemo(() => {
    const datetime =
      lastCheckedAt != null ? formatLastChecked(lastCheckedAt, i18n.language) : t("common.emDash");
    return t("update.lastChecked", { datetime });
  }, [i18n.language, lastCheckedAt, t]);

  const handleCheckNow = useCallback(() => {
    if (checking) {
      return;
    }
    setChecking(true);
    // TODO(update): updater 呼び出しに差し替え。現状は最終確認時刻の見た目更新のみ
    window.setTimeout(() => {
      setLastCheckedAt(new Date());
      setChecking(false);
    }, 400);
  }, [checking]);

  return (
    <div className={styles.page}>
      <SettingsSection>
        <SettingItem
          type="toggle"
          title={t("update.autoCheck")}
          toggleValue={autoCheck}
          onToggleChanged={setAutoCheck}
        />
        <SettingItem
          type="toggle"
          title={t("update.autoDownload")}
          toggleValue={autoDownload}
          onToggleChanged={setAutoDownload}
        />
      </SettingsSection>

      <SettingsSection>
        <SettingItem
          type="button"
          title={t("update.checkNow")}
          description={lastCheckedLabel}
          buttonLabel={checking ? t("update.checking") : t("update.checkButton")}
          disabled={checking}
          onClick={handleCheckNow}
        />
      </SettingsSection>
    </div>
  );
}
