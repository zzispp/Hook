'use client';

import type { TFunction } from 'i18next';
import type { ChartOptions } from 'src/components/chart';
import type { DashboardProviderAggregationItem } from 'src/types/dashboard';
import type { DashboardCostAnalysisFilters, DashboardApiKeyLeaderboardFilters } from 'src/actions/dashboard';

import { useState } from 'react';

import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import TableRow from '@mui/material/TableRow';
import Skeleton from '@mui/material/Skeleton';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import ToggleButton from '@mui/material/ToggleButton';
import TableContainer from '@mui/material/TableContainer';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import {
  useDashboardCostSavings,
  useDashboardCostForecast,
  useDashboardApiKeyLeaderboard,
  useDashboardProviderAggregation,
} from 'src/actions/dashboard';

import { Scrollbar } from 'src/components/scrollbar';
import { Chart, useChart } from 'src/components/chart';
import { TablePaginationCustom } from 'src/components/table';

import { AdminBreadcrumbs } from './shared';
import { DashboardDateRangePicker } from './dashboard-date-range-picker';
import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardPercent,
  formatDashboardCostDetail,
} from '../overview/analytics/view/dashboard-format';

export function CostAnalysisView() {
  const { t, currentLang } = useTranslate('admin');
  const locale = currentLang.numberFormat.code;
  const [filters, setFilters] = useState<DashboardCostAnalysisFilters>(defaultFilters());
  const [leaderboardFilters, setLeaderboardFilters] = useState<DashboardApiKeyLeaderboardFilters>({
    ...defaultFilters(),
    metric: 'cost',
    order: 'desc',
    page: 0,
    pageSize: 10,
  });
  const savings = useDashboardCostSavings(filters);
  const forecast = useDashboardCostForecast(filters);
  const leaderboard = useDashboardApiKeyLeaderboard(leaderboardFilters);
  const providers = useDashboardProviderAggregation(filters);
  const error = savings.error ?? forecast.error ?? leaderboard.error ?? providers.error;

  function updateFilters(next: DashboardCostAnalysisFilters) {
    setFilters(next);
    setLeaderboardFilters((value) => ({ ...value, ...next, page: 0 }));
  }

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.costAnalysis}
        action={<DashboardDateRangePicker t={t} filters={filters} onChange={updateFilters} />}
      />
      {error ? <Alert severity="error" sx={{ mb: 3 }}>{errorMessage(error)}</Alert> : null}
      <Stack spacing={3}>
        <CostSavingsCards t={t} data={savings.data} loading={savings.isLoading} />
        <CostForecastCard t={t} data={forecast.data} loading={forecast.isLoading} />
        <ApiKeyLeaderboardCard
          t={t}
          locale={locale}
          loading={leaderboard.isLoading}
          data={leaderboard.data}
          filters={leaderboardFilters}
          onChange={setLeaderboardFilters}
        />
        <ProviderAggregationTable t={t} locale={locale} data={providers.data} loading={providers.isLoading} />
      </Stack>
    </DashboardContent>
  );
}

