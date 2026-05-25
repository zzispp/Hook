import type { TFunction } from 'i18next';
import type { DashboardScopeFilters } from 'src/actions/dashboard';
import type { DashboardScope, DashboardFilterOptionsResponse } from 'src/types/dashboard';

import { useEffect } from 'react';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

export function DashboardScopeFilter({
  t,
  filters,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: DashboardScopeFilters;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardScopeFilters) => void;
}) {
  useEffect(() => {
    const normalized = normalizeScopeFilters(filters, options);
    if (normalized !== filters) onChange(normalized);
  }, [filters, onChange, options]);

  return (
    <Stack
      direction={{ xs: 'column', sm: 'row' }}
      spacing={1}
      sx={{ width: { xs: '100%', sm: 'auto' } }}
    >
      <TextField
        select
        size="small"
        label={t('common.type')}
        value={filters.scope}
        sx={{ width: { xs: '100%', sm: 128 } }}
        onChange={(event) =>
          onScopeChange(event.target.value as DashboardScope['scope'], options, onChange)
        }
      >
        <MenuItem value="global">{t('dashboard.stats.activity.global')}</MenuItem>
        <MenuItem value="user">{t('dashboard.stats.activity.user')}</MenuItem>
        <MenuItem value="token">{t('dashboard.stats.activity.token')}</MenuItem>
      </TextField>
      {filters.scope === 'user' ? (
        <DashboardUserSelect
          t={t}
          value={filters.user_id ?? ''}
          options={options}
          onChange={onChange}
        />
      ) : null}
      {filters.scope === 'token' ? (
        <DashboardTokenSelect
          t={t}
          value={filters.token_id ?? ''}
          options={options}
          onChange={onChange}
        />
      ) : null}
    </Stack>
  );
}

function DashboardUserSelect({
  t,
  value,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardScopeFilters) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={t('dashboard.stats.activity.user')}
      value={value}
      sx={{ width: { xs: '100%', sm: 260 } }}
      onChange={(event) => onChange({ scope: 'user', user_id: event.target.value })}
    >
      {options?.users.length ? null : (
        <MenuItem value="" disabled>
          {t('common.noData')}
        </MenuItem>
      )}
      {(options?.users ?? []).map((item) => (
        <MenuItem key={item.id} value={item.id}>
          {item.name}
        </MenuItem>
      ))}
    </TextField>
  );
}

function DashboardTokenSelect({
  t,
  value,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardScopeFilters) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={t('dashboard.stats.activity.token')}
      value={value}
      sx={{ width: { xs: '100%', sm: 260 } }}
      onChange={(event) => onChange({ scope: 'token', token_id: event.target.value })}
    >
      {options?.tokens.length ? null : (
        <MenuItem value="" disabled>
          {t('common.noData')}
        </MenuItem>
      )}
      {(options?.tokens ?? []).map((item) => (
        <MenuItem key={item.id} value={item.id}>
          {item.name}
        </MenuItem>
      ))}
    </TextField>
  );
}

function onScopeChange(
  scope: DashboardScope['scope'],
  options: DashboardFilterOptionsResponse | undefined,
  onChange: (filters: DashboardScopeFilters) => void
) {
  if (scope === 'global') onChange({ scope });
  if (scope === 'user') onChange({ scope, user_id: options?.users[0]?.id });
  if (scope === 'token') onChange({ scope, token_id: options?.tokens[0]?.id });
}

function normalizeScopeFilters(
  filters: DashboardScopeFilters,
  options: DashboardFilterOptionsResponse | undefined
) {
  if (filters.scope === 'user' && !filters.user_id && options?.users[0]?.id) {
    return { scope: 'user' as const, user_id: options.users[0].id };
  }
  if (filters.scope === 'token' && !filters.token_id && options?.tokens[0]?.id) {
    return { scope: 'token' as const, token_id: options.tokens[0].id };
  }
  return filters;
}
