import type { TFunction } from 'i18next';
import type { ChartOptions } from 'src/components/chart';
import type {
  DashboardUserStatsMetric,
  DashboardUserStatsTimeSeriesPoint,
} from 'src/types/dashboard';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import Skeleton from '@mui/material/Skeleton';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import ToggleButton from '@mui/material/ToggleButton';
import TableContainer from '@mui/material/TableContainer';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

import { Scrollbar } from 'src/components/scrollbar';
import { Chart, useChart } from 'src/components/chart';
import { TablePaginationCustom } from 'src/components/table';

import {
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardCostDetail,
} from './dashboard-format';

const METRICS: DashboardUserStatsMetric[] = ['requests', 'tokens', 'cost'];

type LeaderboardItem = {
  rank: number;
  name: string;
  requests: number;
  tokens: number;
  cost: number;
  value: number;
};

export function LeaderboardCard({
  t,
  locale,
  metric,
  loading,
  page,
  total,
  items = [],
  rowsPerPage,
  onMetricChange,
  onPageChange,
  onRowsPerPageChange,
}: {
  t: TFunction<'admin'>;
  locale: string;
  metric: DashboardUserStatsMetric;
  loading: boolean;
  page: number;
  total: number;
  items?: LeaderboardItem[];
  rowsPerPage: number;
  onMetricChange: (metric: DashboardUserStatsMetric) => void;
  onPageChange: (page: number) => void;
  onRowsPerPageChange: (pageSize: number) => void;
}) {
  return (
    <Card variant="outlined">
      <CardHeader
        sx={{ pb: 2 }}
        title={t('dashboard.stats.userStats.leaderboard')}
        action={
          <ToggleButtonGroup
            exclusive
            size="small"
            value={metric}
            onChange={(_, value) => value && onMetricChange(value)}
          >
            {METRICS.map((item) => (
              <ToggleButton key={item} value={item}>
                {t(`dashboard.stats.userStats.metrics.${item}`)}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
        }
      />
      {loading ? <Skeleton variant="rectangular" height={248} sx={{ m: 2, mt: 0 }} /> : null}
      {!loading ? <LeaderboardRows t={t} items={items} metric={metric} locale={locale} /> : null}
      <TablePaginationCustom
        page={page}
        count={total}
        rowsPerPage={rowsPerPage}
        rowsPerPageOptions={[5, 10, 25]}
        onPageChange={(_, nextPage) => onPageChange(nextPage)}
        onRowsPerPageChange={(event) => onRowsPerPageChange(Number(event.target.value))}
      />
    </Card>
  );
}

export function SummaryCard({
  t,
  locale,
  loading,
  data,
}: {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  data?: {
    total_requests: number;
    total_tokens: number;
    total_cost: number;
    error_rate: number;
  };
}) {
  const cards = summaryCards(t, locale, data);
  return (
    <Card variant="outlined">
      <CardHeader title={t('dashboard.stats.userStats.summary.title')} sx={{ pb: 2 }} />
      <Grid container spacing={2} sx={{ p: 2, pt: 0 }}>
        {cards.map((card) => (
          <Grid key={card.label} size={{ xs: 12, sm: 6 }}>
            <Card
              variant="outlined"
              sx={{
                p: 2,
                width: 1,
                display: 'flex',
                boxShadow: 'none',
                flexDirection: 'column',
              }}
            >
              <Typography variant="caption" color="text.secondary">
                {card.label}
              </Typography>
              {loading ? <Skeleton width="70%" /> : <Typography variant="h6">{card.value}</Typography>}
            </Card>
          </Grid>
        ))}
      </Grid>
    </Card>
  );
}

export function UserTrendCard({
  t,
  title,
  loading,
  data,
}: {
  t: TFunction<'admin'>;
  title: string;
  loading: boolean;
  data?: DashboardUserStatsTimeSeriesPoint[];
}) {
  const options = useUserTrendOptions(data);
  return (
    <Card variant="outlined">
      <CardHeader title={title} />
      {loading ? <Skeleton variant="rectangular" height={280} sx={{ m: 2 }} /> : null}
      {!loading ? (
        <Chart
          type="line"
          series={[{ name: t('dashboard.stats.userStats.totalCost'), data: costSeries(data) }]}
          options={options}
          sx={{ height: 280, p: 2 }}
        />
      ) : null}
    </Card>
  );
}

export function UserCompareTrendCard({
  t,
  loading,
  enabled,
  data,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  enabled: boolean;
  data?: DashboardUserStatsTimeSeriesPoint[];
}) {
  if (!enabled) return null;
  return (
    <UserTrendCard
      t={t}
      title={t('dashboard.stats.userStats.compareTrend')}
      loading={loading}
      data={data}
    />
  );
}

function LeaderboardRows({
  t,
  items,
  metric,
  locale,
}: {
  t: TFunction<'admin'>;
  items: LeaderboardItem[];
  metric: DashboardUserStatsMetric;
  locale: string;
}) {
  if (items.length === 0) {
    return (
      <Box
        sx={{
          mx: 2,
          mt: 0,
          mb: 2,
          height: 248,
          display: 'grid',
          color: 'text.secondary',
          placeItems: 'center',
          bgcolor: 'background.neutral',
          borderRadius: 1,
        }}
      >
        <Typography variant="body2">{t('dashboard.stats.empty')}</Typography>
      </Box>
    );
  }

  return (
    <TableContainer component={Scrollbar} sx={{ maxHeight: 248 }}>
      <Table size="small" stickyHeader>
        <TableBody>
          {items.map((item) => (
            <TableRow key={`${item.rank}-${item.name}`}>
              <TableCell width={56}>#{item.rank}</TableCell>
              <TableCell>
                <Typography variant="subtitle2" noWrap>
                  {item.name}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {metricValue(metric, item, locale)}
                </Typography>
              </TableCell>
              <TableCell align="right">{formatInteger(item.requests, locale)}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function summaryCards(
  t: TFunction<'admin'>,
  locale: string,
  data?: {
    total_requests: number;
    total_tokens: number;
    total_cost: number;
    error_rate: number;
  }
) {
  return [
    {
      label: t('dashboard.stats.userStats.summary.totalRequests'),
      value: formatInteger(data?.total_requests, locale),
    },
    {
      label: t('dashboard.stats.userStats.summary.totalTokens'),
      value: formatDashboardTokens(data?.total_tokens),
    },
    {
      label: t('dashboard.stats.userStats.summary.totalCost'),
      value: formatDashboardCost(data?.total_cost),
    },
    {
      label: t('dashboard.stats.userStats.summary.errorRate'),
      value: `${(data?.error_rate ?? 0).toFixed(1)}%`,
    },
  ];
}

function useUserTrendOptions(data?: DashboardUserStatsTimeSeriesPoint[]) {
  const theme = useTheme();
  return useChart({
    colors: [hexAlpha(theme.palette.info.main, 0.88)],
    xaxis: { categories: data?.map((point) => point.date) ?? [], labels: { rotate: -35 } },
    legend: { show: false },
    tooltip: { y: { formatter: (value: number) => formatDashboardCostDetail(value) } },
  } satisfies ChartOptions);
}

function costSeries(data?: DashboardUserStatsTimeSeriesPoint[]) {
  return data?.map((point) => point.total_cost) ?? [];
}

function metricValue(
  metric: DashboardUserStatsMetric,
  item: { requests: number; tokens: number; cost: number },
  locale: string
) {
  if (metric === 'cost') return formatDashboardCost(item.cost);
  if (metric === 'tokens') return formatDashboardTokens(item.tokens);
  return formatInteger(item.requests, locale);
}
