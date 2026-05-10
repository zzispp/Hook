'use client';

import type { TokenScope } from './api-token-management-types';

import Card from '@mui/material/Card';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  DASHBOARD_MENU_TITLES,
  DASHBOARD_MENU_SECTIONS,
} from 'src/layouts/dashboard/dashboard-menu-values';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import { useAvailableBillingGroups } from 'src/actions/groups';

import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { RefreshAddActions } from '../admin/admin-page-actions';
import { defaultGroupCode } from './api-token-management-utils';
import { TokenManagementPanel, useTokenManagementPanelState } from './token-management-panel';

export function ApiTokenManagementView() {
  return <TokenManagementView scope="user" />;
}

export function AdminApiTokenManagementView() {
  return <TokenManagementView scope="admin" />;
}

function TokenManagementView({ scope }: { scope: TokenScope }) {
  const groups = useAvailableBillingGroups();
  const panel = useTokenManagementPanelState({ scope });

  return (
    <DashboardContent maxWidth="xl">
      <TokenBreadcrumbs
        scope={scope}
        loading={panel.tokens.isLoading}
        onAdd={() => panel.dialog.openCreate(defaultGroupCode(groups.items))}
        onRefresh={() => void panel.tokens.refresh()}
      />
      <Card>
        <TokenManagementPanel state={panel} />
      </Card>
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
  const breadcrumbs = useDashboardBreadcrumbs({
    heading: isAdmin ? DASHBOARD_MENU_TITLES.tokenManagement : DASHBOARD_MENU_TITLES.apiTokens,
    section: DASHBOARD_MENU_SECTIONS.operations,
  });

  return (
    <CustomBreadcrumbs
      heading={breadcrumbs.heading}
      links={breadcrumbs.links}
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
