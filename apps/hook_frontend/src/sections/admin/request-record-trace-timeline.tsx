'use client';

import type { AdminT } from './shared';
import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';
import type { TraceGroup, TraceAttempt } from './request-record-trace-timeline-utils';
import type { RouteIdentity, RouteScoreExplanation, RoutingDecisionResponse } from 'src/types/routing';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { routeScoreReason } from './routing-i18n';
import { formatDuration } from './request-records-utils';
import { RequestCandidatePayloadPanels } from './request-record-payload-panels';
import {
  attemptKey,
  attemptTime,
  attemptFormat,
  defaultSelection,
  buildTraceGroups,
  requestTraceLabelColor,
} from './request-record-trace-timeline-utils';
import {
  lineSx,
  panelSx,
  trackSx,
  detailSx,
  keyDotSx,
  keyDotsSx,
  nodeWrapSx,
  titleDotSx,
  nodeLabelSx,
  providerDotSx,
  hiddenSummarySx,
} from './request-record-trace-timeline-styles';

export function RequestRecordTraceTimeline({
  record,
  candidates,
  routingDecision,
  routingDecisionError,
  routingDecisionLoading,
  loading,
  locale,
}: {
  record: RequestRecord | null;
  candidates: RequestCandidateDetail[];
  routingDecision: RoutingDecisionResponse | null;
  routingDecisionError?: Error;
  routingDecisionLoading: boolean;
  loading: boolean;
  locale: string;
}) {
  const { t } = useTranslate('admin');
  const groups = useMemo(() => buildTraceGroups(record, candidates), [record, candidates]);
  const [selection, setSelection] = useState({ groupIndex: 0, attemptIndex: 0 });
  const selectedGroup = groups[selection.groupIndex] ?? null;
  const selectedAttempt = selectedGroup?.attempts[selection.attemptIndex] ?? null;

  useEffect(() => {
    setSelection(defaultSelection(groups));
  }, [groups]);

  return (
    <Stack spacing={1.5} sx={panelSx}>
      <TraceHeader record={record} groups={groups} t={t} />
      {loading ? <Typography variant="body2">{t('common.loading')}</Typography> : null}
      <RoutingSelectionSummary
        decision={routingDecision}
        error={routingDecisionError}
        loading={routingDecisionLoading}
        t={t}
      />
      {!loading && groups.length === 0 ? <Typography variant="body2">{t('common.noData')}</Typography> : null}
      {groups.length > 0 ? (
        <>
          <TraceTrack groups={groups} selection={selection} onSelect={setSelection} />
          <TraceAttemptDetail attempt={selectedAttempt} group={selectedGroup} locale={locale} t={t} />
        </>
      ) : null}
    </Stack>
  );
}

function RoutingSelectionSummary({
  decision,
  error,
  loading,
  t,
}: {
  decision: RoutingDecisionResponse | null;
  error?: Error;
  loading: boolean;
  t: AdminT;
}) {
  if (loading) {
    return <Typography variant="body2">{t('common.loading')}</Typography>;
  }
  if (error) {
    return (
      <Typography variant="caption" color="error">
        {t('requestRecords.routingDecisionUnavailable')}: {error.message}
      </Typography>
    );
  }

  const selected = decision ? selectedCandidate(decision) : null;
  if (!decision || !selected) return null;

  return (
    <Stack spacing={0.75} sx={{ p: 1.5, borderRadius: 1, bgcolor: 'background.neutral' }}>
      <Stack direction="row" alignItems="center" spacing={1} useFlexGap flexWrap="wrap">
        <Typography variant="subtitle2">{t('requestRecords.routingDecisionTitle')}</Typography>
        <Label color="info" variant="soft">
          {t(`routing.profile.names.${decision.profile_id}`)}
        </Label>
        <Label color="default" variant="soft">
          {selected.final_score.toFixed(1)}
        </Label>
      </Stack>
      <Typography variant="body2" sx={{ fontWeight: 'medium' }}>
        {selectedRouteLabel(selected)}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {routeScoreReason(selected, t)}
      </Typography>
      <Typography variant="caption" color="text.disabled">
        {decision.profile_version} · {decision.created_at}
      </Typography>
    </Stack>
  );
}

function TraceHeader({
  record,
  groups,
  t,
}: {
  record: RequestRecord | null;
  groups: TraceGroup[];
  t: (key: string) => string;
}) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
      <Stack direction="row" alignItems="center" spacing={1}>
        <Typography variant="subtitle2">{t('requestRecords.traceTitle')}</Typography>
        {record ? <Label color={requestTraceLabelColor(record.status)}>{t(`requestRecords.status.${record.status}`)}</Label> : null}
      </Stack>
      <Typography variant="caption" color="text.secondary">
        {groups.length} {t('requestRecords.traceProviders')}
      </Typography>
    </Stack>
  );
}

function TraceTrack({
  groups,
  selection,
  onSelect,
}: {
  groups: TraceGroup[];
  selection: { groupIndex: number; attemptIndex: number };
  onSelect: (selection: { groupIndex: number; attemptIndex: number }) => void;
}) {
  return (
    <Stack direction="row" alignItems="center" sx={trackSx}>
      {groups.map((group, groupIndex) => (
        <TraceGroupNode
          key={group.id}
          group={group}
          groupIndex={groupIndex}
          selected={selection.groupIndex === groupIndex}
          selectedAttemptIndex={selection.groupIndex === groupIndex ? selection.attemptIndex : -1}
          showLine={groupIndex < groups.length - 1}
          onSelect={onSelect}
        />
      ))}
    </Stack>
  );
}

