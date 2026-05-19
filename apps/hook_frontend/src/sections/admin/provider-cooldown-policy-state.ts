import type { ProviderCooldownPolicy } from 'src/types/provider';
import type { RuleForm, CooldownDurationMode } from './provider-cooldown-policy-utils';

import { useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import {
  emptyRule,
  policyPayload,
  fixedCooldownSeed,
  initialPolicyForm,
} from './provider-cooldown-policy-utils';

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
  const [durationMode, setDurationMode] = useState<CooldownDurationMode>('fixed');
  const [fixedCooldownSeconds, setFixedCooldownSeconds] = useState('');
  const [rules, setRules] = useState<RuleForm[]>([]);

  useEffect(() => {
    if (!open) return;
    const nextForm = initialPolicyForm(policy);
    setDurationMode(nextForm.durationMode);
    setFixedCooldownSeconds(nextForm.fixedCooldownSeconds);
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
    setRules((current) => [...current, emptyRule(durationMode, fixedCooldownSeconds)]);
  }, [durationMode, fixedCooldownSeconds]);

  const changeDurationMode = useDurationModeChanger({
    fixedCooldownSeconds,
    rules,
    setFixedCooldownSeconds,
    setRules,
    setDurationMode,
  });

  return {
    addRule,
    changeDurationMode,
    changeRule,
    deleteRule,
    durationMode,
    fixedCooldownSeconds,
    rules,
    setFixedCooldownSeconds,
  };
}

function useDurationModeChanger({
  fixedCooldownSeconds,
  rules,
  setDurationMode,
  setFixedCooldownSeconds,
  setRules,
}: {
  fixedCooldownSeconds: string;
  rules: RuleForm[];
  setDurationMode: (mode: CooldownDurationMode) => void;
  setFixedCooldownSeconds: React.Dispatch<React.SetStateAction<string>>;
  setRules: React.Dispatch<React.SetStateAction<RuleForm[]>>;
}) {
  return useCallback(
    (nextMode: CooldownDurationMode) => {
      setDurationMode(nextMode);
      if (nextMode === 'fixed')
        setFixedCooldownSeconds((current) => current || fixedCooldownSeed(rules));
      if (nextMode === 'per_rule') {
        setRules((current) =>
          current.map((rule) => ({
            ...rule,
            cooldown_seconds: rule.cooldown_seconds || fixedCooldownSeconds,
          }))
        );
      }
    },
    [fixedCooldownSeconds, rules, setDurationMode, setFixedCooldownSeconds, setRules]
  );
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
      durationMode: form.durationMode,
      fixedCooldownSeconds: form.fixedCooldownSeconds,
      rules: form.rules,
      t,
    }),
  });
}
