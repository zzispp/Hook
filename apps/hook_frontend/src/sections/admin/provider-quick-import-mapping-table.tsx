'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { GlobalModelResponse } from 'src/types/model';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
  ProviderQuickImportModelMappingPreview,
} from 'src/types/provider-quick-import';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import Autocomplete from '@mui/material/Autocomplete';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { defaultMappings } from './provider-quick-import-utils';

type Props = {
  open: boolean;
  preview: ProviderQuickImportPreviewResponse;
  token?: ProviderQuickImportTokenPreview;
  models: GlobalModelResponse[];
  mappings: Record<string, string>;
  onClose: () => void;
  setMappings: Dispatch<SetStateAction<Record<string, string>>>;
};

export function ProviderQuickImportMappingDialog({ open, preview, token, models, mappings, onClose, setMappings }: Props) {
  const { t } = useTranslate('admin');
  const tokenModelIds = token?.models.map((model) => model.upstream_model_id) ?? [];
  const tokenModelSet = new Set(tokenModelIds);
  const visibleMappings = preview.model_mappings.filter(
    (mapping) => tokenModelSet.has(mapping.upstream_model_id) && mapping.upstream_model_id in mappings
  );

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle component="div">
        <Stack direction="row" spacing={1} alignItems="center">
          <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
            <Typography variant="h6">{t('providers.quickImportModelMappings')}</Typography>
            {token ? (
              <Typography variant="caption" color="text.secondary" noWrap>
                {token.name} · {token.masked_key}
              </Typography>
            ) : null}
          </Stack>
          <IconButton onClick={onClose}>
            <Iconify icon="mingcute:close-line" />
          </IconButton>
        </Stack>
      </DialogTitle>
      <DialogContent>
        <Stack spacing={1.5} sx={{ pt: 1 }}>
          <MappingActions preview={preview} tokenModelIds={tokenModelIds} setMappings={setMappings} />
          {visibleMappings.length === 0 ? (
            <Typography variant="body2" color="text.secondary">
              {t('common.noData')}
            </Typography>
          ) : null}
          {visibleMappings.map((mapping) => (
            <MappingRow
              key={mapping.upstream_model_id}
              mapping={mapping}
              models={models}
              value={mappings[mapping.upstream_model_id]}
              setMappings={setMappings}
            />
          ))}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.close')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function MappingActions({
  preview,
  tokenModelIds,
  setMappings,
}: {
  preview: ProviderQuickImportPreviewResponse;
  tokenModelIds: string[];
  setMappings: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1} alignItems="center" justifyContent="flex-end">
      <Button size="small" color="inherit" onClick={() => setMappings((current) => resetTokenMappings(preview, tokenModelIds, current))}>
        {t('providers.quickImportResetMappings')}
      </Button>
      <Button
        size="small"
        variant="outlined"
        onClick={() => setMappings((current) => mergeMissingMappings(preview, tokenModelIds, current))}
      >
        {t('providers.quickImportFetchModels')}
      </Button>
    </Stack>
  );
}

function MappingRow({
  mapping,
  models,
  value,
  setMappings,
}: {
  mapping: ProviderQuickImportModelMappingPreview;
  models: GlobalModelResponse[];
  value?: string;
  setMappings: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} alignItems={{ md: 'center' }}>
      <Typography variant="body2" sx={{ minWidth: 220, fontFamily: 'monospace' }}>
        {mapping.upstream_model_id}
      </Typography>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ flex: 1, minWidth: 0 }}>
        <Autocomplete
          fullWidth
          size="small"
          options={models}
          value={models.find((model) => model.id === value) ?? null}
          getOptionLabel={(model) => model.display_name || model.name}
          isOptionEqualToValue={(option, selected) => option.id === selected.id}
          onChange={(_, selected) =>
            setMappings((current) => ({ ...current, [mapping.upstream_model_id]: selected?.id ?? '' }))
          }
          renderOption={(props, model) => (
            <MenuItem {...props} key={model.id}>
              <Stack>
                <Typography variant="body2">{model.display_name}</Typography>
                <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
                  {model.name}
                </Typography>
              </Stack>
            </MenuItem>
          )}
          renderInput={(params) => (
            <TextField {...params} required label={t('providers.globalModels')} error={!value} />
          )}
        />
        <Tooltip title={t('providers.quickImportRemoveModel')}>
          <IconButton color="error" size="small" onClick={() => removeMapping(mapping.upstream_model_id, setMappings)}>
            <Iconify icon="mingcute:close-line" width={18} />
          </IconButton>
        </Tooltip>
      </Stack>
    </Stack>
  );
}

function resetTokenMappings(
  preview: ProviderQuickImportPreviewResponse,
  tokenModelIds: string[],
  current: Record<string, string>
) {
  const defaults = defaultMappings(preview);
  const next = { ...current };
  for (const id of tokenModelIds) {
    next[id] = defaults[id] ?? '';
  }
  return next;
}

function mergeMissingMappings(
  preview: ProviderQuickImportPreviewResponse,
  tokenModelIds: string[],
  current: Record<string, string>
) {
  const defaults = defaultMappings(preview);
  const additions = Object.fromEntries(
    tokenModelIds.filter((id) => !(id in current)).map((id) => [id, defaults[id] ?? ''])
  );
  return { ...current, ...additions };
}

function removeMapping(upstreamModelId: string, setMappings: Dispatch<SetStateAction<Record<string, string>>>) {
  setMappings((current) => {
    const next = { ...current };
    delete next[upstreamModelId];
    return next;
  });
}