function CostSavingsCards({ t, data, loading }: { t: TFunction<'admin'>; data?: { cache_savings: number; cache_read_cost: number; cache_read_tokens: number; estimated_full_cost: number; cache_creation_cost: number }; loading: boolean }) {
  const cards = [
    { label: t('dashboard.stats.costAnalysis.cacheSavings'), value: formatDashboardCost(data?.cache_savings), detail: t('dashboard.stats.costAnalysis.cacheReadCost', { value: formatDashboardCost(data?.cache_read_cost) }) },
    { label: t('dashboard.stats.costAnalysis.cacheReadTokens'), value: formatDashboardTokens(data?.cache_read_tokens), detail: t('dashboard.stats.costAnalysis.estimatedFullCost', { value: formatDashboardCost(data?.estimated_full_cost) }) },
    { label: t('dashboard.stats.costAnalysis.cacheCreationCost'), value: formatDashboardCost(data?.cache_creation_cost), detail: t('dashboard.stats.costAnalysis.currentRange') },
  ];
  return (
    <Grid container spacing={3}>
      {cards.map((card) => (
        <Grid key={card.label} size={{ xs: 12, md: 4 }}>
          <Card sx={{ p: 2.5, height: 1 }}>
            <Typography variant="caption" sx={{ color: 'text.secondary' }}>{card.label}</Typography>
            {loading ? <Skeleton width="70%" height={36} /> : <Typography variant="h5" sx={{ mt: 0.75 }}>{card.value}</Typography>}
            <Typography variant="caption" sx={{ mt: 0.75, display: 'block', color: 'text.secondary' }}>{card.detail}</Typography>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function CostForecastCard({ t, data, loading }: { t: TFunction<'admin'>; data?: { history: { date: string; total_cost: number }[]; forecast: { date: string; total_cost: number }[] }; loading: boolean }) {
  const options = useForecastOptions(t, data);
  const history = data?.history ?? [];
  const forecast = data?.forecast ?? [];
  return (
    <Card>
      <CardHeader title={t('dashboard.stats.costAnalysis.forecast')} />
      {loading ? <Skeleton variant="rectangular" height={280} sx={{ m: 2 }} /> : null}
      {!loading ? (
        <Chart
          type="line"
          series={[
            { name: t('dashboard.stats.costAnalysis.actualCost'), data: [...history.map((item) => item.total_cost), ...forecast.map(() => null)] },
            { name: t('dashboard.stats.costAnalysis.forecastCost'), data: [...history.map(() => null), ...forecast.map((item) => item.total_cost)] },
          ]}
          options={options}
          sx={{ height: 280, p: 2 }}
        />
      ) : null}
    </Card>
  );
}

function ApiKeyLeaderboardCard({ t, locale, data, loading, filters, onChange }: { t: TFunction<'admin'>; locale: string; loading: boolean; data?: { items: { rank: number; id: string; name: string; requests: number; tokens: number; cost: number }[]; total: number }; filters: DashboardApiKeyLeaderboardFilters; onChange: (filters: DashboardApiKeyLeaderboardFilters) => void }) {
  return (
    <Card>
      <CardHeader
        title={t('dashboard.stats.costAnalysis.apiKeyLeaderboard')}
        action={
          <ToggleButtonGroup exclusive size="small" value={filters.metric} onChange={(_, metric) => metric && onChange({ ...filters, metric, page: 0 })}>
            {(['requests', 'tokens', 'cost'] as const).map((metric) => <ToggleButton key={metric} value={metric}>{t(`dashboard.stats.userStats.metrics.${metric}`)}</ToggleButton>)}
          </ToggleButtonGroup>
        }
      />
      {loading ? <Skeleton variant="rectangular" height={260} sx={{ m: 2 }} /> : <LeaderboardRows t={t} locale={locale} items={data?.items ?? []} />}
      <TablePaginationCustom
        page={filters.page}
        count={data?.total ?? 0}
        rowsPerPage={filters.pageSize}
        rowsPerPageOptions={[10, 20, 50, 100]}
        onPageChange={(_, page) => onChange({ ...filters, page })}
        onRowsPerPageChange={(event) => onChange({ ...filters, page: 0, pageSize: Number(event.target.value) })}
      />
    </Card>
  );
}

function LeaderboardRows({ t, locale, items }: { t: TFunction<'admin'>; locale: string; items: { rank: number; id: string; name: string; requests: number; tokens: number; cost: number }[] }) {
  if (!items.length) return <EmptyState t={t} />;
  return (
    <TableContainer component={Scrollbar} sx={{ maxHeight: 320 }}>
      <Table size="small" stickyHeader>
        <TableHead><TableRow><TableCell>{t('dashboard.stats.costAnalysis.rank')}</TableCell><TableCell>{t('dashboard.stats.costAnalysis.name')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.requests')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.tokens')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.cost')}</TableCell></TableRow></TableHead>
        <TableBody>{items.map((item) => <TableRow key={item.id}><TableCell>#{item.rank}</TableCell><TableCell>{item.name}</TableCell><TableCell align="right">{formatInteger(item.requests, locale)}</TableCell><TableCell align="right">{formatDashboardTokens(item.tokens)}</TableCell><TableCell align="right">{formatDashboardCost(item.cost)}</TableCell></TableRow>)}</TableBody>
      </Table>
    </TableContainer>
  );
}

function ProviderAggregationTable({ t, locale, data = [], loading }: { t: TFunction<'admin'>; locale: string; data?: DashboardProviderAggregationItem[]; loading: boolean }) {
  return (
    <Card>
      <CardHeader title={t('dashboard.stats.costAnalysis.providerAnalysis')} />
      {loading ? <Skeleton variant="rectangular" height={260} sx={{ m: 2 }} /> : null}
      {!loading && !data.length ? <EmptyState t={t} /> : null}
      {!loading && data.length ? (
        <TableContainer component={Scrollbar} sx={{ maxHeight: 360 }}>
          <Table size="small" stickyHeader sx={{ minWidth: 1280 }}>
            <TableHead><TableRow><TableCell>{t('dashboard.stats.costAnalysis.provider')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.requests')}</TableCell><TableCell align="right">{t('dashboard.stats.costAnalysis.inputOutputCache')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.cost')}</TableCell><TableCell align="right">{t('dashboard.stats.costAnalysis.cacheHitRate')}</TableCell><TableCell align="right">{t('dashboard.stats.costAnalysis.successRate')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.responseHeaders')}</TableCell><TableCell align="right">{t('requestRecords.firstChar')}</TableCell><TableCell align="right">{t('requestRecords.firstToken')}</TableCell><TableCell align="right">{t('dashboard.stats.columns.avgLatency')}</TableCell></TableRow></TableHead>
            <TableBody>{data.map((provider) => <TableRow key={provider.provider_key}><TableCell>{provider.provider}</TableCell><TableCell align="right">{formatInteger(provider.request_count, locale)}</TableCell><TableCell align="right"><Stack spacing={0.25}><Typography variant="caption">{formatDashboardTokens(provider.effective_input_tokens)} / {formatDashboardTokens(provider.output_tokens)}</Typography><Typography variant="caption" color="text.secondary">{formatDashboardTokens(provider.cache_read_tokens + provider.cache_creation_tokens)}</Typography></Stack></TableCell><TableCell align="right"><Stack spacing={0.25}><Typography variant="caption" color="primary.main">{formatDashboardCost(provider.total_cost)}</Typography><Typography variant="caption" color="text.secondary">{formatDashboardCost(provider.actual_cost)}</Typography></Stack></TableCell><TableCell align="right">{formatDashboardPercent(provider.cache_hit_rate / 100)}</TableCell><TableCell align="right">{formatDashboardPercent(provider.success_rate / 100)}</TableCell><TableCell align="right">{formatMs(provider.avg_response_headers_ms)}</TableCell><TableCell align="right">{formatMs(provider.avg_first_byte_ms)}</TableCell><TableCell align="right">{formatMs(provider.avg_first_output_ms)}</TableCell><TableCell align="right">{formatMs(provider.avg_response_time_ms)}</TableCell></TableRow>)}</TableBody>
          </Table>
        </TableContainer>
      ) : null}
    </Card>
  );
}

function EmptyState({ t }: { t: TFunction<'admin'> }) {
  return <Typography sx={{ px: 3, py: 4, color: 'text.secondary' }}>{t('dashboard.stats.empty')}</Typography>;
}

function useForecastOptions(t: TFunction<'admin'>, data?: { history: { date: string }[]; forecast: { date: string }[] }) {
  return useChart({
    colors: ['#3b82f6', '#eab308'],
    stroke: { width: 2, dashArray: [0, 6] },
    xaxis: { categories: [...(data?.history ?? []).map((item) => item.date), ...(data?.forecast ?? []).map((item) => item.date)], labels: { rotate: -35 } },
    tooltip: { y: { formatter: (value: number) => formatDashboardCostDetail(value) } },
    noData: { text: t('dashboard.stats.empty') },
  } satisfies ChartOptions);
}

function defaultFilters(): DashboardCostAnalysisFilters {
  return { preset: 'last30days' };
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Request failed';
}
