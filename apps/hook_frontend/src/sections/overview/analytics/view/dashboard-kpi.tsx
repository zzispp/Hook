import type { TFunction } from 'i18next';
import type { CardProps } from '@mui/material/Card';
import type { PaletteColorKey } from 'src/theme/core';
import type { ChartOptions } from 'src/components/chart';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Skeleton from '@mui/material/Skeleton';
import { useTheme } from '@mui/material/styles';

import { CONFIG } from 'src/global-config';

import { SvgColor } from 'src/components/svg-color';
import { Chart, useChart } from 'src/components/chart';

import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
} from './dashboard-format';

type KpiCardData = {
  label: string;
  value: string;
  color: PaletteColorKey;
  icon: string;
  series: number[];
};

export function KpiGrid({
  t,
  data,
  locale,
  loading,
}: {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  data?: DashboardOverviewResponse;
}) {
  const cards = kpiCards(t, locale, data);

  return (
    <Grid container spacing={3} sx={{ mb: 3 }}>
      {cards.map((item) => (
        <Grid key={item.label} size={{ xs: 12, sm: 6, md: 3 }}>
          {loading ? <KpiSkeleton /> : <DashboardKpiCard item={item} />}
        </Grid>
      ))}
    </Grid>
  );
}

function DashboardKpiCard({ item, sx, ...other }: CardProps & { item: KpiCardData }) {
  const theme = useTheme();
  const chartOptions = useChart({
    chart: { sparkline: { enabled: true } },
    colors: [theme.palette[item.color].dark],
    grid: { padding: { top: 6, left: 6, right: 6, bottom: 6 } },
    xaxis: { labels: { show: false } },
    yaxis: { labels: { show: false } },
    tooltip: { y: { formatter: (value: number) => formatInteger(value, 'en-US'), title: { formatter: () => '' } } },
    markers: { strokeWidth: 0 },
  } satisfies ChartOptions);

  return (
    <Card
      sx={[
        () => ({
          p: 3,
          boxShadow: 'none',
          position: 'relative',
          color: `${item.color}.darker`,
          backgroundColor: 'common.white',
          backgroundImage: `linear-gradient(135deg, ${varAlpha(theme.vars.palette[item.color].lighterChannel, 0.48)}, ${varAlpha(theme.vars.palette[item.color].lightChannel, 0.48)})`,
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Box sx={{ width: 48, height: 48, mb: 3 }}>
        <Box component="img" alt={item.label} src={item.icon} sx={{ width: 1, height: 1 }} />
      </Box>
      <Box sx={{ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-end', justifyContent: 'flex-end' }}>
        <Box sx={{ flexGrow: 1, minWidth: 112 }}>
          <Box sx={{ mb: 1, typography: 'subtitle2' }}>{item.label}</Box>
          <Box sx={{ typography: 'h4' }}>{item.value}</Box>
        </Box>
        <Chart type="line" series={[{ data: item.series }]} options={chartOptions} sx={{ width: 84, height: 56 }} />
      </Box>
      <SvgColor
        src={`${CONFIG.assetsDir}/assets/background/shape-square.svg`}
        sx={{
          top: 0,
          left: -20,
          width: 240,
          zIndex: -1,
          height: 240,
          opacity: 0.24,
          position: 'absolute',
          color: `${item.color}.main`,
        }}
      />
    </Card>
  );
}

function KpiSkeleton() {
  return (
    <Card sx={{ p: 3, boxShadow: 'none', bgcolor: 'background.neutral' }}>
      <Skeleton variant="circular" width={48} height={48} sx={{ mb: 3 }} />
      <Skeleton width="48%" />
      <Skeleton width="64%" height={40} />
    </Card>
  );
}

function kpiCards(
  t: TFunction<'admin'>,
  locale: string,
  data?: DashboardOverviewResponse
): KpiCardData[] {
  const summary = data?.summary;
  const points = data?.timeseries ?? [];
  return [
    kpiCard(t('dashboard.stats.kpi.requests'), formatInteger(summary?.request_count, locale), 'primary', 'ic-glass-bag.svg', points.map((point) => point.request_count)),
    kpiCard(t('dashboard.stats.kpi.successRate'), `${((summary?.success_rate ?? 0) * 100).toFixed(1)}%`, 'success', 'ic-glass-users.svg', points.map((point) => ratioPercent(point.success_count, point.success_count + point.failed_count))),
    kpiCard(t('dashboard.stats.kpi.tokens'), formatDashboardTokens(summary?.total_tokens), 'warning', 'ic-glass-buy.svg', points.map((point) => point.total_tokens)),
    kpiCard(t('dashboard.stats.kpi.cost'), formatDashboardCost(summary?.total_cost), 'info', 'ic-glass-bag.svg', points.map((point) => point.total_cost)),
    kpiCard(t('dashboard.stats.kpi.active'), formatInteger(summary?.active_count, locale), 'secondary', 'ic-glass-message.svg', points.map((point) => point.request_count)),
    kpiCard(t('dashboard.stats.kpi.failed'), formatInteger(summary?.failed_count, locale), 'error', 'ic-glass-message.svg', points.map((point) => point.failed_count)),
    kpiCard(t('dashboard.stats.kpi.latency'), formatMs(summary?.avg_latency_ms), 'secondary', 'ic-glass-buy.svg', points.map((point) => point.avg_latency_ms ?? 0)),
    kpiCard(t('dashboard.stats.kpi.models'), formatInteger(summary?.model_count, locale), 'primary', 'ic-glass-users.svg', points.map((point) => point.request_count)),
  ];
}

function kpiCard(label: string, value: string, color: PaletteColorKey, icon: string, series: number[]): KpiCardData {
  return {
    label,
    value,
    color,
    icon: `${CONFIG.assetsDir}/assets/icons/glass/${icon}`,
    series: series.length ? series : [0, 0, 0, 0, 0, 0, 0],
  };
}

function ratioPercent(value: number, total: number) {
  if (total <= 0) return 0;
  return Number(((value / total) * 100).toFixed(1));
}
