import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { CardProps } from '@mui/material/Card';
import type { PaletteColorKey } from 'src/theme/core';
import type { ChartOptions } from 'src/components/chart';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Skeleton from '@mui/material/Skeleton';
import { useTheme } from '@mui/material/styles';

import { CONFIG } from 'src/global-config';

import { Iconify } from 'src/components/iconify';
import { SvgColor } from 'src/components/svg-color';
import { Chart, useChart } from 'src/components/chart';

import { formatInteger } from './dashboard-format';
import { KPI_CARD_CONFIGS, type KpiCardData, type KpiCardConfig } from './dashboard-kpi-config';

const KPI_ICON_SIZE = 48;
const KPI_ICON_GLYPH_SIZE = 30;
const KPI_GRID_SPACING = 3;
const KPI_CARD_PADDING = 3;
const KPI_CARD_RADIUS = 2;
const KPI_TEXT_MIN_WIDTH = 112;
const KPI_CHART_WIDTH = 84;
const KPI_CHART_HEIGHT = 56;
const KPI_ICON_BG_OPACITY = 0.16;
const KPI_GRADIENT_OPACITY = 0.48;
const KPI_SHAPE_LEFT = -20;
const KPI_SHAPE_SIZE = 240;
const KPI_SHAPE_OPACITY = 0.24;
const KPI_SKELETON_VALUE_HEIGHT = 40;
const SPARKLINE_PADDING = 6;
const SPARKLINE_STROKE_WIDTH = 0;
const DEFAULT_SPARKLINE_SERIES = [0, 0, 0, 0, 0, 0, 0];
const KPI_GRID_SX = {
  mb: KPI_GRID_SPACING,
  gap: KPI_GRID_SPACING,
  display: 'grid',
  gridTemplateColumns: {
    xs: '1fr',
    sm: 'repeat(2, minmax(0, 1fr))',
    lg: 'repeat(5, minmax(0, 1fr))',
  },
} as const;

type KpiCardInput = KpiCardData;

type KpiConfigInput = {
  t: TFunction<'admin'>;
  locale: string;
  summary: DashboardOverviewResponse['summary'] | undefined;
  points: DashboardOverviewResponse['timeseries'];
  config: KpiCardConfig;
};

export function KpiGrid({
  t,
  data,
  locale,
  isAdmin,
  loading,
}: {
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
  loading: boolean;
  data?: DashboardOverviewResponse;
}) {
  const cards = kpiCards(t, locale, isAdmin, data);

  return (
    <Box sx={KPI_GRID_SX}>
      {cards.map((item) => (
        <Box key={item.label} sx={{ minWidth: 0 }}>
          {loading ? <KpiSkeleton /> : <DashboardKpiCard item={item} />}
        </Box>
      ))}
    </Box>
  );
}

