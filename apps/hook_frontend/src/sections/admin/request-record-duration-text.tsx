'use client';

import type { RequestRecordStatus } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Typography from '@mui/material/Typography';

import { isLiveTiming, formatRequestTiming } from './request-record-timing';

type DurationMetric = 'response_headers' | 'first_sse_event' | 'first_output' | 'total_latency';

type DurationRecord = Readonly<{
  created_at: string;
  status: RequestRecordStatus;
  response_headers_time_ms?: number | null;
  first_sse_event_time_ms?: number | null;
  first_output_time_ms?: number | null;
  first_byte_time_ms?: number | null;
  total_latency_ms?: number | null;
}>;

export function useRequestRecordDurationNow(records: DurationRecord[]) {
  const shouldTick = useMemo(() => records.some(recordNeedsLiveDuration), [records]);
  const [now, setNow] = useState(0);

  useEffect(() => {
    if (!shouldTick) return undefined;

    let rafId = 0;
    const tick = () => {
      setNow(Date.now());
      rafId = window.requestAnimationFrame(tick);
    };

    setNow(Date.now());
    rafId = window.requestAnimationFrame(tick);

    return () => window.cancelAnimationFrame(rafId);
  }, [shouldTick]);

  return now;
}

export function RequestRecordDurationText({
  record,
  metric,
  now,
}: {
  record: DurationRecord;
  metric: DurationMetric;
  now: number;
}) {
  const live = isLiveTiming(record, metric);
  const text = formatRequestTiming(record, metric, now);

  return (
    <Typography component="span" variant="body2" sx={live ? liveDurationTextSx : durationTextSx}>
      {text}
    </Typography>
  );
}

function recordNeedsLiveDuration(record: DurationRecord) {
  return (
    isLiveTiming(record, 'response_headers') ||
    isLiveTiming(record, 'first_output') ||
    isLiveTiming(record, 'total_latency')
  );
}

const durationTextSx = {
  display: 'inline-block',
  minWidth: 64,
  fontVariantNumeric: 'tabular-nums',
};

const liveDurationTextSx = {
  ...durationTextSx,
  color: 'error.main',
};
