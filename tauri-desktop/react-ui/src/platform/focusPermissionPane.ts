import type { TFunction } from "i18next";

import { getHostOs, type HostOs } from "./hostOs";
import { usesMacOsDeviceControlPermissionLabel } from "./hostOsVersion";

export type FocusPermissionPaneI18nKey =
  | "os.permissionPane.accessibility"
  | "os.permissionPane.deviceControl"
  | "os.permissionPane.focusMonitoring";

/** フォーカス監視に関連する OS 設定ペインまたは機能名の i18n キー。 */
export function focusPermissionPaneI18nKey(
  hostOs: HostOs = getHostOs()
): FocusPermissionPaneI18nKey {
  if (hostOs === "windows") {
    return "os.permissionPane.focusMonitoring";
  }
  return usesMacOsDeviceControlPermissionLabel()
    ? "os.permissionPane.deviceControl"
    : "os.permissionPane.accessibility";
}

export function focusPermissionPaneLabel(t: TFunction, hostOs: HostOs = getHostOs()): string {
  return t(focusPermissionPaneI18nKey(hostOs));
}
