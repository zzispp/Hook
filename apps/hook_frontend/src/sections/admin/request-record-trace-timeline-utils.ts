import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';

import { formatRequestDate } from './request-records-utils';
import { formatApiFormat } from './provider-management-utils';

export type TraceStatus = 'success' | 'failed' | 'active' | 'queued' | 'unscheduled' | 'notScheduled';

export type TraceAttempt = RequestCandidateDetail & {
  traceStatus: TraceStatus;
};

export type TraceGroup = {
  id: string;
  providerName: string;
  status: TraceStatus;
  attempts: TraceAttempt[];
};

const STATUS_ORDER: Record<TraceStatus, number> = {
  notScheduled: 0,
  unscheduled: 1,
  queued: 2,
  failed: 3,
  active: 4,
  success: 5,
};

export function buildTraceGroups(record: RequestRecord | null, candidates: RequestCandidateDetail[]) {
  const attempts = candidates.map((candidate) => ({
    ...candidate,
    traceStatus: traceStatus(candidate, candidates, record),
  }));
  const groups: TraceGroup[] = [];
  for (const attempt of attempts) {
    appendAttempt(groups, attempt);
  }
  return groups;
}

export function defaultSelection(groups: TraceGroup[]) {
  const groupIndex = preferredGroupIndex(groups);
  const group = groups[groupIndex];
  const attemptIndex = group ? Math.max(group.attempts.findIndex((attempt) => attempt.traceStatus === group.status), 0) : 0;
  return { groupIndex, attemptIndex };
}

export function attemptFormat(attempt: RequestCandidateDetail) {
  const client = formatApiFormat(attempt.client_api_format);
  if (!attempt.provider_api_format || attempt.provider_api_format === attempt.client_api_format) return client;
  return `${client} -> ${formatApiFormat(attempt.provider_api_format)}`;
}

export function attemptKey(attempt: RequestCandidateDetail) {
  return [attempt.key_name, attempt.key_preview].filter(Boolean).join(' ') || '-';
}

export function attemptTime(attempt: RequestCandidateDetail, locale: string, t: (key: string) => string) {
  if (!attempt.started_at) return t('requestRecords.traceNotStarted');

  const start = formatRequestDate(attempt.started_at, locale);
  const end = attempt.finished_at ? formatRequestDate(attempt.finished_at, locale) : t('requestRecords.inProgress');
  return `${start} -> ${end}`;
}

export function requestTraceLabelColor(status: string) {
  if (status === 'success') return 'success';
  if (status === 'failed') return 'error';
  if (status === 'active' || status === 'pending' || status === 'streaming') return 'info';
  return 'default';
}

export function statusColor(status: TraceStatus) {
  return {
    success: '#22c55e',
    failed: '#ef4444',
    active: '#3b82f6',
    queued: '#94a3b8',
    unscheduled: '#d1d5db',
    notScheduled: '#9ca3af',
  }[status];
}

function appendAttempt(groups: TraceGroup[], attempt: TraceAttempt) {
  const id = attempt.provider_id || attempt.provider_name || attempt.id;
  const group = groups.find((item) => item.id === id);
  if (group) {
    group.attempts.push(attempt);
    group.status = higherStatus(group.status, attempt.traceStatus);
    return;
  }
  groups.push({ id, providerName: attempt.provider_name || id, status: attempt.traceStatus, attempts: [attempt] });
}

function preferredGroupIndex(groups: TraceGroup[]) {
  for (const status of ['success', 'active', 'failed'] as const) {
    const index = groups.findIndex((group) => group.status === status);
    if (index >= 0) return index;
  }
  return 0;
}

function traceStatus(candidate: RequestCandidateDetail, candidates: RequestCandidateDetail[], record: RequestRecord | null): TraceStatus {
  if (candidate.status === 'success' || successStatusCode(candidate.status_code)) return 'success';
  if (candidate.status === 'failed' || failedStatusCode(candidate.status_code)) return 'failed';
  if (candidate.status === 'streaming' || activeCandidate(candidate)) return 'active';
  if (candidate.status === 'pending' && !candidate.started_at) return 'queued';
  if (candidate.status === 'available') return availableStatus(candidates, record);
  return 'unscheduled';
}

function availableStatus(candidates: RequestCandidateDetail[], record: RequestRecord | null): TraceStatus {
  if (record?.status === 'success' || record?.status === 'failed') return 'notScheduled';
  if (candidates.some(activeCandidate)) return 'queued';
  return 'unscheduled';
}

function activeCandidate(candidate: RequestCandidateDetail) {
  return Boolean(candidate.started_at && !candidate.finished_at);
}

function successStatusCode(value?: number | null) {
  return value !== null && value !== undefined && value >= 200 && value < 300;
}

function failedStatusCode(value?: number | null) {
  return value !== null && value !== undefined && value >= 400;
}

function higherStatus(left: TraceStatus, right: TraceStatus) {
  return STATUS_ORDER[right] > STATUS_ORDER[left] ? right : left;
}
