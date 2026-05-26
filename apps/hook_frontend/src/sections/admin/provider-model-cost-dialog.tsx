'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderModelBinding, ProviderModelCostMode } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogActions from '@mui/material/DialogActions';

import { useTranslate } from 'src/locales/use-locales';
import { upsertProviderModelCosts } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import {
  type ModelCostDrafts,
  ProviderModelCostDialogFields,
} from './provider-model-cost-dialog-fields';
import {
  numberText,
  bindingLabel,
  type TokenCostDraft,
  parseRequiredNumber,
  tokenDraftFromGlobal,
} from './provider-model-cost-utils';

type Props = {
  open: boolean;
  providerId: string;
  apiKeys: ProviderApiKey[];
  bindings: ProviderModelBinding[];
  models: GlobalModelResponse[];
  onClose: () => void;
};

export function ProviderModelCostDialog(props: Props) {
  const { t } = useTranslate('admin');
  const [keyId, setKeyId] = useState('');
  const [mode, setMode] = useState<ProviderModelCostMode>('per_token');
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [pricePerRequest, setPricePerRequest] = useState('');
  const [multiplier, setMultiplier] = useState('1');
  const [tokenDrafts, setTokenDrafts] = useState<ModelCostDrafts>({});
  const [saving, setSaving] = useState(false);
  const options = useMemo(() => sortedBindings(props.bindings, props.models), [props.bindings, props.models]);
  const selected = options.filter((item) => selectedIds.includes(item.id));

  useEffect(() => {
    if (!props.open) return;
    setKeyId(props.apiKeys[0]?.id ?? '');
    setMode('per_token');
    setSelectedIds([]);
    setPricePerRequest('');
    setMultiplier('1');
    setTokenDrafts({});
    setSaving(false);
  }, [props.apiKeys, props.open]);

  const applyMultiplier = () => {
    const factor = parseRequiredNumber(multiplier);
    setTokenDrafts((current) => {
      const next = { ...current };
      for (const binding of selected) {
        next[binding.id] = tokenDraftFromGlobal(binding, props.models, factor);
      }
      return next;
    });
  };

  const submit = async () => {
    if (!keyId || selected.length === 0 || saving) return;
    setSaving(true);
    try {
      const costs = selected.map((binding) => costPayload(binding, mode, pricePerRequest, tokenDrafts[binding.id]));
      await upsertProviderModelCosts(props.providerId, keyId, { costs });
      toast.success(t('messages.providerModelCostSaved'));
      props.onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="md" open={props.open} onClose={props.onClose} PaperProps={{ sx: paperSx }}>
      <DialogHeader onClose={props.onClose} />
      <Box sx={contentSx}>
        <ProviderModelCostDialogFields
          apiKeys={props.apiKeys}
          mode={mode}
          models={props.models}
          multiplier={multiplier}
          options={options}
          pricePerRequest={pricePerRequest}
          selected={selected}
          tokenDrafts={tokenDrafts}
          valueKeyId={keyId}
          onApplyMultiplier={applyMultiplier}
          onDraftChange={(id, patch) => setTokenDrafts((current) => ({ ...current, [id]: { ...current[id], ...patch } }))}
          onKeyChange={setKeyId}
          onModeChange={setMode}
          onMultiplierChange={setMultiplier}
          onPricePerRequestChange={setPricePerRequest}
          onSelectionChange={(values) => {
            setSelectedIds(values.map((value) => value.id));
            setTokenDrafts((current) => ensureDrafts(values, current, props.models));
          }}
        />
      </Box>
      <DialogActions sx={footerSx}>
        <Button variant="outlined" onClick={props.onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={saving} disabled={!keyId || selected.length === 0} onClick={submit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DialogHeader({ onClose }: { onClose: () => void }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" alignItems="center" spacing={1.5} sx={headerSx}>
      <Box sx={headerIconSx}><Iconify icon="solar:bill-list-bold" width={22} /></Box>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6">{t('dialogs.createProviderModelCost')}</Typography>
        <Typography variant="caption" color="text.secondary">{t('providers.modelCostDialogDescription')}</Typography>
      </Box>
      <IconButton onClick={onClose}><Iconify icon="mingcute:close-line" /></IconButton>
    </Stack>
  );
}

function sortedBindings(bindings: ProviderModelBinding[], models: GlobalModelResponse[]) {
  return [...bindings].sort((left, right) => bindingLabel(left, models).localeCompare(bindingLabel(right, models)));
}

function ensureDrafts(bindings: ProviderModelBinding[], current: ModelCostDrafts, models: GlobalModelResponse[]) {
  const next: ModelCostDrafts = {};
  for (const binding of bindings) {
    next[binding.id] = current[binding.id] ?? tokenDraftFromGlobal(binding, models, 1);
  }
  return next;
}

function costPayload(binding: ProviderModelBinding, mode: ProviderModelCostMode, pricePerRequest: string, draft: TokenCostDraft | undefined) {
  if (mode === 'per_request') {
    return { provider_model_id: binding.id, cost_mode: mode, price_per_request: parseRequiredNumber(pricePerRequest) };
  }
  const tokenDraft = draft ?? emptyDraft();
  return {
    provider_model_id: binding.id,
    cost_mode: mode,
    input_price_per_million: parseRequiredNumber(tokenDraft.input_price_per_million),
    output_price_per_million: parseRequiredNumber(tokenDraft.output_price_per_million),
    cache_creation_price_per_million: parseRequiredNumber(tokenDraft.cache_creation_price_per_million),
    cache_read_price_per_million: parseRequiredNumber(tokenDraft.cache_read_price_per_million),
  };
}

function emptyDraft(): TokenCostDraft {
  return {
    input_price_per_million: numberText(0),
    output_price_per_million: numberText(0),
    cache_creation_price_per_million: numberText(0),
    cache_read_price_per_million: numberText(0),
  };
}

const paperSx = (theme: Theme) => ({ borderRadius: 2, border: `1px solid ${theme.vars.palette.divider}` });
const headerSx = { px: 2.5, py: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const headerIconSx = { width: 40, height: 40, borderRadius: 1.5, display: 'grid', placeItems: 'center', bgcolor: 'primary.lighter', color: 'primary.main' };
const contentSx = { px: 2.5, py: 2, display: 'grid', gap: 2 };
const footerSx = { px: 2.5, py: 2, borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, bgcolor: 'background.neutral' };
