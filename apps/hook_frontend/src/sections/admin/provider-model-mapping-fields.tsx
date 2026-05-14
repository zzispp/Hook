'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderModelBinding, ProviderModelReasoningEffort } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Radio from '@mui/material/Radio';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { sortedBindingOptions } from './provider-model-mapping-utils';

type ClientModelFieldProps = {
  disabled: boolean;
  items: ProviderModelBinding[];
  models: GlobalModelResponse[];
  value: string;
  onChange: (value: string) => void;
};

export function ClientModelField({ disabled, items, models, value, onChange }: ClientModelFieldProps) {
  const { t } = useTranslate('admin');
  const placeholder = t('providers.selectClientModel');
  const options = sortedBindingOptions(items, models);

  return (
    <TextField
      select
      fullWidth
      label={t('providers.clientModel')}
      disabled={disabled}
      value={value}
      InputLabelProps={{ shrink: true }}
      SelectProps={{
        displayEmpty: true,
        renderValue: (selected) => {
          if (typeof selected !== 'string' || !selected) {
            return (
              <Typography component="span" sx={{ color: 'text.disabled' }}>
                {placeholder}
              </Typography>
            );
          }
          return options.find((option) => option.id === selected)?.label ?? placeholder;
        },
      }}
      onChange={(event) => onChange(event.target.value)}
    >
      <MenuItem value="">{placeholder}</MenuItem>
      {options.map((option) => (
        <MenuItem key={option.id} value={option.id}>
          {option.label}
        </MenuItem>
      ))}
    </TextField>
  );
}

export function ReasoningEffortField({
  value,
  onChange,
}: {
  value: ProviderModelReasoningEffort | '';
  onChange: (value: ProviderModelReasoningEffort | '') => void;
}) {
  const { t } = useTranslate('admin');
  const placeholder = t('providers.noReasoningEffort');

  return (
    <TextField
      select
      fullWidth
      label={t('providers.reasoningEffort')}
      helperText={t('providers.reasoningEffortHelper')}
      value={value}
      InputLabelProps={{ shrink: true }}
      SelectProps={{
        displayEmpty: true,
        renderValue: (selected) => {
          if (typeof selected !== 'string' || !selected) {
            return (
              <Typography component="span" sx={{ color: 'text.disabled' }}>
                {placeholder}
              </Typography>
            );
          }
          return selected;
        },
      }}
      onChange={(event) => onChange(event.target.value as ProviderModelReasoningEffort | '')}
    >
      <MenuItem value="">{placeholder}</MenuItem>
      {(['minimal', 'low', 'medium', 'high'] as ProviderModelReasoningEffort[]).map((option) => (
        <MenuItem key={option} value={option}>
          {option}
        </MenuItem>
      ))}
    </TextField>
  );
}

type ProviderModelsFieldProps = {
  canAddCustom: boolean;
  customNames: string[];
  loading: boolean;
  query: string;
  reasoningEffort: ProviderModelReasoningEffort | '';
  selectedName: string;
  upstreamModels: string[];
  onAddCustom: () => void;
  onFetch: () => void;
  onQueryChange: (value: string) => void;
  onToggleName: (name: string) => void;
};

export function ProviderModelsField(props: ProviderModelsFieldProps) {
  const { t } = useTranslate('admin');

  return (
    <Box>
      <Stack direction="row" spacing={1} alignItems="center">
        <TextField
          fullWidth
          value={props.query}
          placeholder={t('providers.searchOrAddProviderModel')}
          onChange={(event) => props.onQueryChange(event.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Iconify icon="eva:search-fill" width={18} />
              </InputAdornment>
            ),
          }}
        />
        <Chip size="small" label={t('providers.selectedModelCount', { count: props.selectedName ? 1 : 0 })} />
        <IconButton
          size="small"
          title={t('providers.fetchUpstreamModels')}
          aria-label={t('providers.fetchUpstreamModels')}
          disabled={props.loading}
          sx={fetchButtonSx}
          onClick={props.onFetch}
        >
          {props.loading ? <CircularProgress size={18} thickness={5} /> : <Iconify icon="custom:flash-outline" width={18} />}
        </IconButton>
      </Stack>
      <Box sx={listShellSx}>
        {props.canAddCustom ? <AddCustomRow name={props.query.trim()} onClick={props.onAddCustom} /> : null}
        <MappingOptionGroup names={props.customNames} selectedName={props.selectedName} title={t('providers.customModels')} onToggleName={props.onToggleName} />
        <MappingOptionGroup names={props.upstreamModels} selectedName={props.selectedName} title={t('providers.upstreamModels')} onToggleName={props.onToggleName} />
        {!props.loading && props.customNames.length === 0 && props.upstreamModels.length === 0 ? (
          <Typography variant="body2" color="text.secondary" sx={emptySx}>
            {t('providers.noProviderModelsForMapping')}
          </Typography>
        ) : null}
      </Box>
      <MappingPreview name={props.selectedName} reasoningEffort={props.reasoningEffort} />
    </Box>
  );
}

function AddCustomRow({ name, onClick }: { name: string; onClick: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <Button fullWidth color="inherit" variant="outlined" startIcon={<Iconify icon="mingcute:add-line" />} sx={customButtonSx} onClick={onClick}>
      {t('providers.addCustomProviderModel', { name })}
    </Button>
  );
}

function MappingOptionGroup({
  names,
  selectedName,
  title,
  onToggleName,
}: {
  names: string[];
  selectedName: string;
  title: string;
  onToggleName: (name: string) => void;
}) {
  if (names.length === 0) return null;

  return (
    <Box>
      <Typography variant="overline" sx={sectionTitleSx}>
        {title}
      </Typography>
      <Stack spacing={0.5}>
        {names.map((name) => (
          <Button key={name} color="inherit" sx={optionButtonSx} onClick={() => onToggleName(name)}>
            <Radio size="small" checked={selectedName === name} />
            <Typography variant="body2" sx={{ fontFamily: 'monospace', textTransform: 'none' }}>
              {name}
            </Typography>
          </Button>
        ))}
      </Stack>
    </Box>
  );
}

function MappingPreview({ name, reasoningEffort }: { name: string; reasoningEffort: string }) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={previewSx}>
      <Typography variant="overline" sx={sectionTitleSx}>
        {t('providers.mappingPreview')}
      </Typography>
      {name ? (
        <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{ mt: 1 }}>
          <Chip size="small" label={name} sx={{ fontFamily: 'monospace' }} />
          {reasoningEffort ? <Chip size="small" label={`${t('providers.reasoningEffort')}: ${reasoningEffort}`} /> : null}
        </Stack>
      ) : (
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          {t('providers.emptyMappingPreview')}
        </Typography>
      )}
    </Box>
  );
}

const listShellSx = {
  mt: 1.5,
  p: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
  borderRadius: 1.5,
  minHeight: 240,
  maxHeight: 320,
  overflowY: 'auto',
};
const previewSx = {
  mt: 1.5,
  p: 1.5,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
  borderRadius: 1.5,
};
const customButtonSx = { mb: 1, justifyContent: 'flex-start', textTransform: 'none', borderStyle: 'dashed' };
const sectionTitleSx = { px: 1, color: 'text.secondary' };
const optionButtonSx = { justifyContent: 'flex-start', px: 0.5, py: 0.5, textTransform: 'none' };
const emptySx = { p: 3, textAlign: 'center' };
const fetchButtonSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
