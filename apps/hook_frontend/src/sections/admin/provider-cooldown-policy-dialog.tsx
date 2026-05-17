'use client';

import type { ProviderCooldownRule, ProviderCooldownPolicy } from 'src/types/provider';

import { useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type RuleForm = {
  status_code: string;
  failure_count: string;
  cooldown_seconds: string;
};

type Props = {
  open: boolean;
  policy?: ProviderCooldownPolicy;
  onClose: () => void;
  onSaved: () => void;
};

const EMPTY_POLICY: ProviderCooldownPolicy = { window_seconds: 0, rules: [] };
const MIN_STATUS_CODE = 100;
const MAX_STATUS_CODE = 599;

export function ProviderCooldownPolicyDialog({ open, policy, onClose, onSaved }: Props) {
  const state = usePolicyDialogState({ open, policy, onClose, onSaved });
  const { t } = useTranslate('admin');

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle>{t('providers.cooldownPolicy')}</DialogTitle>
      <DialogContent dividers>
        <Stack spacing={2.5}>
          {state.rules.length > 0 && (
            <TextField
              fullWidth
              type="number"
              label={t('providers.cooldownWindowSeconds')}
              value={state.windowSeconds}
              onChange={(event) => state.setWindowSeconds(event.target.value)}
              sx={{ mt: 1 }}
            />
          )}
          <Stack spacing={1.5}>
            {state.rules.map((rule, index) => (
              <RuleRow
                key={index}
                rule={rule}
                index={index}
                canDelete={state.rules.length > 0}
                onChange={state.changeRule}
                onDelete={state.deleteRule}
              />
            ))}
          </Stack>
          <Box>
            <Button startIcon={<Iconify icon="mingcute:add-line" />} onClick={state.addRule}>
              {t('providers.addCooldownRule')}
            </Button>
          </Box>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button color="inherit" variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={state.submitting} onClick={state.save}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function RuleRow({
  rule,
  index,
  canDelete,
  onChange,
  onDelete,
}: {
  rule: RuleForm;
  index: number;
  canDelete: boolean;
  onChange: (index: number, patch: Partial<RuleForm>) => void;
  onDelete: (index: number) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box
      sx={{
        gap: 1,
        display: 'grid',
        alignItems: 'center',
        gridTemplateColumns: { xs: '1fr', md: '1fr 1fr 1fr auto' },
      }}
    >
      <NumberField label={t('providers.cooldownStatusCode')} value={rule.status_code} onChange={(status_code) => onChange(index, { status_code })} />
      <NumberField
        label={t('providers.cooldownFailureCount')}
        value={rule.failure_count}
        onChange={(failure_count) => onChange(index, { failure_count })}
      />
      <NumberField
        label={t('providers.cooldownSeconds')}
        value={rule.cooldown_seconds}
        onChange={(cooldown_seconds) => onChange(index, { cooldown_seconds })}
      />
      <Tooltip title={t('providers.deleteCooldownRule')}>
        <span>
          <IconButton color="error" disabled={!canDelete} onClick={() => onDelete(index)}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

function NumberField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return <TextField fullWidth type="number" size="small" label={label} value={value} onChange={(event) => onChange(event.target.value)} />;
}

function usePolicyDialogState({ open, policy, onClose, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [windowSeconds, setWindowSeconds] = useState('');
  const [rules, setRules] = useState<RuleForm[]>([]);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!open) return;
    const current = policy ?? EMPTY_POLICY;
    setWindowSeconds(String(current.window_seconds || 0));
    setRules(current.rules.map(ruleForm));
  }, [open, policy]);

  const changeRule = useCallback((index: number, patch: Partial<RuleForm>) => {
    setRules((current) => current.map((rule, itemIndex) => (itemIndex === index ? { ...rule, ...patch } : rule)));
  }, []);

  const deleteRule = useCallback((index: number) => {
    setRules((current) => current.filter((_rule, itemIndex) => itemIndex !== index));
  }, []);

  const addRule = useCallback(() => {
    setRules((current) => [...current, { status_code: '', failure_count: '', cooldown_seconds: '' }]);
  }, []);

  const save = useCallback(async () => {
    try {
      setSubmitting(true);
      await updateSystemSettings({ provider_cooldown_policy: policyPayload(windowSeconds, rules, t) });
      toast.success(t('messages.providerCooldownPolicyUpdated'));
      onSaved();
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [onClose, onSaved, rules, t, windowSeconds]);

  return { addRule, changeRule, deleteRule, rules, save, setWindowSeconds, submitting, windowSeconds };
}

function policyPayload(
  windowSeconds: string,
  rules: RuleForm[],
  t: (key: string) => string
): ProviderCooldownPolicy {
  if (rules.length === 0) return EMPTY_POLICY;
  const window = positiveInteger(windowSeconds);
  if (window === null) {
    throw new Error(t('messages.providerCooldownWindowRequired'));
  }
  const statusCodes = new Set<number>();
  return {
    window_seconds: window,
    rules: rules.map((rule) => rulePayload(rule, statusCodes, t)),
  };
}

function rulePayload(
  rule: RuleForm,
  statusCodes: Set<number>,
  t: (key: string) => string
): ProviderCooldownRule {
  if (ruleIncomplete(rule)) {
    throw new Error(t('messages.providerCooldownRuleRequired'));
  }
  const statusCode = integerValue(rule.status_code);
  const failureCount = positiveInteger(rule.failure_count);
  const cooldownSeconds = positiveInteger(rule.cooldown_seconds);
  if (statusCode === null || statusCode < MIN_STATUS_CODE || statusCode > MAX_STATUS_CODE) {
    throw new Error(t('messages.providerCooldownStatusCodeInvalid'));
  }
  if (failureCount === null || cooldownSeconds === null) {
    throw new Error(t('messages.providerCooldownPositiveValuesRequired'));
  }
  if (statusCodes.has(statusCode)) {
    throw new Error(t('messages.providerCooldownDuplicateStatusCode'));
  }
  statusCodes.add(statusCode);
  return {
    status_code: statusCode,
    failure_count: failureCount,
    cooldown_seconds: cooldownSeconds,
  };
}

function ruleForm(rule: ProviderCooldownRule): RuleForm {
  return {
    status_code: String(rule.status_code),
    failure_count: String(rule.failure_count),
    cooldown_seconds: String(rule.cooldown_seconds),
  };
}

function ruleIncomplete(rule: RuleForm) {
  return !rule.status_code.trim() || !rule.failure_count.trim() || !rule.cooldown_seconds.trim();
}

function positiveInteger(value: string) {
  const number = integerValue(value);
  return number !== null && number > 0 ? number : null;
}

function integerValue(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const number = Number(trimmed);
  return Number.isInteger(number) ? number : null;
}
