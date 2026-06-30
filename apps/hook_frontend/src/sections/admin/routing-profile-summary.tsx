'use client';

import type { AdminT } from './shared';
import type { RoutingProfile, RoutingProfileWeights } from 'src/types/routing';

import Grid from '@mui/material/Grid';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';
import TableContainer from '@mui/material/TableContainer';

import { Iconify } from 'src/components/iconify';

import { routingProfileName, routingProfileDescription } from './routing-i18n';

type Props = {
  profile: RoutingProfile | null;
  t: AdminT;
};

const WEIGHT_KEYS: Array<keyof RoutingProfileWeights> = [
  'success',
  'first_token',
  'latency',
  'tps',
  'cost',
  'headroom',
  'priority',
];

const PARAM_KEYS = [
  'min_samples',
  'exploration_k',
  'prior_sample_cap',
  'conversion_penalty',
  'stale_metric_penalty',
  'affinity_bonus',
  'contextual_exploration_enabled',
  'ema_alpha',
  'ema_max_freshness_seconds',
  'ema_recent_weight',
  'ema_recent_cap',
  'exploration_weight',
  'exploration_cap',
  'exploration_min_success_score',
] as const;

export function RoutingProfileSummary({ profile, t }: Props) {
  if (!profile) return null;

  const adminWeights = profile.learning?.admin_weights || profile.weights;
  const learnedWeights = profile.learning?.learned_weights || null;
  const effectiveWeights = profile.learning?.effective_weights || profile.weights;

  return (
    <Stack spacing={2.5}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle2" color="primary.main">
          {routingProfileName(profile, t)}
        </Typography>
        <Typography variant="body2" color="text.secondary">
          {routingProfileDescription(profile, t)}
        </Typography>
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mt: 0.5 }}
        >
          <Iconify icon="solar:info-circle-bold" width={14} />
          {t('routing.summary.liveWindow')}
        </Typography>
      </Stack>

      <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1.5, overflow: 'hidden' }}>
        <Table size="small">
          <TableHead sx={{ backgroundColor: 'background.neutral' }}>
            <TableRow>
              <TableCell sx={{ py: 1 }}>{t('routing.summary.tableMetric')}</TableCell>
              <TableCell align="right" sx={{ py: 1 }}>{t('routing.summary.tableBaseline')}</TableCell>
              <TableCell align="right" sx={{ py: 1 }}>{t('routing.summary.tableLearned')}</TableCell>
              <TableCell align="right" sx={{ py: 1 }}>{t('routing.summary.tableEffective')}</TableCell>
              <TableCell sx={{ py: 1, width: 70 }} />
            </TableRow>
          </TableHead>
          <TableBody>
            {WEIGHT_KEYS.map((key) => {
              const adminVal = adminWeights[key];
              const learnedVal = learnedWeights ? learnedWeights[key] : null;
              const effectiveVal = effectiveWeights[key];

              return (
                <TableRow key={key} hover sx={{ '&:last-child td, &:last-child th': { border: 0 } }}>
                  <TableCell sx={{ py: 0.75, fontWeight: 'medium', fontSize: '0.8rem' }}>
                    {t(`routing.profile.weightFields.${key}`) || key}
                  </TableCell>
                  <TableCell align="right" sx={{ py: 0.75, fontSize: '0.8rem' }}>
                    {(adminVal * 100).toFixed(1)}%
                  </TableCell>
                  <TableCell
                    align="right"
                    sx={{
                      py: 0.75,
                      fontSize: '0.8rem',
                      color: learnedVal !== null ? 'info.main' : 'text.disabled',
                    }}
                  >
                    {learnedVal !== null ? `${(learnedVal * 100).toFixed(1)}%` : '-'}
                  </TableCell>
                  <TableCell
                    align="right"
                    sx={{ py: 0.75, fontWeight: 'bold', fontSize: '0.8rem', color: 'primary.main' }}
                  >
                    {(effectiveVal * 100).toFixed(1)}%
                  </TableCell>
                  <TableCell sx={{ py: 0.75, verticalAlign: 'middle' }}>
                    <LinearProgress
                      variant="determinate"
                      value={Math.min(effectiveVal * 100, 100)}
                      color="primary"
                      sx={{ height: 6, borderRadius: 1 }}
                    />
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </TableContainer>

      <Grid container spacing={1}>
        {PARAM_KEYS.map((key) => (
          <Grid size={{ xs: 6 }} key={key}>
            <Typography variant="caption" color="text.secondary" display="block">
                {t(`routing.profile.${key === 'contextual_exploration_enabled' ? 'booleanFields' : 'paramFields'}.${key}`)}:{' '}
              <Typography component="span" variant="caption" fontWeight="bold">
                {formatProfileParam(profile, key, t)}
              </Typography>
            </Typography>
          </Grid>
        ))}
      </Grid>

      <Stack
        sx={{
          p: 1.5,
          borderRadius: 1,
          backgroundColor: 'background.neutral',
          border: 1,
          borderColor: 'divider',
        }}
        spacing={1}
      >
        <Typography
          variant="subtitle2"
          sx={{ display: 'flex', alignItems: 'center', gap: 0.75, fontSize: '0.75rem' }}
        >
          <Iconify
            icon="solar:settings-bold"
            width={16}
            color={profile.learning ? 'warning.main' : 'text.disabled'}
          />
          {t('routing.summary.autoTuneStatus')}:{' '}
          {profile.learning
            ? t('routing.summary.autoTuneActive')
            : t('routing.summary.autoTuneInactive')}
        </Typography>

        {profile.learning ? (
          <Grid container spacing={1}>
            <Grid size={{ xs: 6 }}>
              <Typography variant="caption" color="text.secondary" display="block">
                {t('routing.summary.rewardWindow')}:{' '}
                <Typography component="span" variant="caption" fontWeight="bold">
                  {profile.learning.reward_window}
                </Typography>
              </Typography>
            </Grid>
            <Grid size={{ xs: 6 }}>
              <Typography variant="caption" color="text.secondary" display="block">
                {t('routing.summary.samples')}:{' '}
                <Typography component="span" variant="caption" fontWeight="bold">
                  {profile.learning.sample_count}
                </Typography>
              </Typography>
            </Grid>
            <Grid size={{ xs: 6 }}>
              <Typography variant="caption" color="text.secondary" display="block">
                {t('routing.summary.confidence')}:{' '}
                <Typography component="span" variant="caption" fontWeight="bold">
                  {(profile.learning.confidence * 100).toFixed(0)}%
                </Typography>
              </Typography>
            </Grid>
            <Grid size={{ xs: 6 }}>
              <Typography variant="caption" color="text.secondary" display="block" noWrap>
                {t('routing.summary.updatedAt')}:{' '}
                <Typography component="span" variant="caption" fontWeight="bold" sx={{ fontSize: '0.7rem' }}>
                  {profile.learning.updated_at.split(' ')[0]}
                </Typography>
              </Typography>
            </Grid>
          </Grid>
        ) : (
          <Typography variant="caption" color="text.secondary">
            {t('routing.summary.autoTuneDisabled')}
          </Typography>
        )}
      </Stack>
    </Stack>
  );
}

function formatProfileParam(profile: RoutingProfile, key: (typeof PARAM_KEYS)[number], t: AdminT) {
  if (key === 'contextual_exploration_enabled') {
    return profile.contextual_exploration_enabled
      ? t('routing.profile.booleanValues.enabled')
      : t('routing.profile.booleanValues.disabled');
  }
  return String(profile[key]);
}
