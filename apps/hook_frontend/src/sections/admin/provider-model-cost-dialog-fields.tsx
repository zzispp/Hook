'use client';

import type { Theme } from '@mui/material/styles';
import type { PaperProps } from '@mui/material/Paper';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderModelBinding, ProviderModelCostMode } from 'src/types/provider';

import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import ToggleButton from '@mui/material/ToggleButton';
import ListItemText from '@mui/material/ListItemText';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import {
  bindingLabel,
  type TokenCostDraft,
  tokenDraftFromGlobal,
} from './provider-model-cost-utils';

export type ModelCostDrafts = Record<string, TokenCostDraft>;

type Props = {
  apiKeys: ProviderApiKey[];
  mode: ProviderModelCostMode;
  models: GlobalModelResponse[];
  multiplier: string;
  options: ProviderModelBinding[];
  pricePerRequest: string;
  selected: ProviderModelBinding[];
  tokenDrafts: ModelCostDrafts;
  valueKeyId: string;
  onApplyMultiplier: () => void;
  onDraftChange: (id: string, patch: Partial<TokenCostDraft>) => void;
  onKeyChange: (value: string) => void;
  onModeChange: (value: ProviderModelCostMode) => void;
  onMultiplierChange: (value: string) => void;
  onPricePerRequestChange: (value: string) => void;
  onSelectionChange: (values: ProviderModelBinding[]) => void;
};

export function ProviderModelCostDialogFields(props: Props) {
  return (
    <>
      <KeySelect apiKeys={props.apiKeys} value={props.valueKeyId} onChange={props.onKeyChange} />
      <ModelSelect
        models={props.models}
        options={props.options}
        selected={props.selected}
        onChange={props.onSelectionChange}
      />
      <ModeSelect value={props.mode} onChange={props.onModeChange} />
      {props.mode === 'per_request' ? (
        <PerRequestField value={props.pricePerRequest} onChange={props.onPricePerRequestChange} />
      ) : (
        <TokenPriceEditor
          drafts={props.tokenDrafts}
          multiplier={props.multiplier}
          selected={props.selected}
          models={props.models}
          onApplyMultiplier={props.onApplyMultiplier}
          onDraftChange={props.onDraftChange}
          onMultiplierChange={props.onMultiplierChange}
        />
      )}
    </>
  );
}

