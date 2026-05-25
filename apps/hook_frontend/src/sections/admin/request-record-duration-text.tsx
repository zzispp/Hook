'use client';

import type { RequestRecordStatus } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Typography from '@mui/material/Typography';

import { formatDuration } from './request-records-utils';

type DurationMetric = 'first_byte' | 'total_latency';

type DurationRecord = Readonly<{
  created_at: string;
  status: RequestRecordStatus;
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
  const live = isLiveDuration(record, metric);
  const value = durationValue(record, metric);
  const text = live ? formatLiveDuration(record.created_at, now) : formatDuration(value);

  return (
    <Typography component="span" variant="body2" sx={live ? liveDurationTextSx : durationTextSx}>
      {text}
    </Typography>
  );
}

function recordNeedsLiveDuration(record: DurationRecord) {
  return isLiveDuration(record, 'first_byte') || isLiveDuration(record, 'total_latency');
}

function isLiveDuration(record: DurationRecord, metric: DurationMetric) {
  if (!isActiveStatus(record.status)) return false;
  if (metric === 'total_latency') return true;
  return record.first_byte_time_ms === null || record.first_byte_time_ms === undefined;
}

function isActiveStatus(status: RequestRecordStatus) {
  return status === 'pending' || status === 'streaming';
}

function durationValue(record: DurationRecord, metric: DurationMetric) {
  return metric === 'first_byte' ? record.first_byte_time_ms : record.total_latency_ms;
}

function formatLiveDuration(createdAt: string, now: number) {
  const createdAtMs = parseRequestTimestampMs(createdAt);
  if (Number.isNaN(createdAtMs)) return 'N/A';

  const elapsedMs = Math.max(0, now - createdAtMs);
  return formatDuration(elapsedMs);
}

function parseRequestTimestampMs(value: string) {
  const normalized = /(?:Z|[+-]\d{2}:\d{2})$/i.test(value) ? value : `${value}Z`;
  return new Date(normalized).getTime();
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
