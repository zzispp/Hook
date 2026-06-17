'use client';

import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { RoutingProfile, RoutingProfileId } from 'src/types/routing';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';
import { updateGlobalModel } from 'src/actions/models';
import { updateBillingGroup } from 'src/actions/groups';

import { toast } from 'src/components/snackbar';

import { routingProfileName } from './routing-i18n';

type Props = {
  group: BillingGroup | null;
  model: GlobalModelResponse | null;
  profiles: RoutingProfile[];
  loading: boolean;
  onSaved: VoidFunction;
};

type Scope = 'billing_group' | 'model';

export function RoutingProfileSettings({ group, model, profiles, loading, onSaved }: Props) {
  const { t } = useTranslate('admin');
  const [scope, setScope] = useState<Scope>('billing_group');
  const [selectedProfileId, setSelectedProfileId] = useState<RoutingProfileId | ''>('');
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setSelectedProfileId(currentProfileId(scope, group, model));
  }, [group, model, scope]);

  const effectiveProfile = useMemo(() => {
    const hasProfileContext = Boolean(group || model);
    const profileId =
      model?.routing_profile_id ??
      group?.routing_profile_id ??
      (hasProfileContext ? 'balanced' : null);
    if (!profileId) {
      return null;
    }
    return profiles.find((item) => item.id === profileId) ?? null;
  }, [group, model, profiles]);

  const scopeDisabled = !group || (scope === 'model' && !model);
  const saveDisabled = loading || submitting || scopeDisabled;

  const save = useCallback(async () => {
    if (!group) {
      toast.error(t('validation.groupRequired'));
      return;
    }
    setSubmitting(true);
    try {
      if (scope === 'billing_group') {
        await updateBillingGroup(group.id, {
          routing_profile_id: selectedProfileId || null,
        });
      } else {
        if (!model) {
          throw new Error(t('routing.runtime.modelScopeUnavailable'));
        }
        await updateGlobalModel(model.id, {
          routing_profile_id: selectedProfileId || null,
        });
      }
      toast.success(t('messages.routingProfileUpdated'));
      onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [group, model, onSaved, scope, selectedProfileId, t]);

  return (
    <Stack spacing={2.5}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextField
          select
          size="small"
          label={t('routing.runtime.scope')}
          value={scope}
          onChange={(event) => setScope(event.target.value as Scope)}
          sx={{ minWidth: 180 }}
        >
          <MenuItem value="billing_group">{t('routing.runtime.scopeBillingGroup')}</MenuItem>
          <MenuItem value="model">{t('routing.runtime.scopeModel')}</MenuItem>
        </TextField>

        <TextField
          select
          size="small"
          label={t('fields.routingProfile')}
          value={selectedProfileId}
          onChange={(event) => setSelectedProfileId(event.target.value as RoutingProfileId | '')}
          disabled={scopeDisabled}
          sx={{ minWidth: 260 }}
        >
          <MenuItem value="">
            {scope === 'model' ? t('routing.runtime.followBillingGroup') : t('routing.profileInherited')}
          </MenuItem>
          {profiles.map((profile) => (
            <MenuItem key={profile.id} value={profile.id}>
              {routingProfileName(profile, t)}
            </MenuItem>
          ))}
        </TextField>

        <Button
          variant="contained"
          loading={submitting}
          disabled={saveDisabled}
          onClick={save}
          sx={{ alignSelf: { xs: 'stretch', md: 'center' } }}
        >
          {t('common.save')}
        </Button>
      </Stack>

      {!group ? <Alert severity="info">{t('routing.runtime.groupRequired')}</Alert> : null}
      {scope === 'model' && !model ? <Alert severity="info">{t('routing.runtime.modelRequired')}</Alert> : null}

      {effectiveProfile ? (
        <Alert severity="info">
          {t('routing.runtime.effectiveProfile', {
            profile: routingProfileName(effectiveProfile, t),
            modelScope: model?.routing_profile_id ? t('routing.runtime.scopeModel') : t('common.none'),
            groupScope: group?.routing_profile_id ? t('routing.runtime.scopeBillingGroup') : t('common.none'),
          })}
        </Alert>
      ) : null}
    </Stack>
  );
}

function currentProfileId(scope: Scope, group: BillingGroup | null, model: GlobalModelResponse | null): RoutingProfileId | '' {
  if (scope === 'model') {
    return model?.routing_profile_id ?? '';
  }
  return group?.routing_profile_id ?? '';
}
