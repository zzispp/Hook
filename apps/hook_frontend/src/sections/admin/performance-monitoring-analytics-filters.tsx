'use client';

import type { PerformanceMonitoringAnalyticsQuery } from 'src/types/performance-monitoring';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import CardHeader from '@mui/material/CardHeader';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

type FilterState = Pick<
  PerformanceMonitoringAnalyticsQuery,
  'provider_id' | 'model' | 'api_format' | 'is_stream' | 'needs_conversion'
> & {
  limit: number;
  slow_threshold_ms: number;
};

export const DEFAULT_ANALYTICS_FILTERS: FilterState = {
  provider_id: '',
  model: '',
  api_format: '',
  is_stream: undefined,
  needs_conversion: undefined,
  limit: 8,
  slow_threshold_ms: 10000,
};

export function toAnalyticsQueryFilters(filters: FilterState) {
  return {
    limit: filters.limit,
    slow_threshold_ms: filters.slow_threshold_ms,
    provider_id: textOrUndefined(filters.provider_id),
    model: textOrUndefined(filters.model),
    api_format: textOrUndefined(filters.api_format),
    is_stream: filters.is_stream,
    needs_conversion: filters.needs_conversion,
  };
}

export function AnalyticsFilters({
  filters,
  onChange,
}: {
  filters: FilterState;
  onChange: (filters: FilterState) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Card>
      <CardHeader
        title={t('performanceMonitoring.filters.title')}
        action={
          <Button size="small" color="inherit" onClick={() => onChange(DEFAULT_ANALYTICS_FILTERS)}>
            {t('common.clear')}
          </Button>
        }
      />
      <FilterFields filters={filters} onChange={onChange} />
    </Card>
  );
}

function FilterFields({ filters, onChange }: { filters: FilterState; onChange: (filters: FilterState) => void }) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={filterGridSx}>
      <TextFilter label={t('performanceMonitoring.filters.providerId')} value={filters.provider_id} onChange={(provider_id) => onChange({ ...filters, provider_id })} />
      <TextFilter label={t('performanceMonitoring.filters.model')} value={filters.model} onChange={(model) => onChange({ ...filters, model })} />
      <TextFilter label={t('performanceMonitoring.filters.apiFormat')} value={filters.api_format} onChange={(api_format) => onChange({ ...filters, api_format })} />
      <LimitFilter value={filters.limit} onChange={(limit) => onChange({ ...filters, limit })} />
      <SlowThresholdFilter value={filters.slow_threshold_ms} onChange={(slow_threshold_ms) => onChange({ ...filters, slow_threshold_ms })} />
      <SwitchFilters filters={filters} onChange={onChange} />
    </Box>
  );
}

function TextFilter({ label, value, onChange }: { label: string; value?: string; onChange: (value: string) => void }) {
  return <TextField size="small" label={label} value={value} onChange={(event) => onChange(event.target.value)} />;
}

function LimitFilter({ value, onChange }: { value: number; onChange: (value: number) => void }) {
  const { t } = useTranslate('admin');

  return (
    <TextField select size="small" label={t('performanceMonitoring.filters.limit')} value={value} onChange={(event) => onChange(Number(event.target.value))}>
      {[5, 8, 10, 20].map((item) => <MenuItem key={item} value={item}>{item}</MenuItem>)}
    </TextField>
  );
}

function SlowThresholdFilter({ value, onChange }: { value: number; onChange: (value: number) => void }) {
  const { t } = useTranslate('admin');

  return <TextField size="small" type="number" label={t('performanceMonitoring.filters.slowThresholdMs')} value={value} onChange={(event) => onChange(Number(event.target.value))} />;
}

function SwitchFilters({ filters, onChange }: { filters: FilterState; onChange: (filters: FilterState) => void }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.25}>
      <TriStateSwitch label={t('performanceMonitoring.filters.streamOnly')} value={filters.is_stream} onChange={(is_stream) => onChange({ ...filters, is_stream })} />
      <TriStateSwitch label={t('performanceMonitoring.filters.conversionOnly')} value={filters.needs_conversion} onChange={(needs_conversion) => onChange({ ...filters, needs_conversion })} />
    </Stack>
  );
}

function TriStateSwitch({
  label,
  value,
  onChange,
}: {
  label: string;
  value?: boolean;
  onChange: (value?: boolean) => void;
}) {
  return (
    <FormControlLabel
      label={label}
      control={
        <Switch
          size="small"
          checked={value === true}
          onChange={(event) => onChange(event.target.checked ? true : undefined)}
        />
      }
    />
  );
}

function textOrUndefined(value?: string) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

const filterGridSx = {
  gap: 1.5,
  p: 2.5,
  display: 'grid',
  gridTemplateColumns: {
    xs: '1fr',
    md: 'repeat(2, minmax(180px, 1fr))',
    xl: 'repeat(6, minmax(140px, 1fr))',
  },
};
