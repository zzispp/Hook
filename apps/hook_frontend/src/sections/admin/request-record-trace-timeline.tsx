'use client';

import type { Theme } from '@mui/material/styles';
import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';
import type { TraceGroup, TraceStatus, TraceAttempt } from './request-record-trace-timeline-utils';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { formatDuration } from './request-records-utils';
import {
  attemptKey,
  attemptTime,
  statusColor,
  attemptFormat,
  defaultSelection,
  buildTraceGroups,
  requestTraceLabelColor,
} from './request-record-trace-timeline-utils';

export function RequestRecordTraceTimeline({
  record,
  candidates,
  loading,
  locale,
}: {
  record: RequestRecord | null;
  candidates: RequestCandidateDetail[];
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
      </Stack>
      {attempt.error_message ? (
        <Typography variant="caption" color="error">
          {attempt.error_message}
        </Typography>
      ) : null}
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

const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

const trackSx = { gap: 0, px: 2, py: 4, overflowX: 'auto', overflowY: 'visible' };
const nodeWrapSx = { position: 'relative', minWidth: 128, px: 1 };
const nodeLabelSx = { position: 'absolute', bottom: 'calc(100% + 8px)', maxWidth: 90 };
const keyDotsSx = {
  position: 'absolute',
  top: 'calc(100% + 8px)',
  left: '50%',
  px: 0.5,
  py: 0.5,
  width: 'max-content',
  overflow: 'visible',
  transform: 'translateX(-50%)',
};
const hiddenSummarySx = {
  position: 'absolute',
  top: 'calc(100% + 28px)',
  left: '50%',
  color: 'text.secondary',
  transform: 'translateX(-50%)',
};
const lineSx = { width: 64, height: 2, bgcolor: 'divider' };
const detailSx = { p: 1.5, borderRadius: 1, bgcolor: 'background.neutral' };

function providerDotSx(status: TraceStatus, selected: boolean) {
  return {
    m: 0,
    width: 16,
    height: 16,
    p: 0,
    display: 'inline-flex',
    position: 'relative',
    alignItems: 'center',
    justifyContent: 'center',
    appearance: 'none',
    borderRadius: '50%',
    color: statusColor(status),
    border: '2px solid currentColor',
    bgcolor: 'transparent',
    cursor: 'pointer',
    overflow: 'visible',
    transform: selected ? 'scale(1.18)' : 'scale(1)',
    '&::before': dotBeforeSx(8),
  };
}

function keyDotSx(status: TraceStatus, selected: boolean) {
  return {
    width: 10,
    height: 10,
    p: 0,
    border: 0,
    borderRadius: '50%',
    bgcolor: statusColor(status),
    cursor: 'pointer',
    opacity: selected ? 1 : 0.55,
    boxShadow: selected ? `0 0 0 2px #fff, 0 0 0 3px ${statusColor(status)}` : 'none',
  };
}

function titleDotSx(status: TraceStatus) {
  return { width: 10, height: 10, borderRadius: '50%', bgcolor: statusColor(status), flexShrink: 0 };
}

function dotBeforeSx(size: number) {
  return {
    content: '""',
    position: 'absolute',
    top: '50%',
    left: '50%',
    width: size,
    height: size,
    borderRadius: '50%',
    bgcolor: 'currentColor',
    transform: 'translate(-50%, -50%)',
  };
}
