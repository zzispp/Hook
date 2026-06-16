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
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/components/iconify';

import { RefreshButton } from './shared';
import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

export const ROUTING_WINDOWS: RoutingMetricWindow[] = ['1m', '5m', '15m', '1h', '24h', '7d'];

export function RoutingFilters(props: FilterProps) {
  return (
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, sm: 6 }}>
        <TextField
          fullWidth
          select
          size="small"
          label={props.t('routing.filters.user')}
          value={props.userId}
          onChange={(event) => props.onUserChange(event.target.value)}
        >
          <MenuItem disabled value="">
            {props.t(props.users.length === 0 ? 'common.noData' : 'routing.filters.selectUserFirst')}
          </MenuItem>
          {props.users.map((user) => (
            <MenuItem key={user.id} value={user.id}>
              {user.username} · {user.email}
            </MenuItem>
          ))}
        </TextField>
      </Grid>
      <Grid size={{ xs: 12, sm: 6 }}>
        <TextField
          fullWidth
          select
          size="small"
          disabled={!props.userId}
          label={props.t('routing.filters.apiToken')}
          value={props.apiTokenId}
          onChange={(event) => props.onApiTokenChange(event.target.value)}
        >
          <MenuItem disabled value="">
            {props.t(props.userId && props.apiTokens.length === 0 ? 'common.noData' : 'routing.filters.selectApiTokenFirst')}
          </MenuItem>
          {props.apiTokens.map((token) => (
            <MenuItem key={token.id} value={token.id}>
              {token.name} · {token.token_prefix} · {token.group_code} · {token.token_type}
            </MenuItem>
          ))}
        </TextField>
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
        <TextField
          fullWidth
          select
          size="small"
          disabled={!props.apiTokenId || props.models.length === 0}
          label={props.t('routing.filters.model')}
          value={props.modelName}
          onChange={(event) => props.onModelChange(event.target.value)}
        >
          <MenuItem disabled value="">
            {props.t(props.apiTokenId && props.models.length === 0 ? 'routing.filters.noAccessibleModels' : 'routing.filters.selectApiTokenFirst')}
          </MenuItem>
          {props.models.map((model) => (
            <MenuItem key={model.id} value={model.name}>
              {model.name}
            </MenuItem>
          ))}
        </TextField>
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
  userId: string;
  apiTokenId: string;
  groupCode: string;
  modelName: string;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  requestInput: string;
  canSimulate: boolean;
  onUserChange: (value: string) => void;
  onApiTokenChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
  onRequestInputChange: (value: string) => void;
  onSimulate: VoidFunction;
  onDecisionLookup: VoidFunction;
};
