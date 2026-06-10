'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { ProviderQuickImportTokenPreview } from 'src/types/provider-quick-import';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { useTranslate } from 'src/locales/use-locales';

import { type QuickImportTokenDraft } from './provider-quick-import-utils';
import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

type Props = {
  token: ProviderQuickImportTokenPreview;
  draft?: QuickImportTokenDraft;
  mappings: Record<string, string>;
  rechargeMultiplier: number;
  setTokens: Dispatch<SetStateAction<Record<string, QuickImportTokenDraft>>>;
  onMapModels: (token: ProviderQuickImportTokenPreview) => void;
};

type DetailProps = {
  formats: string[];
  costMultiplier: string;
  importable: boolean;
  selected: boolean;
  modelMappingConfigured: boolean;
  modelMappingMissing: boolean;
  onMapModels: () => void;
  onUpdate: (patch: Partial<QuickImportTokenDraft>) => void;
};

export function ProviderQuickImportTokenRow(props: Props) {
  const tokenState = useTokenRowState(props);

  return (
    <>
      <TokenMainRow {...tokenState} />
      <TableRow>
        <TableCell sx={detailRowCellSx} />
        <TableCell colSpan={4} sx={detailRowCellSx}>
          <TokenDetailFields {...tokenState} />
        </TableCell>
      </TableRow>
    </>
  );
}

function useTokenRowState({ token, draft, mappings, rechargeMultiplier, setTokens, onMapModels }: Props) {
  const formats = draft?.endpointFormats ?? [];
  const name = draft?.name ?? token.name;
  const costMultiplier = draft?.costMultiplier ?? String(token.effective_cost_multiplier);
  const selected = !!draft?.selected;
  const modelValues = token.models
    .map((model) => mappings[model.upstream_model_id])
    .filter((value) => value !== undefined);
  const modelMappingConfigured = modelValues.length > 0 && modelValues.every(Boolean);
  const modelMappingMissing = selected && !modelMappingConfigured;
  const onUpdate = tokenDraftUpdater(token, setTokens);

  return {
    token,
    name,
    formats,
    selected,
    costMultiplier,
    rechargeMultiplier,
    importable: token.importable,
    modelMappingConfigured,
    modelMappingMissing,
    onUpdate,
    onMapModels: () => onMapModels(token),
  };
}

function tokenDraftUpdater(
  token: ProviderQuickImportTokenPreview,
  setTokens: Dispatch<SetStateAction<Record<string, QuickImportTokenDraft>>>
) {
  return (patch: Partial<QuickImportTokenDraft>) =>
    setTokens((current) => {
      const currentDraft = current[token.upstream_token_id];
      return {
        ...current,
        [token.upstream_token_id]: {
          selected: currentDraft?.selected ?? false,
          name: currentDraft?.name ?? token.name,
          endpointFormats: currentDraft?.endpointFormats ?? [],
          costMultiplier: currentDraft?.costMultiplier ?? String(token.effective_cost_multiplier),
          ...patch,
        },
      };
    });
}

function TokenMainRow({
  token,
  name,
  selected,
  rechargeMultiplier,
  onUpdate,
}: ReturnType<typeof useTokenRowState>) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell padding="checkbox" sx={mainRowCellSx}>
        <Checkbox checked={selected} disabled={!token.importable} onChange={(event) => onUpdate({ selected: event.target.checked })} />
      </TableCell>
      <TableCell sx={mainRowCellSx}>
        <TextField required fullWidth size="small" placeholder={t('fields.keyName')} value={name} disabled={!token.importable} error={selected && !name.trim()} onChange={(event) => onUpdate({ name: event.target.value })} />
        <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
          {token.masked_key}
        </Typography>
      </TableCell>
      <TableCell sx={mainRowCellSx}>
        <Chip size="small" color={token.importable ? 'success' : 'default'} variant="soft" label={token.importable ? t('common.enabled') : t('common.disabled')} />
      </TableCell>
      <TableCell sx={mainRowCellSx}>
        <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
          <Chip size="small" color="info" variant="soft" label={token.group ?? '-'} />
          <Chip size="small" color="warning" variant="soft" label={formatMultiplierLabel(token.group_ratio)} />
        </Stack>
      </TableCell>
      <TableCell sx={mainRowCellSx}>
        <Chip size="small" color="secondary" variant="soft" label={formatMultiplierLabel(rechargeMultiplier)} />
      </TableCell>
    </TableRow>
  );
}

function TokenDetailFields(props: DetailProps) {
  return (
    <Box sx={detailGridSx}>
      <CostMultiplierField {...props} />
      <ModelMappingField {...props} />
      <EndpointFormatsField {...props} />
    </Box>
  );
}

function CostMultiplierField({
  costMultiplier,
  importable,
  selected,
  onUpdate,
}: Pick<DetailProps, 'costMultiplier' | 'importable' | 'selected' | 'onUpdate'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.quickImportEffectiveCostMultiplier')}
      </Typography>
      <TextField
        required
        fullWidth
        size="small"
        type="number"
        placeholder={t('providers.quickImportEffectiveCostMultiplier')}
        value={costMultiplier}
        disabled={!importable}
        error={selected && Number(costMultiplier) <= 0}
        onChange={(event) => onUpdate({ costMultiplier: event.target.value })}
      />
    </Stack>
  );
}

function ModelMappingField({
  importable,
  modelMappingConfigured,
  modelMappingMissing,
  onMapModels,
}: Pick<DetailProps, 'importable' | 'modelMappingConfigured' | 'modelMappingMissing' | 'onMapModels'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.models')}
      </Typography>
      <Button
        size="small"
        variant="outlined"
        color={modelMappingMissing ? 'error' : modelMappingConfigured ? 'success' : 'warning'}
        disabled={!importable}
        sx={{ minHeight: 40 }}
        onClick={onMapModels}
      >
        {t(modelMappingConfigured ? 'providers.quickImportModelsConfigured' : 'providers.quickImportModelsUnconfigured')}
      </Button>
    </Stack>
  );
}

function EndpointFormatsField({ formats, selected, onUpdate }: Pick<DetailProps, 'formats' | 'selected' | 'onUpdate'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.endpoints')}
        <Box component="span" sx={{ color: 'error.main', ml: 0.25 }}>
          *
        </Box>
      </Typography>
      <Autocomplete
        multiple
        size="small"
        options={API_FORMAT_OPTIONS}
        value={formats}
        getOptionLabel={formatApiFormat}
        onChange={(_, values) => onUpdate({ endpointFormats: values })}
        renderInput={(params) => (
          <TextField {...params} required placeholder={t('providers.endpoints')} error={selected && formats.length === 0} />
        )}
      />
    </Stack>
  );
}

function formatMultiplierLabel(value: number) {
  return `${Number(value).toLocaleString(undefined, { maximumFractionDigits: 6 })}x`;
}

const mainRowCellSx = {
  borderBottom: 0,
  pb: 0.75,
};

const detailRowCellSx = {
  pt: 0,
  pb: 2,
};

const detailGridSx = {
  display: 'grid',
  gridTemplateColumns: { xs: '1fr', md: '190px 120px minmax(0, 1fr)' },
  gap: 1.5,
  alignItems: 'start',
};
