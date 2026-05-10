'use client';

import type { TranslationLanguage } from 'src/types/i18n';
import type { TranslationValueRow, TranslationValueForm, TranslationLanguageForm } from './translation-management-utils';

import { useState, useCallback } from 'react';

import {
  valueFormFromRow,
  languageFormFromRecord,
  DEFAULT_TRANSLATION_VALUE_FORM,
  DEFAULT_TRANSLATION_LANGUAGE_FORM,
} from './translation-management-utils';

export type TranslationTab = 'values' | 'languages';

export function useTranslationValueForm(languages: TranslationLanguage[]) {
  const [form, setForm] = useState<TranslationValueForm>(DEFAULT_TRANSLATION_VALUE_FORM);
  const [editing, setEditing] = useState<TranslationValueRow | null>(null);
  const [creating, setCreating] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_TRANSLATION_VALUE_FORM, values: defaultValues(languages) });
  }, [languages]);

  const openEdit = useCallback((row: TranslationValueRow) => {
    setEditing(row);
    setCreating(false);
    setForm(valueFormFromRow(row));
  }, []);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_TRANSLATION_VALUE_FORM);
  }, []);

  return { close, creating, editing, form, open: creating || !!editing, openCreate, openEdit, setForm };
}

export function useTranslationLanguageForm() {
  const [form, setForm] = useState<TranslationLanguageForm>(DEFAULT_TRANSLATION_LANGUAGE_FORM);
  const [editing, setEditing] = useState<TranslationLanguage | null>(null);
  const [creating, setCreating] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_TRANSLATION_LANGUAGE_FORM);
  }, []);

  const openEdit = useCallback((language: TranslationLanguage) => {
    setEditing(language);
    setCreating(false);
    setForm(languageFormFromRecord(language));
  }, []);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_TRANSLATION_LANGUAGE_FORM);
  }, []);

  return { close, creating, editing, form, open: creating || !!editing, openCreate, openEdit, setForm };
}

export function useTranslationDeleteState() {
  const [valueTarget, setValueTarget] = useState<TranslationValueRow | null>(null);
  const [languageTarget, setLanguageTarget] = useState<TranslationLanguage | null>(null);

  return { languageTarget, setLanguageTarget, setValueTarget, valueTarget };
}

function defaultValues(languages: TranslationLanguage[]) {
  return Object.fromEntries(languages.map((language) => [language.code, '']));
}

export type TranslationValueFormState = ReturnType<typeof useTranslationValueForm>;
export type TranslationLanguageFormState = ReturnType<typeof useTranslationLanguageForm>;
export type TranslationDeleteState = ReturnType<typeof useTranslationDeleteState>;

