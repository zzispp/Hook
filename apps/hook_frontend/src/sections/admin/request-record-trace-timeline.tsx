'use client';

import type { AdminT } from './shared';
import type { RoutingDecisionResponse } from 'src/types/routing';
import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';
import type { TraceGroup, TraceAttempt } from './request-record-trace-timeline-utils';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { formatAttemptTiming } from './request-record-timing';
import { RoutingDecisionBlock } from './request-record-trace-routing-decision';
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
  loading,
  locale,
}: {
  record: RequestRecord | null;
  candidates: RequestCandidateDetail[];
  routingDecision?: RoutingDecisionResponse | null;
  loading: boolean;
  locale: string;
}) {
  const { t } = useTranslate('admin');
  const groups = useMemo(
    () => buildTraceGroups(record, candidates, routingDecision),
    [record, candidates, routingDecision]
  );
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
      {!loading && groups.length === 0 ? (
        <Typography variant="body2">{t('common.noData')}</Typography>
      ) : null}
      {groups.length > 0 ? (
        <>
          <TraceTrack groups={groups} selection={selection} onSelect={setSelection} />
          <TraceAttemptDetail
            attempt={selectedAttempt}
            group={selectedGroup}
            routingDecision={routingDecision}
            locale={locale}
            t={t}
          />
        </>
      ) : null}
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
  t: AdminT;
}) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
      <Stack direction="row" alignItems="center" spacing={1}>
        <Typography variant="subtitle2">{t('requestRecords.traceTitle')}</Typography>
        {record ? (
          <Label color={requestTraceLabelColor(record.status)}>
            {t(`requestRecords.status.${record.status}`)}
          </Label>
        ) : null}
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
        <Box
          component="button"
          sx={providerDotSx(group.status, selected)}
          onClick={() => onSelect({ groupIndex, attemptIndex: 0 })}
        />
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
  routingDecision,
  locale,
  t,
}: {
  attempt: TraceAttempt | null;
  group: TraceGroup | null;
  routingDecision?: RoutingDecisionResponse | null;
  locale: string;
  t: AdminT;
}) {
  if (!attempt || !group) return null;
  const statusLabel = t(`requestRecords.traceStatus.${attempt.traceStatus}`);

  return (
    <Stack spacing={1.25} sx={detailSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Box sx={titleDotSx(attempt.traceStatus)} />
          <Typography variant="subtitle2">{group.providerName}</Typography>
          <Label color={requestTraceLabelColor(attempt.traceStatus)}>
            {attempt.status_code ?? statusLabel}
          </Label>
        </Stack>
        <Typography variant="caption" color="text.secondary">
          {attempt.candidate_index + 1} / {attempt.retry_index + 1}
          {group.hiddenAttemptCount > 0 ? ` (+${group.hiddenAttemptCount})` : ''}
        </Typography>
      </Stack>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        <TraceInfo
          label={t('requestRecords.traceTimeRange')}
          value={attemptTime(attempt, locale, t)}
        />
        <TraceInfo label={t('requestRecords.apiFormat')} value={attemptFormat(attempt)} />
        <TraceInfo label={t('requestRecords.traceEndpoint')} value={attemptEndpoint(attempt)} />
        <TraceInfo label={t('requestRecords.traceKey')} value={attemptKey(attempt)} />
        <TraceInfo
          label={t('requestRecords.responseHeaders')}
          value={formatAttemptTiming(attempt, 'response_headers')}
        />
        <TraceInfo
          label={t('requestRecords.firstChar')}
          value={formatAttemptTiming(attempt, 'first_sse_event')}
        />
        <TraceInfo
          label={t('requestRecords.firstToken')}
          value={formatAttemptTiming(attempt, 'first_output')}
        />
        <TraceInfo
          label={t('requestRecords.totalLatency')}
          value={formatAttemptTiming(attempt, 'total_latency')}
        />
        {attempt.skip_reason ? (
          <TraceInfo
            label={t('requestRecords.traceSkipReason')}
            value={skipReasonLabel(t, attempt.skip_reason)}
          />
        ) : null}
        {attempt.error_code ? (
          <TraceInfo label={t('requestRecords.traceErrorCode')} value={attempt.error_code} />
        ) : null}
        {attempt.error_param ? (
          <TraceInfo label={t('requestRecords.traceErrorParam')} value={attempt.error_param} />
        ) : null}
      </Stack>
      {attempt.error_message ? (
        <Typography variant="caption" color="error">
          {attempt.error_message}
        </Typography>
      ) : null}
      <RoutingDecisionBlock attempt={attempt} routingDecision={routingDecision} t={t} />
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

function skipReasonLabel(t: AdminT, reason: string) {
  return t(`requestRecords.traceSkipReasonLabels.${reason}`);
}

function attemptEndpoint(attempt: TraceAttempt) {
  return attempt.endpoint_name || attempt.endpoint_id || '-';
}
