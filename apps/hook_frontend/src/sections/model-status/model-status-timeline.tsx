'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusValue, ModelStatusTimelinePoint } from 'src/types/model-status';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';

const STATUS_COLORS: Record<ModelStatusValue, string> = {
  operational: '#16a34a',
  degraded: '#d97706',
  failed: '#dc2626',
  error: '#b91c1c',
};

export function ModelStatusTimeline({
  points,
  t,
}: {
  points: ModelStatusTimelinePoint[];
  t: TFunction<'admin'>;
}) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'row-reverse',
        gap: '2px',
        width: 1,
        height: 28,
        p: '2px',
      }}
    >
      {points.slice(0, 60).map((point) => (
        <TimelineSegment key={`${point.checked_at}-${point.status}`} point={point} t={t} />
      ))}
    </Box>
  );
}

function TimelineSegment({ point, t }: { point: ModelStatusTimelinePoint; t: TFunction<'admin'> }) {
  const label = segmentLabel(point, t);
  return (
    <Tooltip arrow title={<TooltipContent point={point} t={t} />}>
      <Box
        component="button"
        type="button"
        aria-label={label}
        sx={{
          position: 'relative',
          display: 'block',
          flex: 1,
          width: 1,
          height: 1,
          minWidth: 2,
          border: 0,
          borderRadius: '1px',
          cursor: 'default',
          bgcolor: STATUS_COLORS[point.status],
          transition: 'opacity 200ms ease, transform 200ms ease',
          '&:hover': { opacity: 0.8, transform: 'scaleY(1.1)' },
        }}
      />
    </Tooltip>
  );
}

function TooltipContent({ point, t }: { point: ModelStatusTimelinePoint; t: TFunction<'admin'> }) {
  return (
    <Box>
      <Typography variant="caption" component="div">
        {formatTime(point.checked_at)} · {statusLabel(point.status, t)}
      </Typography>
      <Typography variant="caption" component="div">
        {latencyLabel(point.latency_ms)}
        {point.status_code ? ` · HTTP ${point.status_code}` : ''}
      </Typography>
      {point.message ? (
        <Typography variant="caption" component="div">
          {point.message}
        </Typography>
      ) : null}
    </Box>
  );
}

function segmentLabel(point: ModelStatusTimelinePoint, t: TFunction<'admin'>) {
  return `${point.checked_at} · ${statusLabel(point.status, t)}`;
}

export function statusLabel(status: ModelStatusValue | null | undefined, t: TFunction<'admin'>) {
  return t(`modelStatus.statusLabel.${status ?? 'unknown'}`);
}

export function latencyLabel(value: number | null | undefined) {
  return typeof value === 'number' ? `${value} ms` : '-';
}

function formatTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}
