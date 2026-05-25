import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { DashboardActivityResponse } from 'src/types/dashboard';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import Skeleton from '@mui/material/Skeleton';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

import {
  formatPlainInteger,
  formatDashboardTokens,
  formatDashboardCostDetail,
} from './dashboard-format';

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
  activity,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  activity?: DashboardActivityResponse;
}) {
  const theme = useTheme();
  const max = Math.max(activity?.max_request_count ?? 0, 1);

  return (
    <Card>
      <CardHeader
        sx={{ '& .MuiCardHeader-content': { minWidth: 0 } }}
        title={t('dashboard.stats.activity.title')}
        subheader={t('dashboard.stats.activity.subtitle')}
      />
      <Box sx={{ p: 3, minWidth: 0 }}>
        {loading ? <Skeleton variant="rectangular" height={126} /> : null}
        {!loading ? <ActivityCells t={t} theme={theme} days={activity?.days ?? []} max={max} /> : null}
        <ActivityLegend t={t} theme={theme} />
      </Box>
    </Card>
  );
}

function ActivityCells({
  t,
  theme,
  days,
  max,
}: {
  t: TFunction<'admin'>;
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
        <Tooltip key={day.date} arrow title={<ActivityTooltip t={t} day={day} />}>
          <Box
            sx={{
              width: ACTIVITY_CELL_SIZE,
              height: ACTIVITY_CELL_SIZE,
              borderRadius: 0.5,
              bgcolor: activityColor(theme, day.request_count, max),
            }}
          />
        </Tooltip>
      ))}
    </Box>
  );
}

function ActivityTooltip({
  t,
  day,
}: {
  t: TFunction<'admin'>;
  day: DashboardActivityResponse['days'][number];
}) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="caption" sx={{ color: 'common.white' }}>
        {day.date}
      </Typography>
      <Typography variant="caption" sx={{ color: 'common.white' }}>
        {t('dashboard.stats.activity.requestCount', {
          count: formatPlainInteger(day.request_count),
        })}{' '}
        · {formatDashboardTokens(day.total_tokens)}
      </Typography>
      <Typography variant="caption" sx={{ color: 'common.white' }}>
        {t('dashboard.stats.activity.cost')}{formatDashboardCostDetail(day.total_cost)} ·{' '}
        {t('dashboard.stats.activity.baseCost')}{formatDashboardCostDetail(day.base_cost)}
      </Typography>
    </Stack>
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

function activityColor(theme: Theme, count: number, max: number) {
  if (count <= 0) return hexAlpha(theme.palette.grey[500], 0.16);
  const ratio = Math.min(count / max, 1);
  if (ratio < 0.25) return theme.palette.success.lighter;
  if (ratio < 0.5) return theme.palette.success.light;
  if (ratio < 0.75) return theme.palette.success.main;
  return theme.palette.success.dark;
}
