'use client';

import type { AdminT } from './shared';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { RoutingMetricWindow } from 'src/types/routing';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import FormControlLabel from '@mui/material/FormControlLabel';

import { RefreshButton } from './shared';
import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

export const ROUTING_WINDOWS: RoutingMetricWindow[] = ['1m', '5m', '15m', '1h', '24h', '7d'];

export function RoutingFilters(props: RoutingFilterProps) {
  return (
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, sm: 6 }}>
        <Autocomplete<BillingGroup, false, false, false>
          fullWidth
          size="small"
          loading={props.groupsLoading}
          options={props.groups}
          value={props.selectedGroup}
          inputValue={props.groupSearch}
          filterOptions={(items) => items}
          getOptionLabel={groupLabel}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          noOptionsText={props.t('common.noResults')}
          onInputChange={(_event, value) => props.onGroupSearchChange(value)}
          onChange={(_event, group) => props.onGroupChange(group)}
          renderInput={(params) => <TextField {...params} label={props.t('routing.filters.group')} />}
        />
      </Grid>

      <Grid size={{ xs: 12, sm: 6 }}>
        <Autocomplete<GlobalModelResponse, false, false, false>
          fullWidth
          size="small"
          disabled={!props.selectedGroup}
          loading={props.modelsLoading}
          options={props.models}
          value={props.selectedModel}
          inputValue={props.modelSearch}
          filterOptions={(items) => items}
          getOptionLabel={modelLabel}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          noOptionsText={props.modelSearch.trim() ? props.t('common.noResults') : props.t('routing.filters.selectGroupFirst')}
          onInputChange={(_event, value) => props.onModelSearchChange(value)}
          onChange={(_event, model) => props.onModelChange(model)}
          renderInput={(params) => <TextField {...params} label={props.t('routing.filters.model')} />}
        />
      </Grid>

      <Grid size={{ xs: 12, sm: 6 }}>
        <TextField
          fullWidth
          select
          size="small"
          label={props.t('routing.filters.apiFormat')}
          value={props.apiFormat}
          onChange={(event) => props.onApiFormatChange(event.target.value)}
        >
          {API_FORMAT_OPTIONS.map((format) => (
            <MenuItem key={format} value={format}>
              {formatApiFormat(format)}
            </MenuItem>
          ))}
        </TextField>
      </Grid>

      <Grid size={{ xs: 12, sm: 6 }}>
        <TextField
          fullWidth
          select
          size="small"
          label={props.t('routing.filters.window')}
          value={props.metricWindow}
          onChange={(event) => props.onWindowChange(event.target.value as RoutingMetricWindow)}
        >
          {ROUTING_WINDOWS.map((item) => (
            <MenuItem key={item} value={item}>
              {item}
            </MenuItem>
          ))}
        </TextField>
      </Grid>

      <SwitchFilter
        checked={props.isStream}
        label={props.t('routing.filters.stream')}
        onChange={props.onStreamChange}
      />
      <SwitchFilter
        checked={props.includeExcluded}
        label={props.t('routing.filters.includeExcluded')}
        onChange={props.onIncludeExcludedChange}
      />
    </Grid>
  );
}

export function RoutingHeaderActions(props: {
  autoRefresh: boolean;
  loading: boolean;
  t: AdminT;
  onRefresh: VoidFunction;
  onAutoRefreshChange: (value: boolean) => void;
}) {
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <FormControlLabel
        control={
          <Switch
            checked={props.autoRefresh}
            onChange={(event) => props.onAutoRefreshChange(event.target.checked)}
          />
        }
        label={props.t('routing.filters.autoRefresh')}
      />
      <RefreshButton loading={props.loading} onClick={props.onRefresh} />
    </Stack>
  );
}

function groupLabel(group: BillingGroup) {
  return `${group.name} · ${group.code}`;
}

function modelLabel(model: GlobalModelResponse) {
  return model.display_name && model.display_name !== model.name ? `${model.display_name} · ${model.name}` : model.name;
}

function SwitchFilter({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (value: boolean) => void;
}) {
  return (
    <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex', alignItems: 'center' }}>
      <FormControlLabel
        control={<Switch checked={checked} onChange={(event) => onChange(event.target.checked)} />}
        label={label}
      />
    </Grid>
  );
}

export type RoutingFilterProps = {
  t: AdminT;
  groups: BillingGroup[];
  models: GlobalModelResponse[];
  selectedGroup: BillingGroup | null;
  selectedModel: GlobalModelResponse | null;
  groupSearch: string;
  modelSearch: string;
  groupsLoading: boolean;
  modelsLoading: boolean;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  onGroupSearchChange: (value: string) => void;
  onModelSearchChange: (value: string) => void;
  onGroupChange: (value: BillingGroup | null) => void;
  onModelChange: (value: GlobalModelResponse | null) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
};
