import { invoke } from "@tauri-apps/api/core";
import type { SwitcherStateDto } from "@/features/inputSource";
import type { QuickSetSessionDto } from "./types";

export const getQuickSetSession = () => invoke<QuickSetSessionDto | null>("get_quick_set_session");

export const applyQuickSetLanguage = (language: string) =>
  invoke<SwitcherStateDto>("apply_quick_set_language", { language });

export const hideQuickSet = () => invoke<void>("hide_quick_set");

export const openQuickSetAdvanced = (appId: string) =>
  invoke<void>("open_quick_set_advanced", { appId });
