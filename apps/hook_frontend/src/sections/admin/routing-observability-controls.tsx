import type { AdminT } from './shared';
import type { SystemUser } from 'src/types/rbac';
import type { ApiToken } from 'src/types/api-token';
import type { GlobalModelResponse } from 'src/types/model';
import type { RoutingMetricWindow } from 'src/types/routing';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/components/iconify';

import { RefreshButton } from './shared';
import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

export const ROUTING_WINDOWS: RoutingMetricWindow[] = ['1m', '5m', '15m', '1h', '24h', '7d'];

export function RoutingFilters(props: FilterProps) {
  return (
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, sm: 6 }}>
        <Autocomplete<SystemUser, false, false, false>
          fullWidth
          size="small"
          loading={props.usersLoading}
          options={props.users}
          value={props.selectedUser}
          inputValue={props.userSearch}
          filterOptions={(items) => items}
          getOptionLabel={userLabel}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          noOptionsText={props.t('common.noResults')}
          onInputChange={(_event, value) => props.onUserSearchChange(value)}
          onChange={(_event, user) => props.onUserChange(user)}
          renderInput={(params) => <TextField {...params} label={props.t('routing.filters.user')} />}
        />
      </Grid>
      <Grid size={{ xs: 12, sm: 6 }}>
        <Autocomplete<ApiToken, false, false, false>
          fullWidth
          size="small"
          disabled={!props.selectedUser}
          loading={props.apiTokensLoading}
          options={props.apiTokens}
          value={props.selectedToken}
          inputValue={props.apiTokenSearch}
          filterOptions={(items) => items}
          getOptionLabel={apiTokenLabel}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          noOptionsText={props.t(props.selectedUser ? 'common.noResults' : 'routing.filters.selectUserFirst')}
          onInputChange={(_event, value) => props.onApiTokenSearchChange(value)}
          onChange={(_event, token) => props.onApiTokenChange(token)}
          renderInput={(params) => <TextField {...params} label={props.t('routing.filters.apiToken')} />}
        />
      </Grid>
      <Grid size={{ xs: 12, sm: 6 }}>
        <TextField
          fullWidth
          disabled
          size="small"
          label={props.t('routing.filters.group')}
          value={props.groupCode}
        />
      </Grid>
      <Grid size={{ xs: 12, sm: 6 }}>
        <Autocomplete<GlobalModelResponse, false, false, false>
          fullWidth
          size="small"
          disabled={!props.selectedToken || !props.groupCode}
          loading={props.modelsLoading}
          options={props.models}
          value={props.selectedModel}
          inputValue={props.modelSearch}
          filterOptions={(items) => items}
          getOptionLabel={modelLabel}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          noOptionsText={modelNoOptionsText(props)}
          onInputChange={(_event, value) => props.onModelSearchChange(value)}
          onChange={(_event, model) => props.onModelChange(model)}
          renderInput={(params) => <TextField {...params} label={props.t('routing.filters.model')} />}
        />
      </Grid>
      <ApiFormatFilter {...props} />
      <WindowFilter {...props} />
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
      <Grid size={{ xs: 12 }}>
        <Button
          fullWidth
          variant="contained"
          startIcon={<Iconify icon="eva:search-fill" />}
          disabled={!props.canSimulate}
          onClick={props.onSimulate}
          sx={{ height: 40 }}
        >
          {props.t('routing.actions.simulate')}
        </Button>
      </Grid>
      <Grid size={{ xs: 12, sm: 8 }}>
        <TextField
          fullWidth
          size="small"
          value={props.requestInput}
          label={props.t('routing.filters.requestId')}
          onChange={(event) => props.onRequestInputChange(event.target.value)}
        />
      </Grid>
      <Grid size={{ xs: 12, sm: 4 }}>
        <Button
          fullWidth
          variant="outlined"
          color="inherit"
          startIcon={<Iconify icon="eva:search-fill" />}
          disabled={!props.requestInput.trim()}
          onClick={props.onDecisionLookup}
          sx={{ height: 40 }}
        >
          {props.t('routing.actions.lookup')}
        </Button>
      </Grid>
    </Grid>
  );
}

function userLabel(user: SystemUser) {
  return `${user.username} · ${user.email}`;
}

function apiTokenLabel(token: ApiToken) {
  return `${token.name} · ${token.token_prefix} · ${token.group_code} · ${token.token_type}`;
}

function modelLabel(model: GlobalModelResponse) {
  return model.display_name && model.display_name !== model.name ? `${model.display_name} · ${model.name}` : model.name;
}

function modelNoOptionsText(props: FilterProps) {
  if (!props.selectedToken) return props.t('routing.filters.selectApiTokenFirst');
  if (props.modelSearch.trim()) return props.t('common.noResults');
  return props.t('routing.filters.noAccessibleModels');
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

function ApiFormatFilter(props: FilterProps) {
  return (
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
  );
}

function WindowFilter(props: FilterProps) {
  return (
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
  );
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
    <Grid size={{ xs: 6 }} sx={{ display: 'flex', alignItems: 'center' }}>
      <FormControlLabel
        control={<Switch checked={checked} onChange={(event) => onChange(event.target.checked)} />}
        label={label}
      />
    </Grid>
  );
}

type FilterProps = {
  t: AdminT;
  users: SystemUser[];
  apiTokens: ApiToken[];
  models: GlobalModelResponse[];
  selectedUser: SystemUser | null;
  selectedToken: ApiToken | null;
  selectedModel: GlobalModelResponse | null;
  userSearch: string;
  apiTokenSearch: string;
  modelSearch: string;
  usersLoading: boolean;
  apiTokensLoading: boolean;
  modelsLoading: boolean;
  groupCode: string;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  requestInput: string;
  canSimulate: boolean;
  onUserSearchChange: (value: string) => void;
  onApiTokenSearchChange: (value: string) => void;
  onModelSearchChange: (value: string) => void;
  onUserChange: (value: SystemUser | null) => void;
  onApiTokenChange: (value: ApiToken | null) => void;
  onModelChange: (value: GlobalModelResponse | null) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
  onRequestInputChange: (value: string) => void;
  onSimulate: VoidFunction;
  onDecisionLookup: VoidFunction;
};

export type RoutingFilterProps = FilterProps;
