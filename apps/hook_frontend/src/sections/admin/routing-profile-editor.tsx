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

type RoutingProfileSettingKey =
  | 'min_samples'
  | 'exploration_k'
  | 'conversion_penalty'
  | 'stale_metric_penalty'
  | 'affinity_bonus'
  | 'prior_sample_cap';

const SETTING_FIELDS: Array<{
  key: RoutingProfileSettingKey;
  label: string;
  min: number;
  step: number;
  integer?: boolean;
}> = [
  { key: 'min_samples', label: 'minSamples', min: 1, step: 1, integer: true },
  { key: 'exploration_k', label: 'explorationK', min: 0, step: 0.1 },
  { key: 'conversion_penalty', label: 'conversionPenalty', min: 0, step: 0.5 },
  { key: 'stale_metric_penalty', label: 'staleMetricPenalty', min: 0, step: 0.5 },
  { key: 'affinity_bonus', label: 'affinityBonus', min: 0, step: 0.5 },
  { key: 'prior_sample_cap', label: 'priorSampleCap', min: 0, step: 1, integer: true },
];

type Props = {
  profile: RoutingProfile | null;
  onSaved: VoidFunction;
};

export function RoutingProfileEditor({ profile, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [autoTuneEnabled, setAutoTuneEnabled] = useState(false);
  const [contextualExplorationEnabled, setContextualExplorationEnabled] = useState(true);
  const [weights, setWeights] = useState<Record<keyof RoutingProfileWeights, string>>(emptyWeights);
  const [settings, setSettings] = useState<Record<RoutingProfileSettingKey, string>>(emptySettings);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!profile) {
      setAutoTuneEnabled(false);
      setContextualExplorationEnabled(true);
      setWeights(emptyWeights);
      setSettings(emptySettings);
      return;
    }
    setAutoTuneEnabled(profile.auto_tune_enabled);
    setContextualExplorationEnabled(profile.contextual_exploration_enabled);
    setWeights(toWeightForm(profile.learning?.admin_weights || profile.weights));
    setSettings(toSettingsForm(profile));
  }, [profile]);

  const totalWeight = useMemo(
    () => WEIGHT_FIELDS.reduce((sum, key) => sum + parseWeight(weights[key]), 0),
    [weights]
  );
  const invalidTotal = Math.abs(totalWeight - 1) > 0.001;
  const invalidSettings = useMemo(() => SETTING_FIELDS.some((field) => invalidSetting(settings[field.key], field)), [settings]);
  const saveDisabled = !profile || submitting || invalidTotal || invalidSettings;
  const priorityLocked = !profile || profile.id !== 'fixed_priority_plus';

  const save = useCallback(async () => {
    if (!profile) return;
    setSubmitting(true);
    try {
      await updateRoutingProfile(profile.id, {
        auto_tune_enabled: autoTuneEnabled,
        contextual_exploration_enabled: contextualExplorationEnabled,
        weights: fromWeightForm(weights),
        ...fromSettingsForm(settings),
      });
      toast.success(t('messages.routingProfileUpdated'));
      onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [autoTuneEnabled, contextualExplorationEnabled, onSaved, profile, settings, t, weights]);

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

      <Divider />

      <Typography variant="subtitle2">{t('routing.profile.advancedTitle')}</Typography>

      <Grid container spacing={1.5}>
        {SETTING_FIELDS.map((field) => (
          <Grid size={{ xs: 6 }} key={field.key}>
            <TextField
              fullWidth
              size="small"
              type="number"
              label={t(`routing.profile.settingFields.${field.label}`)}
              value={settings[field.key]}
              onChange={(event) =>
                setSettings((current) => ({ ...current, [field.key]: event.target.value }))
              }
              error={invalidSetting(settings[field.key], field)}
              inputProps={{ step: field.step, min: field.min }}
            />
          </Grid>
        ))}
      </Grid>

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
            {t('routing.profile.settingFields.contextualExploration')}
          </Typography>
        }
      />

      <Alert severity={invalidTotal ? 'warning' : 'info'} sx={{ py: 0.5 }}>
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

const emptySettings: Record<RoutingProfileSettingKey, string> = {
  min_samples: '20',
  exploration_k: '3',
  conversion_penalty: '6',
  stale_metric_penalty: '8',
  affinity_bonus: '6',
  prior_sample_cap: '20',
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

function toSettingsForm(profile: RoutingProfile): Record<RoutingProfileSettingKey, string> {
  return {
    min_samples: String(profile.min_samples),
    exploration_k: String(profile.exploration_k),
    conversion_penalty: String(profile.conversion_penalty),
    stale_metric_penalty: String(profile.stale_metric_penalty),
    affinity_bonus: String(profile.affinity_bonus),
    prior_sample_cap: String(profile.prior_sample_cap),
  };
}

function fromSettingsForm(settings: Record<RoutingProfileSettingKey, string>): RoutingProfileUpsert {
  return {
    min_samples: parseInteger(settings.min_samples),
    exploration_k: parseSetting(settings.exploration_k),
    conversion_penalty: parseSetting(settings.conversion_penalty),
    stale_metric_penalty: parseSetting(settings.stale_metric_penalty),
    affinity_bonus: parseSetting(settings.affinity_bonus),
    prior_sample_cap: parseInteger(settings.prior_sample_cap),
  };
}

function invalidSetting(
  value: string,
  field: (typeof SETTING_FIELDS)[number]
) {
  const parsed = parseSetting(value);
  if (!Number.isFinite(parsed) || parsed < field.min) {
    return true;
  }
  return Boolean(field.integer && !Number.isInteger(parsed));
}

function parseWeight(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function parseSetting(value: string) {
  return Number(value);
}

function parseInteger(value: string) {
  return Math.trunc(parseSetting(value));
}
