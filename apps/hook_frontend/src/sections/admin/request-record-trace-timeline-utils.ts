import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';
import type { RouteScoreExplanation, RoutingDecisionResponse } from 'src/types/routing';

import { formatRequestDate } from './request-records-utils';
import { formatApiFormat } from './provider-management-utils';

export type TraceStatus = 'success' | 'cancelled' | 'failed' | 'active' | 'queued' | 'unscheduled' | 'notScheduled';

export type TraceAttempt = RequestCandidateDetail & {
  traceStatus: TraceStatus;
  routingDecision?: RouteScoreExplanation | null;
};

export type TraceGroup = {
  id: string;
  providerName: string;
  status: TraceStatus;
  attempts: TraceAttempt[];
  totalAttemptCount: number;
  hiddenAttemptCount: number;
  endpointCount: number;
  keyCount: number;
};

const STATUS_ORDER: Record<TraceStatus, number> = {
  notScheduled: 0,
  unscheduled: 1,
  queued: 2,
  failed: 3,
  cancelled: 4,
  active: 5,
  success: 6,
};

export function buildTraceGroups(
  record: RequestRecord | null,
  candidates: RequestCandidateDetail[],
  routingDecision?: RoutingDecisionResponse | null
) {
  const attempts = candidates.map((candidate) => ({
    ...candidate,
    traceStatus: traceStatus(candidate, candidates, record),
    routingDecision: candidateRoutingDecision(record, candidate, routingDecision),
  }));
  return groupedAttempts(attempts).map(traceGroup);
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
  if (status === 'cancelled') return 'warning';
  if (status === 'failed') return 'error';
  if (status === 'active' || status === 'pending' || status === 'streaming') return 'info';
  return 'default';
}

export function statusColor(status: TraceStatus) {
  return {
    success: '#22c55e',
    cancelled: '#f59e0b',
    failed: '#ef4444',
    active: '#3b82f6',
    queued: '#94a3b8',
    unscheduled: '#d1d5db',
    notScheduled: '#9ca3af',
  }[status];
}

function groupedAttempts(attempts: TraceAttempt[]) {
  const groups = new Map<string, TraceAttempt[]>();
  for (const attempt of attempts) {
    const id = attemptGroupId(attempt);
    groups.set(id, [...(groups.get(id) ?? []), attempt]);
  }
  return [...groups.values()];
}

function traceGroup(attempts: TraceAttempt[]): TraceGroup {
  const visibleAttempts = attempts.filter(visibleTraceAttempt);
  const shownAttempts = visibleAttempts.length > 0 ? visibleAttempts : attempts.slice(0, 1);
  const first = attempts[0];
  const status = attempts.reduce((current, attempt) => higherStatus(current, attempt.traceStatus), 'notScheduled' as TraceStatus);
  return {
    id: attemptGroupId(first),
    providerName: first.provider_name || attemptGroupId(first),
    status,
    attempts: shownAttempts,
    totalAttemptCount: attempts.length,
    hiddenAttemptCount: attempts.length - shownAttempts.length,
    endpointCount: uniqueCount(attempts.map((attempt) => attempt.endpoint_id || attempt.provider_api_format || attempt.id)),
    keyCount: uniqueCount(attempts.map((attempt) => attempt.key_id || attempt.key_preview || attempt.id)),
  };
}

function attemptGroupId(attempt: TraceAttempt) {
  return attempt.provider_id || attempt.provider_name || attempt.id;
}

function visibleTraceAttempt(attempt: TraceAttempt) {
  return attempt.traceStatus !== 'notScheduled' && attempt.traceStatus !== 'unscheduled';
}

function preferredGroupIndex(groups: TraceGroup[]) {
  for (const status of ['success', 'active', 'cancelled', 'failed'] as const) {
    const index = groups.findIndex((group) => group.status === status);
    if (index >= 0) return index;
  }
  return 0;
}

function traceStatus(candidate: RequestCandidateDetail, candidates: RequestCandidateDetail[], record: RequestRecord | null): TraceStatus {
  if (candidate.status === 'success' || successStatusCode(candidate.status_code)) return 'success';
  if (candidate.status === 'cancelled' || cancelledStatusCode(candidate.status_code)) return 'cancelled';
  if (candidate.status === 'failed' || failedStatusCode(candidate.status_code)) return 'failed';
  if (candidate.status === 'streaming' || activeCandidate(candidate)) return 'active';
  if (candidate.status === 'pending' && !candidate.started_at) return 'queued';
  if (candidate.status === 'skipped') return 'notScheduled';
  if (candidate.status === 'scheduled') return scheduledStatus(candidates, record);
  return 'unscheduled';
}

function scheduledStatus(candidates: RequestCandidateDetail[], record: RequestRecord | null): TraceStatus {
  if (record?.status === 'success' || record?.status === 'failed' || record?.status === 'cancelled') return 'notScheduled';
  if (candidates.some(activeCandidate)) return 'queued';
  return 'unscheduled';
}

function activeCandidate(candidate: RequestCandidateDetail) {
  return Boolean(candidate.started_at && !candidate.finished_at);
}

function successStatusCode(value?: number | null) {
  return value !== null && value !== undefined && value >= 200 && value < 300;
}

function cancelledStatusCode(value?: number | null) {
  return value === 499;
}

function failedStatusCode(value?: number | null) {
  return value !== null && value !== undefined && value >= 400;
}

function higherStatus(left: TraceStatus, right: TraceStatus) {
  return STATUS_ORDER[right] > STATUS_ORDER[left] ? right : left;
}

function uniqueCount(values: string[]) {
  return new Set(values).size;
}

function candidateRoutingDecision(
  record: RequestRecord | null,
  candidate: RequestCandidateDetail,
  decision?: RoutingDecisionResponse | null
) {
  if (!record?.global_model_id || !decision) return null;
  return decision.candidates.find((item) => routeMatchesCandidate(record, candidate, item)) ?? null;
}

function routeMatchesCandidate(
  record: RequestRecord,
  candidate: RequestCandidateDetail,
  item: RouteScoreExplanation
) {
  const route = item.route;
  return (
    route.provider_id === candidate.provider_id &&
    route.key_id === candidate.key_id &&
    route.endpoint_id === candidate.endpoint_id &&
    route.global_model_id === record.global_model_id &&
    route.client_api_format === candidate.client_api_format &&
    route.provider_api_format === providerFormat(candidate) &&
    route.is_stream === candidate.is_stream
  );
}

function providerFormat(candidate: RequestCandidateDetail) {
  return candidate.provider_api_format ?? candidate.client_api_format;
}
