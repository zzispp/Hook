'use client';

import type { ProviderTab } from './provider-management-page-state';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ConfirmDialog } from 'src/components/custom-dialog';

import { ProviderTable } from './provider-table';
import { ProviderFormDialog } from './provider-form-dialog';
import { ProviderGroupsCard } from './provider-groups-card';
import { ProviderModelDialog } from './provider-model-dialog';
import { ProviderApiKeyDialog } from './provider-api-key-dialog';
import { ProviderBindingsPanel } from './provider-bindings-panel';
import { ProviderCooldownTable } from './provider-cooldown-table';
import { ProviderEndpointDialog } from './provider-endpoint-dialog';
import { ProviderFiltersToolbar } from './provider-filters-toolbar';
import { ProviderPriorityDialog } from './provider-priority-dialog';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { useProviderManagementState } from './provider-management-page-state';
import { ProviderCooldownPolicyDialog } from './provider-cooldown-policy-dialog';
import { ProviderGroupAssociationDialog } from './provider-group-association-dialog';

export function ProviderManagementView() {
  const state = useProviderManagementState();

  return (
    <DashboardContent maxWidth="xl">
      <ProviderHeader state={state} />
      <ProviderTabs state={state} />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      {state.tab === 'providers' ? <ProviderTableCard state={state} /> : null}
      {state.tab === 'groups' ? <ProviderGroupsCardWrapper state={state} /> : null}
      {state.tab === 'cooldowns' ? <ProviderCooldownCard state={state} /> : null}
      <ProviderDialogs state={state} />
    </DashboardContent>
  );
}

function ProviderHeader({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  const loading =
    (state.tab === 'providers' && state.providers.isLoading) ||
    (state.tab === 'groups' && (state.providerGroups.isLoading || state.providerKeyGroups.isLoading)) ||
    (state.tab === 'cooldowns' && state.cooldowns.isLoading);
  const refresh = state.tab === 'providers'
    ? state.providers.refresh
    : state.tab === 'groups'
      ? state.refreshProviderGroups
      : state.cooldowns.refresh;

  return (
    <AdminBreadcrumbs
      headingCode={DASHBOARD_MENU_CODES.providerManagement}
      action={
        <Stack direction="row" spacing={1}>
          <RefreshButton loading={loading} onClick={() => void refresh()} />
          {state.tab === 'providers' ? (
            <AddButton onClick={state.dialog.openCreate}>{state.t('actions.addProvider')}</AddButton>
          ) : null}
        </Stack>
      }
    />
  );
}

function ProviderTabs({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  return (
    <Tabs value={state.tab} onChange={(_event, next: ProviderTab) => state.setTab(next)} sx={{ mb: 3 }}>
      <Tab value="providers" label={state.t('providers.providerListTab')} />
      <Tab value="groups" label={state.t('providers.groupsTab')} />
      <Tab value="cooldowns" label={state.t('providers.cooldownsTab')} />
    </Tabs>
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
        onOpenPriority={state.openPriorityDialog}
        onOpenCooldownPolicy={() => state.setCooldownPolicyOpen(true)}
      />
      <ProviderTable
        rows={state.providers.items}
        groups={state.providerGroups.items}
        total={state.providers.total}
        loading={state.providers.isLoading}
        table={state.table}
        selectedId={state.bindingsOpen ? state.selectedProvider?.id : undefined}
        onSelect={state.openProviderBindings}
        onEdit={state.dialog.openEdit}
        onDelete={state.deleteDialog.setDeleteTarget}
        onAssociateGroups={state.providerGroupAssociation.openForProvider}
      />
    </Card>
  );
}

function ProviderGroupsCardWrapper({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  return (
    <ProviderGroupsCard
      providerGroups={state.providerGroups}
      providerKeyGroups={state.providerKeyGroups}
      providers={state.priorityProviders.items}
      keysByProvider={state.priorityKeys.itemsByProvider}
    />
  );
}

function ProviderCooldownCard({ state }: { state: ReturnType<typeof useProviderManagementState> }) {
  return (
    <Card>
      <ProviderCooldownTable
        rows={state.cooldowns.items}
        total={state.cooldowns.total}
        loading={state.cooldowns.isLoading}
        table={state.cooldownTable}
        filters={state.cooldownFilters}
        locale={state.currentLang.numberFormat.code}
        releasingId={state.releasingCooldownId}
        onFiltersChange={state.handleCooldownFiltersChange}
        onRelease={state.releaseCooldown}
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
        providerKeyGroups={state.providerKeyGroups.items}
        dialogs={state.childDialogs}
        onAssociateKeyGroups={state.providerKeyGroupAssociation.openForKey}
        onClose={state.closeProviderBindings}
      />
      <ProviderGroupAssociationDialog
        kind="provider"
        open={state.providerGroupAssociation.open}
        targetName={state.providerGroupAssociation.target?.name ?? ''}
        groups={state.providerGroups.items}
        selectedIds={state.providerGroupAssociation.selectedIds}
        submitting={state.providerGroupAssociation.submitting}
        onClose={state.providerGroupAssociation.close}
        onSubmit={state.providerGroupAssociation.submit}
        onSelectedIdsChange={state.providerGroupAssociation.setSelectedIds}
      />
      <ProviderGroupAssociationDialog
        kind="key"
        open={state.providerKeyGroupAssociation.open}
        targetName={state.providerKeyGroupAssociation.target?.name ?? ''}
        groups={state.providerKeyGroups.items}
        selectedIds={state.providerKeyGroupAssociation.selectedIds}
        submitting={state.providerKeyGroupAssociation.submitting}
        onClose={state.providerKeyGroupAssociation.close}
        onSubmit={state.providerKeyGroupAssociation.submit}
        onSelectedIdsChange={state.providerKeyGroupAssociation.setSelectedIds}
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
        keysByProvider={state.priorityKeys.itemsByProvider}
        loading={state.priorityProviders.isLoading || state.priorityKeys.isLoading}
        schedulingMode={state.settings.data?.scheduling_mode ?? 'cache_affinity'}
        priorityMode={state.settings.data?.provider_priority_mode ?? 'provider'}
        keyPrioritySnapshotInitialized={state.settings.data?.key_priority_snapshot_initialized ?? false}
        cacheAffinityTtlMinutes={state.settings.data?.cache_affinity_ttl_minutes ?? 5}
        onClose={() => state.setPriorityOpen(false)}
        onSaved={() => {
          void state.providers.refresh();
          void state.priorityProviders.refresh();
          void state.priorityKeys.refresh();
          void state.settings.refresh();
        }}
      />
      <ProviderCooldownPolicyDialog
        open={state.cooldownPolicyOpen}
        policy={state.settings.data?.provider_cooldown_policy}
        onClose={() => state.setCooldownPolicyOpen(false)}
        onSaved={() => {
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

function ErrorAlert({ message }: { message: string }) {
  return <Alert severity="error" sx={{ mb: 3 }}>{message}</Alert>;
}
