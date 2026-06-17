'use client';

import type { RoutingProfile, RoutingProfileUpsert, RoutingProfileWeights } from 'src/types/routing';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
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

const PARAM_FIELDS = [
  'min_samples',
  'exploration_k',
  'prior_sample_cap',
  'conversion_penalty',
  'stale_metric_penalty',
  'affinity_bonus',
  'ema_alpha',
  'ema_max_freshness_seconds',
  'ema_recent_weight',
  'ema_recent_cap',
  'exploration_weight',
  'exploration_cap',
  'exploration_min_success_score',
] as const;

type RoutingParamField = (typeof PARAM_FIELDS)[number];

type ParamRule = {
  min: number;
  max?: number;
  integer?: boolean;
  step: number;
};

const PARAM_RULES: Record<RoutingParamField, ParamRule> = {
  min_samples: { min: 1, integer: true, step: 1 },
  exploration_k: { min: 0, step: 0.1 },
  prior_sample_cap: { min: 0, integer: true, step: 1 },
  conversion_penalty: { min: 0, step: 0.1 },
  stale_metric_penalty: { min: 0, step: 0.1 },
  affinity_bonus: { min: 0, step: 0.1 },
  ema_alpha: { min: 0, max: 1, step: 0.01 },
  ema_max_freshness_seconds: { min: 0, integer: true, step: 1 },
  ema_recent_weight: { min: 0, step: 0.01 },
  ema_recent_cap: { min: 0, step: 0.01 },
  exploration_weight: { min: 0, step: 0.01 },
  exploration_cap: { min: 0, step: 0.01 },
  exploration_min_success_score: { min: 0, max: 100, step: 0.1 },
};

type Props = {
  profile: RoutingProfile | null;
  onSaved: VoidFunction;
};

export function RoutingProfileEditor({ profile, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [autoTuneEnabled, setAutoTuneEnabled] = useState(false);
  const [contextualExplorationEnabled, setContextualExplorationEnabled] = useState(true);
  const [weights, setWeights] = useState<Record<keyof RoutingProfileWeights, string>>(emptyWeights);
  const [params, setParams] = useState<Record<RoutingParamField, string>>(emptyParams);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!profile) {
      setAutoTuneEnabled(false);
      setContextualExplorationEnabled(true);
      setWeights(emptyWeights);
      setParams(emptyParams);
      return;
    }
    setAutoTuneEnabled(profile.auto_tune_enabled);
    setContextualExplorationEnabled(profile.contextual_exploration_enabled);
    setWeights(toWeightForm(profile.learning?.admin_weights || profile.weights));
    setParams(toParamForm(profile));
  }, [profile]);

  const totalWeight = useMemo(
    () => WEIGHT_FIELDS.reduce((sum, key) => sum + parseNumber(weights[key]), 0),
    [weights]
  );
  const invalidTotal = Math.abs(totalWeight - 1) > 0.001;
  const invalidParams = useMemo(
    () => PARAM_FIELDS.some((field) => invalidParam(params[field], PARAM_RULES[field])),
    [params]
  );
  const saveDisabled = !profile || submitting || invalidTotal || invalidParams;
  const priorityLocked = !profile || profile.id !== 'fixed_priority_plus';

  const save = useCallback(async () => {
    if (!profile) return;
    setSubmitting(true);
    try {
      await updateRoutingProfile(profile.id, {
        auto_tune_enabled: autoTuneEnabled,
        contextual_exploration_enabled: contextualExplorationEnabled,
        weights: fromWeightForm(weights),
        ...fromParamForm(params),
      });
      toast.success(t('messages.routingProfileUpdated'));
      onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [autoTuneEnabled, contextualExplorationEnabled, onSaved, params, profile, t, weights]);

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
              onChange={(event) => setWeights((current) => ({ ...current, [field]: event.target.value }))}
              disabled={field === 'priority' && priorityLocked}
              inputProps={{ step: 0.01, min: 0, max: 1 }}
            />
          </Grid>
        ))}
      </Grid>

      <Stack spacing={0.5}>
        <Typography variant="subtitle2">{t('routing.profile.paramSection')}</Typography>
        <Typography variant="caption" color="text.secondary">
          {t('routing.profile.paramHelper')}
        </Typography>
      </Stack>

      <FormControlLabel
        control={
          <Switch
            size="small"
            checked={contextualExplorationEnabled}
            onChange={(event) => setContextualExplorationEnabled(event.target.checked)}
          />
        }
        label={
          <Typography variant="body2" sx={{ fontWeight: 'medium' }}>
            {t('routing.profile.booleanFields.contextual_exploration_enabled')}
          </Typography>
        }
      />

      <Grid container spacing={1.5}>
        {PARAM_FIELDS.map((field) => (
          <Grid size={{ xs: 6 }} key={field}>
            <TextField
              fullWidth
              size="small"
              type="number"
              label={t(`routing.profile.paramFields.${field}`)}
              value={params[field]}
              onChange={(event) => setParams((current) => ({ ...current, [field]: event.target.value }))}
              error={invalidParam(params[field], PARAM_RULES[field])}
              inputProps={{
                step: PARAM_RULES[field].step,
                min: PARAM_RULES[field].min,
                max: PARAM_RULES[field].max,
              }}
            />
          </Grid>
        ))}
      </Grid>

      <Alert severity={invalidTotal || invalidParams ? 'warning' : 'info'} sx={{ py: 0.5 }}>
        <Typography variant="caption" display="block">
          {t('routing.profile.weightTotal', { total: totalWeight.toFixed(3) })}
        </Typography>
      </Alert>

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

