'use client';

import type { TokenScope, TokenModelOption, BillingGroupOption } from './api-token-management-types';

import Card from '@mui/material/Card';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';

import { useUsers } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';
import { useUserModelCatalog } from 'src/actions/models';
import { DashboardContent } from 'src/layouts/dashboard';
import { useScopedApiTokens } from 'src/actions/api-tokens';
import { useAvailableBillingGroups } from 'src/actions/groups';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { ApiTokenTable } from './api-token-table';
import { ApiTokenDialog } from './api-token-dialog';
import { RefreshAddActions } from '../admin/admin-page-actions';
import { defaultGroupCode } from './api-token-management-utils';
import { ApiTokenCreatedDialog } from './api-token-created-dialog';
import { toggleToken, useCopyToken, useTokenDialog, useDeleteDialog } from './api-token-management-state';

export function ApiTokenManagementView() {
  return <TokenManagementView scope="user" />;
}

export function AdminApiTokenManagementView() {
  return <TokenManagementView scope="admin" />;
}

function TokenManagementView({ scope }: { scope: TokenScope }) {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const tokens = useScopedApiTokens(scope, table.page, table.rowsPerPage);
  const groups = useAvailableBillingGroups();
  const models = useUserModelCatalog();
  const users = useUsers(scope === 'admin' ? 0 : -1, scope === 'admin' ? 100 : 0);
  const dialog = useTokenDialog(scope, t, groups.items);
  const deleteDialog = useDeleteDialog(scope, t);
  const copyToken = useCopyToken(scope, t);

  return (
    <DashboardContent maxWidth={scope === 'user' ? 'xl' : 'lg'}>
      <TokenBreadcrumbs
        scope={scope}
        loading={tokens.isLoading}
        onAdd={() => dialog.openCreate(defaultGroupCode(groups.items))}
        onRefresh={() => void tokens.refresh()}
      />
      <Card>
        <ApiTokenTable
          rows={tokens.items}
          total={tokens.total}
          loading={tokens.isLoading}
          table={table}
          showOwner={scope === 'admin'}
          onCopy={copyToken}
          onEdit={dialog.openEdit}
          onToggle={(token) => void toggleAndNotify(scope, token, t)}
          onDelete={deleteDialog.setDeleteTarget}
        />
      </Card>
      <ApiTokenDialog
        scope={scope}
        dialog={dialog}
        groups={groups.items}
        models={modelsForGroup(models.items, groups.items, dialog.form.group_code)}
        users={users.items}
      />
      <ApiTokenCreatedDialog rawToken={dialog.createdToken} onClose={dialog.closeCreatedToken} />
      <ConfirmDialog
        open={!!deleteDialog.deleteTarget}
        onClose={() => deleteDialog.setDeleteTarget(null)}
        title={t('dialogs.deleteApiToken')}
        content={t('tokens.deleteConfirm', { name: deleteDialog.deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

function TokenBreadcrumbs({
  scope,
  loading,
  onAdd,
  onRefresh,
}: {
  scope: TokenScope;
  loading: boolean;
  onAdd: VoidFunction;
  onRefresh: VoidFunction;
}) {
  const { t } = useTranslate('admin');
  const isAdmin = scope === 'admin';

  return (
    <CustomBreadcrumbs
      heading={t(isAdmin ? 'pages.adminApiTokens' : 'pages.apiTokens')}
      links={[
        { name: t('nav.dashboard'), href: paths.dashboard.root },
        { name: t(isAdmin ? 'nav.systemManagement' : 'nav.resources') },
        { name: t(isAdmin ? 'pages.adminApiTokens' : 'pages.apiTokens') },
      ]}
      action={
        <RefreshAddActions
          loading={loading}
          addLabel={t('actions.addApiToken')}
          onAdd={onAdd}
          onRefresh={onRefresh}
        />
      }
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

async function toggleAndNotify(scope: TokenScope, token: Parameters<typeof toggleToken>[1], t: (key: string) => string) {
  try {
    await toggleToken(scope, token);
    toast.success(t('messages.apiTokenUpdated'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
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
