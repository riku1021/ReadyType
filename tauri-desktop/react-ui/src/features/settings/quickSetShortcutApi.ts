import { invoke } from "@tauri-apps/api/core";

type QuickSetShortcutSettings = {
  quickSetShortcut: string;
  quickSetShortcutRegistered: boolean;
};

export type QuickSetShortcutLoad = {
  accelerator: string;
  registered: boolean;
};

/** 設定全体から Quick Set ショートカットを読む（専用 get コマンドは持たない）。 */
export async function loadQuickSetShortcut(): Promise<QuickSetShortcutLoad> {
  const settings = await invoke<QuickSetShortcutSettings>("get_input_source_settings");
  return {
    accelerator: settings.quickSetShortcut,
    registered: settings.quickSetShortcutRegistered,
  };
}

export const setQuickSetShortcut = (shortcut: string) =>
  invoke<string>("set_quick_set_shortcut", { shortcut });
