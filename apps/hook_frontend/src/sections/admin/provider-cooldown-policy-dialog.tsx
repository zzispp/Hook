'use client';

import type { ProviderCooldownPolicy } from 'src/types/provider';
import type { RuleForm } from './provider-cooldown-policy-utils';
import type { PolicyDialogState } from './provider-cooldown-policy-state';

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

import { Iconify } from 'src/components/iconify';

import { usePolicyDialogState } from './provider-cooldown-policy-state';

type Props = {
  open: boolean;
  policy?: ProviderCooldownPolicy;
  onClose: () => void;
  onSaved: () => void;
};

type DialogActionsProps = {
  submitting: boolean;
  onClose: () => void;
  onSave: () => void;
};

type RuleRowProps = {
  rule: RuleForm;
  index: number;
  canDelete: boolean;
  onChange: (index: number, patch: Partial<RuleForm>) => void;
  onDelete: (index: number) => void;
};

export function ProviderCooldownPolicyDialog({ open, policy, onClose, onSaved }: Props) {
  const state = usePolicyDialogState({ open, policy, onClose, onSaved });

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <CooldownDialogTitle />
      <CooldownDialogContent state={state} />
      <CooldownDialogActions submitting={state.submitting} onClose={onClose} onSave={state.save} />
    </Dialog>
  );
}

function CooldownDialogTitle() {
  const { t } = useTranslate('admin');

  return <DialogTitle>{t('providers.cooldownPolicy')}</DialogTitle>;
}

function CooldownDialogContent({ state }: { state: PolicyDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <DialogContent dividers>
      <Stack spacing={2.5}>
        <WindowSecondsField value={state.windowSeconds} onChange={state.setWindowSeconds} />
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
  );
}

function WindowSecondsField({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      fullWidth
      type="number"
      label={t('providers.cooldownWindowSeconds')}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{
        inputLabel: {
          shrink: true,
          sx: { bgcolor: 'background.paper', px: 0.5, zIndex: 1 },
        },
      }}
      sx={{ mt: 1 }}
    />
  );
}

function CooldownDialogActions({ submitting, onClose, onSave }: DialogActionsProps) {
  const { t } = useTranslate('admin');

  return (
    <DialogActions>
      <Button color="inherit" variant="outlined" onClick={onClose}>
        {t('common.cancel')}
      </Button>
      <Button variant="contained" loading={submitting} onClick={onSave}>
        {t('common.save')}
      </Button>
    </DialogActions>
  );
}

function RuleRow({ rule, index, canDelete, onChange, onDelete }: RuleRowProps) {
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
      <TextField
        fullWidth
        size="small"
        label={t('providers.cooldownStatusCode')}
        value={rule.status_code}
        onChange={(event) => onChange(index, { status_code: event.target.value })}
      />
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
  return (
    <TextField
      fullWidth
      type="number"
      size="small"
      label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
