import type { ApiEnvelope } from './rbac';

export type I18nResources = Record<string, unknown>;

export type I18nResourceResponse = {
  lang: string;
  namespace: string;
  resources: I18nResources;
};

export type TranslationLanguage = {
  code: string;
  name: string;
  native_name: string;
  enabled: boolean;
  system: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
};

export type TranslationLanguageInput = {
  code: string;
  name: string;
  native_name: string;
  enabled: boolean;
  sort_order: number;
};

export type TranslationEntry = {
  id: string;
  namespace: string;
  group_key: string;
  item_key: string;
  lang_code: string;
  value: string;
  description?: string | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
};

export type TranslationEntryInput = {
  namespace: string;
  group_key: string;
  item_key: string;
  lang_code: string;
  value: string;
  description?: string | null;
  enabled: boolean;
};

export type TranslationLanguageListResponse = {
  languages: TranslationLanguage[];
  total: number;
};

export type TranslationEntryListResponse = {
  translations: TranslationEntry[];
  total: number;
};

export type TranslationBundleResponse = {
  namespace: string;
  group_key: string;
  item_key: string;
  entries: TranslationEntry[];
};

export type I18nApiEnvelope<T> = ApiEnvelope<T>;

