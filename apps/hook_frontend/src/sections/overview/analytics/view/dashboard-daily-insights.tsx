import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { ChartOptions } from 'src/components/chart';
import type {
  DashboardPreset,
  DashboardDailyStats,
  DashboardDailyBreakdownItem,
} from 'src/types/dashboard';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Skeleton from '@mui/material/Skeleton';
import { useTheme } from '@mui/material/styles';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';

import { Chart, useChart, ChartLegends } from 'src/components/chart';

import { dashboardPeriodLabel } from './dashboard-period';
import { DailyTotalsGrid } from './dashboard-daily-totals';
import {
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardCostDetail,
} from './dashboard-format';

type DailyInsightProps = {
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
  loading: boolean;
  preset: DashboardPreset;
  data?: DashboardDailyStats;
};

type DailyChartProps = Omit<DailyInsightProps, 'isAdmin'>;

const CHART_HEIGHT = 320;
const DAILY_GRID_SX = {
  gap: 3,
  display: 'grid',
  gridTemplateColumns: { xs: '1fr', lg: 'repeat(2, minmax(0, 1fr))' },
} as const;

export function DailyInsights({ t, locale, isAdmin, loading, preset, data }: DailyInsightProps) {
  const period = dashboardPeriodLabel(t, preset);

  return (
    <Stack spacing={3}>
      <DailyTotalsGrid t={t} locale={locale} loading={loading} preset={preset} data={data} />
      <Box sx={DAILY_GRID_SX}>
        {!isAdmin ? <DailyUsageTrendCard t={t} locale={locale} loading={loading} data={data} /> : null}
        <DailyModelCostCard t={t} locale={locale} loading={loading} period={period} data={data} />
        {isAdmin ? <ProviderCostCard t={t} loading={loading} data={data} /> : null}
      </Box>
    </Stack>
  );
}

function DailyUsageTrendCard({
  t,
  locale,
  loading,
  data,
}: {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  data?: DashboardDailyStats;
}) {
  const days = data?.days ?? [];
  const options = useDailyUsageOptions(t, locale, days.map((day) => day.date));

  return (
    <DailyChartCard title={t('dashboard.stats.daily.usageTrend')} loading={loading} empty={!days.length}>
      <Chart
        type="line"
        series={[
          { name: t('dashboard.stats.columns.requests'), data: days.map((day) => day.request_count) },
          { name: t('dashboard.stats.columns.tokens'), data: days.map((day) => day.total_tokens) },
        ]}
        options={options}
        sx={{ height: CHART_HEIGHT }}
      />
    </DailyChartCard>
  );
}

function DailyModelCostCard({
  t,
  locale,
  loading,
  period,
  data,
}: {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  period: string;
  data?: DashboardDailyStats;
}) {
  const days = data?.days ?? [];
  const models = modelNames(days.flatMap((day) => day.model_breakdown));
  const options = useModelCostOptions(locale, days.map((day) => day.date));
  const series = models.map((name) => ({
    name: compactModelName(name),
    data: days.map((day) => modelCost(day.model_breakdown, name)),
  }));

  return (
    <DailyChartCard title={t('dashboard.stats.period.modelCost', { period })} loading={loading} empty={!series.length}>
      <Chart type="bar" series={series} options={options} sx={{ height: CHART_HEIGHT }} />
    </DailyChartCard>
  );
}

function ProviderCostCard({
  t,
  loading,
  data,
}: Pick<DailyChartProps, 't' | 'loading' | 'data'>) {
  const theme = useTheme();
  const items = data?.provider_summary ?? [];
  const labels = items.map((item) => item.name);
  const colors = paletteColors(theme).slice(0, items.length);
  const options = useProviderCostOptions(labels, colors);

  return (
    <DailyChartCard title={t('dashboard.stats.daily.providerCost')} loading={loading} empty={!items.length}>
      <Chart type="donut" series={items.map((item) => item.total_cost)} options={options} sx={{ height: CHART_HEIGHT }} />
      <ChartLegends
        labels={labels}
        colors={colors}
        values={items.map((item) => formatDashboardCost(item.total_cost))}
        sx={{ px: 3, pb: 3, justifyContent: 'center' }}
      />
    </DailyChartCard>
  );
}

function DailyChartCard({
  title,
  loading,
  empty,
  children,
}: {
  title: string;
  loading: boolean;
  empty: boolean;
  children: React.ReactNode;
}) {
  return (
    <Card>
      <CardHeader title={title} />
      {loading ? <Skeleton variant="rectangular" height={CHART_HEIGHT} sx={{ m: 3 }} /> : null}
      {!loading && empty ? <DailyEmpty /> : null}
      {!loading && !empty ? children : null}
    </Card>
  );
}

function DailyEmpty() {
  return (
    <Typography sx={{ px: 3, py: 8, color: 'text.secondary', textAlign: 'center' }}>
      -
    </Typography>
  );
}

function useDailyUsageOptions(t: TFunction<'admin'>, locale: string, dates: string[]) {
  return useChart({
    xaxis: { categories: dates.map((date) => formatDateLabel(date, locale)) },
    legend: { show: true, position: 'top', horizontalAlign: 'left' },
    tooltip: { y: { formatter: (value: number, context) => usageTooltip(t, value, context) } },
  } satisfies ChartOptions);
}

function useModelCostOptions(locale: string, dates: string[]) {
  return useChart({
    chart: { stacked: true },
    xaxis: { categories: dates.map((date) => formatDateLabel(date, locale)) },
    legend: { show: true, position: 'top', horizontalAlign: 'left' },
    tooltip: { y: { formatter: formatDashboardCostDetail } },
    yaxis: { labels: { formatter: (value: number) => formatDashboardCost(value) } },
    plotOptions: { bar: { columnWidth: '48%' } },
  } satisfies ChartOptions);
}

function useProviderCostOptions(labels: string[], colors: string[]) {
  return useChart({
    labels,
    colors,
    stroke: { width: 0 },
    legend: { show: false },
    tooltip: { y: { formatter: formatDashboardCostDetail } },
  } satisfies ChartOptions);
}

function usageTooltip(t: TFunction<'admin'>, value: number, context?: { seriesIndex?: number }) {
  if (context?.seriesIndex === 1) return formatDashboardTokens(value);
  return t('dashboard.stats.activity.requestCount', { count: value });
}

function modelNames(items: DashboardDailyBreakdownItem[]) {
  const totals = new Map<string, number>();
  items.forEach((item) => totals.set(item.name, (totals.get(item.name) ?? 0) + item.total_cost));
  return [...totals.entries()].sort((left, right) => right[1] - left[1]).map(([name]) => name);
}

function modelCost(items: DashboardDailyBreakdownItem[], name: string) {
  return items.find((item) => item.name === name)?.total_cost ?? 0;
}

function compactModelName(name: string) {
  return name.replace(/^claude-/, '').replace(/^gpt-/, '');
}

function formatDateLabel(date: string, locale: string) {
  return new Intl.DateTimeFormat(locale, { month: '2-digit', day: '2-digit' }).format(new Date(`${date}T00:00:00`));
}

function paletteColors(theme: Theme) {
  return [
    theme.palette.primary.main,
    theme.palette.warning.main,
    theme.palette.info.main,
    theme.palette.error.main,
    theme.palette.success.main,
    theme.palette.secondary.main,
    theme.palette.primary.dark,
    theme.palette.warning.dark,
  ];
}
