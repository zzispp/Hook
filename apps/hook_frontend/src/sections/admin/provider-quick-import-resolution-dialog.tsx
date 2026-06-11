'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
  ProviderQuickImportResolutionResponse,
} from 'src/types/provider-quick-import';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import {
  relinkProviderQuickImportKey,
  getProviderQuickImportResolution,
  acceptProviderQuickImportCurrent,
} from 'src/actions/provider-quick-import';

import { toast } from 'src/components/snackbar';

import { ProviderQuickImportMappingDialog } from './provider-quick-import-mapping-table';
import {
  mappingInputs,
  defaultMappings,
  globalModelHasCost,
  selectedMappedUpstreamModels,
} from './provider-quick-import-utils';
import {
  QuickImportResolutionHeader,
  QuickImportResolutionStatusChips,
  QuickImportResolutionTokenSummary,
  QuickImportResolutionLoadingState,
} from './provider-quick-import-resolution-parts';

type Props = {
  open: boolean;
  provider: Provider | null;
  apiKey: ProviderApiKey | null;
  models: GlobalModelResponse[];
  onClose: () => void;
};

export function ProviderQuickImportResolutionDialog(props: Props) {
  const { open, provider, apiKey, models, onClose } = props;
  const { t } = useTranslate('admin');
  const [response, setResponse] = useState<ProviderQuickImportResolutionResponse | null>(null);
  const [selectedTokenId, setSelectedTokenId] = useState('');
  const [mappings, setMappings] = useState<Record<string, string>>({});
  const [mappingOpen, setMappingOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState<'accept' | 'relink' | null>(null);
  const preview = useMemo(() => responsePreview(response), [response]);
  const token = useMemo(() => tokenById(response, selectedTokenId), [response, selectedTokenId]);
  const candidates = useMemo(() => relinkCandidates(response), [response]);
  const selectedModelIds = useMemo(
    () => (token ? selectedMappedUpstreamModels([token], mappings) : []),
    [mappings, token]
  );
  const mappingInvalid = selectedModelIds.length === 0 || selectedModelIds.some((id) => !mappings[id]);
  const costMissing = selectedModelIds.some((id) => !globalModelHasCost(models, mappings[id]));

  const load = useCallback(async () => {
    if (!provider || !apiKey) return;
    setLoading(true);
    try {
      const next = await getProviderQuickImportResolution(provider.id, apiKey.id);
      const nextPreview = responsePreview(next);
      const first = relinkCandidates(next)[0];
      setResponse(next);
      setSelectedTokenId(first?.upstream_token_id ?? '');
      setMappings(first && nextPreview ? tokenMappings(nextPreview, first) : {});
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
    } finally {
      setLoading(false);
    }
  }, [apiKey, provider, t]);

  useEffect(() => {
    if (open) void load();
  }, [load, open]);

  const close = () => {
    setResponse(null);
    setSelectedTokenId('');
    setMappings({});
    setMappingOpen(false);
    setLoading(false);
    setSubmitting(null);
    onClose();
  };

  const acceptCurrent = async () => {
    if (!provider || !apiKey) return;
    setSubmitting('accept');
    try {
      await acceptProviderQuickImportCurrent(provider.id, apiKey.id);
      toast.success(t('messages.providerQuickImportResolved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(null);
    }
  };

  const relink = async () => {
    if (!provider || !apiKey || !token || mappingInvalid || costMissing) return;
    setSubmitting('relink');
    try {
      await relinkProviderQuickImportKey(provider.id, apiKey.id, {
        upstream_token_id: token.upstream_token_id,
        selected_model_ids: selectedModelIds,
        model_mappings: mappingInputs(selectedModelIds, mappings, models),
      });
      toast.success(t('messages.providerQuickImportResolved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(null);
    }
  };

  const selectToken = (nextId: string) => {
    const nextToken = tokenById(response, nextId);
    setSelectedTokenId(nextId);
    setMappings(nextToken && preview ? tokenMappings(preview, nextToken) : {});
  };

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={close}>
      <DialogTitle component="div">
        <QuickImportResolutionHeader title={t('providers.quickImportResolutionTitle')} subtitle={apiKey?.name} onClose={close} />
      </DialogTitle>
      <DialogContent>
        {loading ? <QuickImportResolutionLoadingState /> : null}
        {!loading && response && preview ? (
          <Stack spacing={2.25} sx={{ pt: 1 }}>
            <QuickImportResolutionStatusChips statuses={response.statuses} />
            <Alert severity="info">{t('providers.quickImportResolutionDescription')}</Alert>
            <Stack spacing={1}>
              <Typography variant="subtitle2">{t('providers.quickImportAcceptCurrent')}</Typography>
              <Typography variant="body2" color="text.secondary">
                {t('providers.quickImportAcceptCurrentDescription')}
              </Typography>
              <Button
                variant="contained"
                loading={submitting === 'accept'}
                onClick={acceptCurrent}
                sx={{ alignSelf: 'flex-start' }}
              >
                {t('providers.quickImportAcceptCurrent')}
              </Button>
            </Stack>
            <Divider />
            <Stack spacing={1.5}>
              <Typography variant="subtitle2">{t('providers.quickImportRelinkToken')}</Typography>
              {candidates.length === 0 ? (
                <Alert severity="warning">{t('providers.quickImportNoRelinkCandidates')}</Alert>
              ) : (
                <>
                  <TextField
                    select
                    fullWidth
                    size="small"
                    label={t('providers.quickImportRelinkToken')}
                    value={selectedTokenId}
                    onChange={(event) => selectToken(event.target.value)}
                  >
                    {candidates.map((item) => (
                      <MenuItem key={item.upstream_token_id} value={item.upstream_token_id}>
                        {item.name} · {item.masked_key}
                      </MenuItem>
                    ))}
                  </TextField>
                  <QuickImportResolutionTokenSummary token={token} />
                  <Button variant="outlined" disabled={!token} onClick={() => setMappingOpen(true)}>
                    {t('providers.quickImportModelMappings')}
                  </Button>
                  {costMissing ? <Alert severity="error">{t('providers.quickImportCostMissing')}</Alert> : null}
                  <Button
                    variant="contained"
                    loading={submitting === 'relink'}
                    disabled={!token || mappingInvalid || costMissing}
                    onClick={relink}
                    sx={{ alignSelf: 'flex-start' }}
                  >
                    {t('providers.quickImportRelinkSubmit')}
                  </Button>
                </>
              )}
            </Stack>
          </Stack>
        ) : null}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={close}>
          {t('common.close')}
        </Button>
      </DialogActions>
      {preview ? (
        <ProviderQuickImportMappingDialog
          open={mappingOpen}
          preview={preview}
          token={token}
          models={models}
          mappings={mappings}
          setMappings={setMappings}
          onClose={() => setMappingOpen(false)}
        />
      ) : null}
    </Dialog>
  );
}

function responsePreview(response: ProviderQuickImportResolutionResponse | null): ProviderQuickImportPreviewResponse | null {
  if (!response) return null;
  return {
    provider_id: response.provider_id,
    source_kind: response.source_kind,
    provider_name: response.key_name,
    recharge_multiplier: 1,
    tokens: response.tokens,
    model_mappings: response.model_mappings,
  };
}

function relinkCandidates(response: ProviderQuickImportResolutionResponse | null) {
  return response?.tokens.filter((token) => token.importable && token.group && token.upstream_token_id !== response.current_upstream_token_id) ?? [];
}

function tokenById(response: ProviderQuickImportResolutionResponse | null, id: string) {
  return response?.tokens.find((token) => token.upstream_token_id === id);
}

function tokenMappings(preview: ProviderQuickImportPreviewResponse, token: ProviderQuickImportTokenPreview) {
  const defaults = defaultMappings(preview);
  return Object.fromEntries(token.models.map((model) => [model.upstream_model_id, defaults[model.upstream_model_id] ?? '']));
}
