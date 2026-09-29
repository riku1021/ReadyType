import registryJson from "@/locales/registry.json";

export type UiLocaleRegistryEntry = {
  code: string;
  nativeName: string;
};

export type UiLocaleRegistry = {
  defaultPreference: string;
  defaultResolved: string;
  locales: UiLocaleRegistryEntry[];
};

/** FE-only UI locale registry (select labels / defaults). Rust uses tray.json keys. */
export const UI_LOCALE_REGISTRY = registryJson as UiLocaleRegistry;

export const SYSTEM_LANGUAGE_PREFERENCE = "system";

export const SUPPORTED_LANGUAGES = UI_LOCALE_REGISTRY.locales.map(
  (locale) => locale.code
) as readonly string[];

export const LANGUAGE_PREFERENCES = [
  SYSTEM_LANGUAGE_PREFERENCE,
  ...SUPPORTED_LANGUAGES,
] as const;

export type ResolvedUiLocale = (typeof SUPPORTED_LANGUAGES)[number];
export type UiLocalePreference = (typeof LANGUAGE_PREFERENCES)[number];

export function isResolvedUiLocale(value: string): value is ResolvedUiLocale {
  return SUPPORTED_LANGUAGES.includes(value);
}

export function isUiLocalePreference(value: string): value is UiLocalePreference {
  return (LANGUAGE_PREFERENCES as readonly string[]).includes(value);
}

export function nativeNameForLocale(code: string): string | undefined {
  return UI_LOCALE_REGISTRY.locales.find((locale) => locale.code === code)?.nativeName;
}
