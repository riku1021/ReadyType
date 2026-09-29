import translation from "./translation.json";

export type I18nNamespaces = {
  translation: typeof translation;
};

export const defaultNS = "translation";

declare module "i18next" {
  interface CustomTypeOptions {
    resources: I18nNamespaces;
    defaultNS: typeof defaultNS;
  }
}

export default {
  translation,
};
