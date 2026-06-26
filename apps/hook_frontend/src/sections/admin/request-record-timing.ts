import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';

import { formatDuration } from './request-records-utils';

type TimingRecord = Pick<
  RequestRecord,
  | 'created_at'
  | 'status'
  | 'response_headers_time_ms'
  | 'first_sse_event_time_ms'
  | 'first_output_time_ms'
  | 'total_latency_ms'
>;

type TimingAttempt = Pick<
  RequestCandidateDetail,
  | 'response_headers_time_ms'
  | 'first_sse_event_time_ms'
  | 'first_output_time_ms'
  | 'latency_ms'
>;

export type RequestTimingMetric = 'response_headers' | 'first_sse_event' | 'first_output' | 'total_latency';

export function formatRequestTiming(record: TimingRecord, metric: RequestTimingMetric, now?: number) {
  const live = now !== undefined && isLiveTiming(record, metric);
  if (live) {
    return formatDuration(liveDurationMs(record.created_at, now));
  }
  return formatDuration(requestTimingValue(record, metric));
}

export function formatAttemptTiming(attempt: TimingAttempt, metric: RequestTimingMetric) {
  return formatDuration(attemptTimingValue(attempt, metric));
}

export function requestTimingValue(record: TimingRecord, metric: RequestTimingMetric) {
  if (metric === 'response_headers') return record.response_headers_time_ms;
  if (metric === 'first_sse_event') return record.first_sse_event_time_ms;
  if (metric === 'first_output') return record.first_output_time_ms;
  return record.total_latency_ms;
}

export function attemptTimingValue(attempt: TimingAttempt, metric: RequestTimingMetric) {
  if (metric === 'response_headers') return attempt.response_headers_time_ms;
  if (metric === 'first_sse_event') return attempt.first_sse_event_time_ms;
  if (metric === 'first_output') return attempt.first_output_time_ms;
  return attempt.latency_ms;
}

export function isLiveTiming(record: TimingRecord, metric: RequestTimingMetric) {
  if (!isActiveStatus(record.status)) return false;
  if (metric === 'total_latency') return true;
  return requestTimingValue(record, metric) === null || requestTimingValue(record, metric) === undefined;
}

function isActiveStatus(status: string) {
  return status === 'pending' || status === 'streaming';
}

function liveDurationMs(createdAt: string, now: number) {
  const createdAtMs = parseRequestTimestampMs(createdAt);
  if (Number.isNaN(createdAtMs)) return null;
  return Math.max(0, now - createdAtMs);
}

function parseRequestTimestampMs(value: string) {
  const normalized = /(?:Z|[+-]\d{2}:\d{2})$/i.test(value) ? value : `${value}Z`;
  return new Date(normalized).getTime();
}
