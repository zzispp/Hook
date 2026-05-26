'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderModelCost, ProviderModelBinding } from 'src/types/provider';

import { useMemo, useState } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { formatMoneyCompact } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';
import { deleteProviderModelCost } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';
import { ProviderModelCostDialog } from './provider-model-cost-dialog';
import {
  bindingLabel,
  globalDefaultMode,
  effectiveTokenDraft,
  effectiveRequestPrice,
} from './provider-model-cost-utils';

type Props = {
  providerId: string;
  apiKeys: ProviderApiKey[];
  bindings: ProviderModelBinding[];
  costs: ProviderModelCost[];
  loading: boolean;
  models: GlobalModelResponse[];
};

export function ProviderModelCostsSection({ providerId, apiKeys, bindings, costs, loading, models }: Props) {
  const { t } = useTranslate('admin');
  const [dialogOpen, setDialogOpen] = useState(false);
  const rows = useMemo(() => costRows(apiKeys, bindings, costs, models), [apiKeys, bindings, costs, models]);

  return (
    <>
      <Box sx={panelSx}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
          <Typography variant="subtitle2">{t('providers.modelCosts')}</Typography>
          <Button
            color="inherit"
            size="small"
            variant="outlined"
            disabled={apiKeys.length === 0 || bindings.length === 0}
            startIcon={<Iconify icon="mingcute:add-line" width={14} />}
            onClick={() => setDialogOpen(true)}
          >
            {t('actions.addProviderModelCost')}
          </Button>
        </Stack>
        <Box sx={listSx}>
          {rows.map((row) => (
            <CostRow key={`${row.key.id}:${row.binding.id}`} providerId={providerId} row={row} />
          ))}
          <EmptyList loading={loading} length={rows.length} />
        </Box>
      </Box>
      <ProviderModelCostDialog
        open={dialogOpen}
        providerId={providerId}
        apiKeys={apiKeys}
        bindings={bindings}
        models={models}
        onClose={() => setDialogOpen(false)}
      />
    </>
  );
}

function CostRow({ providerId, row }: { providerId: string; row: CostRowItem }) {
  const { t } = useTranslate('admin');
  const [deleting, setDeleting] = useState(false);
  const configured = Boolean(row.cost);

  const deleteCost = async () => {
    if (!row.cost || deleting) return;
    setDeleting(true);
    try {
      await deleteProviderModelCost(providerId, row.key.id, row.binding.id);
      toast.success(t('messages.providerModelCostDeleted'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Box sx={rowSx}>
      <Stack direction="row" justifyContent="space-between" spacing={1.5} alignItems="flex-start">
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap>{row.modelLabel}</Typography>
          <Typography variant="caption" sx={monoSx}>{row.binding.provider_model_name}</Typography>
        </Box>
        <Stack direction="row" alignItems="center" spacing={0.75}>
          <Chip size="small" color={configured ? 'success' : 'default'} label={sourceLabel(row.source, t)} />
          {configured ? (
            <Tooltip title={t('common.delete')}>
              <IconButton size="small" disabled={deleting} onClick={deleteCost}>
                <Iconify icon="solar:trash-bin-trash-bold" width={16} />
              </IconButton>
            </Tooltip>
          ) : null}
        </Stack>
      </Stack>
      <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap" sx={metaSx}>
        <Chip size="small" variant="outlined" label={`${t('providers.key')}: ${row.key.name}`} />
        <Chip size="small" variant="outlined" label={modeLabel(row.mode, t)} />
        <Typography variant="caption" sx={priceSx}>{priceSummary(row, t)}</Typography>
      </Stack>
    </Box>
  );
}

type CostRowItem = ReturnType<typeof costRows>[number];

function costRows(
  apiKeys: ProviderApiKey[],
  bindings: ProviderModelBinding[],
  costs: ProviderModelCost[],
  models: GlobalModelResponse[]
) {
  const costMap = new Map(costs.map((cost) => [`${cost.key_id}:${cost.provider_model_id}`, cost]));
  return apiKeys.flatMap((key) =>
    bindings.map((binding) => {
      const cost = costMap.get(`${key.id}:${binding.id}`);
      return {
        key,
        binding,
        cost,
        modelLabel: bindingLabel(binding, models),
        mode: cost?.cost_mode ?? globalDefaultMode(binding, models),
        source: cost ? 'configured' : 'global_default',
        requestPrice: effectiveRequestPrice(binding, models, cost),
        tokenDraft: effectiveTokenDraft(binding, models, cost),
      };
    })
  );
}

function priceSummary(row: CostRowItem, t: (key: string) => string) {
  if (row.mode === 'per_request') {
    return `${t('providers.pricePerRequest')}: ${formatPrice(row.requestPrice)}`;
  }
  const draft = row.tokenDraft;
  return [
    `${t('requestRecords.inputPrice')}: ${formatPrice(Number(draft.input_price_per_million || 0))}`,
    `${t('requestRecords.outputPrice')}: ${formatPrice(Number(draft.output_price_per_million || 0))}`,
    `${t('requestRecords.cacheCreationPrice')}: ${formatPrice(Number(draft.cache_creation_price_per_million || 0))}`,
    `${t('requestRecords.cacheReadPrice')}: ${formatPrice(Number(draft.cache_read_price_per_million || 0))}`,
  ].join(' / ');
}

function modeLabel(mode: string, t: (key: string) => string) {
  return mode === 'per_request' ? t('providers.perRequestCost') : t('providers.perTokenCost');
}

function sourceLabel(source: string, t: (key: string) => string) {
  return source === 'configured' ? t('providers.configuredCost') : t('providers.globalDefaultCost');
}

function formatPrice(value: number | null | undefined) {
  return formatMoneyCompact(value);
}

const panelSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 2, overflow: 'hidden' };
const headerSx = { p: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const listSx = { '& > * + *': { borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` } };
const rowSx = { px: 2, py: 1.5, transition: (theme: Theme) => theme.transitions.create('background-color'), '&:hover': { bgcolor: 'action.hover' } };
const monoSx = { fontFamily: 'monospace', color: 'text.secondary' };
const metaSx = { mt: 1, alignItems: 'center', color: 'text.secondary' };
const priceSx = { fontFamily: 'monospace', color: 'text.secondary' };
