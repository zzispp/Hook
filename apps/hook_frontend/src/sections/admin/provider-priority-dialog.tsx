'use client';

import type { PriorityItem } from './provider-priority-utils';
import type { Provider, ProviderSchedulingMode } from 'src/types/provider';

import { useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import ToggleButton from '@mui/material/ToggleButton';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/locales/use-locales';
import { updateSchedulingMode } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { ProviderPriorityList } from './provider-priority-list';
import {
  orderProviders,
  parsePriorities,
  movePriorityItem,
  changeItemPriority,
  savePriorityChanges,
} from './provider-priority-utils';

type ProviderPriorityDialogProps = {
  open: boolean;
  providers: Provider[];
  loading: boolean;
  schedulingMode: ProviderSchedulingMode;
  onClose: () => void;
  onSaved: () => void;
};

export function ProviderPriorityDialog(props: ProviderPriorityDialogProps) {
  const state = usePriorityDialogState(props);

  return (
    <Dialog fullWidth maxWidth="md" open={props.open} onClose={props.onClose}>
      <PriorityDialogTitle onClose={props.onClose} />
      <DialogContent dividers sx={{ px: 3, py: 2 }}>
        <ProviderPriorityList
          items={state.items}
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
        mode={state.mode}
        submitting={state.submitting}
        onModeChange={state.setMode}
        onClose={props.onClose}
        onSave={state.save}
      />
    </Dialog>
  );
}

function usePriorityDialogState({
  open,
  providers,
  schedulingMode,
  onClose,
  onSaved,
}: ProviderPriorityDialogProps) {
  const { t } = useTranslate('admin');
  const [items, setItems] = useState(orderProviders(providers));
  const [mode, setMode] = useState<ProviderSchedulingMode>(schedulingMode);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!open) return;
    setItems(orderProviders(providers));
    setMode(schedulingMode);
    setEditingId(null);
    setDraggingId(null);
  }, [open, providers, schedulingMode]);

  const changePriority = useCallback((id: string, value: string) => {
    setItems((current) => changeItemPriority(current, id, value));
  }, []);

  const dropOn = useCallback(
    (targetId: string) => {
      if (!draggingId || draggingId === targetId) return;
      setItems((current) => movePriorityItem(current, draggingId, targetId));
      setDraggingId(null);
    },
    [draggingId]
  );

  const save = useCallback(async () => {
    const priorities = parsePriorities(items);
    if (!priorities) {
      toast.error(t('messages.providerPriorityInvalid'));
      return;
    }

    setSubmitting(true);
    try {
      await savePriorityState({ items, mode, priorities, providers, schedulingMode });
      toast.success(t('messages.providerPriorityUpdated'));
      onSaved();
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [items, mode, onClose, onSaved, providers, schedulingMode, t]);

  return { changePriority, draggingId, dropOn, editingId, items, mode, save, setDraggingId, setEditingId, setMode, submitting };
}

async function savePriorityState({
  items,
  mode,
  providers,
  priorities,
  schedulingMode,
}: {
  items: PriorityItem[];
  mode: ProviderSchedulingMode;
  providers: Provider[];
  priorities: Map<string, number>;
  schedulingMode: ProviderSchedulingMode;
}) {
  await savePriorityChanges(items, providers, priorities);
  if (mode !== schedulingMode) await updateSchedulingMode(mode);
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
  mode,
  submitting,
  onModeChange,
  onClose,
  onSave,
}: {
  mode: ProviderSchedulingMode;
  submitting: boolean;
  onModeChange: (mode: ProviderSchedulingMode) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <DialogActions sx={{ px: 3, py: 2, display: 'block' }}>
      <Stack direction={{ xs: 'column', md: 'row' }} alignItems={{ md: 'center' }} justifyContent="space-between" spacing={2}>
        <Stack direction={{ xs: 'column', sm: 'row' }} alignItems={{ sm: 'center' }} spacing={1.5}>
          <Typography variant="caption" color="text.secondary">
            {t('providers.currentMode')}: <Box component="span" sx={{ color: 'text.primary', fontWeight: 600 }}>{t('providers.providerFirst')}</Box>
          </Typography>
          <Divider flexItem orientation="vertical" sx={{ display: { xs: 'none', sm: 'block' } }} />
          <SchedulingModePicker value={mode} onChange={onModeChange} />
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