function TraceGroupNode({
  group,
  groupIndex,
  selected,
  selectedAttemptIndex,
  showLine,
  onSelect,
}: {
  group: TraceGroup;
  groupIndex: number;
  selected: boolean;
  selectedAttemptIndex: number;
  showLine: boolean;
  onSelect: (selection: { groupIndex: number; attemptIndex: number }) => void;
}) {
  return (
    <Stack direction="row" alignItems="center" sx={{ flexShrink: 0 }}>
      <Stack alignItems="center" sx={nodeWrapSx}>
        <Typography variant="caption" noWrap sx={nodeLabelSx}>
          {group.providerName}
        </Typography>
        <Box component="button" sx={providerDotSx(group.status, selected)} onClick={() => onSelect({ groupIndex, attemptIndex: 0 })} />
        {selected && group.attempts.length > 1 ? (
          <Stack direction="row" spacing={0.75} sx={keyDotsSx}>
            {group.attempts.map((attempt, attemptIndex) => (
              <Box
                key={attempt.id}
                component="button"
                title={attempt.key_name || attempt.key_preview || attempt.id}
                sx={keyDotSx(attempt.traceStatus, selectedAttemptIndex === attemptIndex)}
                onClick={() => onSelect({ groupIndex, attemptIndex })}
              />
            ))}
          </Stack>
        ) : null}
        {selected && group.hiddenAttemptCount > 0 ? (
          <Typography variant="caption" sx={hiddenSummarySx}>
            +{group.hiddenAttemptCount}
          </Typography>
        ) : null}
      </Stack>
      {showLine ? <Box sx={lineSx} /> : null}
    </Stack>
  );
}

function TraceAttemptDetail({
  attempt,
  group,
  locale,
  t,
}: {
  attempt: TraceAttempt | null;
  group: TraceGroup | null;
  locale: string;
  t: (key: string) => string;
}) {
  if (!attempt || !group) return null;
  const statusLabel = t(`requestRecords.traceStatus.${attempt.traceStatus}`);

  return (
    <Stack spacing={1.25} sx={detailSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Box sx={titleDotSx(attempt.traceStatus)} />
          <Typography variant="subtitle2">{group.providerName}</Typography>
          <Label color={requestTraceLabelColor(attempt.traceStatus)}>{attempt.status_code ?? statusLabel}</Label>
        </Stack>
        <Typography variant="caption" color="text.secondary">
          {attempt.candidate_index + 1} / {attempt.retry_index + 1}
          {group.hiddenAttemptCount > 0 ? ` (+${group.hiddenAttemptCount})` : ''}
        </Typography>
      </Stack>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        <TraceInfo label={t('requestRecords.traceTimeRange')} value={attemptTime(attempt, locale, t)} />
        <TraceInfo label={t('requestRecords.apiFormat')} value={attemptFormat(attempt)} />
        <TraceInfo label={t('requestRecords.traceKey')} value={attemptKey(attempt)} />
        <TraceInfo label={t('requestRecords.totalLatency')} value={formatDuration(attempt.latency_ms)} />
        {attempt.skip_reason ? (
          <TraceInfo label={t('requestRecords.traceSkipReason')} value={skipReasonLabel(t, attempt.skip_reason)} />
        ) : null}
        {attempt.error_code ? <TraceInfo label={t('requestRecords.traceErrorCode')} value={attempt.error_code} /> : null}
        {attempt.error_param ? <TraceInfo label={t('requestRecords.traceErrorParam')} value={attempt.error_param} /> : null}
      </Stack>
      {attempt.error_message ? (
        <Typography variant="caption" color="error">
          {attempt.error_message}
        </Typography>
      ) : null}
      <RequestCandidatePayloadPanels
        providerRequestHeaders={attempt.provider_request_headers}
        providerRequestBody={attempt.provider_request_body}
        providerResponseHeaders={attempt.provider_response_headers}
        providerResponseBody={attempt.provider_response_body}
      />
    </Stack>
  );
}

function TraceInfo({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.25} sx={{ minWidth: 140 }}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function selectedCandidate(decision: RoutingDecisionResponse) {
  if (!decision.selected) return decision.candidates[0] ?? null;
  return (
    decision.candidates.find((candidate) => routeKey(candidate.route) === routeKey(decision.selected)) ??
    decision.candidates[0] ??
    null
  );
}

function selectedRouteLabel(item: RouteScoreExplanation) {
  const provider = item.provider_name || item.route.provider_id;
  const key = item.key_name || item.route.key_id;
  const endpoint = item.endpoint_name || item.route.endpoint_id;
  return `${provider} / ${key} / ${endpoint}`;
}

function routeKey(route: RouteIdentity) {
  return `${route.provider_id}:${route.key_id}:${route.endpoint_id}:${route.global_model_id}:${route.client_api_format}:${route.provider_api_format}:${route.is_stream}`;
}

function skipReasonLabel(t: (key: string) => string, reason: string) {
  return t(`requestRecords.traceSkipReasonLabels.${reason}`);
}