function KeySelect({ apiKeys, value, onChange }: { apiKeys: ProviderApiKey[]; value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');
  return (
    <TextField select fullWidth label={t('providers.key')} value={value} onChange={(event) => onChange(event.target.value)}>
      {apiKeys.map((apiKey) => (
        <MenuItem key={apiKey.id} value={apiKey.id}>
          {apiKey.name}
        </MenuItem>
      ))}
    </TextField>
  );
}

function ModelSelect({
  models,
  options,
  selected,
  onChange,
}: {
  models: GlobalModelResponse[];
  options: ProviderModelBinding[];
  selected: ProviderModelBinding[];
  onChange: (values: ProviderModelBinding[]) => void;
}) {
  const { t } = useTranslate('admin');
  const allSelected = options.length > 0 && selected.length === options.length;

  return (
    <Autocomplete
      multiple
      disableCloseOnSelect
      options={options}
      value={selected}
      slots={{
        paper: (params) => (
          <ModelSelectPaper
            {...params}
            clearDisabled={selected.length === 0}
            selectAllDisabled={allSelected}
            onClear={() => onChange([])}
            onSelectAll={() => onChange(options)}
          />
        ),
      }}
      getOptionLabel={(option) => bindingLabel(option, models)}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      noOptionsText={t('providers.noBindableModels')}
      onChange={(_, values) => onChange(values)}
      renderOption={(params, option, state) => (
        <MenuItem {...params} key={option.id} value={option.id}>
          <Checkbox readOnly checked={state.selected} size="small" tabIndex={-1} sx={optionCheckboxSx} />
          <ListItemText primary={bindingLabel(option, models)} secondary={option.provider_model_name} />
        </MenuItem>
      )}
      renderInput={(params) => <TextField {...params} label={t('providers.model')} />}
    />
  );
}

function ModelSelectPaper({
  children,
  clearDisabled,
  selectAllDisabled,
  onClear,
  onSelectAll,
  ...paperProps
}: PaperProps & {
  clearDisabled: boolean;
  selectAllDisabled: boolean;
  onClear: () => void;
  onSelectAll: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Paper {...paperProps}>
      <Stack direction="row" spacing={1} sx={selectActionsSx} onMouseDown={(event) => event.preventDefault()}>
        <Button
          size="small"
          variant="text"
          disabled={selectAllDisabled}
          startIcon={<Iconify icon="solar:check-circle-bold" />}
          onClick={onSelectAll}
        >
          {t('common.selectAll')}
        </Button>
        <Button
          size="small"
          variant="text"
          color="inherit"
          disabled={clearDisabled}
          startIcon={<Iconify icon="solar:close-circle-bold" />}
          onClick={onClear}
        >
          {t('common.deselectAll')}
        </Button>
      </Stack>
      <Divider />
      {children}
    </Paper>
  );
}

function ModeSelect({ value, onChange }: { value: ProviderModelCostMode; onChange: (value: ProviderModelCostMode) => void }) {
  const { t } = useTranslate('admin');
  return (
    <ToggleButtonGroup exclusive fullWidth value={value} onChange={(_, next) => next && onChange(next)}>
      <ToggleButton value="per_token">{t('providers.perTokenCost')}</ToggleButton>
      <ToggleButton value="per_request">{t('providers.perRequestCost')}</ToggleButton>
    </ToggleButtonGroup>
  );
}

function PerRequestField({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');
  return (
    <TextField
      fullWidth
      type="number"
      label={t('providers.pricePerRequest')}
      value={value}
      inputProps={{ min: 0, step: 0.000001 }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function TokenPriceEditor({
  drafts,
  multiplier,
  selected,
  models,
  onApplyMultiplier,
  onDraftChange,
  onMultiplierChange,
}: {
  drafts: ModelCostDrafts;
  multiplier: string;
  selected: ProviderModelBinding[];
  models: GlobalModelResponse[];
  onApplyMultiplier: () => void;
  onDraftChange: (id: string, patch: Partial<TokenCostDraft>) => void;
  onMultiplierChange: (value: string) => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Stack spacing={1.5}>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5}>
        <TextField
          fullWidth
          type="number"
          label={t('providers.costMultiplier')}
          value={multiplier}
          inputProps={{ min: 0, step: 0.01 }}
          onChange={(event) => onMultiplierChange(event.target.value)}
        />
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:restart-bold" />}
          sx={applyMultiplierButtonSx}
          onClick={onApplyMultiplier}
        >
          {t('providers.applyGlobalPriceMultiplier')}
        </Button>
      </Stack>
      <Stack spacing={1}>
        {selected.map((binding) => (
          <TokenDraftRow
            key={binding.id}
            binding={binding}
            draft={drafts[binding.id] ?? tokenDraftFromGlobal(binding, models, 1)}
            label={bindingLabel(binding, models)}
            onChange={(patch) => onDraftChange(binding.id, patch)}
          />
        ))}
      </Stack>
    </Stack>
  );
}

function TokenDraftRow({ binding, draft, label, onChange }: { binding: ProviderModelBinding; draft: TokenCostDraft; label: string; onChange: (patch: Partial<TokenCostDraft>) => void }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={rowSx}>
      <TypographyLine label={label} value={binding.provider_model_name} />
      <Box sx={priceGridSx}>
        <PriceField label={t('requestRecords.inputPrice')} value={draft.input_price_per_million} onChange={(value) => onChange({ input_price_per_million: value })} />
        <PriceField label={t('requestRecords.outputPrice')} value={draft.output_price_per_million} onChange={(value) => onChange({ output_price_per_million: value })} />
        <PriceField label={t('requestRecords.cacheCreationPrice')} value={draft.cache_creation_price_per_million} onChange={(value) => onChange({ cache_creation_price_per_million: value })} />
        <PriceField label={t('requestRecords.cacheReadPrice')} value={draft.cache_read_price_per_million} onChange={(value) => onChange({ cache_read_price_per_million: value })} />
      </Box>
    </Box>
  );
}

function TypographyLine({ label, value }: { label: string; value: string }) {
  return (
    <>
      <Box component="span" sx={titleSx}>{label}</Box>
      <Box component="span" sx={subtitleSx}>{value}</Box>
    </>
  );
}

function PriceField({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <TextField size="small" type="number" label={label} value={value} inputProps={{ min: 0, step: 0.000001 }} onChange={(event) => onChange(event.target.value)} />;
}

const rowSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1, p: 1.5 };
const selectActionsSx = { px: 1, py: 0.75 };
const optionCheckboxSx = { p: 0.5, mr: 1 };
const titleSx = { display: 'block', typography: 'subtitle2', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' };
const subtitleSx = { display: 'block', typography: 'caption', fontFamily: 'monospace', color: 'text.secondary' };
const priceGridSx = { mt: 1.5, display: 'grid', gap: 1, gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, minmax(0, 1fr))' } };
const applyMultiplierButtonSx = {
  width: { xs: 1, sm: 'auto' },
  minWidth: { sm: 168 },
  flexShrink: 0,
  whiteSpace: 'nowrap',
};
