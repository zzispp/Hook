'use client';

import type { AdminT } from './shared';
import type { RoutingProfile, RoutingProfileWeights } from 'src/types/routing';

import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';

type Props = {
  profile: RoutingProfile | null;
  t: AdminT;
};

const WEIGHT_KEYS: Array<keyof RoutingProfileWeights> = [
  'success',
  'ttfb',
  'latency',
  'tps',
  'cost',
  'headroom',
  'priority',
];

export function RoutingProfileSummary({ profile, t }: Props) {
  if (!profile) return null;

  return (
    <Stack spacing={2}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle1">{profile.name}</Typography>
        <Typography variant="body2" color="text.secondary">
          {profile.description}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {t('routing.summary.liveWindow')}
        </Typography>
      </Stack>

      <Stack direction={{ xs: 'column', xl: 'row' }} spacing={2} divider={<Divider flexItem orientation="vertical" />}>
        <WeightColumn title={t('routing.summary.effectiveWeights')} weights={profile.learning?.effective_weights || profile.weights} />
        <WeightColumn title={t('routing.summary.adminWeights')} weights={profile.learning?.admin_weights || profile.weights} />
        <WeightColumn title={t('routing.summary.learnedWeights')} weights={profile.learning?.learned_weights || null} />
      </Stack>

      <Typography variant="caption" color="text.secondary">
        {profile.learning
          ? `${t('routing.summary.rewardWindow')}: ${profile.learning.reward_window} · ${t('routing.summary.samples')}: ${profile.learning.sample_count} · ${t('routing.summary.confidence')}: ${(profile.learning.confidence * 100).toFixed(0)}% · ${t('routing.summary.updatedAt')}: ${profile.learning.updated_at}`
          : t('routing.summary.autoTuneDisabled')}
      </Typography>
    </Stack>
  );
}

function WeightColumn({
  title,
  weights,
}: {
  title: string;
  weights: RoutingProfileWeights | null;
}) {
  return (
    <Stack spacing={1.25} sx={{ minWidth: 0, flex: 1 }}>
      <Typography variant="subtitle2">{title}</Typography>
      {weights ? (
        WEIGHT_KEYS.map((key) => (
          <Stack key={key} spacing={0.35}>
            <Stack direction="row" justifyContent="space-between" spacing={1}>
              <Typography variant="caption" color="text.secondary">
                {key}
              </Typography>
              <Typography variant="caption">{(weights[key] * 100).toFixed(1)}%</Typography>
            </Stack>
            <LinearProgress
              variant="determinate"
              value={Math.min(weights[key] * 100, 100)}
              sx={{ height: 5, borderRadius: 0.5 }}
            />
          </Stack>
        ))
      ) : (
        <Typography variant="body2" color="text.secondary">
          -
        </Typography>
      )}
    </Stack>
  );
}
