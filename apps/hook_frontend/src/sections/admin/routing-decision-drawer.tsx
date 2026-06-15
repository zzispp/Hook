'use client';

import type { AdminT } from './shared';
import type {
  RoutingProfile,
  RoutingMetricWindow,
  RouteScoreExplanation,
  RoutingRankingResponse,
  RoutingDecisionResponse,
} from 'src/types/routing';

import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import LinearProgress from '@mui/material/LinearProgress';

import { fNumber } from 'src/utils/format-number';

import { Iconify } from 'src/components/iconify';

import { formatRouteApiFormat, routeNeedsConversion } from './routing-format-utils';
import {
  routeScoreReason,
  scoreComponentLabel,
  routingWindowDetail,
  routingProfileDescription,
  translatedExclusionReason,
} from './routing-i18n';

type Props = {
  open: boolean;
  item: RouteScoreExplanation | null;
  decision?: RoutingDecisionResponse | null;
  profile: RoutingProfile | null;
  windowSnapshots: Partial<Record<RoutingMetricWindow, RoutingRankingResponse>>;
  selectedWindow: RoutingMetricWindow;
  t: AdminT;
  onClose: VoidFunction;
};

export function RoutingDecisionDrawer({
  open,
  item,
  decision,
  profile,
  windowSnapshots,
  selectedWindow,
  t,
  onClose,
}: Props) {
  const active = item ?? decision?.candidates[0] ?? null;

  return (
    <Drawer
      anchor="right"
      open={open}
      PaperProps={{ sx: { width: { xs: 1, sm: 560 } } }}
      onClose={onClose}
    >
      <Stack spacing={2.5} sx={{ p: 3 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">{t('routing.drawer.title')}</Typography>
          <IconButton onClick={onClose}>
            <Iconify icon="mingcute:close-line" />
          </IconButton>
        </Stack>

        {decision ? <DecisionHeader decision={decision} t={t} /> : null}
        {active ? (
          <RouteDetail
            item={active}
            profile={profile}
            selectedWindow={selectedWindow}
            windowSnapshots={windowSnapshots}
            t={t}
          />
        ) : null}

        {decision ? (
          <>
            <Divider />
            <Stack spacing={1.5}>
              <Typography variant="subtitle2">{t('routing.drawer.candidates')}</Typography>
              {decision.candidates.map((candidate) => (
                <CandidateLine key={candidateKey(candidate)} item={candidate} />
              ))}
            </Stack>
          </>
        ) : null}
      </Stack>
    </Drawer>
  );
}

function DecisionHeader({ decision, t }: { decision: RoutingDecisionResponse; t: AdminT }) {
  return (
    <Stack spacing={0.75}>
      <Typography variant="body2" color="text.secondary">
        {t('routing.drawer.requestId')}: {decision.request_id}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        {decision.profile_id} · {decision.profile_version} · {decision.created_at}
      </Typography>
    </Stack>
  );
}

function RouteDetail({
  item,
  profile,
  selectedWindow,
  windowSnapshots,
  t,
}: {
  item: RouteScoreExplanation;
  profile: RoutingProfile | null;
  selectedWindow: RoutingMetricWindow;
  windowSnapshots: Partial<Record<RoutingMetricWindow, RoutingRankingResponse>>;
  t: AdminT;
}) {
  const formatLabel = formatRouteApiFormat(item.route);
  const isConversion = routeNeedsConversion(item.route);

  return (
    <Stack spacing={2}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle1">{item.provider_name || item.route.provider_id}</Typography>
        <Typography variant="body2" color="text.secondary">
          {item.key_name || item.route.key_id} · {item.endpoint_name || item.route.endpoint_id}
        </Typography>
        <Typography variant="caption" color={isConversion ? 'warning.main' : 'text.disabled'}>
          {formatLabel}
        </Typography>
        <Typography variant="body2">{routeScoreReason(item, t)}</Typography>
      </Stack>
      <StateBlock item={item} t={t} />
      <FormulaBlock profile={profile} t={t} />
      <WindowBlock
        item={item}
        selectedWindow={selectedWindow}
        windowSnapshots={windowSnapshots}
        t={t}
      />
      <ComponentBlock item={item} t={t} />
    </Stack>
  );
}

function StateBlock({ item, t }: { item: RouteScoreExplanation; t: AdminT }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="subtitle2">{t('routing.drawer.state')}</Typography>
      <Typography variant="body2" color="text.secondary">
        {`${t(`routing.states.${item.state}`)} · ${t('routing.drawer.sourceWindow')}: ${item.metric_window} · ${t('routing.drawer.freshness')}: ${item.metric_freshness_seconds}s`}
      </Typography>
      {item.exclusion_reason ? (
        <Typography variant="body2" color="error.main">
          {`${t('routing.drawer.exclusion')}: ${translatedExclusionReason(item.exclusion_reason, t)}`}
        </Typography>
      ) : null}
    </Stack>
  );
}

function FormulaBlock({ profile, t }: { profile: RoutingProfile | null; t: AdminT }) {
  if (!profile) return null;

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('routing.drawer.formula')}</Typography>
      <Typography variant="body2" color="text.secondary">
        {routingProfileDescription(profile, t)}
      </Typography>
      <WeightLine
        label={t('routing.summary.effectiveWeights')}
        weights={profile.learning?.effective_weights || profile.weights}
        t={t}
      />
      <WeightLine
        label={t('routing.summary.adminWeights')}
        weights={profile.learning?.admin_weights || profile.weights}
        t={t}
      />
      {profile.learning?.learned_weights ? (
        <WeightLine
          label={t('routing.summary.learnedWeights')}
          weights={profile.learning.learned_weights}
          t={t}
        />
      ) : null}
    </Stack>
  );
}

