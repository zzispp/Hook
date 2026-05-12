'use client';

import type { ApiTokenFilters } from 'src/actions/api-tokens';
import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { TokenFilterState } from './api-token-filters-toolbar';
import type {
  TokenScope,
  TokenModelOption,
  BillingGroupOption,
} from './api-token-management-types';

import { useState, useCallback } from 'react';

import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';

import { useUsers } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';
import { useUserModelCatalog } from 'src/actions/models';
import { useScopedApiTokens } from 'src/actions/api-tokens';
import { useAvailableBillingGroups } from 'src/actions/groups';
import { useSystemSettings, useUsdCnyExchangeRate } from 'src/actions/system-settings';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { ApiTokenTable } from './api-token-table';
import { ApiTokenDialog } from './api-token-dialog';
import { ApiTokenCreatedDialog } from './api-token-created-dialog';
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
  const users = useUsers(
    scope === 'admin' && !fixedUserId ? 0 : -1,
    scope === 'admin' && !fixedUserId ? 100 : 0
  );
  const dialog = useTokenDialog(scope, t, groups.items, fixedUserId ?? '');
  const deleteDialog = useDeleteDialog(scope, t);
  const copyToken = useCopyToken(scope, t);
  const currency = useTokenCurrencyState(scope === 'admin' && !disabled, t);
  const handleFiltersChange = useCallback(
    (nextFilters: TokenFilterState) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return {
    copyToken,
    currency,
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
      {state.currency.exchangeRateError ? (
        <Alert severity="error" sx={{ m: 2.5, mb: 0 }}>
          {t('requestRecords.exchangeRateLoadFailed')}
        </Alert>
      ) : null}
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
        currencyDisplay={state.currency.display}
        onCopy={state.copyToken}
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

function useTokenCurrencyState(enabled: boolean, t: (key: string) => string) {
  const settings = useSystemSettings(enabled);
  const exchangeRate = useUsdCnyExchangeRate(enabled && settings.data?.currency === 'CNY');
  const display: CurrencyDisplay | undefined = enabled
    ? {
        currency: settings.data?.currency ?? 'USD',
        usdCnyRate: exchangeRate.data,
        unavailableLabel: t('requestRecords.exchangeRateUnavailable'),
      }
    : undefined;

  return {
    display,
    exchangeRateError: Boolean(enabled && settings.data?.currency === 'CNY' && exchangeRate.error),
  };
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
