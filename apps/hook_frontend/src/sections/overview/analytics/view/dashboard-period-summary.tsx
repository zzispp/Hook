import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { PaletteColorKey } from 'src/theme/core';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardPreset, DashboardSummary } from 'src/types/dashboard';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Skeleton from '@mui/material/Skeleton';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { dashboardPeriodLabel } from './dashboard-period';
import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardPercent,
} from './dashboard-format';

type PeriodItem = {
  label: string;
  value: string;
  icon: IconifyName;
  color: PaletteColorKey;
  detail?: string;
};

const PERIOD_GRID_SX = {
  mb: 3,
  gap: 2,
  display: 'grid',
  gridTemplateColumns: {
    xs: '1fr',
    sm: 'repeat(2, minmax(0, 1fr))',
    lg: 'repeat(4, minmax(0, 1fr))',
  },
} as const;

export function PeriodSummaryGrid({
  t,
  locale,
  isAdmin,
  loading,
  preset,
  summary,
}: {
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
  loading: boolean;
  preset: DashboardPreset;
  summary?: DashboardSummary;
}) {
  const items = periodItems(t, locale, isAdmin, preset, summary);

  return (
    <Box sx={PERIOD_GRID_SX}>
      {items.map((item) => (
        <Box key={item.label} sx={{ minWidth: 0 }}>
          {loading ? <PeriodSkeleton /> : <PeriodCard item={item} />}
        </Box>
      ))}
    </Box>
  );
}

function PeriodCard({ item }: { item: PeriodItem }) {
  const theme = useTheme();

  return (
    <Card sx={periodCardSx(theme, item.color)}>
      <Stack direction="row" spacing={2} sx={{ alignItems: 'flex-start' }}>
        <Box sx={periodIconSx(theme, item.color)}>
          <Iconify aria-hidden icon={item.icon} width={22} />
        </Box>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {item.label}
          </Typography>
          <Typography variant="h5" sx={{ mt: 0.5 }}>
            {item.value}
          </Typography>
          {item.detail ? (
            <Typography variant="caption" sx={{ mt: 0.5, display: 'block', color: 'text.secondary' }}>
              {item.detail}
            </Typography>
          ) : null}
        </Box>
      </Stack>
    </Card>
  );
}

function PeriodSkeleton() {
  return (
    <Card sx={{ p: 2.5, boxShadow: 'none' }}>
      <Skeleton width={42} height={42} sx={{ mb: 1 }} />
      <Skeleton width="52%" />
      <Skeleton width="70%" height={34} />
    </Card>
  );
}

function periodItems(
  t: TFunction<'admin'>,
  locale: string,
  isAdmin: boolean,
  preset: DashboardPreset,
  summary?: DashboardSummary
): PeriodItem[] {
  const period = dashboardPeriodLabel(t, preset);
  if (isAdmin) return adminPeriodItems(t, locale, period, summary);
  return userPeriodItems(t, period, summary);
}

function adminPeriodItems(
  t: TFunction<'admin'>,
  locale: string,
  period: string,
  summary?: DashboardSummary
): PeriodItem[] {
  return [
    metric(
      t('dashboard.stats.period.avgResponse', { period }),
      formatMs(summary?.avg_latency_ms),
      'solar:clock-circle-bold',
      'primary'
    ),
    metric(
      t('dashboard.stats.period.errorRate', { period }),
      formatDashboardPercent(summary?.error_rate),
      'solar:danger-triangle-bold',
      'error'
    ),
    metric(
      t('dashboard.stats.period.failovers', { period }),
      formatInteger(summary?.failover_count, locale),
      'solar:transfer-horizontal-bold-duotone',
      'warning'
    ),
    metric(
      t('dashboard.stats.period.cost', { period }),
      formatDashboardCost(summary?.total_cost),
      'solar:bill-list-bold',
      'info',
      t('dashboard.stats.period.upstreamCost', {
        value: formatDashboardCost(summary?.upstream_total_cost),
      })
    ),
  ];
}

function userPeriodItems(
  t: TFunction<'admin'>,
  period: string,
  summary?: DashboardSummary
): PeriodItem[] {
  return [
    metric(
      t('dashboard.stats.period.cacheHitRate', { period }),
      formatDashboardPercent(summary?.cache_hit_rate),
      'solar:chart-square-outline',
      'success'
    ),
    metric(
      t('dashboard.stats.period.cacheRead', { period }),
      formatDashboardTokens(summary?.cache_read_input_tokens),
      'solar:ssd-round-bold',
      'primary'
    ),
    metric(
      t('dashboard.stats.period.cacheCreate', { period }),
      formatDashboardTokens(summary?.cache_creation_input_tokens),
      'solar:archive-down-minimlistic-bold',
      'warning'
    ),
    metric(
      t('dashboard.stats.period.cost', { period }),
      formatDashboardCost(summary?.total_cost),
      'solar:bill-list-bold',
      'info'
    ),
  ];
}

function metric(
  label: string,
  value: string,
  icon: IconifyName,
  color: PaletteColorKey,
  detail?: string
): PeriodItem {
  return { label, value, icon, color, detail };
}

function periodCardSx(theme: Theme, color: PaletteColorKey) {
  return {
    p: 2.5,
    height: 1,
    boxShadow: 'none',
    border: `1px solid ${varAlpha(theme.vars.palette[color].mainChannel, 0.16)}`,
  };
}

function periodIconSx(theme: Theme, color: PaletteColorKey) {
  return {
    width: 42,
    height: 42,
    display: 'grid',
    flexShrink: 0,
    borderRadius: 1.5,
    placeItems: 'center',
    color: `${color}.main`,
    bgcolor: varAlpha(theme.vars.palette[color].mainChannel, 0.12),
  };
}
