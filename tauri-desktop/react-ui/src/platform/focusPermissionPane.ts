import type { TFunction } from "i18next";

import { usesMacOsDeviceControlPermissionLabel } from "./hostOsVersion";

export type FocusPermissionPaneI18nKey =
  | "os.permissionPane.accessibility"
  | "os.permissionPane.deviceControl";

/** System Settings 上のフォーカス監視用権限ペイン名の i18n キー。 */
export function focusPermissionPaneI18nKey(): FocusPermissionPaneI18nKey {
  return usesMacOsDeviceControlPermissionLabel()
    ? "os.permissionPane.deviceControl"
    : "os.permissionPane.accessibility";
}

export function focusPermissionPaneLabel(t: TFunction): string {
  return t(focusPermissionPaneI18nKey());
}