const emptyParams: Record<RoutingParamField, string> = {
  min_samples: '12',
  exploration_k: '4.5',
  prior_sample_cap: '20',
  conversion_penalty: '6',
  stale_metric_penalty: '8',
  affinity_bonus: '6',
  ema_alpha: '0.35',
  ema_max_freshness_seconds: '300',
  ema_recent_weight: '0.35',
  ema_recent_cap: '8',
  exploration_weight: '0.05',
  exploration_cap: '5',
  exploration_min_success_score: '65',
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
    success: parseNumber(weights.success),
    ttfb: parseNumber(weights.ttfb),
    latency: parseNumber(weights.latency),
    tps: parseNumber(weights.tps),
    cost: parseNumber(weights.cost),
    headroom: parseNumber(weights.headroom),
    priority: parseNumber(weights.priority),
  };
}

function toParamForm(profile: RoutingProfile): Record<RoutingParamField, string> {
  return {
    min_samples: String(profile.min_samples),
    exploration_k: String(profile.exploration_k),
    prior_sample_cap: String(profile.prior_sample_cap),
    conversion_penalty: String(profile.conversion_penalty),
    stale_metric_penalty: String(profile.stale_metric_penalty),
    affinity_bonus: String(profile.affinity_bonus),
    ema_alpha: String(profile.ema_alpha),
    ema_max_freshness_seconds: String(profile.ema_max_freshness_seconds),
    ema_recent_weight: String(profile.ema_recent_weight),
    ema_recent_cap: String(profile.ema_recent_cap),
    exploration_weight: String(profile.exploration_weight),
    exploration_cap: String(profile.exploration_cap),
    exploration_min_success_score: String(profile.exploration_min_success_score),
  };
}

function fromParamForm(params: Record<RoutingParamField, string>): RoutingProfileUpsert {
  return {
    min_samples: parseInteger(params.min_samples),
    exploration_k: parseNumber(params.exploration_k),
    prior_sample_cap: parseInteger(params.prior_sample_cap),
    conversion_penalty: parseNumber(params.conversion_penalty),
    stale_metric_penalty: parseNumber(params.stale_metric_penalty),
    affinity_bonus: parseNumber(params.affinity_bonus),
    ema_alpha: parseNumber(params.ema_alpha),
    ema_max_freshness_seconds: parseInteger(params.ema_max_freshness_seconds),
    ema_recent_weight: parseNumber(params.ema_recent_weight),
    ema_recent_cap: parseNumber(params.ema_recent_cap),
    exploration_weight: parseNumber(params.exploration_weight),
    exploration_cap: parseNumber(params.exploration_cap),
    exploration_min_success_score: parseNumber(params.exploration_min_success_score),
  };
}

function invalidParam(value: string, rule: ParamRule) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < rule.min) {
    return true;
  }
  if (rule.max !== undefined && parsed > rule.max) {
    return true;
  }
  return Boolean(rule.integer && !Number.isInteger(parsed));
}

function parseNumber(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function parseInteger(value: string) {
  return Math.trunc(parseNumber(value));
}
