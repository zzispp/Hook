'use client';

import type { SystemSettings } from 'src/types/system-setting';
import type { SystemSettingsForm } from './system-settings-utils';

import { useState, useEffect, useCallback } from 'react';

import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import {
  settingsPayload,
  formFromSettings,
  DEFAULT_SETTINGS_FORM,
} from './system-settings-utils';

export function useSystemSettingsForm(
  settings: SystemSettings | undefined,
  t: (key: string) => string
) {
  const [form, setForm] = useState<SystemSettingsForm>(DEFAULT_SETTINGS_FORM);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (settings) {
      setForm(formFromSettings(settings));
    }
  }, [settings]);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      const updated = await updateSystemSettings(settingsPayload(form));
      setForm(formFromSettings(updated));
      toast.success(t('messages.systemSettingsUpdated'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [form, t]);

  return { form, setForm, submit, submitting };
}