function DashboardKpiCard({ item, sx, ...other }: CardProps & { item: KpiCardData }) {
  const theme = useTheme();
  const chartOptions = useKpiChartOptions(item.color);

  return (
    <Card
      sx={[
        () => kpiCardSurfaceSx(theme, item.color),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <KpiIcon color={item.color} icon={item.icon} />
      <KpiContent item={item} chartOptions={chartOptions} />
      <KpiShape color={item.color} />
    </Card>
  );
}

function useKpiChartOptions(color: PaletteColorKey) {
  const theme = useTheme();

  return useChart({
    chart: { sparkline: { enabled: true } },
    colors: [theme.palette[color].dark],
    grid: { padding: sparklinePadding() },
    xaxis: { labels: { show: false } },
    yaxis: { labels: { show: false } },
    tooltip: { y: { formatter: formatTooltipValue, title: { formatter: emptyTooltipTitle } } },
    markers: { strokeWidth: SPARKLINE_STROKE_WIDTH },
  } satisfies ChartOptions);
}

function KpiIcon({ color, icon }: { color: PaletteColorKey; icon: IconifyName }) {
  const theme = useTheme();

  return (
    <Box
      sx={{
        mb: KPI_GRID_SPACING,
        width: KPI_ICON_SIZE,
        height: KPI_ICON_SIZE,
        display: 'grid',
        borderRadius: KPI_CARD_RADIUS,
        placeItems: 'center',
        bgcolor: varAlpha(theme.vars.palette[color].mainChannel, KPI_ICON_BG_OPACITY),
      }}
    >
      <Iconify aria-hidden icon={icon} width={KPI_ICON_GLYPH_SIZE} />
    </Box>
  );
}

function KpiContent({ item, chartOptions }: { item: KpiCardData; chartOptions: ChartOptions }) {
  return (
    <Box sx={{ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-end', justifyContent: 'flex-end' }}>
      <Box sx={{ flexGrow: 1, minWidth: KPI_TEXT_MIN_WIDTH }}>
        <Box sx={{ mb: 1, typography: 'subtitle2' }}>{item.label}</Box>
        <Box sx={{ typography: 'h4' }}>{item.value}</Box>
      </Box>
      <Chart
        type="line"
        series={[{ data: item.series }]}
        options={chartOptions}
        sx={{ width: KPI_CHART_WIDTH, height: KPI_CHART_HEIGHT }}
      />
    </Box>
  );
}

function KpiShape({ color }: { color: PaletteColorKey }) {
  return (
    <SvgColor
      src={`${CONFIG.assetsDir}/assets/background/shape-square.svg`}
      sx={{
        top: 0,
        zIndex: -1,
        position: 'absolute',
        left: KPI_SHAPE_LEFT,
        width: KPI_SHAPE_SIZE,
        height: KPI_SHAPE_SIZE,
        opacity: KPI_SHAPE_OPACITY,
        color: `${color}.main`,
      }}
    />
  );
}

function KpiSkeleton() {
  return (
    <Card sx={{ p: KPI_CARD_PADDING, boxShadow: 'none', bgcolor: 'background.neutral' }}>
      <Skeleton
        variant="circular"
        width={KPI_ICON_SIZE}
        height={KPI_ICON_SIZE}
        sx={{ mb: KPI_GRID_SPACING }}
      />
      <Skeleton width="48%" />
      <Skeleton width="64%" height={KPI_SKELETON_VALUE_HEIGHT} />
    </Card>
  );
}

function kpiCards(
  t: TFunction<'admin'>,
  locale: string,
  isAdmin: boolean,
  data?: DashboardOverviewResponse
): KpiCardData[] {
  const summary = data?.summary;
  const points = data?.timeseries ?? [];
  return KPI_CARD_CONFIGS
    .filter((config) => isAdmin || !config.adminOnly)
    .map((config) => cardFromConfig({ t, locale, summary, points, config }));
}

function kpiCard({ label, value, color, icon, series }: KpiCardInput): KpiCardData {
  return {
    label,
    value,
    color,
    icon,
    series: series.length ? series : DEFAULT_SPARKLINE_SERIES,
  };
}

function cardFromConfig({ t, locale, summary, points, config }: KpiConfigInput) {
  return kpiCard({
    label: t(config.labelKey),
    value: config.value(summary, locale),
    color: config.color,
    icon: config.icon,
    series: config.series(points),
  });
}

function sparklinePadding() {
  return {
    top: SPARKLINE_PADDING,
    left: SPARKLINE_PADDING,
    right: SPARKLINE_PADDING,
    bottom: SPARKLINE_PADDING,
  };
}

function formatTooltipValue(value: number) {
  return formatInteger(value, 'en-US');
}

function emptyTooltipTitle() {
  return '';
}

function kpiCardSurfaceSx(theme: Theme, color: PaletteColorKey) {
  return {
    p: KPI_CARD_PADDING,
    boxShadow: 'none',
    position: 'relative',
    color: `${color}.darker`,
    backgroundColor: 'common.white',
    backgroundImage: `linear-gradient(135deg, ${varAlpha(theme.vars.palette[color].lighterChannel, KPI_GRADIENT_OPACITY)}, ${varAlpha(theme.vars.palette[color].lightChannel, KPI_GRADIENT_OPACITY)})`,
  };
}
