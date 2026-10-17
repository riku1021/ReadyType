import { atom } from "jotai";

import type {
  AccessibilityPermission,
  InputSourceSettingsDto,
  RuleDto,
  SwitcherStateDto,
} from "../types";

export type InputSourceFeatureState = {
  switcher: SwitcherStateDto | null;
  permission: AccessibilityPermission | null;
  settings: InputSourceSettingsDto | null;
  rules: RuleDto[];
  /** OS login-item status; `null` until first successful fetch (keep across remounts). */
  autostartEnabled: boolean | null;
  initError: string | null;
  ready: boolean;
};

export const initialInputSourceFeatureState: InputSourceFeatureState = {
  switcher: null,
  permission: null,
  settings: null,
  rules: [],
  autostartEnabled: null,
  initError: null,
  ready: false,
};

export const inputSourceStateAtom = atom<InputSourceFeatureState>(initialInputSourceFeatureState);
