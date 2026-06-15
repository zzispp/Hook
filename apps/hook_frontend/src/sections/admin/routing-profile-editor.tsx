'use client';

import type { RoutingProfile, RoutingProfileWeights } from 'src/types/routing';

import { useMemo, useState, useEffect, useCallback } from 'react';

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

type Props = {
  profile: RoutingProfile | null;
  onSaved: VoidFunction;
};

export function RoutingProfileEditor({ profile, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [autoTuneEnabled, setAutoTuneEnabled] = useState(false);
  const [weights, setWeights] = useState<Record<keyof RoutingProfileWeights, string>>(emptyWeights);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!profile) {
      setAutoTuneEnabled(false);
      setWeights(emptyWeights);
      return;
    }
    setAutoTuneEnabled(profile.auto_tune_enabled);
    setWeights(toWeightForm(profile.learning?.admin_weights || profile.weights));
  }, [profile]);

  const totalWeight = useMemo(
    () => WEIGHT_FIELDS.reduce((sum, key) => sum + parseWeight(weights[key]), 0),
    [weights]
  );
  const invalidTotal = Math.abs(totalWeight - 1) > 0.001;
  const saveDisabled = !profile || submitting || invalidTotal;
  const priorityLocked = !profile || profile.id !== 'fixed_priority_plus';

  const save = useCallback(async () => {
    if (!profile) return;
    setSubmitting(true);
    try {
      await updateRoutingProfile(profile.id, {
        auto_tune_enabled: autoTuneEnabled,
        weights: fromWeightForm(weights),
      });
      toast.success(t('messages.routingProfileUpdated'));
      onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [autoTuneEnabled, onSaved, profile, t, weights]);

  if (!profile) {
    return null;
  }

  return (
    <Stack spacing={2}>
      <Divider />
      <Stack spacing={0.5}>
        <Typography variant="subtitle1">{t('routing.profile.editorTitle')}</Typography>
        <Typography variant="body2" color="text.secondary">
          {t('routing.profile.editorHelper')}
        </Typography>
      </Stack>

      <FormControlLabel
        control={
          <Switch
            checked={autoTuneEnabled}
            onChange={(event) => setAutoTuneEnabled(event.target.checked)}
          />
        }
        label={t('routing.profile.autoTune')}
      />

      <Stack direction={{ xs: 'column', lg: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        {WEIGHT_FIELDS.map((field) => (
          <TextField
            key={field}
            size="small"
            type="number"
            label={t(`routing.profile.weightFields.${field}`)}
            value={weights[field]}
            onChange={(event) =>
              setWeights((current) => ({ ...current, [field]: event.target.value }))
            }
            disabled={field === 'priority' && priorityLocked}
            inputProps={{ step: 0.01, min: 0, max: 1 }}
            sx={{ minWidth: 140 }}
          />
        ))}
      </Stack>

      <Alert severity={invalidTotal ? 'warning' : 'info'}>
        {t('routing.profile.weightTotal', { total: totalWeight.toFixed(3) })}
      </Alert>

      <Stack direction="row" justifyContent="flex-end">
        <Button variant="contained" loading={submitting} disabled={saveDisabled} onClick={save}>
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
