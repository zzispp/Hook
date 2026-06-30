import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { ChartOptions } from 'src/components/chart';
import type { DashboardBreakdownItem } from 'src/types/dashboard';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Skeleton from '@mui/material/Skeleton';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

import { Chart, useChart, ChartLegends } from 'src/components/chart';

import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardPercent,
} from './dashboard-format';

type BreakdownVariant = 'ranking' | 'distribution';

export function BreakdownCard({
  t,
  title,
  items,
  locale,
  loading,
  isAdmin,
  variant = 'ranking',
}: {
  t: TFunction<'admin'>;
  title: string;
  locale: string;
  isAdmin: boolean;
  loading: boolean;
  variant?: BreakdownVariant;
  items?: DashboardBreakdownItem[];
}) {
  return (
    <Card sx={{ height: 1 }}>
      <CardHeader title={title} />
      {loading ? <BreakdownLoading /> : null}
      {!loading && !items?.length ? <BreakdownEmpty t={t} /> : null}
      {!loading && items?.length ? (
        variant === 'distribution' ? (
          <DistributionBreakdown items={items} locale={locale} t={t} isAdmin={isAdmin} />
        ) : (
          <RankingBreakdown items={items} locale={locale} t={t} isAdmin={isAdmin} />
        )
      ) : null}
    </Card>
  );
}

function RankingBreakdown({
  items,
  locale,
  t,
  isAdmin,
}: {
  items: DashboardBreakdownItem[];
  locale: string;
  t: TFunction<'admin'>;
  isAdmin: boolean;
}) {
  const theme = useTheme();
  const options = useChart({
    colors: [theme.palette.primary.dark, hexAlpha(theme.palette.primary.dark, 0.24)],
    stroke: { width: 2, colors: ['transparent'] },
    xaxis: { categories: items.map((item) => item.name) },
    tooltip: { y: { formatter: (value: number) => formatInteger(value, locale) } },
    dataLabels: {
      enabled: true,
      offsetX: -6,
      style: { fontSize: '10px', colors: ['#FFFFFF', theme.vars.palette.text.primary] },
    },
    plotOptions: {
      bar: {
        horizontal: true,
        borderRadius: 2,
        barHeight: '48%',
        dataLabels: { position: 'top' },
      },
    },
  } satisfies ChartOptions);

  return (
    <>
      <Chart
        type="bar"
        series={[{ name: t('dashboard.stats.columns.requests'), data: items.map((item) => item.request_count) }]}
        options={options}
        slotProps={{ loading: { p: 2.5 } }}
        sx={{ pl: 1, py: 2.5, pr: 2.5, height: Math.max(280, items.length * 44 + 112) }}
      />
      <Divider sx={{ borderStyle: 'dashed' }} />
      <BreakdownDetails items={items} t={t} locale={locale} isAdmin={isAdmin} />
    </>
  );
}

function DistributionBreakdown({
  items,
  locale,
  t,
  isAdmin,
}: {
  items: DashboardBreakdownItem[];
  locale: string;
  t: TFunction<'admin'>;
  isAdmin: boolean;
}) {
  const theme = useTheme();
  const labels = items.map((item) => item.name);
  const colors = paletteColors(theme).slice(0, items.length);
  const options = useChart({
    chart: { sparkline: { enabled: true } },
    colors,
    labels,
    stroke: { width: 0 },
    dataLabels: { enabled: true, dropShadow: { enabled: false } },
    tooltip: { y: { formatter: (value: number) => formatInteger(value, locale) } },
    plotOptions: { pie: { donut: { labels: { show: false } } } },
  } satisfies ChartOptions);

  return (
    <>
      <Chart
        type="pie"
        series={items.map((item) => item.request_count)}
        options={options}
        sx={{ my: 6, mx: 'auto', width: { xs: 240, xl: 260 }, height: { xs: 240, xl: 260 } }}
      />
      <Divider sx={{ borderStyle: 'dashed' }} />
      <ChartLegends
        labels={labels}
        colors={colors}
        values={items.map((item) => formatInteger(item.request_count, locale))}
        sx={{ p: 3, justifyContent: 'center' }}
      />
      <Divider sx={{ borderStyle: 'dashed' }} />
      <BreakdownDetails items={items} t={t} locale={locale} isAdmin={isAdmin} showLatency />
    </>
  );
}

function paletteColors(theme: Theme) {
  return [
    theme.palette.primary.main,
    theme.palette.warning.light,
    theme.palette.info.dark,
    theme.palette.error.main,
    theme.palette.success.main,
    theme.palette.secondary.main,
    theme.palette.primary.dark,
    theme.palette.warning.dark,
    theme.palette.info.main,
    theme.palette.error.dark,
  ];
}

function BreakdownLoading() {
  return (
    <Stack spacing={2} sx={{ p: 3 }}>
      {Array.from({ length: 4 }).map((_, index) => (
        <Stack key={index} spacing={1}>
          <Skeleton width="55%" />
          <Skeleton variant="rectangular" height={24} />
        </Stack>
      ))}
    </Stack>
  );
}

function BreakdownEmpty({ t }: { t: TFunction<'admin'> }) {
  return <Typography sx={{ px: 3, py: 4, color: 'text.secondary' }}>{t('dashboard.stats.empty')}</Typography>;
}

function BreakdownDetails({
  t,
  items,
  locale,
  isAdmin,
  showLatency = false,
}: {
  t: TFunction<'admin'>;
  items: DashboardBreakdownItem[];
  locale: string;
  isAdmin: boolean;
  showLatency?: boolean;
}) {
  return (
    <Stack divider={<Divider flexItem />} sx={{ px: 3, pb: 3 }}>
      {items.map((item) => (
        <Stack key={`${item.id ?? item.name}`} spacing={0.5} sx={{ py: 1.25 }}>
          <Typography variant="subtitle2" noWrap>
            {item.name}
          </Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {detailText(item, t, locale, showLatency, isAdmin)}
          </Typography>
        </Stack>
      ))}
    </Stack>
  );
}

function detailText(
  item: DashboardBreakdownItem,
  t: TFunction<'admin'>,
  locale: string,
  showLatency: boolean,
  isAdmin: boolean
) {
  const parts = [
    `${t('dashboard.stats.columns.requests')}: ${formatInteger(item.request_count, locale)}`,
    `${t('dashboard.stats.columns.tokens')}: ${formatDashboardTokens(item.total_tokens)}`,
  ];
  if (showLatency) {
    parts.push(`${t('requestRecords.responseHeaders')}: ${formatMs(item.avg_response_headers_ms)}`);
    parts.push(`${t('requestRecords.firstByte')}: ${formatMs(item.avg_first_byte_ms)}`);
    parts.push(`${t('dashboard.stats.columns.avgLatency')}: ${formatMs(item.avg_latency_ms)}`);
    parts.push(`${t('dashboard.stats.columns.firstToken')}: ${formatMs(item.avg_first_token_ms)}`);
  }
  parts.push(`${t('dashboard.stats.columns.cost')}: ${formatDashboardCost(item.total_cost)}`);
  if (isAdmin) {
    parts.push(`${t('dashboard.stats.columns.upstreamCost')}: ${formatDashboardCost(item.upstream_total_cost)}`);
    parts.push(`${t('dashboard.stats.columns.profitRate')}: ${formatDashboardPercent(item.profit_rate)}`);
  }
  return parts.join(' · ');
}
