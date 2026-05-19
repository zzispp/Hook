'use client';

import type { ProviderCooldownPolicy } from 'src/types/provider';
import type { PolicyDialogState } from './provider-cooldown-policy-state';
import type { RuleForm, CooldownDurationMode } from './provider-cooldown-policy-utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import ToggleButton from '@mui/material/ToggleButton';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

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
  durationMode: CooldownDurationMode;
  onChange: (index: number, patch: Partial<RuleForm>) => void;
  onDelete: (index: number) => void;
};

type DurationModePickerProps = {
  value: CooldownDurationMode;
  onChange: (mode: CooldownDurationMode) => void;
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
        {state.rules.length > 0 && <PolicyFields state={state} />}
        <Stack spacing={1.5}>
          {state.rules.map((rule, index) => (
            <RuleRow
              key={index}
              rule={rule}
              index={index}
              canDelete={state.rules.length > 0}
              durationMode={state.durationMode}
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

function PolicyFields({ state }: { state: PolicyDialogState }) {
  return (
    <Stack spacing={2} sx={{ mt: 1 }}>
      <DurationModePicker value={state.durationMode} onChange={state.changeDurationMode} />
      {state.durationMode === 'fixed' && (
        <FixedCooldownField
          value={state.fixedCooldownSeconds}
          onChange={state.setFixedCooldownSeconds}
        />
      )}
    </Stack>
  );
}

function FixedCooldownField({
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
      label={t('providers.cooldownFixedSeconds')}
      value={value}
      onChange={(event) => onChange(event.target.value)}
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

function RuleRow({ rule, index, canDelete, durationMode, onChange, onDelete }: RuleRowProps) {
  const { t } = useTranslate('admin');
  const gridTemplateColumns =
    durationMode === 'fixed'
      ? { xs: '1fr', md: '1fr 1fr auto' }
      : { xs: '1fr', md: '1fr 1fr 1fr auto' };

  return (
    <Box
      sx={{
        gap: 1,
        display: 'grid',
        alignItems: 'center',
        gridTemplateColumns,
      }}
    >
      <NumberField
        label={t('providers.cooldownStatusCode')}
        value={rule.status_code}
        onChange={(status_code) => onChange(index, { status_code })}
      />
      <NumberField
        label={t('providers.cooldownFailureCount')}
        value={rule.failure_count}
        onChange={(failure_count) => onChange(index, { failure_count })}
      />
      {durationMode === 'per_rule' && (
        <NumberField
          label={t('providers.cooldownSeconds')}
          value={rule.cooldown_seconds}
          onChange={(cooldown_seconds) => onChange(index, { cooldown_seconds })}
        />
      )}
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

function DurationModePicker({ value, onChange }: DurationModePickerProps) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} alignItems={{ sm: 'center' }} spacing={1}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.cooldownDurationMode')}:
      </Typography>
      <ToggleButtonGroup
        exclusive
        size="small"
        value={value}
        onChange={(_, nextValue: CooldownDurationMode | null) => {
          if (nextValue) onChange(nextValue);
        }}
      >
        <ToggleButton value="fixed">{t('providers.cooldownDurationFixed')}</ToggleButton>
        <ToggleButton value="per_rule">{t('providers.cooldownDurationPerRule')}</ToggleButton>
      </ToggleButtonGroup>
    </Stack>
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
