'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type { Provider, ProviderListResponse } from 'src/types/provider';

import { useState, useCallback } from 'react';

import { useGlobalModels } from 'src/actions/models';
import { useTranslate } from 'src/locales/use-locales';
import { useSystemSettings } from 'src/actions/system-settings';
import { useProviders, useProviderPriorityKeys } from 'src/actions/providers';
import { useProviderGroups, useProviderKeyGroups } from 'src/actions/provider-groups';

import { useTable } from 'src/components/table';

import { useProviderCooldownState } from './provider-cooldown-state';
import { toProviderFilters, DEFAULT_PROVIDER_FILTERS } from './provider-filters-toolbar';
import {
  useProviderGroupAssociation,
  useProviderKeyGroupAssociation,
} from './provider-group-association-state';
import {
  useProviderDialog,
  useDeleteProviderDialog,
  useProviderChildDialogs,
} from './provider-management-state';

const PROVIDER_PRIORITY_LIMIT = 1000;

export type ProviderTab = 'providers' | 'groups' | 'cooldowns';

export function useProviderManagementState() {
  const { t, currentLang } = useTranslate('admin');
  const ui = useProviderUiState();
  const providers = useProviders(ui.table.page, ui.table.rowsPerPage, toProviderFilters(ui.filters));
  const cooldownState = useProviderCooldownState(t, ui.cooldownTable);
  const priorityProviders = useProviders(0, PROVIDER_PRIORITY_LIMIT);
  const priorityKeys = useProviderPriorityKeys(priorityProviders.items);
  const providerGroups = useProviderGroups(0, PROVIDER_PRIORITY_LIMIT);
  const providerKeyGroups = useProviderKeyGroups(0, PROVIDER_PRIORITY_LIMIT);
  const refreshProviderGroups = useCallback(async () => {
    await Promise.all([
      providerGroups.refresh(),
      providerKeyGroups.refresh(),
      priorityProviders.refresh(),
      priorityKeys.refresh(),
    ]);
  }, [priorityKeys, priorityProviders, providerGroups, providerKeyGroups]);
  const settings = useSystemSettings();
  const models = useGlobalModels(0, 1000);
  const dialog = useProviderDialog({ t });
  const deleteDialog = useDeleteProviderDialog(t);
  const childDialogs = useProviderChildDialogs(t, ui.selectedProvider?.id);
  const providerGroupAssociation = useProviderGroupAssociation(t, providerGroups.items);
  const providerKeyGroupAssociation = useProviderKeyGroupAssociation(t, providerKeyGroups.items);
  const openPriorityDialog = useOpenPriorityDialog({
    setPriorityOpen: ui.setPriorityOpen,
    refreshProviders: priorityProviders.refresh,
    refreshKeysForProviders: priorityKeys.refreshForProviders,
  });

  return {
    ...ui,
    t,
    models,
    dialog,
    providers,
    settings,
    currentLang,
    providerGroups,
    providerKeyGroups,
    providerGroupAssociation,
    providerKeyGroupAssociation,
    refreshProviderGroups,
    childDialogs,
    deleteDialog,
    cooldowns: cooldownState.cooldowns,
    errorMessage: errorMessage(
      providers.error,
      providerGroups.error,
      providerKeyGroups.error,
      cooldownState.cooldowns.error,
      settings.error,
      models.error,
      priorityKeys.error
    ),
    priorityProviders,
    priorityKeys,
    openPriorityDialog,
    cooldownFilters: cooldownState.filters,
    releasingCooldownId: cooldownState.releasingId,
    releaseCooldown: cooldownState.release,
    handleCooldownFiltersChange: cooldownState.changeFilters,
  };
}

function useOpenPriorityDialog({
  setPriorityOpen,
  refreshProviders,
  refreshKeysForProviders,
}: {
  setPriorityOpen: (value: boolean) => void;
  refreshProviders: () => Promise<ApiEnvelope<ProviderListResponse> | undefined>;
  refreshKeysForProviders: (providers: Pick<Provider, 'id'>[]) => Promise<unknown>;
}) {
  return useCallback(() => {
    void (async () => {
      const response = await refreshProviders();
      await refreshKeysForProviders(response?.data?.providers ?? []);
      setPriorityOpen(true);
    })();
  }, [refreshKeysForProviders, refreshProviders, setPriorityOpen]);
}

function useProviderUiState() {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const cooldownTable = useTable({ defaultRowsPerPage: 10 });
  const [tab, setTab] = useState<ProviderTab>('providers');
  const [filters, setFilters] = useState(DEFAULT_PROVIDER_FILTERS);
  const [selectedProvider, setSelectedProvider] = useState<Provider | undefined>();
  const [bindingsOpen, setBindingsOpen] = useState(false);
  const [priorityOpen, setPriorityOpen] = useState(false);
  const [cooldownPolicyOpen, setCooldownPolicyOpen] = useState(false);

  const handleFiltersChange = useCallback((nextFilters: typeof DEFAULT_PROVIDER_FILTERS) => {
    table.onResetPage();
    setFilters(nextFilters);
    setSelectedProvider(undefined);
    setBindingsOpen(false);
  }, [table]);

  const openProviderBindings = useCallback((provider: Provider) => {
    setSelectedProvider(provider);
    setBindingsOpen(true);
  }, []);

  const closeProviderBindings = useCallback(() => {
    setBindingsOpen(false);
  }, []);

  return {
    tab,
    table,
    filters,
    bindingsOpen,
    priorityOpen,
    cooldownTable,
    selectedProvider,
    cooldownPolicyOpen,
    setTab,
    setPriorityOpen,
    setCooldownPolicyOpen,
    openProviderBindings,
    closeProviderBindings,
    handleFiltersChange,
  };
}

function errorMessage(...errors: unknown[]) {
  const error = errors.find(Boolean);
  return error instanceof Error ? error.message : undefined;
}
