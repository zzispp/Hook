'use client';

import type {
  RoutingProfile,
  RoutingProfileWeights,
  RoutingCacheAffinityMode,
} from 'src/types/routing';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';
import { updateRoutingProfile } from 'src/actions/routing';

import { toast } from 'src/components/snackbar';

const WEIGHT_FIELDS: Array<keyof RoutingProfileWeights> = [
  'success',
  'ttfb',
  'latency',
  'tps',
  'cost',
  'headroom',
  'priority',
];
const CACHE_AFFINITY_MODES: RoutingCacheAffinityMode[] = [
  'disabled',
  'score_bonus',
  'prefer_cached',
];

type Props = {
  profile: RoutingProfile | null;
  onSaved: VoidFunction;
};

export function RoutingProfileEditor({ profile, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [autoTuneEnabled, setAutoTuneEnabled] = useState(false);
  const [weights, setWeights] = useState<Record<keyof RoutingProfileWeights, string>>(emptyWeights);
  const [explorationBudgetPercent, setExplorationBudgetPercent] = useState('10');
  const [emaRegressionPenalty, setEmaRegressionPenalty] = useState('6');
  const [cacheAffinityMode, setCacheAffinityMode] =
    useState<RoutingCacheAffinityMode>('score_bonus');
  const [affinityBonus, setAffinityBonus] = useState('2');
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!profile) {
      setAutoTuneEnabled(false);
      setWeights(emptyWeights);
      setExplorationBudgetPercent('10');
      setEmaRegressionPenalty('6');
      setCacheAffinityMode('score_bonus');
      setAffinityBonus('2');
      return;
    }
    setAutoTuneEnabled(profile.auto_tune_enabled);
    setWeights(toWeightForm(profile.learning?.admin_weights || profile.weights));
    setExplorationBudgetPercent(String(profile.exploration_budget_percent));
    setEmaRegressionPenalty(String(profile.ema_regression_penalty));
    setCacheAffinityMode(profile.cache_affinity_mode);
    setAffinityBonus(String(profile.affinity_bonus));
  }, [profile]);

  const totalWeight = useMemo(
    () => WEIGHT_FIELDS.reduce((sum, key) => sum + parseWeight(weights[key]), 0),
    [weights]
  );
  const invalidTotal = Math.abs(totalWeight - 1) > 0.001;
  const invalidExplorationBudget = !isPercent(explorationBudgetPercent);
  const invalidEmaRegressionPenalty = !isPercent(emaRegressionPenalty);
  const invalidAffinityBonus = !isNonNegativeNumber(affinityBonus);
  const saveDisabled =
    !profile ||
    submitting ||
    invalidTotal ||
    invalidExplorationBudget ||
    invalidEmaRegressionPenalty ||
    invalidAffinityBonus;
  const priorityLocked = !profile || profile.id !== 'fixed_priority_plus';

  const save = useCallback(async () => {
    if (!profile) return;
    setSubmitting(true);
    try {
      await updateRoutingProfile(profile.id, {
        auto_tune_enabled: autoTuneEnabled,
        weights: fromWeightForm(weights),
        exploration_budget_percent: parseNumber(explorationBudgetPercent),
        ema_regression_penalty: parseNumber(emaRegressionPenalty),
        cache_affinity_mode: cacheAffinityMode,
        affinity_bonus: parseNumber(affinityBonus),
      });
      toast.success(t('messages.routingProfileUpdated'));
      onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [
    affinityBonus,
    autoTuneEnabled,
    cacheAffinityMode,
    emaRegressionPenalty,
    explorationBudgetPercent,
    onSaved,
    profile,
    t,
    weights,
  ]);

  if (!profile) {
    return null;
  }

  return (
    <Stack spacing={2.5}>
      <Divider />
      <Stack spacing={0.5}>
        <Typography variant="subtitle2">{t('routing.profile.editorTitle')}</Typography>
        <Typography variant="caption" color="text.secondary">
          {t('routing.profile.editorHelper')}
        </Typography>
      </Stack>

      <FormControlLabel
        control={
          <Switch
            size="small"
            checked={autoTuneEnabled}
            onChange={(event) => setAutoTuneEnabled(event.target.checked)}
          />
        }
        label={
          <Typography variant="body2" sx={{ fontWeight: 'medium' }}>
            {t('routing.profile.autoTune')}
          </Typography>
        }
      />

      <Grid container spacing={1.5}>
        {WEIGHT_FIELDS.map((field) => (
          <Grid size={{ xs: 6 }} key={field}>
            <TextField
              fullWidth
              size="small"
              type="number"
              label={t(`routing.profile.weightFields.${field}`)}
              value={weights[field]}
              onChange={(event) =>
                setWeights((current) => ({ ...current, [field]: event.target.value }))
              }
              disabled={field === 'priority' && priorityLocked}
              inputProps={{ step: 0.01, min: 0, max: 1 }}
            />
          </Grid>
        ))}
      </Grid>

      <Alert severity={invalidTotal ? 'warning' : 'info'} sx={{ py: 0.5 }}>
        <Typography variant="caption" display="block">
          {t('routing.profile.weightTotal', { total: totalWeight.toFixed(3) })}
        </Typography>
      </Alert>

      <Grid container spacing={1.5}>
        <Grid size={{ xs: 12, sm: 6 }}>
          <TextField
            fullWidth
            select
            size="small"
            label={t('routing.profile.cacheAffinityMode')}
            value={cacheAffinityMode}
            onChange={(event) =>
              setCacheAffinityMode(event.target.value as RoutingCacheAffinityMode)
            }
          >
            {CACHE_AFFINITY_MODES.map((mode) => (
              <MenuItem key={mode} value={mode}>
                {t(`routing.profile.cacheAffinityModes.${mode}`)}
              </MenuItem>
            ))}
          </TextField>
        </Grid>
        <Grid size={{ xs: 12, sm: 6 }}>
          <TextField
            fullWidth
            size="small"
            type="number"
            label={t('routing.profile.affinityBonus')}
            value={affinityBonus}
            error={invalidAffinityBonus}
            onChange={(event) => setAffinityBonus(event.target.value)}
            inputProps={{ step: 0.5, min: 0 }}
          />
        </Grid>
        <Grid size={{ xs: 12, sm: 6 }}>
          <TextField
            fullWidth
            size="small"
            type="number"
            label={t('routing.profile.explorationBudgetPercent')}
            value={explorationBudgetPercent}
            error={invalidExplorationBudget}
            onChange={(event) => setExplorationBudgetPercent(event.target.value)}
            inputProps={{ step: 1, min: 0, max: 100 }}
          />
        </Grid>
        <Grid size={{ xs: 12, sm: 6 }}>
          <TextField
            fullWidth
            size="small"
            type="number"
            label={t('routing.profile.emaRegressionPenalty')}
            value={emaRegressionPenalty}
            error={invalidEmaRegressionPenalty}
            onChange={(event) => setEmaRegressionPenalty(event.target.value)}
            inputProps={{ step: 1, min: 0, max: 100 }}
          />
        </Grid>
      </Grid>

      <Stack direction="row" justifyContent="flex-end">
        <Button variant="contained" size="medium" loading={submitting} disabled={saveDisabled} onClick={save}>
          {t('common.save')}
        </Button>
      </Stack>
    </Stack>
  );
}

const emptyWeights: Record<keyof RoutingProfileWeights, string> = {
  success: '0',
  ttfb: '0',
  latency: '0',
  tps: '0',
  cost: '0',
  headroom: '0',
  priority: '0',
};

function toWeightForm(weights: RoutingProfileWeights): Record<keyof RoutingProfileWeights, string> {
  return {
    success: String(weights.success),
    ttfb: String(weights.ttfb),
    latency: String(weights.latency),
    tps: String(weights.tps),
    cost: String(weights.cost),
    headroom: String(weights.headroom),
    priority: String(weights.priority),
  };
}

function fromWeightForm(weights: Record<keyof RoutingProfileWeights, string>): RoutingProfileWeights {
  return {
    success: parseWeight(weights.success),
    ttfb: parseWeight(weights.ttfb),
    latency: parseWeight(weights.latency),
    tps: parseWeight(weights.tps),
    cost: parseWeight(weights.cost),
    headroom: parseWeight(weights.headroom),
    priority: parseWeight(weights.priority),
  };
}

function parseWeight(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function parseNumber(value: string) {
  return Number(value);
}

function isPercent(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 && parsed <= 100;
}

function isNonNegativeNumber(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0;
}
