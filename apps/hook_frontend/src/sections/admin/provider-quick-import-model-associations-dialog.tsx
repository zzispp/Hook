'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { GlobalModelResponse } from 'src/types/model';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type {
  ProviderQuickImportModelAssociationCandidate,
  ProviderQuickImportModelAssociationsResponse,
} from 'src/types/provider-quick-import';

import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

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
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';
import {
  getProviderQuickImportModelAssociations,
  updateProviderQuickImportModelAssociations,
} from 'src/actions/provider-quick-import';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type Props = {
  open: boolean;
  provider: Provider | null;
  apiKey: ProviderApiKey | null;
  models: GlobalModelResponse[];
  onClose: () => void;
};

export function ProviderQuickImportModelAssociationsDialog(props: Props) {
  const { open, provider, apiKey, models, onClose } = props;
  const { t } = useTranslate('admin');
  const [response, setResponse] = useState<ProviderQuickImportModelAssociationsResponse | null>(null);
  const [mappings, setMappings] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const loadingKeyRef = useRef<string | null>(null);
  const candidates = useMemo(() => visibleCandidates(response, mappings), [mappings, response]);
  const disabled = loading || submitting || Object.keys(mappings).length === 0 || Object.values(mappings).some((id) => !id);

  const load = useCallback(async () => {
    if (!provider || !apiKey) return;
    const loadKey = `${provider.id}:${apiKey.id}`;
    if (loadingKeyRef.current === loadKey) return;
    loadingKeyRef.current = loadKey;
    setLoading(true);
    try {
      const next = await getProviderQuickImportModelAssociations(provider.id, apiKey.id);
      setResponse(next);
      setMappings(associationMappings(next));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
    } finally {
      loadingKeyRef.current = null;
      setLoading(false);
    }
  }, [apiKey, provider, t]);

  useEffect(() => {
    if (open) void load();
  }, [load, open]);

  const close = () => {
    setResponse(null);
    setMappings({});
    setLoading(false);
    setSubmitting(false);
    loadingKeyRef.current = null;
    onClose();
  };

  const submit = async () => {
    if (!provider || !apiKey || disabled) return;
    setSubmitting(true);
    try {
      await updateProviderQuickImportModelAssociations(provider.id, apiKey.id, {
        model_mappings: associationPayload(mappings),
      });
      toast.success(t('messages.providerQuickImportAssociationsSaved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={close}>
      <DialogTitle component="div">
        <Header title={t('providers.quickImportModelAssociationsTitle')} subtitle={apiKey?.name} onClose={close} />
      </DialogTitle>
      <DialogContent>
        {loading ? <LoadingState /> : null}
        {!loading && response ? (
          <Stack spacing={2} sx={{ pt: 1 }}>
            <Typography variant="subtitle2">{t('providers.quickImportAssociatedModels')}</Typography>
            {Object.keys(mappings).map((upstreamId) => (
              <MappingRow
                key={upstreamId}
                upstreamId={upstreamId}
                value={mappings[upstreamId]}
                models={models}
                onChange={(globalId) => setMappings((current) => ({ ...current, [upstreamId]: globalId }))}
                onRemove={() => removeMapping(upstreamId, setMappings)}
              />
            ))}
            {Object.keys(mappings).length === 0 ? (
              <Typography variant="body2" color="text.secondary">
                {t('providers.quickImportNoAssociatedModels')}
              </Typography>
            ) : null}
            <Typography variant="subtitle2">{t('providers.quickImportModelCandidates')}</Typography>
            <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
              {candidates.map((candidate) => (
                <Button
                  key={candidate.upstream_model_id}
                  size="small"
                  variant="outlined"
                  startIcon={<Iconify icon="mingcute:add-line" />}
                  onClick={() => addCandidate(candidate, setMappings)}
                >
                  {candidate.upstream_model_id}
                </Button>
              ))}
              {candidates.length === 0 ? (
                <Typography variant="body2" color="text.secondary">
                  {t('common.noData')}
                </Typography>
              ) : null}
            </Stack>
          </Stack>
        ) : null}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={close}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} disabled={disabled} onClick={submit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function Header({ title, subtitle, onClose }: { title: string; subtitle?: string; onClose: () => void }) {
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6">{title}</Typography>
        {subtitle ? (
          <Typography variant="caption" color="text.secondary" noWrap>
            {subtitle}
          </Typography>
        ) : null}
      </Stack>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

function MappingRow({
  upstreamId,
  value,
  models,
  onChange,
  onRemove,
}: {
  upstreamId: string;
  value: string;
  models: GlobalModelResponse[];
  onChange: (globalId: string) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} alignItems={{ md: 'center' }}>
      <Typography variant="body2" sx={{ minWidth: 220, fontFamily: 'monospace' }}>
        {upstreamId}
      </Typography>
      <Autocomplete
        fullWidth
        size="small"
        options={models}
        value={models.find((model) => model.id === value) ?? null}
        getOptionLabel={(model) => model.display_name || model.name}
        isOptionEqualToValue={(option, selected) => option.id === selected.id}
        onChange={(_, selected) => onChange(selected?.id ?? '')}
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
        renderInput={(params) => <TextField {...params} required label={t('providers.globalModels')} error={!value} />}
      />
      <Tooltip title={t('providers.quickImportRemoveModel')}>
        <IconButton color="error" onClick={onRemove}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </Stack>
  );
}

function LoadingState() {
  return (
    <Stack alignItems="center" justifyContent="center" sx={{ py: 6 }}>
      <CircularProgress size={24} />
    </Stack>
  );
}

function associationMappings(response: ProviderQuickImportModelAssociationsResponse) {
  return Object.fromEntries(response.associations.map((item) => [item.upstream_model_id, item.global_model_id]));
}

function associationPayload(mappings: Record<string, string>) {
  return Object.entries(mappings).map(([upstream_model_id, global_model_id]) => ({
    upstream_model_id,
    global_model_id,
  }));
}

function visibleCandidates(
  response: ProviderQuickImportModelAssociationsResponse | null,
  mappings: Record<string, string>
) {
  return response?.candidates.filter((candidate) => !(candidate.upstream_model_id in mappings)) ?? [];
}

function addCandidate(
  candidate: ProviderQuickImportModelAssociationCandidate,
  setMappings: Dispatch<SetStateAction<Record<string, string>>>
) {
  setMappings((current) => ({
    ...current,
    [candidate.upstream_model_id]: candidate.suggested_global_model_id ?? '',
  }));
}

function removeMapping(
  upstreamId: string,
  setMappings: Dispatch<SetStateAction<Record<string, string>>>
) {
  setMappings((current) => {
    const next = { ...current };
    delete next[upstreamId];
    return next;
  });
}
