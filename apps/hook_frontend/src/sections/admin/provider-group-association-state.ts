'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';

import { useState, useCallback } from 'react';

import { updateProviderGroup, updateProviderKeyGroup } from 'src/actions/provider-groups';

import { toast } from 'src/components/snackbar';

import {
  providerKeyGroupIdsForKey,
  providerGroupIdsForProvider,
} from './provider-groups-utils';

type AdminT = ReturnType<typeof useTranslate>['t'];

export function useProviderGroupAssociation(t: AdminT, groups: ProviderGroup[]) {
  const [target, setTarget] = useState<Provider | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const openForProvider = useCallback((provider: Provider) => {
    setTarget(provider);
    setSelectedIds(providerGroupIdsForProvider(groups, provider.id));
  }, [groups]);

  const close = useCallback(() => {
    setTarget(null);
    setSelectedIds([]);
  }, []);

  const submit = useCallback(async () => {
    if (!target) return;
    setSubmitting(true);
    try {
      await saveProviderGroupAssociations({ providerId: target.id, selectedIds, groups });
      toast.success(t('messages.providerGroupAssociationSaved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, groups, selectedIds, target, t]);

  return { close, open: !!target, openForProvider, selectedIds, setSelectedIds, submit, submitting, target };
}

export function useProviderKeyGroupAssociation(t: AdminT, groups: ProviderKeyGroup[]) {
  const [target, setTarget] = useState<ProviderApiKey | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const openForKey = useCallback((apiKey: ProviderApiKey) => {
    setTarget(apiKey);
    setSelectedIds(providerKeyGroupIdsForKey(groups, apiKey.id));
  }, [groups]);

  const close = useCallback(() => {
    setTarget(null);
    setSelectedIds([]);
  }, []);

  const submit = useCallback(async () => {
    if (!target) return;
    setSubmitting(true);
    try {
      await saveProviderKeyGroupAssociations({ keyId: target.id, selectedIds, groups });
      toast.success(t('messages.providerKeyGroupAssociationSaved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, groups, selectedIds, target, t]);

  return { close, open: !!target, openForKey, selectedIds, setSelectedIds, submit, submitting, target };
}

async function saveProviderGroupAssociations({
  providerId,
  selectedIds,
  groups,
}: {
  providerId: string;
  selectedIds: string[];
  groups: ProviderGroup[];
}) {
  const selected = new Set(selectedIds);
  const updates = groups.flatMap((group) => {
    const nextIds = associatedMemberIds(group.provider_ids, providerId, selected.has(group.id));
    return idsEqual(nextIds, group.provider_ids) ? [] : updateProviderGroup(group.id, { provider_ids: nextIds });
  });
  await Promise.all(updates);
}

async function saveProviderKeyGroupAssociations({
  keyId,
  selectedIds,
  groups,
}: {
  keyId: string;
  selectedIds: string[];
  groups: ProviderKeyGroup[];
}) {
  const selected = new Set(selectedIds);
  const updates = groups.flatMap((group) => {
    const nextIds = associatedMemberIds(group.provider_key_ids, keyId, selected.has(group.id));
    return idsEqual(nextIds, group.provider_key_ids) ? [] : updateProviderKeyGroup(group.id, { provider_key_ids: nextIds });
  });
  await Promise.all(updates);
}

function associatedMemberIds(currentIds: string[], targetId: string, associated: boolean) {
  if (associated) {
    return currentIds.includes(targetId) ? currentIds : [...currentIds, targetId];
  }

  return currentIds.filter((id) => id !== targetId);
}

function idsEqual(left: string[], right: string[]) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}
