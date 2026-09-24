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
  initError: string | null;
  ready: boolean;
};

export const initialInputSourceFeatureState: InputSourceFeatureState = {
  switcher: null,
  permission: null,
  settings: null,
  rules: [],
  initError: null,
  ready: false,
};

export const inputSourceStateAtom = atom<InputSourceFeatureState>(initialInputSourceFeatureState);
