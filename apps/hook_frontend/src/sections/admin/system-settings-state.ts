'use client';

import type { SystemSettings } from 'src/types/system-setting';
import type { SystemSettingsForm } from './system-settings-utils';
import type { SystemSettingsValidationContext } from './system-settings-validation';

import { useState, useEffect, useCallback } from 'react';

import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import { validateSystemSettingsBeforeSubmit } from './system-settings-validation';
import {
  settingsPayload,
  formFromSettings,
  DEFAULT_SETTINGS_FORM,
} from './system-settings-utils';

type SystemSettingsSubmitOptions = {
  save?: (form: SystemSettingsForm) => Promise<SystemSettings>;
  afterSave?: () => Promise<void>;
};

export function useSystemSettingsForm(
  settings: SystemSettings | undefined,
  t: (key: string) => string,
  validationContext: SystemSettingsValidationContext = {},
  options: SystemSettingsSubmitOptions = {}
) {
  const [form, setForm] = useState<SystemSettingsForm>(DEFAULT_SETTINGS_FORM);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (settings) {
      setForm(formFromSettings(settings));
    }
  }, [settings]);

  const submit = useCallback(async () => {
    const validationError = validateSystemSettingsBeforeSubmit(form, validationContext, t);
    if (validationError) {
      toast.error(validationError);
      return;
    }
    setSubmitting(true);
    try {
      const updated = await (options.save?.(form) ?? updateSystemSettings(settingsPayload(form)));
      setForm(formFromSettings(updated));
      await options.afterSave?.();
      toast.success(t('messages.systemSettingsUpdated'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [form, options, validationContext, t]);

  return { form, setForm, submit, submitting };
}
