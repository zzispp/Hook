'use client';

import type { AddEndpointForm, QuickAddEndpointForm } from './provider-endpoint-add-card';
import type { Provider, ProviderEndpoint, ProviderEndpointUpdate } from 'src/types/provider';
import type { EditableBodyRule, EndpointEditState, EditableHeaderRule } from './provider-endpoint-rule-types';

import { useMemo, useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { createProviderEndpoint, deleteProviderEndpoint, updateProviderEndpoint } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';

import { editStateFromEndpoint, validateEndpointEditState } from './provider-endpoint-card';
import { OPENAI_COMPACT_API_FORMAT, defaultOpenAiCompactBodyRules } from './provider-endpoint-default-rules';
import {
  addEndpointPayload,
  emptyAddEndpointForm,
  quickAddEndpointPayloads,
  emptyQuickAddEndpointForm,
} from './provider-endpoint-add-card';

type BusyState = { id: string; action: 'save' | 'delete' | 'toggle' } | null;

export function useEndpointManager(
  provider: Provider | undefined,
  endpoints: ProviderEndpoint[],
  refresh: () => Promise<unknown>
) {
  const { t } = useTranslate('admin');
  const [busy, setBusy] = useState<BusyState>(null);
  const [adding, setAdding] = useState(false);
  const [quickAdding, setQuickAdding] = useState(false);
  const [expanded, setExpandedState] = useState<Record<string, boolean>>({});
  const [editStates, setEditStates] = useState<Record<string, EndpointEditState>>({});
  const [addForm, setAddForm] = useState<AddEndpointForm>(() => emptyAddEndpointForm());
  const [quickAddForm, setQuickAddForm] = useState<QuickAddEndpointForm>(() => emptyQuickAddEndpointForm());
  const [addRulesOpen, setAddRulesOpen] = useState(false);
  const [addBodyRulesCustomized, setAddBodyRulesCustomized] = useState(false);

  useEffect(() => {
    setAddForm(emptyAddEndpointForm());
    setQuickAddForm(emptyQuickAddEndpointForm());
    setAddRulesOpen(false);
    setAddBodyRulesCustomized(false);
  }, [provider?.id]);

  useEffect(() => {
    setEditStates(Object.fromEntries(endpoints.map((endpoint) => [endpoint.id, editStateFromEndpoint(endpoint)])));
    setExpandedState(Object.fromEntries(endpoints.map((endpoint) => [endpoint.id, endpointHasRules(endpoint)])));
  }, [endpoints]);

  const setEditState = useCallback((id: string, state: EndpointEditState) => {
    setEditStates((current) => ({ ...current, [id]: state }));
  }, []);

  const setExpanded = useCallback((id: string, open: boolean) => {
    setExpandedState((current) => ({ ...current, [id]: open }));
  }, []);

  const setAddApiFormat = useCallback((apiFormat: string) => {
    const selection = selectAddApiFormat(addForm, apiFormat, addBodyRulesCustomized);
    setAddForm(selection.form);
    if (selection.appliedDefaults) setAddRulesOpen(true);
  }, [addBodyRulesCustomized, addForm]);

  const setQuickAddApiFormats = useCallback((apiFormats: string[]) => {
    setQuickAddForm((current) => ({ ...current, apiFormats }));
  }, []);

  const setAddHeaderRules = useCallback((headerRules: EditableHeaderRule[]) => {
    setAddForm((current) => ({ ...current, headerRules }));
    setAddRulesOpen(true);
  }, []);

  const setAddBodyRules = useCallback((bodyRules: EditableBodyRule[]) => {
    setAddForm((current) => ({ ...current, bodyRules }));
    setAddBodyRulesCustomized(true);
    setAddRulesOpen(true);
  }, []);

  const saveEndpoint = useCallback(async (endpoint: ProviderEndpoint, payload: ProviderEndpointUpdate) => {
    if (!provider) return;
    const error = validateEndpointEditState(editStates[endpoint.id] ?? editStateFromEndpoint(endpoint), t);
    if (error) {
      toast.error(error);
      return;
    }
    setBusy({ id: endpoint.id, action: 'save' });
    try {
      await updateProviderEndpoint(provider.id, endpoint.id, payload);
      toast.success(t('messages.providerEndpointUpdated'));
      await refresh();
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : t('messages.saveFailed'));
    } finally {
      setBusy(null);
    }
  }, [editStates, provider, refresh, t]);

  const deleteEndpoint = useCallback(async (endpoint: ProviderEndpoint) => {
    if (!provider) return;
    setBusy({ id: endpoint.id, action: 'delete' });
    try {
      await deleteProviderEndpoint(provider.id, endpoint.id);
      toast.success(t('messages.providerEndpointDeleted'));
      await refresh();
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : t('messages.deleteFailed'));
    } finally {
      setBusy(null);
    }
  }, [provider, refresh, t]);

  const toggleEndpoint = useCallback(async (endpoint: ProviderEndpoint) => {
    if (!provider) return;
    setBusy({ id: endpoint.id, action: 'toggle' });
    try {
      await updateProviderEndpoint(provider.id, endpoint.id, { is_active: !endpoint.is_active });
      await refresh();
    } finally {
      setBusy(null);
    }
  }, [provider, refresh]);

  const addEndpoint = useCallback(async () => {
    if (!provider) return;
    if (!addForm.apiFormat) {
      toast.error(t('providers.apiFormatRequired'));
      return;
    }
    const payload = addEndpointPayload(addForm);
    if (!isValidEndpointForm(addForm, t)) return;
    setAdding(true);
    try {
      await createProviderEndpoint(provider.id, payload);
      toast.success(t('messages.providerEndpointCreated'));
      setAddForm(emptyAddEndpointForm(payload.base_url));
      setAddRulesOpen(false);
      setAddBodyRulesCustomized(false);
      await refresh();
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : t('messages.saveFailed'));
    } finally {
      setAdding(false);
    }
  }, [addForm, provider, refresh, t]);

  const quickAddEndpoints = useCallback(async () => {
    if (!provider) return;
    if (!quickAddForm.apiFormats.length) {
      toast.error(t('providers.apiFormatRequired'));
      return;
    }
    if (!isValidQuickAddForm(quickAddForm, t)) return;
    setQuickAdding(true);
    try {
      const payloads = quickAddEndpointPayloads(quickAddForm);
      for (const payload of payloads) {
        await createProviderEndpoint(provider.id, payload);
      }
      toast.success(t('messages.providerEndpointCreated'));
      setQuickAddForm(emptyQuickAddEndpointForm(payloads[0].base_url));
      await refresh();
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : t('messages.saveFailed'));
    } finally {
      setQuickAdding(false);
    }
  }, [provider, quickAddForm, refresh, t]);

  return useMemo(() => ({
    addForm,
    addRulesOpen,
    adding,
    busy,
    editStates,
    expanded,
    quickAddForm,
    quickAdding,
    setAddApiFormat,
    setAddBodyRules,
    setAddForm,
    setAddHeaderRules,
    setAddRulesOpen,
    setEditState,
    setExpanded,
    setQuickAddApiFormats,
    setQuickAddForm,
    saveEndpoint,
    deleteEndpoint,
    toggleEndpoint,
    addEndpoint,
    quickAddEndpoints,
  }), [
    addEndpoint,
    addForm,
    addRulesOpen,
    adding,
    busy,
    deleteEndpoint,
    editStates,
    expanded,
    quickAddEndpoints,
    quickAddForm,
    quickAdding,
    saveEndpoint,
    setAddApiFormat,
    setAddBodyRules,
    setAddHeaderRules,
    setEditState,
    setExpanded,
    setQuickAddApiFormats,
    toggleEndpoint,
  ]);
}

export type EndpointManager = ReturnType<typeof useEndpointManager>;

function selectAddApiFormat(form: AddEndpointForm, apiFormat: string, addBodyRulesCustomized: boolean) {
  if (apiFormat !== OPENAI_COMPACT_API_FORMAT || addBodyRulesCustomized || form.bodyRules.length) {
    return { form: { ...form, apiFormat }, appliedDefaults: false };
  }
  return { form: { ...form, apiFormat, bodyRules: defaultOpenAiCompactBodyRules() }, appliedDefaults: true };
}

function isValidEndpointForm(form: AddEndpointForm, t: ReturnType<typeof useTranslate>['t']) {
  const error = validateEndpointEditState(form, t);
  if (error) toast.error(error);
  return !error;
}

function isValidQuickAddForm(form: QuickAddEndpointForm, t: ReturnType<typeof useTranslate>['t']) {
  return isValidEndpointForm({ ...form, apiFormat: form.apiFormats[0], customPath: '', headerRules: [], bodyRules: [] }, t);
}

function endpointHasRules(endpoint: ProviderEndpoint) {
  return Boolean((endpoint.header_rules?.length ?? 0) + (endpoint.body_rules?.length ?? 0));
}
