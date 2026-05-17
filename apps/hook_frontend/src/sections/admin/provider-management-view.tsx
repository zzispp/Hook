'use client';

import type { Provider } from 'src/types/provider';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useGlobalModels } from 'src/actions/models';
import { useProviders } from 'src/actions/providers';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { ProviderTable } from './provider-table';
import { ProviderFormDialog } from './provider-form-dialog';
import { ProviderModelDialog } from './provider-model-dialog';
import { ProviderApiKeyDialog } from './provider-api-key-dialog';
import { ProviderBindingsPanel } from './provider-bindings-panel';
import { ProviderEndpointDialog } from './provider-endpoint-dialog';
import { ProviderPriorityDialog } from './provider-priority-dialog';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import {
  toProviderFilters,
  ProviderFiltersToolbar,
  DEFAULT_PROVIDER_FILTERS,
} from './provider-filters-toolbar';
import {
  useProviderDialog,
  useDeleteProviderDialog,
  useProviderChildDialogs,
} from './provider-management-state';

const PROVIDER_PRIORITY_LIMIT = 1000;

export function ProviderManagementView() {
  const state = useProviderManagementState();

  return (
    <DashboardContent maxWidth="xl">
      <ProviderHeader state={state} />
      <ProviderTableCard state={state} />
      <ProviderDialogs state={state} />
    </DashboardContent>
  );
}

function useProviderManagementState() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const [filters, setFilters] = useState(DEFAULT_PROVIDER_FILTERS);
  const [selectedProvider, setSelectedProvider] = useState<Provider | undefined>();
  const [bindingsOpen, setBindingsOpen] = useState(false);
  const [priorityOpen, setPriorityOpen] = useState(false);
  const providers = useProviders(table.page, table.rowsPerPage, toProviderFilters(filters));
  const priorityProviders = useProviders(0, PROVIDER_PRIORITY_LIMIT);
  const settings = useSystemSettings();
  const models = useGlobalModels(0, 1000);
  const dialog = useProviderDialog(t);
  const deleteDialog = useDeleteProviderDialog(t);
  const childDialogs = useProviderChildDialogs(t, selectedProvider?.id);

  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_PROVIDER_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
      setSelectedProvider(undefined);
      setBindingsOpen(false);
    },
    [table]
  );

  const openProviderBindings = useCallback((provider: Provider) => {
    setSelectedProvider(provider);
    setBindingsOpen(true);
  }, []);

  const closeProviderBindings = useCallback(() => {
    setBindingsOpen(false);
  }, []);

  return {
    t,
    table,
    models,
    dialog,
    filters,
    providers,
    settings,
    childDialogs,
    bindingsOpen,
    priorityOpen,
    deleteDialog,
    selectedProvider,
    priorityProviders,
    setPriorityOpen,
    openProviderBindings,
    closeProviderBindings,
    handleFiltersChange,
  };
}

function ProviderHeader({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  return (
    <AdminBreadcrumbs
      headingCode={DASHBOARD_MENU_CODES.providerManagement}
      action={
        <Stack direction="row" spacing={1}>
          <RefreshButton loading={state.providers.isLoading} onClick={() => void state.providers.refresh()} />
          <AddButton onClick={state.dialog.openCreate}>{state.t('actions.addProvider')}</AddButton>
        </Stack>
      }
    />
  );
}

function ProviderTableCard({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  return (
    <Card>
      <ProviderFiltersToolbar
        filters={state.filters}
        models={state.models.items}
        schedulingLabel={schedulingModeLabel(state.settings.data?.scheduling_mode ?? 'cache_affinity', state.t)}
        onChange={state.handleFiltersChange}
        onOpenPriority={() => state.setPriorityOpen(true)}
      />
      <ProviderTable
        rows={state.providers.items}
        total={state.providers.total}
        loading={state.providers.isLoading}
        table={state.table}
        selectedId={state.bindingsOpen ? state.selectedProvider?.id : undefined}
        onSelect={state.openProviderBindings}
        onEdit={state.dialog.openEdit}
        onDelete={state.deleteDialog.setDeleteTarget}
      />
    </Card>
  );
}

function schedulingModeLabel(value: string, t: (key: string) => string) {
  const labels: Record<string, string> = {
    cache_affinity: t('providers.schedulingCacheAffinity'),
    fixed_order: t('providers.schedulingFixedOrder'),
    load_balance: t('providers.schedulingLoadBalance'),
  };

  return labels[value] ?? value;
}

function ProviderDialogs({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <ProviderFormDialog dialog={state.dialog} />
      <ProviderBindingsPanel
        open={state.bindingsOpen}
        provider={state.selectedProvider}
        models={state.models.items}
        dialogs={state.childDialogs}
        onClose={state.closeProviderBindings}
      />
      <ProviderEndpointDialog dialogs={state.childDialogs} provider={state.selectedProvider} />
      <ProviderApiKeyDialog
        dialogs={state.childDialogs}
        models={state.models.items}
        providerId={state.selectedProvider?.id}
      />
      <ProviderModelDialog
        dialogs={state.childDialogs}
        models={state.models.items}
        providerId={state.selectedProvider?.id}
        providerName={state.selectedProvider?.name}
      />
      <ProviderPriorityDialog
        open={state.priorityOpen}
        providers={state.priorityProviders.items}
        loading={state.priorityProviders.isLoading}
        schedulingMode={state.settings.data?.scheduling_mode ?? 'cache_affinity'}
        onClose={() => state.setPriorityOpen(false)}
        onSaved={() => {
          void state.providers.refresh();
          void state.priorityProviders.refresh();
          void state.settings.refresh();
        }}
      />
      <ConfirmDialog
        open={!!state.deleteDialog.deleteTarget}
        onClose={() => state.deleteDialog.setDeleteTarget(null)}
        title={t('dialogs.deleteProvider')}
        content={t('providers.deleteConfirm', { name: state.deleteDialog.deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={state.deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
      <ConfirmDialog
        open={!!state.childDialogs.deletingApiKey}
        onClose={() => state.childDialogs.setDeletingApiKey(null)}
        title={t('dialogs.deleteProviderKey')}
        content={t('dialogs.deleteContent', { name: state.childDialogs.deletingApiKey?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={state.childDialogs.confirmDeleteApiKey}>
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}
