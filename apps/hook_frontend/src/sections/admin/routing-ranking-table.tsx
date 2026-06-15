'use client';

import type { AdminT } from './shared';
import type { RoutingRouteState, RouteScoreExplanation } from 'src/types/routing';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';
import TableContainer from '@mui/material/TableContainer';

import { fNumber } from 'src/utils/format-number';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableHeadCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import {
  routeScoreReason,
  scoreComponentLabel,
  routingMetricSummary,
  translatedExclusionReason,
} from './routing-i18n';

type Props = {
  rows: RouteScoreExplanation[];
  loading: boolean;
  t: AdminT;
  onOpen: (item: RouteScoreExplanation) => void;
};

const HEAD = [
  { id: 'rank', width: 72 },
  { id: 'route', minWidth: 220 },
  { id: 'state', width: 132 },
  { id: 'score', width: 120 },
  { id: 'reason', minWidth: 260 },
  { id: 'breakdown', minWidth: 260 },
  { id: 'metrics', minWidth: 260 },
  { id: 'penalty', minWidth: 220 },
  { id: 'actions', width: 72 },
];

export function RoutingRankingTable({ rows, loading, t, onOpen }: Props) {
  const head = HEAD.map((item) => {
    if (item.id === 'actions') {
      return withStickyActionHeadCell({
        id: 'actions',
        label: '',
        width: 72,
      });
    }
    const { minWidth, ...rest } = item;
    return {
      ...rest,
      label: t(`routing.table.${item.id}`),
      sx: minWidth ? { minWidth } : undefined,
    };
  });

  return (
    <TableContainer sx={{ position: 'relative', overflow: 'hidden' }}>
      <Scrollbar>
        <Table size="small" sx={{ minWidth: 1500 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {rows.map((row) => (
              <RankingRow key={routeKey(row)} row={row} t={t} onOpen={onOpen} />
            ))}
            <TableNoData
              title={loading ? t('routing.loading') : t('routing.empty')}
              notFound={!loading && rows.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
    </TableContainer>
  );
}

function RankingRow({ row, t, onOpen }: { row: RouteScoreExplanation; t: AdminT; onOpen: Props['onOpen'] }) {
  return (
    <TableRow hover>
      <TableCell>{row.rank}</TableCell>
      <TableCell>
        <RouteCell row={row} />
      </TableCell>
      <TableCell>
        <StateLabel state={row.state} t={t} />
      </TableCell>
      <TableCell>{fNumber(row.final_score, { maximumFractionDigits: 1 })}</TableCell>
      <TableCell>
        <Typography variant="body2" sx={{ maxWidth: 360 }}>
          {routeScoreReason(row, t)}
        </Typography>
      </TableCell>
      <TableCell>
        <ScoreBreakdown row={row} t={t} />
      </TableCell>
      <TableCell>
        <MetricSummary row={row} t={t} />
      </TableCell>
      <TableCell>
        <PenaltySummary row={row} t={t} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Tooltip title={t('routing.actions.details')}>
          <Button size="small" color="inherit" onClick={() => onOpen(row)}>
            <Iconify icon="solar:eye-bold" />
          </Button>
        </Tooltip>
      </TableCell>
    </TableRow>
  );
}

function RouteCell({ row }: { row: RouteScoreExplanation }) {
  return (
    <Stack spacing={0.25}>
      <Typography variant="subtitle2">{row.provider_name || row.route.provider_id}</Typography>
      <Typography variant="body2" color="text.secondary">
        {row.key_name || row.route.key_id} · {row.key_preview || row.route.key_id}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {row.endpoint_name || row.route.endpoint_id}
      </Typography>
    </Stack>
  );
}

function ScoreBreakdown({ row, t }: { row: RouteScoreExplanation; t: AdminT }) {
  return (
    <Stack spacing={0.75}>
      {row.components.slice(0, 5).map((component) => (
        <Stack key={component.code} spacing={0.25}>
          <Stack direction="row" justifyContent="space-between" spacing={1}>
            <Typography variant="caption" color="text.secondary">
              {scoreComponentLabel(component, t)}
            </Typography>
            <Typography variant="caption">{component.contribution.toFixed(1)}</Typography>
          </Stack>
          <LinearProgress
            variant="determinate"
            value={Math.min(Math.abs(component.contribution), 100)}
            color={component.contribution < 0 ? 'error' : 'primary'}
            sx={{ height: 4, borderRadius: 0.5 }}
          />
        </Stack>
      ))}
    </Stack>
  );
}

function MetricSummary({ row, t }: { row: RouteScoreExplanation; t: AdminT }) {
  return (
    <Typography variant="body2" color="text.secondary">
      {routingMetricSummary(row.metric_window, row.raw_metrics, t)}
    </Typography>
  );
}

function PenaltySummary({ row, t }: { row: RouteScoreExplanation; t: AdminT }) {
  const penalties = row.components
    .filter((component) => component.contribution < 0)
    .map((component) => `${scoreComponentLabel(component, t)} ${component.contribution.toFixed(1)}`);
  const text = row.exclusion_reason
    ? translatedExclusionReason(row.exclusion_reason, t)
    : penalties.join(', ') || '-';

  return (
    <Typography variant="body2" color={row.exclusion_reason ? 'error.main' : 'text.secondary'}>
      {text}
    </Typography>
  );
}

function StateLabel({ state, t }: { state: RoutingRouteState; t: AdminT }) {
  const color =
    (state === 'eligible' && 'success') ||
    (state === 'warming' && 'info') ||
    (state === 'degraded' && 'warning') ||
    (state === 'circuit_open' && 'error') ||
    'default';

  return (
    <Label color={color} variant="soft">
      {t(`routing.states.${state}`)}
    </Label>
  );
}

function routeKey(row: RouteScoreExplanation) {
  const route = row.route;
  return `${route.provider_id}:${route.key_id}:${route.endpoint_id}:${route.global_model_id}:${route.provider_api_format}:${route.is_stream}`;
}
