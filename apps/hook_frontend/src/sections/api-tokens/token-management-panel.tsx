'use client';

import type { ApiTokenFilters } from 'src/actions/api-tokens';
import type { TokenFilterState } from './api-token-filters-toolbar';
import type {
  TokenScope,
  TokenModelOption,
  BillingGroupOption,
} from './api-token-management-types';

import { useState, useCallback } from 'react';

import Button from '@mui/material/Button';

import { useUsers } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';
import { useUserModelCatalog } from 'src/actions/models';
import { useSiteInfo } from 'src/actions/system-settings';
import { useScopedApiTokens } from 'src/actions/api-tokens';
import { useAvailableBillingGroups } from 'src/actions/groups';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { ApiTokenTable } from './api-token-table';
import { ApiTokenDialog } from './api-token-dialog';
import { ApiTokenCreatedDialog } from './api-token-created-dialog';
import { ApiTokenCcSwitchDialog } from './api-token-cc-switch-dialog';
import { useCcSwitchImportDialog } from './api-token-cc-switch-state';
import {
  toApiTokenFilters,
  DEFAULT_TOKEN_FILTERS,
  ApiTokenFiltersToolbar,
} from './api-token-filters-toolbar';
import {
  useCopyToken,
  useTokenDialog,
  useDeleteDialog,
  toggleTokenAndNotify,
} from './api-token-management-state';

type Props = {
  state: TokenManagementPanelState;
};

export type TokenManagementPanelState = ReturnType<typeof useTokenManagementPanelState>;

export function useTokenManagementPanelState({
  scope,
  fixedUserId,
  disabled = false,
}: {
  scope: TokenScope;
  fixedUserId?: string;
  disabled?: boolean;
}) {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const [filters, setFilters] = useState(DEFAULT_TOKEN_FILTERS);
  const requestFilters = tokenFilters(filters, fixedUserId);
  const tokens = useScopedApiTokens(
    scope,
    disabled ? -1 : table.page,
    table.rowsPerPage,
    requestFilters
  );
  const groups = useAvailableBillingGroups();
  const models = useUserModelCatalog();
  const site = useSiteInfo();
  const users = useUsers(
    scope === 'admin' && !fixedUserId ? 0 : -1,
    scope === 'admin' && !fixedUserId ? 100 : 0
  );
  const dialog = useTokenDialog(scope, t, groups.items, fixedUserId ?? '');
  const ccSwitchImport = useCcSwitchImportDialog({
    scope,
    t,
    catalog: models.items,
    siteName: site.data?.site_name,
  });
  const deleteDialog = useDeleteDialog(scope, t);
  const copyToken = useCopyToken(scope, t);
  const handleFiltersChange = useCallback(
    (nextFilters: TokenFilterState) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return {
    copyToken,
    ccSwitchImport,
    deleteDialog,
    dialog,
    filters,
    fixedUserId,
    groups,
    handleFiltersChange,
    models,
    scope,
    table,
    tokens,
    users,
  };
}

export function TokenManagementPanel({ state }: Props) {
  const { t } = useTranslate('admin');

  return (
    <>
      <ApiTokenFiltersToolbar
        filters={state.filters}
        showTokenType={state.scope === 'admin' && !state.fixedUserId}
        onChange={state.handleFiltersChange}
      />
      <ApiTokenTable
        rows={state.tokens.items}
        total={state.tokens.total}
        loading={state.tokens.isLoading}
        table={state.table}
        showOwner={state.scope === 'admin' && !state.fixedUserId}
        onCopy={state.copyToken}
        onImportCcSwitch={state.ccSwitchImport.openImport}
        onEdit={state.dialog.openEdit}
        onToggle={(token) => void toggleTokenAndNotify(state.scope, token, t)}
        onDelete={state.deleteDialog.setDeleteTarget}
      />
      <ApiTokenDialog
        scope={state.scope}
        dialog={state.dialog}
        groups={state.groups.items}
        models={modelsForGroup(
          state.models.items,
          state.groups.items,
          state.dialog.form.group_code
        )}
        users={state.users.items}
        fixedUserId={state.fixedUserId}
      />
      <ApiTokenCreatedDialog
        rawToken={state.dialog.createdToken}
        onClose={state.dialog.closeCreatedToken}
      />
      <ApiTokenCcSwitchDialog state={state.ccSwitchImport} />
      <ConfirmDialog
        open={!!state.deleteDialog.deleteTarget}
        onClose={() => state.deleteDialog.setDeleteTarget(null)}
        title={t('dialogs.deleteApiToken')}
        content={t('tokens.deleteConfirm', { name: state.deleteDialog.deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={state.deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}

function tokenFilters(filters: TokenFilterState, fixedUserId?: string): ApiTokenFilters {
  return toApiTokenFilters(filters, fixedUserId);
}

function modelsForGroup(
  models: TokenModelOption[],
  groups: BillingGroupOption[],
  groupCode: string
) {
  const allowedIds = groups.find((group) => group.code === groupCode)?.allowed_model_ids ?? [];
  if (allowedIds.length === 0) {
    return models;
  }
  return models.filter((model) => allowedIds.includes(model.id));
}
