/** Lab 切替テスト欄。Rust `presets.rs` の AXIdentifier / 言語割当と一致させる。 */
export type LabSwitchLanguage = "ja" | "en";

export type LabSwitchField = {
  id: string;
  language: LabSwitchLanguage;
};

export const LAB_SWITCH_FIELDS: readonly LabSwitchField[] = [
  { id: "readytype-lab-switch-01", language: "ja" },
  { id: "readytype-lab-switch-02", language: "en" },
  { id: "readytype-lab-switch-03", language: "ja" },
  { id: "readytype-lab-switch-04", language: "en" },
  { id: "readytype-lab-switch-05", language: "ja" },
  { id: "readytype-lab-switch-06", language: "en" },
] as const;
