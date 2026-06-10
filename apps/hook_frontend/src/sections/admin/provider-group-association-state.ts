'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type {
  ProviderGroup,
  ProviderKeyGroup,
  ProviderGroupMember,
  ProviderKeyGroupMember,
} from 'src/types/provider-group';

import { useState, useCallback } from 'react';

import { updateProviderGroup, updateProviderKeyGroup } from 'src/actions/provider-groups';

import { toast } from 'src/components/snackbar';

import { providerKeyGroupIdsForKey, providerGroupIdsForProvider } from './provider-groups-utils';

type AdminT = ReturnType<typeof useTranslate>['t'];

export function useProviderGroupAssociation(t: AdminT, groups: ProviderGroup[]) {
  const [target, setTarget] = useState<Provider | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const openForProvider = useCallback(
    (provider: Provider) => {
      setTarget(provider);
      setSelectedIds(providerGroupIdsForProvider(groups, provider.id));
    },
    [groups]
  );

  const close = useCallback(() => {
    setTarget(null);
    setSelectedIds([]);
  }, []);

  const submit = useCallback(async () => {
    if (!target) return;
    setSubmitting(true);
    try {
      await saveProviderGroupAssociations({ provider: target, selectedIds, groups });
      toast.success(t('messages.providerGroupAssociationSaved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, groups, selectedIds, target, t]);

  return {
    close,
    open: !!target,
    openForProvider,
    selectedIds,
    setSelectedIds,
    submit,
    submitting,
    target,
  };
}

export function useProviderKeyGroupAssociation(t: AdminT, groups: ProviderKeyGroup[]) {
  const [target, setTarget] = useState<ProviderApiKey | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const openForKey = useCallback(
    (apiKey: ProviderApiKey) => {
      setTarget(apiKey);
      setSelectedIds(providerKeyGroupIdsForKey(groups, apiKey.id));
    },
    [groups]
  );

  const close = useCallback(() => {
    setTarget(null);
    setSelectedIds([]);
  }, []);

  const submit = useCallback(async () => {
    if (!target) return;
    setSubmitting(true);
    try {
      await saveProviderKeyGroupAssociations({ apiKey: target, selectedIds, groups });
      toast.success(t('messages.providerKeyGroupAssociationSaved'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, groups, selectedIds, target, t]);

  return {
    close,
    open: !!target,
    openForKey,
    selectedIds,
    setSelectedIds,
    submit,
    submitting,
    target,
  };
}

async function saveProviderGroupAssociations({
  provider,
  selectedIds,
  groups,
}: {
  provider: Provider;
  selectedIds: string[];
  groups: ProviderGroup[];
}) {
  const selected = new Set(selectedIds);
  const updates = groups.flatMap((group) => {
    const nextMembers = associatedProviderMembers(
      group.provider_members,
      provider,
      selected.has(group.id)
    );
    return providerMembersEqual(nextMembers, group.provider_members)
      ? []
      : updateProviderGroup(group.id, { provider_members: nextMembers });
  });
  await Promise.all(updates);
}

async function saveProviderKeyGroupAssociations({
  apiKey,
  selectedIds,
  groups,
}: {
  apiKey: ProviderApiKey;
  selectedIds: string[];
  groups: ProviderKeyGroup[];
}) {
  const selected = new Set(selectedIds);
  const updates = groups.flatMap((group) => {
    const nextMembers = associatedProviderKeyMembers(
      group.provider_key_members,
      apiKey,
      selected.has(group.id)
    );
    return providerKeyMembersEqual(nextMembers, group.provider_key_members)
      ? []
      : updateProviderKeyGroup(group.id, { provider_key_members: nextMembers });
  });
  await Promise.all(updates);
}

function associatedProviderMembers(
  currentMembers: ProviderGroupMember[],
  provider: Provider,
  associated: boolean
) {
  if (associated) {
    return currentMembers.some((member) => member.provider_id === provider.id)
      ? currentMembers
      : [...currentMembers, { provider_id: provider.id, priority: provider.priority }];
  }

  return currentMembers.filter((member) => member.provider_id !== provider.id);
}

function associatedProviderKeyMembers(
  currentMembers: ProviderKeyGroupMember[],
  apiKey: ProviderApiKey,
  associated: boolean
) {
  if (associated) {
    return currentMembers.some((member) => member.provider_key_id === apiKey.id)
      ? currentMembers
      : [...currentMembers, { provider_key_id: apiKey.id, priority: apiKey.internal_priority }];
  }

  return currentMembers.filter((member) => member.provider_key_id !== apiKey.id);
}

function providerMembersEqual(left: ProviderGroupMember[], right: ProviderGroupMember[]) {
  return (
    left.length === right.length &&
    left.every(
      (value, index) =>
        value.provider_id === right[index]?.provider_id && value.priority === right[index]?.priority
    )
  );
}

function providerKeyMembersEqual(left: ProviderKeyGroupMember[], right: ProviderKeyGroupMember[]) {
  return (
    left.length === right.length &&
    left.every(
      (value, index) =>
        value.provider_key_id === right[index]?.provider_key_id &&
        value.priority === right[index]?.priority
    )
  );
}
