'use client';

import type { AdminWallet } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAdminWallets, adjustAdminWallet } from 'src/actions/wallet';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AdminWalletTable } from './admin-wallet-table';
import { AdminWalletGlobalLedger } from './admin-wallet-global-ledger';
import { AdminWalletLedgerDialog } from './admin-wallet-ledger-dialog';
import { AdminWalletAdjustDialog } from './admin-wallet-adjust-dialog';
import {
  toAdminWalletFilters,
  AdminWalletFiltersToolbar,
  DEFAULT_ADMIN_WALLET_FILTERS,
} from './admin-wallet-filters';

type AdminWalletTab = 'ledger' | 'wallets';

export function AdminWalletManagementView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<AdminWalletTab>('ledger');
  const walletState = useAdminWalletListState(t);
  const locale = currentLang.numberFormat.code;

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.walletManagement}
        action={tab === 'wallets' ? <RefreshButton loading={walletState.wallets.isLoading} onClick={() => void walletState.wallets.refresh()} /> : null}
      />
      {walletState.wallets.error ? <WalletErrorAlert message={walletState.wallets.error.message} /> : null}
      <AdminWalletTabs t={t} value={tab} onChange={setTab} />
      {tab === 'ledger' ? <AdminWalletGlobalLedger t={t} locale={locale} /> : null}
      {tab === 'wallets' ? <AdminWalletListPanel t={t} locale={locale} state={walletState} /> : null}
    </DashboardContent>
  );
}

function AdminWalletTabs({
  t,
  value,
  onChange,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  value: AdminWalletTab;
  onChange: (tab: AdminWalletTab) => void;
}) {
  return (
    <Tabs value={value} onChange={(_event, next: AdminWalletTab) => onChange(next)} sx={{ mb: 3 }}>
      <Tab value="ledger" label={t('adminWallets.tabs.ledger')} />
      <Tab value="wallets" label={t('adminWallets.tabs.wallets')} />
    </Tabs>
  );
}

function AdminWalletListPanel({
  t,
  locale,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useAdminWalletListState>;
}) {
  return (
    <>
      <Card>
        <AdminWalletFiltersToolbar t={t} filters={state.filters} onChange={state.handleFiltersChange} />
        <AdminWalletTable
          t={t}
          locale={locale}
          rows={state.wallets.data?.items ?? []}
          total={state.wallets.data?.total ?? 0}
          loading={state.wallets.isLoading}
          page={state.table.page}
          rowsPerPage={state.table.rowsPerPage}
          onOpenLedger={state.setLedgerWallet}
          onOpenAdjust={state.adjustDialog.open}
          onPageChange={state.table.onChangePage}
          onRowsPerPageChange={state.table.onChangeRowsPerPage}
        />
      </Card>
      <AdminWalletListDialogs t={t} locale={locale} state={state} />
    </>
  );
}

function AdminWalletListDialogs({
  t,
  locale,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useAdminWalletListState>;
}) {
  return (
    <>
      <AdminWalletLedgerDialog
        t={t}
        locale={locale}
        wallet={state.ledgerWallet}
        onClose={() => state.setLedgerWallet(null)}
      />
      <AdminWalletAdjustDialog
        t={t}
        open={Boolean(state.adjustDialog.wallet)}
        wallet={state.adjustDialog.wallet}
        submitting={state.adjustDialog.submitting}
        onClose={state.adjustDialog.close}
        onSubmit={state.adjustDialog.submit}
      />
    </>
  );
}

function useAdminWalletListState(t: ReturnType<typeof useTranslate>['t']) {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'updated_at' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_WALLET_FILTERS);
  const walletFilters = useMemo(() => toAdminWalletFilters(filters), [filters]);
  const wallets = useAdminWallets(table.page, table.rowsPerPage, walletFilters);
  const [ledgerWallet, setLedgerWallet] = useState<AdminWallet | null>(null);
  const adjustDialog = useAdjustDialog(t, wallets.refresh);
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_WALLET_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return { adjustDialog, filters, handleFiltersChange, ledgerWallet, setLedgerWallet, table, wallets };
}

function useAdjustDialog(t: ReturnType<typeof useTranslate>['t'], refresh: VoidFunction) {
  const [wallet, setWallet] = useState<AdminWallet | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const close = useCallback(() => setWallet(null), []);
  const open = useCallback((target: AdminWallet) => setWallet(target), []);
  const submit = useCallback(
    async (input: Parameters<typeof adjustAdminWallet>[1]) => {
      if (!wallet) return;
      setSubmitting(true);
      try {
        await adjustAdminWallet(wallet.id, input);
        toast.success(t('adminWallets.messages.adjusted'));
        refresh();
        close();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      } finally {
        setSubmitting(false);
      }
    },
    [close, refresh, t, wallet]
  );

  return { wallet, submitting, open, close, submit };
}

function WalletErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}
