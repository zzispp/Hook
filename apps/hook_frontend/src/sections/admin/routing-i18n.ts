import type { AdminT } from './shared';
import type {
  ScoreComponent,
  RoutingProfile,
  RoutingMetricSnapshot,
  RouteScoreExplanation,
} from 'src/types/routing';

export function routingProfileName(profile: RoutingProfile, t: AdminT) {
  return t(`routing.profile.names.${profile.id}`);
}

export function routingProfileDescription(profile: RoutingProfile, t: AdminT) {
  return t(`routing.profile.descriptions.${profile.id}`);
}

export function routeScoreReason(item: RouteScoreExplanation, t: AdminT) {
  if (item.components.length === 0) {
    return translatedExclusionReason(item.selected_reason, t);
  }
  const components = item.components
    .filter((component) => Math.abs(component.contribution) > 0.01)
    .map((component) => `${scoreComponentLabel(component, t)} ${signedScore(component.contribution)}`)
    .join(', ');

  return t('routing.scoreReason', {
    score: item.final_score.toFixed(1),
    components,
  });
}

export function scoreComponentLabel(component: ScoreComponent, t: AdminT) {
  return t(`routing.scoreComponents.${component.code}`);
}

export function routingMetricSummary(
  window: string,
  metrics: RoutingMetricSnapshot,
  t: AdminT
) {
  return t('routing.metrics.summary', {
    window,
    success: successRate(metrics),
    ttfb: formatMs(metrics.ttfb_avg_ms),
    latency: formatMs(metrics.latency_avg_ms),
    tps: formatNumber(metrics.output_tps, 1),
    rpm: `${metrics.rpm_used}/${metrics.rpm_limit ?? t('routing.metrics.unlimited')}`,
    samples: metrics.sample_count,
  });
}

export function routingWindowDetail(
  item: RouteScoreExplanation,
  metrics: RoutingMetricSnapshot,
  t: AdminT
) {
  return t('routing.metrics.windowDetail', {
    state: t(`routing.states.${item.state}`),
    source: item.metric_window,
    success: successRate(metrics),
    ttfb: formatMs(metrics.ttfb_avg_ms),
    latency: formatMs(metrics.latency_avg_ms),
    tps: formatNumber(metrics.output_tps, 2),
    samples: metrics.sample_count,
  });
}

export function translatedExclusionReason(reason: string, t: AdminT) {
  const normalized = reason.split(';')[0]?.trim() ?? reason;
  if (normalized === 'provider_key_rate_limit_exhausted') {
    return reason.replace(normalized, t('routing.exclusionReasons.provider_key_rate_limit_exhausted'));
  }
  return reason;
}

function successRate(metrics: RoutingMetricSnapshot) {
  if (!metrics.request_count) {
    return '0.0';
  }
  return ((metrics.success_count / metrics.request_count) * 100).toFixed(1);
}

function signedScore(value: number) {
  return `${value >= 0 ? '+' : ''}${value.toFixed(1)}`;
}

function formatMs(value?: number | null) {
  return value == null ? '-' : `${value.toFixed(0)}ms`;
}

function formatNumber(value?: number | null, digits = 1) {
  return value == null ? '-' : value.toFixed(digits);
}
