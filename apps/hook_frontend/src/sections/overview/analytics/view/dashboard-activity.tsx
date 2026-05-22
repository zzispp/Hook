import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { DashboardActivityFilters } from 'src/actions/dashboard';
import type {
  DashboardScope,
  DashboardActivityResponse,
  DashboardFilterOptionsResponse,
} from 'src/types/dashboard';

import { useEffect } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import Skeleton from '@mui/material/Skeleton';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

const ACTIVITY_CELL_SIZE = 12;
const ACTIVITY_CELL_GAP = 4;
const ACTIVITY_CELL_TRACK = `${ACTIVITY_CELL_SIZE}px`;
const ACTIVITY_CELL_GAP_VALUE = `${ACTIVITY_CELL_GAP}px`;
const ACTIVITY_GRID_ROWS = {
  xs: `repeat(22, ${ACTIVITY_CELL_TRACK})`,
  sm: `repeat(14, ${ACTIVITY_CELL_TRACK})`,
  md: `repeat(8, ${ACTIVITY_CELL_TRACK})`,
  lg: `repeat(12, ${ACTIVITY_CELL_TRACK})`,
  xl: `repeat(8, ${ACTIVITY_CELL_TRACK})`,
};

export function ActivityGridCard({
  t,
  loading,
  isAdmin,
  filters,
  activity,
  filterOptions,
  onFiltersChange,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  isAdmin: boolean;
  filters: DashboardActivityFilters;
  activity?: DashboardActivityResponse;
  filterOptions?: DashboardFilterOptionsResponse;
  onFiltersChange: (filters: DashboardActivityFilters) => void;
}) {
  const theme = useTheme();
  const max = Math.max(activity?.max_request_count ?? 0, 1);

  useEffect(() => {
    const normalized = normalizeActivityFilters(filters, filterOptions);
    if (normalized !== filters) onFiltersChange(normalized);
  }, [filterOptions, filters, onFiltersChange]);

  return (
    <Card>
      <CardHeader
        sx={{
          gap: 2,
          alignItems: { xs: 'stretch', md: 'center' },
          flexDirection: { xs: 'column', md: 'row' },
          '& .MuiCardHeader-content': { minWidth: 0 },
          '& .MuiCardHeader-action': {
            mr: 0,
            mt: 0,
            width: { xs: '100%', md: 'auto' },
            alignSelf: { xs: 'stretch', md: 'center' },
          },
        }}
        title={t('dashboard.stats.activity.title')}
        subheader={t('dashboard.stats.activity.subtitle')}
        action={
          isAdmin ? (
            <ActivityFilters t={t} filters={filters} options={filterOptions} onChange={onFiltersChange} />
          ) : null
        }
      />
      <Box sx={{ p: 3, minWidth: 0 }}>
        {loading ? <Skeleton variant="rectangular" height={126} /> : null}
        {!loading ? <ActivityCells theme={theme} days={activity?.days ?? []} max={max} /> : null}
        <ActivityLegend t={t} theme={theme} />
      </Box>
    </Card>
  );
}

function ActivityFilters({
  t,
  filters,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: DashboardActivityFilters;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardActivityFilters) => void;
}) {
  return (
    <Stack
      direction={{ xs: 'column', sm: 'row' }}
      spacing={1}
      sx={{
        width: { xs: '100%', sm: 'auto' },
        minWidth: { md: 420 },
        justifyContent: { sm: 'flex-end' },
      }}
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
        <UserSelect t={t} value={filters.user_id ?? ''} options={options} onChange={onChange} />
      ) : null}
      {filters.scope === 'token' ? (
        <TokenSelect t={t} value={filters.token_id ?? ''} options={options} onChange={onChange} />
      ) : null}
    </Stack>
  );
}

function UserSelect({
  t,
  value,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardActivityFilters) => void;
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

function TokenSelect({
  t,
  value,
  options,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  options?: DashboardFilterOptionsResponse;
  onChange: (filters: DashboardActivityFilters) => void;
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

function ActivityCells({
  theme,
  days,
  max,
}: {
  theme: Theme;
  days: DashboardActivityResponse['days'];
  max: number;
}) {
  return (
    <Box
      sx={{
        gap: ACTIVITY_CELL_GAP_VALUE,
        display: 'grid',
        gridAutoFlow: 'column',
        gridAutoColumns: ACTIVITY_CELL_TRACK,
        gridTemplateRows: ACTIVITY_GRID_ROWS,
      }}
    >
      {days.map((day) => (
        <Box
          key={day.date}
          title={`${day.date}: ${day.request_count}`}
          sx={{
            width: ACTIVITY_CELL_SIZE,
            height: ACTIVITY_CELL_SIZE,
            borderRadius: 0.5,
            bgcolor: activityColor(theme, day.request_count, max),
          }}
        />
      ))}
    </Box>
  );
}

function ActivityLegend({ t, theme }: { t: TFunction<'admin'>; theme: Theme }) {
  return (
    <Stack direction="row" spacing={1} alignItems="center" justifyContent="flex-end" sx={{ mt: 2 }}>
      <Typography variant="caption" sx={{ color: 'text.secondary' }}>
        {t('dashboard.stats.activity.less')}
      </Typography>
      {[0, 1, 2, 3, 4].map((level) => (
        <Box
          key={level}
          sx={{
            width: ACTIVITY_CELL_SIZE,
            height: ACTIVITY_CELL_SIZE,
            borderRadius: 0.5,
            bgcolor: activityColor(theme, level, 4),
          }}
        />
      ))}
      <Typography variant="caption" sx={{ color: 'text.secondary' }}>
        {t('dashboard.stats.activity.more')}
      </Typography>
    </Stack>
  );
}

function onScopeChange(
  scope: DashboardScope['scope'],
  options: DashboardFilterOptionsResponse | undefined,
  onChange: (filters: DashboardActivityFilters) => void
) {
  if (scope === 'global') onChange({ scope });
  if (scope === 'user') onChange({ scope, user_id: options?.users[0]?.id });
  if (scope === 'token') onChange({ scope, token_id: options?.tokens[0]?.id });
}

function normalizeActivityFilters(
  filters: DashboardActivityFilters,
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

function activityColor(theme: Theme, count: number, max: number) {
  if (count <= 0) return hexAlpha(theme.palette.grey[500], 0.16);
  const ratio = Math.min(count / max, 1);
  if (ratio < 0.25) return theme.palette.success.lighter;
  if (ratio < 0.5) return theme.palette.success.light;
  if (ratio < 0.75) return theme.palette.success.main;
  return theme.palette.success.dark;
}
