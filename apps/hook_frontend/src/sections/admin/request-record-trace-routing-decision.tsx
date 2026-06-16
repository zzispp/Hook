'use client';

import type { AdminT } from './shared';
import type { TraceAttempt } from './request-record-trace-timeline-utils';
import type {
  ScoreComponent,
  RouteScoreExplanation,
  RoutingDecisionResponse,
} from 'src/types/routing';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { routeScoreReason, scoreComponentLabel, translatedExclusionReason } from './routing-i18n';

export function RoutingDecisionBlock({
  attempt,
  routingDecision,
  t,
}: {
  attempt: TraceAttempt;
  routingDecision?: RoutingDecisionResponse | null;
  t: AdminT;
}) {
  const item = attempt.routingDecision;

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('requestRecords.traceRoutingDecision')}</Typography>
      {!routingDecision || !item ? (
        <Typography variant="caption" color="text.secondary">
          {t('requestRecords.traceRoutingDecisionMissing')}
        </Typography>
      ) : (
        <RoutingDecisionContent attempt={attempt} item={item} decision={routingDecision} t={t} />
      )}
    </Stack>
  );
}

function RoutingDecisionContent({
  attempt,
  item,
  decision,
  t,
}: {
  attempt: TraceAttempt;
  item: RouteScoreExplanation;
  decision: RoutingDecisionResponse;
  t: AdminT;
}) {
  const components = item.components.filter(visibleComponent).slice(0, 6);

  return (
    <Stack spacing={1}>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        <RoutingInfo
          label={t('requestRecords.traceRoutingProfile')}
          value={`${decision.profile_id} / ${decision.profile_version}`}
        />
        <RoutingInfo
          label={t('requestRecords.traceRoutingState')}
          value={routingStateText(item, t)}
        />
        <RoutingInfo
          label={t('requestRecords.traceRoutingSourceWindow')}
          value={`${item.metric_window} / ${item.metric_freshness_seconds}s`}
        />
        <RoutingInfo
          label={t('requestRecords.traceCacheAffinity')}
          value={cacheAffinityText(attempt, item, t)}
        />
      </Stack>
      <RoutingLongInfo
        label={t('requestRecords.traceRoutingReason')}
        value={routeScoreReason(item, t)}
      />
      {item.exclusion_reason ? (
        <Typography variant="caption" color="error">
          {translatedExclusionReason(item.exclusion_reason, t)}
        </Typography>
      ) : null}
      {components.length > 0 ? <ScoreComponents components={components} t={t} /> : null}
    </Stack>
  );
}

function ScoreComponents({ components, t }: { components: ScoreComponent[]; t: AdminT }) {
  return (
    <Stack spacing={0.5}>
      {components.map((component) => (
        <Stack key={component.code} direction="row" justifyContent="space-between" spacing={1}>
          <Typography variant="caption" color="text.secondary">
            {scoreComponentLabel(component, t)}
          </Typography>
          <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
            {signedScore(component.contribution)}
          </Typography>
        </Stack>
      ))}
    </Stack>
  );
}

function RoutingInfo({ label, value }: { label: string; value: string }) {
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

function RoutingLongInfo({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.25}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="caption">{value}</Typography>
    </Stack>
  );
}

function routingStateText(item: RouteScoreExplanation, t: AdminT) {
  return `#${item.rank} / ${item.final_score.toFixed(1)} / ${t(`routing.states.${item.state}`)}`;
}

function cacheAffinityText(attempt: TraceAttempt, item: RouteScoreExplanation, t: AdminT) {
  const component = item.components.find((score) => score.code === 'affinity');
  if (attempt.is_cached) return t('requestRecords.traceCacheAffinityHit');
  if (component && component.contribution > 0) {
    return t('requestRecords.traceCacheAffinityBonus', {
      score: signedScore(component.contribution),
    });
  }
  return t('requestRecords.traceCacheAffinityMiss');
}

function visibleComponent(component: ScoreComponent) {
  return Math.abs(component.contribution) > 0.01;
}

function signedScore(value: number) {
  return `${value >= 0 ? '+' : ''}${value.toFixed(1)}`;
}