function WindowBlock({
  item,
  selectedWindow,
  windowSnapshots,
  t,
}: {
  item: RouteScoreExplanation;
  selectedWindow: RoutingMetricWindow;
  windowSnapshots: Partial<Record<RoutingMetricWindow, RoutingRankingResponse>>;
  t: AdminT;
}) {
  const windows = [selectedWindow, '1h', '24h', '7d'].filter(uniqueWindow);

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('routing.drawer.windowMetrics')}</Typography>
      {windows.map((window) => {
        const detail = findWindowItem(item, window as RoutingMetricWindow, windowSnapshots);

        return (
          <Stack key={window} spacing={0.25}>
            <Typography variant="body2">
              {window}
              {detail ? '' : ' · -'}
            </Typography>
            {detail ? (
              <Typography variant="caption" color="text.secondary">
                {routingWindowDetail(detail, detail.raw_metrics, t)}
              </Typography>
            ) : null}
          </Stack>
        );
      })}
    </Stack>
  );
}

function ComponentBlock({ item, t }: { item: RouteScoreExplanation; t: AdminT }) {
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('routing.drawer.components')}</Typography>
      {item.components.map((component) => (
        <Stack key={component.code} spacing={0.5}>
          <Stack direction="row" justifyContent="space-between">
            <Typography variant="body2">{scoreComponentLabel(component, t)}</Typography>
            <Typography variant="body2">{component.contribution.toFixed(2)}</Typography>
          </Stack>
          <LinearProgress
            variant="determinate"
            value={Math.min(Math.abs(component.contribution), 100)}
            color={component.contribution < 0 ? 'error' : 'primary'}
            sx={{ height: 5, borderRadius: 0.5 }}
          />
        </Stack>
      ))}
    </Stack>
  );
}

function CandidateLine({ item }: { item: RouteScoreExplanation }) {
  const formatLabel = formatRouteApiFormat(item.route);

  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
      <Stack sx={{ minWidth: 0 }} spacing={0.25}>
        <Typography variant="body2" noWrap>
          #{item.rank} {item.provider_name || item.route.provider_id} /{' '}
          {item.key_name || item.route.key_id}
        </Typography>
        <Typography variant="caption" color="text.secondary" noWrap>
          {formatLabel}
        </Typography>
      </Stack>
      <Typography variant="body2">
        {fNumber(item.final_score, { maximumFractionDigits: 1 })}
      </Typography>
    </Stack>
  );
}

function WeightLine({
  label,
  weights,
  t,
}: {
  label: string;
  weights: RoutingProfile['weights'];
  t: AdminT;
}) {
  return (
    <Typography variant="caption" color="text.secondary">
      {`${label}: ${t('routing.profile.weightFields.success')} ${(weights.success * 100).toFixed(1)}%, ${t('routing.profile.weightFields.ttfb')} ${(weights.ttfb * 100).toFixed(1)}%, ${t('routing.profile.weightFields.latency')} ${(weights.latency * 100).toFixed(1)}%, ${t('routing.profile.weightFields.tps')} ${(weights.tps * 100).toFixed(1)}%, ${t('routing.profile.weightFields.cost')} ${(weights.cost * 100).toFixed(1)}%, ${t('routing.profile.weightFields.headroom')} ${(weights.headroom * 100).toFixed(1)}%, ${t('routing.profile.weightFields.priority')} ${(weights.priority * 100).toFixed(1)}%`}
    </Typography>
  );
}

function findWindowItem(
  item: RouteScoreExplanation,
  window: RoutingMetricWindow,
  snapshots: Partial<Record<RoutingMetricWindow, RoutingRankingResponse>>
) {
  return snapshots[window]?.items.find(
    (candidate) => candidateKey(candidate) === candidateKey(item)
  );
}

function uniqueWindow(value: string, index: number, items: string[]) {
  return items.indexOf(value) === index;
}

function candidateKey(item: RouteScoreExplanation) {
  const route = item.route;
  return `${route.provider_id}:${route.key_id}:${route.endpoint_id}:${route.global_model_id}:${route.client_api_format}:${route.provider_api_format}:${route.is_stream}`;
}
