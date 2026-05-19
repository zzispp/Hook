import type { ProviderCooldownPolicy } from 'src/types/provider';
import type { RuleForm } from './provider-cooldown-policy-utils';

import { useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import { emptyRule, policyPayload, initialPolicyForm } from './provider-cooldown-policy-utils';

type Props = {
  open: boolean;
  policy?: ProviderCooldownPolicy;
  onClose: () => void;
  onSaved: () => void;
};

export type PolicyDialogState = ReturnType<typeof usePolicyDialogState>;

export function usePolicyDialogState({ open, policy, onClose, onSaved }: Props) {
  const form = usePolicyFormState(open, policy);
  const { t } = useTranslate('admin');
  const [submitting, setSubmitting] = useState(false);

  const save = useCallback(async () => {
    try {
      setSubmitting(true);
      await savePolicy({ form, t });
      toast.success(t('messages.providerCooldownPolicyUpdated'));
      onSaved();
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [form, onClose, onSaved, t]);

  return { ...form, save, submitting };
}

function usePolicyFormState(open: boolean, policy?: ProviderCooldownPolicy) {
  const [windowSeconds, setWindowSeconds] = useState('');
  const [rules, setRules] = useState<RuleForm[]>([]);

  useEffect(() => {
    if (!open) return;
    const nextForm = initialPolicyForm(policy);
    setWindowSeconds(nextForm.windowSeconds);
    setRules(nextForm.rules);
  }, [open, policy]);

  const changeRule = useCallback((index: number, patch: Partial<RuleForm>) => {
    setRules((current) =>
      current.map((rule, itemIndex) => (itemIndex === index ? { ...rule, ...patch } : rule))
    );
  }, []);

  const deleteRule = useCallback((index: number) => {
    setRules((current) => current.filter((_rule, itemIndex) => itemIndex !== index));
  }, []);

  const addRule = useCallback(() => {
    setRules((current) => [...current, emptyRule()]);
  }, []);

  return {
    addRule,
    changeRule,
    deleteRule,
    rules,
    setWindowSeconds,
    windowSeconds,
  };
}

async function savePolicy({
  form,
  t,
}: {
  form: ReturnType<typeof usePolicyFormState>;
  t: (key: string) => string;
}) {
  await updateSystemSettings({
    provider_cooldown_policy: policyPayload({
      windowSeconds: form.windowSeconds,
      rules: form.rules,
      t,
    }),
  });
}
