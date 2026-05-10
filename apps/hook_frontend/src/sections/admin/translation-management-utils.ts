'use client';

import type { TranslationEntry, TranslationLanguage, TranslationEntryInput, TranslationLanguageInput } from 'src/types/i18n';

export type TranslationValueForm = {
  namespace: string;
  group_key: string;
  item_key: string;
  values: Record<string, string>;
  enabled: boolean;
};

export type TranslationLanguageForm = TranslationLanguageInput;

export type TranslationValueRow = {
  id: string;
  namespace: string;
  group_key: string;
  item_key: string;
  enabled: boolean;
  entries: TranslationEntry[];
  values: Record<string, string>;
};

export const DEFAULT_TRANSLATION_VALUE_FORM: TranslationValueForm = {
  namespace: 'admin',
  group_key: '',
  item_key: '',
  values: {},
  enabled: true,
};

export const DEFAULT_TRANSLATION_LANGUAGE_FORM: TranslationLanguageForm = {
  code: '',
  name: '',
  native_name: '',
  enabled: true,
  sort_order: 0,
};

export function translationRows(entries: TranslationEntry[]): TranslationValueRow[] {
  return Array.from(bundleMap(entries).values()).sort(compareRows);
}

export function valueFormFromRow(row: TranslationValueRow): TranslationValueForm {
  return {
    namespace: row.namespace,
    group_key: row.group_key,
    item_key: row.item_key,
    values: row.values,
    enabled: row.enabled,
  };
}

export function languageFormFromRecord(language: TranslationLanguage): TranslationLanguageForm {
  return {
    code: language.code,
    name: language.name,
    native_name: language.native_name,
    enabled: language.enabled,
    sort_order: language.sort_order,
  };
}

export function entryInputFromForm(form: TranslationValueForm, langCode: string): TranslationEntryInput {
  return {
    namespace: form.namespace,
    group_key: form.group_key,
    item_key: form.item_key,
    lang_code: langCode,
    value: form.values[langCode] ?? '',
    description: null,
    enabled: form.enabled,
  };
}

function bundleMap(entries: TranslationEntry[]) {
  const map = new Map<string, TranslationValueRow>();
  for (const entry of entries) {
    const id = rowId(entry);
    const row = map.get(id) ?? emptyRow(id, entry);
    row.entries.push(entry);
    row.values[entry.lang_code] = entry.value;
    row.enabled = row.enabled && entry.enabled;
    map.set(id, row);
  }
  return map;
}

function emptyRow(id: string, entry: TranslationEntry): TranslationValueRow {
  return {
    id,
    namespace: entry.namespace,
    group_key: entry.group_key,
    item_key: entry.item_key,
    enabled: entry.enabled,
    entries: [],
    values: {},
  };
}

function compareRows(left: TranslationValueRow, right: TranslationValueRow) {
  return (
    left.namespace.localeCompare(right.namespace) ||
    left.group_key.localeCompare(right.group_key) ||
    left.item_key.localeCompare(right.item_key)
  );
}

function rowId(entry: TranslationEntry) {
  return `${entry.namespace}:${entry.group_key}:${entry.item_key}`;
}

