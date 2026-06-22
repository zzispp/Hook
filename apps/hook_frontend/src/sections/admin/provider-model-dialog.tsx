'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderChildDialogs } from './provider-management-state';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogActions from '@mui/material/DialogActions';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderModels } from 'src/actions/providers';
import { batchUpdateProviderModels } from 'src/actions/provider-model-bindings';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type Props = {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  models: GlobalModelResponse[];
  providerId?: string;
  providerName?: string;
};

export function ProviderModelDialog({ dialogs, models, providerId, providerName }: Props) {
  const { t } = useTranslate('admin');
  const providerModels = useProviderModels(dialogs.modelOpen ? providerId : null);
  const [query, setQuery] = useState('');
  const [saving, setSaving] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [initialIds, setInitialIds] = useState<Set<string>>(new Set());
  const filteredModels = useMemo(() => filterModels(models, query), [models, query]);
  const changes = pendingChanges(selectedIds, initialIds);
  const allVisibleSelected = filteredModels.length > 0 && filteredModels.every((model) => selectedIds.has(model.id));

  useEffect(() => {
    if (!dialogs.modelOpen) {
      resetDialog();
      return;
    }
    const ids = new Set(providerModels.items.map((item) => item.global_model_id));
    setSelectedIds(ids);
    setInitialIds(ids);
  }, [dialogs.modelOpen, providerModels.items]);

  const resetDialog = () => {
    setQuery('');
    setSelectedIds(new Set());
    setInitialIds(new Set());
  };

  const toggleModel = (id: string) => {
    setSelectedIds((current) => nextSelectedIds(current, id));
  };

  const toggleAll = () => {
    setSelectedIds((current) => nextVisibleSelection(current, filteredModels, allVisibleSelected));
  };

  const close = () => {
    dialogs.closeModel();
  };

  const submit = async () => {
    if (!providerId || saving || changes.count === 0) return;
    setSaving(true);
    try {
      await saveChanges(providerId, models, providerModels.items, changes);
      toast.success(t('messages.providerModelBindingsSaved'));
      dialogs.closeModel();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="md" open={dialogs.modelOpen} onClose={close} PaperProps={{ sx: dialogPaperSx }}>
      <DialogHeader providerName={providerName} onClose={close} />
      <Box sx={contentSx}>
        <TextField
          fullWidth
          size="small"
          value={query}
          placeholder={t('providers.searchModels')}
          InputProps={{ startAdornment: <Iconify icon="eva:search-fill" width={18} /> }}
          onChange={(event) => setQuery(event.target.value)}
        />
        <Box sx={listShellSx}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" sx={listHeaderSx}>
            <Stack direction="row" spacing={1} alignItems="center">
              <Typography variant="caption" sx={{ fontWeight: 600 }}>
                {t('providers.globalModels')}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                ({filteredModels.length})
              </Typography>
            </Stack>
            <Button size="small" variant="text" onClick={toggleAll}>
              {allVisibleSelected ? t('common.clear') : t('common.selectAll')}
            </Button>
          </Stack>
          <Stack spacing={0.5} sx={modelListSx}>
            {providerModels.isLoading ? <Typography variant="body2">{t('common.loading')}</Typography> : null}
            {!providerModels.isLoading && filteredModels.length === 0 ? (
              <Typography variant="body2" color="text.secondary" sx={{ p: 3, textAlign: 'center' }}>
                {query ? t('common.noResults') : t('providers.noBindableModels')}
              </Typography>
            ) : null}
            {filteredModels.map((model) => (
              <ModelSelectRow
                key={model.id}
                model={model}
                checked={selectedIds.has(model.id)}
                onToggle={() => toggleModel(model.id)}
              />
            ))}
          </Stack>
        </Box>
      </Box>
      <DialogActions sx={footerSx}>
        <Typography variant="caption" color="text.secondary" sx={{ flexGrow: 1 }}>
          {changes.count > 0 ? t('providers.pendingModelChanges', { count: changes.count }) : ''}
        </Typography>
        <Button variant="contained" loading={saving} disabled={changes.count === 0} onClick={submit}>
          {t('common.save')}
        </Button>
        <Button variant="outlined" onClick={close}>
          {t('common.close')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DialogHeader({ providerName, onClose }: { providerName?: string; onClose: () => void }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" alignItems="center" spacing={1.5} sx={headerSx}>
      <Box sx={headerIconSx}>
        <Iconify icon="solar:list-bold" width={22} />
      </Box>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6" noWrap>
          {providerName ? t('dialogs.manageProviderModelsFor', { name: providerName }) : t('dialogs.manageProviderModels')}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {t('providers.modelAssociationHint')}
        </Typography>
      </Box>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

function ModelSelectRow({ model, checked, onToggle }: { model: GlobalModelResponse; checked: boolean; onToggle: () => void }) {
  return (
    <Stack direction="row" alignItems="center" spacing={1} sx={selectRowSx} onClick={onToggle}>
      <Box sx={checkboxSx(checked)}>{checked ? <Iconify icon="eva:checkmark-fill" width={12} /> : null}</Box>
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="body2" noWrap sx={{ fontWeight: 600 }}>
          {model.display_name}
        </Typography>
        <Typography variant="caption" noWrap sx={{ display: 'block', fontFamily: 'monospace', color: 'text.secondary' }}>
          {model.name}
        </Typography>
      </Box>
    </Stack>
  );
}

async function saveChanges(
  providerId: string,
  models: GlobalModelResponse[],
  current: ReturnType<typeof useProviderModels>['items'],
  changes: PendingChanges
) {
  await batchUpdateProviderModels(providerId, {
    create: providerModelCreates(models, changes.add),
    delete_ids: providerModelDeleteIds(current, changes.remove),
  });
}

type PendingChanges = { add: string[]; remove: string[]; count: number };

function pendingChanges(selectedIds: Set<string>, initialIds: Set<string>): PendingChanges {
  const add = [...selectedIds].filter((id) => !initialIds.has(id));
  const remove = [...initialIds].filter((id) => !selectedIds.has(id));
  return { add, remove, count: add.length + remove.length };
}

function filterModels(models: GlobalModelResponse[], query: string) {
  const normalized = query.trim().toLowerCase();
  return models
    .filter((model) => !normalized || model.name.toLowerCase().includes(normalized) || model.display_name.toLowerCase().includes(normalized))
    .sort((left, right) => left.display_name.localeCompare(right.display_name));
}

function providerModelCreates(models: GlobalModelResponse[], ids: string[]) {
  return ids.flatMap((id) => {
    const model = models.find((item) => item.id === id);
    return model ? [{ global_model_id: id }] : [];
  });
}

function providerModelDeleteIds(current: ReturnType<typeof useProviderModels>['items'], ids: string[]) {
  return ids.flatMap((id) => {
    const binding = current.find((item) => item.global_model_id === id);
    return binding ? [binding.id] : [];
  });
}

function nextSelectedIds(current: Set<string>, id: string) {
  const next = new Set(current);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

function nextVisibleSelection(current: Set<string>, models: GlobalModelResponse[], removeVisible: boolean) {
  const next = new Set(current);
  for (const model of models) {
    if (removeVisible) next.delete(model.id);
    else next.add(model.id);
  }
  return next;
}

const dialogPaperSx = { borderRadius: 1.5, overflow: 'hidden' };
const headerSx = { px: 3, py: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const headerIconSx = { width: 36, height: 36, borderRadius: 1, display: 'grid', placeItems: 'center', color: 'primary.main', bgcolor: 'primary.lighter' };
const contentSx = { px: 3, py: 2, display: 'grid', gap: 2, minHeight: 0 };
const listShellSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1, overflow: 'hidden' };
const listHeaderSx = { px: 1.5, py: 1, bgcolor: 'background.neutral', position: 'sticky', top: 0, zIndex: 1 };
const modelListSx = { p: 1, maxHeight: 384, overflowY: 'auto' };
const selectRowSx = { px: 1, py: 0.75, borderRadius: 1, cursor: 'pointer', '&:hover': { bgcolor: 'action.hover' } };
const footerSx = { px: 3, py: 2, borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, bgcolor: 'background.neutral' };

function checkboxSx(checked: boolean) {
  return { width: 16, height: 16, borderRadius: 0.5, border: (theme: Theme) => `1px solid ${checked ? theme.vars.palette.primary.main : theme.vars.palette.divider}`, bgcolor: checked ? 'primary.main' : 'transparent', color: 'primary.contrastText', display: 'grid', placeItems: 'center', flexShrink: 0 };
}
