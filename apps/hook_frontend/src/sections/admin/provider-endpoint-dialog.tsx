'use client';

import type { EndpointEditState } from './provider-endpoint-rule-types';
import type { useProviderChildDialogs } from './provider-management-state';
import type { Provider, ProviderEndpoint, ProviderEndpointUpdate } from 'src/types/provider';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import LoadingButton from '@mui/lab/LoadingButton';
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

import { ProviderEndpointSearchSelect } from './provider-endpoint-select';
import { formatApiFormat, API_FORMAT_OPTIONS, defaultEndpointPath } from './provider-management-utils';
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
          <AddEndpointCard manager={manager} existingEndpoints={endpointQuery.items} />
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

function AddEndpointCard({
  manager,
  existingEndpoints,
}: {
  manager: EndpointManager;
  existingEndpoints: ProviderEndpoint[];
}) {
  const { t } = useTranslate('admin');
  const availableFormats = API_FORMAT_OPTIONS.filter((format) => !existingEndpoints.some((endpoint) => endpoint.api_format === format));
  const selectedPath = defaultEndpointPath(manager.addForm.apiFormat);

  return (
    <Box sx={addCardSx}>
      <Stack direction="row" spacing={1.5} alignItems="center" sx={{ px: 2, py: 1.25, bgcolor: 'action.hover' }}>
        <ProviderEndpointSearchSelect
          value={manager.addForm.apiFormat}
          options={availableFormats.map((value) => ({ value, label: formatApiFormat(value) }))}
          placeholder={t('providers.selectFormat')}
          sx={{ minWidth: 220 }}
          onChange={(apiFormat) => manager.setAddForm({ ...manager.addForm, apiFormat })}
        />
        <Box sx={{ flex: 1 }} />
        <LoadingButton size="small" variant="outlined" loading={manager.adding} disabled={!manager.addForm.apiFormat || !manager.addForm.baseUrl.trim()} onClick={() => void manager.addEndpoint()}>
          {t('common.add')}
        </LoadingButton>
      </Stack>
      <Divider />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} sx={{ p: 2 }}>
        <TextField fullWidth size="small" label={t('providers.baseUrl')} value={manager.addForm.baseUrl} placeholder={t('providers.baseUrlPlaceholder')} onChange={(event) => manager.setAddForm({ ...manager.addForm, baseUrl: event.target.value })} />
        <TextField
          fullWidth
          size="small"
          label={t('providers.customPath')}
          value={manager.addForm.customPath}
          placeholder={selectedPath || '留空使用默认'}
          helperText={selectedPath ? `${t('providers.defaultWhenBlank')} ${selectedPath}` : t('providers.defaultWhenBlank')}
          onChange={(event) => manager.setAddForm({ ...manager.addForm, customPath: event.target.value })}
        />
      </Stack>
    </Box>
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
  const [addForm, setAddForm] = useState({ apiFormat: '', baseUrl: '', customPath: '' });

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
    setAdding(true);
    try {
      await createProviderEndpoint(provider.id, addEndpointPayload(addForm));
      toast.success(t('messages.providerEndpointCreated'));
      setAddForm({ apiFormat: '', baseUrl: addForm.baseUrl, customPath: '' });
      await refresh();
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : t('messages.saveFailed'));
    } finally {
      setAdding(false);
    }
  }, [addForm, provider, refresh, t]);

  return useMemo(() => ({ addForm, adding, busy, editStates, expanded, setAddForm, setEditState, setExpanded, saveEndpoint, deleteEndpoint, toggleEndpoint, addEndpoint }), [addEndpoint, addForm, adding, busy, deleteEndpoint, editStates, expanded, saveEndpoint, setEditState, setExpanded, toggleEndpoint]);
}

function addEndpointPayload(form: { apiFormat: string; baseUrl: string; customPath: string }) {
  return {
    api_format: form.apiFormat,
    base_url: form.baseUrl,
    custom_path: form.customPath.trim() || null,
    is_active: true,
  };
}

type EndpointManager = ReturnType<typeof useEndpointManager>;

const labelSx = { color: 'text.secondary', fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1 };
const addCardSx = { border: '1px dashed', borderColor: 'divider', borderRadius: 1, overflow: 'hidden' };
