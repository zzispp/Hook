'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type {
  Provider,
  ProviderApiKey,
  ProviderKeyModelMappingsUpdate,
  ProviderKeyModelMappingsForKeyResponse,
} from 'src/types/provider';

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
  updateProviderKeyModelMappings,
  getProviderKeyModelMappingsForKey,
} from 'src/actions/provider-quick-import';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { ReasoningEffortField } from './provider-model-mapping-fields';
import {
  addManualMapping,
  visibleCandidates,
  type MappingDraft,
  associationPayload,
  removeMappingDraft,
  addCandidateMapping,
  associationMappings,
} from './provider-quick-import-model-associations-utils';

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
  const [response, setResponse] = useState<ProviderKeyModelMappingsForKeyResponse | null>(null);
  const [mappings, setMappings] = useState<Record<string, MappingDraft>>({});
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const loadingKeyRef = useRef<string | null>(null);
  const candidates = useMemo(() => visibleCandidates(response, mappings), [mappings, response]);
  const disabled =
    loading ||
    submitting ||
    Object.keys(mappings).length === 0 ||
    Object.values(mappings).some((draft) => !draft.global_model_id || !draft.upstream_model_name.trim());

  const load = useCallback(async () => {
    if (!provider || !apiKey) return;
    const loadKey = `${provider.id}:${apiKey.id}`;
    if (loadingKeyRef.current === loadKey) return;
    loadingKeyRef.current = loadKey;
    setLoading(true);
    try {
      const next = await getProviderKeyModelMappingsForKey(provider.id, apiKey.id);
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
      const payload: ProviderKeyModelMappingsUpdate = {
        model_mappings: associationPayload(mappings),
      };
      await updateProviderKeyModelMappings(provider.id, apiKey.id, payload);
      toast.success(t('messages.providerModelMappingSaved'));
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
        <Header title={t('providers.modelMappings')} subtitle={apiKey?.name} onClose={close} />
      </DialogTitle>
      <DialogContent>
        {loading ? <LoadingState /> : null}
        {!loading && response ? (
          <Stack spacing={2} sx={{ pt: 1 }}>
            <Typography variant="subtitle2">{t('providers.quickImportAssociatedModels')}</Typography>
            <Stack direction="row" justifyContent="flex-end">
              <Button
                size="small"
                variant="outlined"
                startIcon={<Iconify icon="mingcute:add-line" />}
                onClick={() => setMappings((current) => addManualMapping(current))}
              >
                {t('common.add')}
              </Button>
            </Stack>
            {Object.keys(mappings).map((providerModelId) => (
              <MappingRow
                key={providerModelId}
                draft={mappings[providerModelId]}
                models={models}
                onChange={(patch) =>
                  setMappings((current) => ({
                    ...current,
                    [providerModelId]: { ...current[providerModelId], ...patch },
                  }))
                }
                onRemove={() =>
                  setMappings((current) => removeMappingDraft(providerModelId, current))
                }
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
                  key={candidate.upstream_model_name}
                  size="small"
                  variant="outlined"
                  startIcon={<Iconify icon="mingcute:add-line" />}
                  onClick={() =>
                    setMappings((current) => addCandidateMapping(candidate, current))
                  }
                >
                  {candidate.upstream_model_name}
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
  draft,
  models,
  onChange,
  onRemove,
}: {
  draft: MappingDraft;
  models: GlobalModelResponse[];
  onChange: (patch: Partial<MappingDraft>) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslate('admin');
  const selectedModel = models.find((model) => model.id === draft.global_model_id) ?? null;

  return (
    <Stack spacing={1.5} sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1.5, p: 1.5 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} alignItems={{ md: 'center' }}>
        <Autocomplete
          fullWidth
          size="small"
          options={models}
          value={selectedModel}
          getOptionLabel={(model) => model.display_name || model.name}
          isOptionEqualToValue={(option, selected) => option.id === selected.id}
          onChange={(_, selected) => onChange({ global_model_id: selected?.id ?? '' })}
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
            <TextField {...params} required label={t('providers.globalModels')} error={!draft.global_model_id} />
          )}
        />
        <Tooltip title={t('providers.quickImportRemoveModel')}>
          <IconButton color="error" onClick={onRemove}>
            <Iconify icon="mingcute:close-line" />
          </IconButton>
        </Tooltip>
      </Stack>
      <TextField
        fullWidth
        size="small"
        required
        label={t('providers.upstreamModels')}
        value={draft.upstream_model_name}
        onChange={(event) => onChange({ upstream_model_name: event.target.value })}
      />
      <ReasoningEffortField value={draft.reasoning_effort ?? ''} onChange={(value) => onChange({ reasoning_effort: value || null })} />
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
