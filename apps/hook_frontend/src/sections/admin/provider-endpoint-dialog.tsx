'use client';

import type { AddEndpointForm } from './provider-endpoint-add-card';
import type { useProviderChildDialogs } from './provider-management-state';
import type { Provider, ProviderEndpoint, ProviderEndpointUpdate } from 'src/types/provider';
import type {
  EditableBodyRule,
  EndpointEditState,
  EditableHeaderRule,
} from './provider-endpoint-rule-types';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import {
  useProviderEndpoints,
  createProviderEndpoint,
  updateProviderEndpoint,
  deleteProviderEndpoint,
} from 'src/actions/providers';

import { toast } from 'src/components/snackbar';

import { OPENAI_COMPACT_API_FORMAT, defaultOpenAiCompactBodyRules } from './provider-endpoint-default-rules';
import { addEndpointPayload, emptyAddEndpointForm, ProviderEndpointAddCard } from './provider-endpoint-add-card';
import { ProviderEndpointCard, editStateFromEndpoint, validateEndpointEditState } from './provider-endpoint-card';

type BusyState = { id: string; action: 'save' | 'delete' | 'toggle' } | null;

export function ProviderEndpointDialog({
  dialogs,
  provider,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  provider?: Provider;
}) {
  const { t } = useTranslate('admin');
  const endpointQuery = useProviderEndpoints(dialogs.endpointOpen ? provider?.id : null);
  const manager = useEndpointManager(provider, endpointQuery.items, endpointQuery.refresh);

  return (
    <Dialog fullWidth maxWidth="md" open={dialogs.endpointOpen} onClose={dialogs.closeEndpoint}>
      <DialogTitle>
        {t('providers.endpointManagement')}
        <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
          {t('providers.endpointManagementDescription', { name: provider?.name ?? '' })}
        </Typography>
      </DialogTitle>
      <DialogContent dividers sx={{ px: 3, py: 2 }}>
        <Stack spacing={2}>
          <ConfiguredEndpoints loading={endpointQuery.isLoading} manager={manager} endpoints={endpointQuery.items} />
          <ProviderEndpointAddCard
            form={manager.addForm}
            rulesOpen={manager.addRulesOpen}
            adding={manager.adding}
            existingEndpoints={endpointQuery.items}
            onFormChange={manager.setAddForm}
            onApiFormatChange={manager.setAddApiFormat}
            onRulesOpenChange={manager.setAddRulesOpen}
            onHeaderRulesChange={manager.setAddHeaderRules}
            onBodyRulesChange={manager.setAddBodyRules}
            onAdd={() => void manager.addEndpoint()}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={dialogs.closeEndpoint}>{t('common.close')}</Button>
      </DialogActions>
    </Dialog>
  );
}

function ConfiguredEndpoints({
  loading,
  manager,
  endpoints,
}: {
  loading: boolean;
  manager: EndpointManager;
  endpoints: ProviderEndpoint[];
}) {
  const { t } = useTranslate('admin');
  if (loading) return <Typography variant="body2" color="text.secondary">{t('common.loading')}</Typography>;
  return (
    <Stack spacing={1.5}>
      <Typography variant="caption" sx={labelSx}>已配置的端点</Typography>
      {endpoints.map((endpoint) => (
        <ProviderEndpointCard
          key={endpoint.id}
          endpoint={endpoint}
          editState={manager.editStates[endpoint.id] ?? editStateFromEndpoint(endpoint)}
          expanded={manager.expanded[endpoint.id] ?? false}
          saving={manager.busy?.id === endpoint.id && manager.busy.action === 'save'}
          deleting={manager.busy?.id === endpoint.id && manager.busy.action === 'delete'}
          toggling={manager.busy?.id === endpoint.id && manager.busy.action === 'toggle'}
          onEditStateChange={(state) => manager.setEditState(endpoint.id, state)}
          onExpandedChange={(open) => manager.setExpanded(endpoint.id, open)}
          onSave={(payload) => void manager.saveEndpoint(endpoint, payload)}
          onDelete={() => void manager.deleteEndpoint(endpoint)}
          onToggle={() => void manager.toggleEndpoint(endpoint)}
        />
      ))}
      {!endpoints.length && <Typography variant="body2" color="text.secondary">{t('common.noData')}</Typography>}
    </Stack>
  );
}

function useEndpointManager(
  provider: Provider | undefined,
  endpoints: ProviderEndpoint[],
  refresh: () => Promise<unknown>
) {
  const { t } = useTranslate('admin');
  const [busy, setBusy] = useState<BusyState>(null);
  const [adding, setAdding] = useState(false);
  const [expanded, setExpandedState] = useState<Record<string, boolean>>({});
  const [editStates, setEditStates] = useState<Record<string, EndpointEditState>>({});
  const [addForm, setAddForm] = useState<AddEndpointForm>(() => emptyAddEndpointForm());
  const [addRulesOpen, setAddRulesOpen] = useState(false);
  const [addBodyRulesCustomized, setAddBodyRulesCustomized] = useState(false);

  useEffect(() => {
    setEditStates(Object.fromEntries(endpoints.map((endpoint) => [endpoint.id, editStateFromEndpoint(endpoint)])));
    setExpandedState(Object.fromEntries(endpoints.map((endpoint) => [endpoint.id, Boolean((endpoint.header_rules?.length ?? 0) + (endpoint.body_rules?.length ?? 0))])));
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
    const error = validateEndpointEditState(editStates[endpoint.id] ?? editStateFromEndpoint(endpoint));
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
      toast.error('请选择端点格式');
      return;
    }
    const error = validateEndpointEditState(addForm);
    if (error) {
      toast.error(error);
      return;
    }
    setAdding(true);
    try {
      const payload = addEndpointPayload(addForm);
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

  return useMemo(() => ({
    addForm,
    addRulesOpen,
    adding,
    busy,
    editStates,
    expanded,
    setAddApiFormat,
    setAddBodyRules,
    setAddForm,
    setAddHeaderRules,
    setAddRulesOpen,
    setEditState,
    setExpanded,
    saveEndpoint,
    deleteEndpoint,
    toggleEndpoint,
    addEndpoint,
  }), [
    addEndpoint,
    addForm,
    addRulesOpen,
    adding,
    busy,
    deleteEndpoint,
    editStates,
    expanded,
    saveEndpoint,
    setAddApiFormat,
    setAddBodyRules,
    setAddHeaderRules,
    setEditState,
    setExpanded,
    toggleEndpoint,
  ]);
}

type EndpointManager = ReturnType<typeof useEndpointManager>;

function selectAddApiFormat(form: AddEndpointForm, apiFormat: string, addBodyRulesCustomized: boolean) {
  if (!shouldApplyCompactDefaults(form, apiFormat, addBodyRulesCustomized)) {
    return { form: { ...form, apiFormat }, appliedDefaults: false };
  }
  return {
    form: { ...form, apiFormat, bodyRules: defaultOpenAiCompactBodyRules() },
    appliedDefaults: true,
  };
}

function shouldApplyCompactDefaults(form: AddEndpointForm, apiFormat: string, addBodyRulesCustomized: boolean) {
  return apiFormat === OPENAI_COMPACT_API_FORMAT && !addBodyRulesCustomized && !form.bodyRules.length;
}

const labelSx = { color: 'text.secondary', fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1 };
