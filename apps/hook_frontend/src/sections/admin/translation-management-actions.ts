'use client';

import type { TranslationDeleteState, TranslationValueFormState, TranslationLanguageFormState } from './translation-management-state';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import {
  deleteTranslationEntry,
  upsertTranslationBundle,
  deleteTranslationLanguage,
  createTranslationLanguage,
  updateTranslationLanguage,
} from 'src/actions/i18n';

import { toast } from 'src/components/snackbar';

type Options = {
  deleteState: TranslationDeleteState;
  languageForm: TranslationLanguageFormState;
  valueForm: TranslationValueFormState;
};

export function useTranslationManagementActions({ deleteState, languageForm, valueForm }: Options) {
  const [submitting, setSubmitting] = useState(false);
  const value = useValueActions({ deleteState, setSubmitting, valueForm });
  const language = useLanguageActions({ deleteState, languageForm, setSubmitting });

  return { ...value, ...language, submitting };
}

function useValueActions({
  deleteState,
  setSubmitting,
  valueForm,
}: Pick<Options, 'deleteState' | 'valueForm'> & SubmitState) {
  const { t } = useTranslate('admin');

  const submitValue = useCallback(async () => {
    setSubmitting(true);
    try {
      const form = valueForm.form;
      await upsertTranslationBundle(form.namespace, form.group_key, form.item_key, form.values, form.enabled);
      toast.success(t(valueForm.editing ? 'translations.messages.valueUpdated' : 'translations.messages.valueCreated'));
      valueForm.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [setSubmitting, t, valueForm]);

  const confirmDeleteValue = useCallback(async () => {
    if (!deleteState.valueTarget) return;
    try {
      await Promise.all(deleteState.valueTarget.entries.map((entry) => deleteTranslationEntry(entry.id)));
      toast.success(t('translations.messages.valueDeleted'));
      deleteState.setValueTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteState, t]);

  return { confirmDeleteValue, submitValue };
}

function useLanguageActions({
  deleteState,
  languageForm,
  setSubmitting,
}: Pick<Options, 'deleteState' | 'languageForm'> & SubmitState) {
  const { t } = useTranslate('admin');

  const submitLanguage = useCallback(async () => {
    setSubmitting(true);
    try {
      if (languageForm.editing) {
        await updateTranslationLanguage(languageForm.editing.code, languageForm.form);
      } else {
        await createTranslationLanguage(languageForm.form);
      }
      toast.success(t(languageForm.editing ? 'translations.messages.languageUpdated' : 'translations.messages.languageCreated'));
      languageForm.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [languageForm, setSubmitting, t]);

  const confirmDeleteLanguage = useCallback(async () => {
    if (!deleteState.languageTarget) return;
    try {
      await deleteTranslationLanguage(deleteState.languageTarget.code);
      toast.success(t('translations.messages.languageDeleted'));
      deleteState.setLanguageTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteState, t]);

  return { confirmDeleteLanguage, submitLanguage };
}

type SubmitState = {
  setSubmitting: (value: boolean) => void;
};

