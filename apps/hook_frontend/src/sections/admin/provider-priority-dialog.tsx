'use client';

import type { ProviderPriorityDialogProps } from './provider-priority-state';
import type { ProviderPriorityMode, ProviderSchedulingMode } from 'src/types/provider';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
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

import { ProviderPriorityList } from './provider-priority-list';
import { usePriorityDialogState } from './provider-priority-state';

export function ProviderPriorityDialog(props: ProviderPriorityDialogProps) {
  const state = usePriorityDialogState(props);

  return (
    <Dialog fullWidth maxWidth="md" open={props.open} onClose={props.onClose}>
      <PriorityDialogTitle onClose={props.onClose} />
      <DialogContent dividers sx={{ px: 3, py: 2 }}>
        {state.kind === 'key' ? (
          <PriorityFormatPicker
            formats={state.priorityFormats}
            value={state.activeFormat}
            onChange={state.setActiveFormat}
          />
        ) : null}
        <ProviderPriorityList
          items={state.kind === 'key' ? state.itemsByFormat[state.activeFormat] ?? [] : state.items}
          loading={props.loading}
          editingId={state.editingId}
          draggingId={state.draggingId}
          onDragStart={state.setDraggingId}
          onDragEnd={() => state.setDraggingId(null)}
          onDrop={state.dropOn}
          onEdit={state.setEditingId}
          onPriorityChange={state.changePriority}
        />
      </DialogContent>
      <PriorityDialogActions
        kind={state.kind}
        mode={state.mode}
        submitting={state.submitting}
        cacheAffinityTtlMinutes={state.cacheAffinityTtlMinutes}
        onKindChange={state.changeKind}
        onModeChange={state.setMode}
        onCacheAffinityTtlChange={state.setCacheAffinityTtlMinutes}
        onClose={props.onClose}
        onSave={state.save}
      />
    </Dialog>
  );
}

function PriorityFormatPicker({
  formats,
  value,
  onChange,
}: {
  formats: string[];
  value: string;
  onChange: (value: string) => void;
}) {
  if (formats.length === 0) return null;

  return (
    <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap" sx={{ pb: 1.5 }}>
      {formats.map((format) => (
        <Button
          key={format}
          size="small"
          variant={format === value ? 'contained' : 'outlined'}
          color={format === value ? 'primary' : 'inherit'}
          onClick={() => onChange(format)}
        >
          {format}
        </Button>
      ))}
    </Stack>
  );
}

function PriorityDialogTitle({ onClose }: { onClose: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <DialogTitle component="div" sx={{ px: 3, py: 2 }}>
      <Stack direction="row" alignItems="center" spacing={2}>
        <Box sx={{ p: 1, borderRadius: 1, bgcolor: 'primary.lighter', color: 'primary.main' }}>
          <Iconify icon="solar:list-bold" width={24} />
        </Box>
        <Box sx={{ minWidth: 0, flex: 1 }}>
          <Typography variant="h6">{t('providers.priorityManagement')}</Typography>
          <Typography variant="caption" color="text.secondary">
            {t('providers.priorityManagementHelper')}
          </Typography>
        </Box>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Stack>
    </DialogTitle>
  );
}

function PriorityDialogActions({
  kind,
  mode,
  submitting,
  cacheAffinityTtlMinutes,
  onKindChange,
  onModeChange,
  onCacheAffinityTtlChange,
  onClose,
  onSave,
}: {
  kind: ProviderPriorityMode;
  mode: ProviderSchedulingMode;
  submitting: boolean;
  cacheAffinityTtlMinutes: string;
  onKindChange: (kind: ProviderPriorityMode) => void;
  onModeChange: (mode: ProviderSchedulingMode) => void;
  onCacheAffinityTtlChange: (value: string) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <DialogActions sx={{ px: 3, py: 2, display: 'block' }}>
      <Stack direction={{ xs: 'column', md: 'row' }} alignItems={{ md: 'center' }} justifyContent="space-between" spacing={2}>
        <Stack direction={{ xs: 'column', sm: 'row' }} alignItems={{ sm: 'center' }} spacing={1.5}>
          <PriorityTargetPicker value={kind} onChange={onKindChange} />
          <Divider flexItem orientation="vertical" sx={{ display: { xs: 'none', sm: 'block' } }} />
          <Typography variant="caption" color="text.secondary">
            {t('providers.currentMode')}:{' '}
            <Box component="span" sx={{ color: 'text.primary', fontWeight: 600 }}>
              {schedulingModeLabel(mode, t)}
            </Box>
          </Typography>
          <Divider flexItem orientation="vertical" sx={{ display: { xs: 'none', sm: 'block' } }} />
          <SchedulingModePicker value={mode} onChange={onModeChange} />
          {mode === 'cache_affinity' && (
            <CacheAffinityTtlField
              value={cacheAffinityTtlMinutes}
              onChange={onCacheAffinityTtlChange}
            />
          )}
        </Stack>
        <Stack direction="row" justifyContent="flex-end" spacing={1}>
          <Button variant="outlined" color="inherit" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button variant="contained" loading={submitting} onClick={onSave}>
            {t('common.save')}
          </Button>
        </Stack>
      </Stack>
    </DialogActions>
  );
}

function PriorityTargetPicker({
  value,
  onChange,
}: {
  value: ProviderPriorityMode;
  onChange: (mode: ProviderPriorityMode) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center" spacing={1}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.priorityTarget')}:
      </Typography>
      <ToggleButtonGroup
        exclusive
        size="small"
        value={value}
        onChange={(_, nextValue: ProviderPriorityMode | null) => {
          if (nextValue) onChange(nextValue);
        }}
      >
        <ToggleButton value="provider">{t('providers.priorityTargetProvider')}</ToggleButton>
        <ToggleButton value="key">{t('providers.priorityTargetKey')}</ToggleButton>
      </ToggleButtonGroup>
    </Stack>
  );
}

function CacheAffinityTtlField({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      type="number"
      size="small"
      label={t('providers.cacheAffinityTtlMinutes')}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      inputProps={{ min: 1, step: 1 }}
      sx={{ width: { xs: 1, sm: 180 } }}
    />
  );
}

function schedulingModeLabel(mode: ProviderSchedulingMode, t: (key: string) => string) {
  const labels: Record<ProviderSchedulingMode, string> = {
    cache_affinity: t('providers.schedulingCacheAffinity'),
    fixed_order: t('providers.schedulingFixedOrder'),
    load_balance: t('providers.schedulingLoadBalance'),
  };

  return labels[mode];
}

function SchedulingModePicker({
  value,
  onChange,
}: {
  value: ProviderSchedulingMode;
  onChange: (mode: ProviderSchedulingMode) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center" spacing={1}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.scheduling')}:
      </Typography>
      <ToggleButtonGroup
        exclusive
        size="small"
        value={value}
        onChange={(_, nextValue: ProviderSchedulingMode | null) => {
          if (nextValue) onChange(nextValue);
        }}
      >
        <ToggleButton value="cache_affinity">{t('providers.schedulingCacheAffinity')}</ToggleButton>
        <ToggleButton value="load_balance">{t('providers.schedulingLoadBalance')}</ToggleButton>
        <ToggleButton value="fixed_order">{t('providers.schedulingFixedOrder')}</ToggleButton>
      </ToggleButtonGroup>
    </Stack>
  );
}
