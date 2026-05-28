import type { TFunction } from 'i18next';
import type { DashboardPreset, DashboardDailyStats } from 'src/types/dashboard';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Skeleton from '@mui/material/Skeleton';
import Typography from '@mui/material/Typography';

import { dashboardPeriodLabel } from './dashboard-period';
import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
} from './dashboard-format';

type DailyTotals = {
  requestCount: number;
  totalTokens: number;
  totalCost: number;
  avgLatencyMs?: number;
};

const TOTALS_GRID_SX = {
  gap: 2,
  display: 'grid',
  gridTemplateColumns: {
    xs: '1fr',
    sm: 'repeat(2, minmax(0, 1fr))',
    lg: 'repeat(4, minmax(0, 1fr))',
  },
} as const;

export function DailyTotalsGrid({
  t,
  locale,
  loading,
  preset,
  data,
}: {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  preset: DashboardPreset;
  data?: DashboardDailyStats;
}) {
  const totals = dailyTotals(data);
  const period = dashboardPeriodLabel(t, preset);
  const items = [
    [t('dashboard.stats.period.requests', { period }), formatInteger(totals.requestCount, locale)],
    [t('dashboard.stats.period.tokens', { period }), formatDashboardTokens(totals.totalTokens)],
    [t('dashboard.stats.period.cost', { period }), formatDashboardCost(totals.totalCost)],
    [t('dashboard.stats.period.avgResponse', { period }), formatMs(totals.avgLatencyMs)],
  ];

  return (
    <Box sx={TOTALS_GRID_SX}>
      {items.map(([label, value]) => (
        <Card key={label} sx={{ p: 2.5, boxShadow: 'none' }}>
          {loading ? <TotalsSkeleton /> : <TotalsValue label={label} value={value} />}
        </Card>
      ))}
    </Box>
  );
}

function TotalsValue({ label, value }: { label: string; value: string }) {
  return (
    <>
      <Typography variant="caption" sx={{ color: 'text.secondary' }}>
        {label}
      </Typography>
      <Typography variant="h5" sx={{ mt: 0.5 }}>
        {value}
      </Typography>
    </>
  );
}

function TotalsSkeleton() {
  return (
    <>
      <Skeleton width="55%" />
      <Skeleton width="72%" height={34} />
    </>
  );
}

function dailyTotals(data?: DashboardDailyStats): DailyTotals {
  return (data?.days ?? []).reduce(
    (totals, day) => {
      const latency = day.avg_latency_ms ?? 0;
      return {
        requestCount: totals.requestCount + day.request_count,
        totalTokens: totals.totalTokens + day.total_tokens,
        totalCost: totals.totalCost + day.total_cost,
        avgLatencyMs: weightedLatency(totals, latency, day.request_count),
      };
    },
    { requestCount: 0, totalTokens: 0, totalCost: 0, avgLatencyMs: undefined } as DailyTotals
  );
}

function weightedLatency(totals: DailyTotals, latency: number, requestCount: number) {
  const totalRequests = totals.requestCount + requestCount;
  if (totalRequests <= 0) return undefined;
  return (((totals.avgLatencyMs ?? 0) * totals.requestCount) + latency * requestCount) / totalRequests;
}
