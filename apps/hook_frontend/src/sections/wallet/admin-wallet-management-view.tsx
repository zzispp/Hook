'use client';

import type { AdminWallet } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAdminWallets, adjustAdminWallet } from 'src/actions/wallet';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AdminWalletTable } from './admin-wallet-table';
import { AdminWalletLedgerDialog } from './admin-wallet-ledger-dialog';
import { AdminWalletAdjustDialog } from './admin-wallet-adjust-dialog';
import {
  toAdminWalletFilters,
  AdminWalletFiltersToolbar,
  DEFAULT_ADMIN_WALLET_FILTERS,
} from './admin-wallet-filters';

type WalletManagementBodyProps = {
  t: ReturnType<typeof useTranslate>['t'];
  table: ReturnType<typeof useTable>;
  wallets: ReturnType<typeof useAdminWallets>;
  filters: typeof DEFAULT_ADMIN_WALLET_FILTERS;
  locale: string;
  adjustDialog: ReturnType<typeof useAdjustDialog>;
  ledgerWallet: AdminWallet | null;
  setLedgerWallet: React.Dispatch<React.SetStateAction<AdminWallet | null>>;
  handleFiltersChange: (filters: typeof DEFAULT_ADMIN_WALLET_FILTERS) => void;
};

export function AdminWalletManagementView() {
  const { t, currentLang } = useTranslate('admin');
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

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.walletManagement')}
        action={<RefreshButton loading={wallets.isLoading} onClick={() => void wallets.refresh()} />}
      />
      {wallets.error ? <WalletErrorAlert message={wallets.error.message} /> : null}
      <WalletManagementBody
        t={t}
        table={table}
        wallets={wallets}
        filters={filters}
        locale={currentLang.numberFormat.code}
        adjustDialog={adjustDialog}
        ledgerWallet={ledgerWallet}
        setLedgerWallet={setLedgerWallet}
        handleFiltersChange={handleFiltersChange}
      />
    </DashboardContent>
  );
}

function WalletManagementBody({
  t,
  table,
  wallets,
  filters,
  locale,
  adjustDialog,
  ledgerWallet,
  setLedgerWallet,
  handleFiltersChange,
}: WalletManagementBodyProps) {
  return (
    <>
      <AdminWalletCard
        t={t}
        table={table}
        wallets={wallets}
        filters={filters}
        locale={locale}
        onOpenLedger={setLedgerWallet}
        onOpenAdjust={adjustDialog.open}
        onFiltersChange={handleFiltersChange}
      />
      <AdminWalletDialogs
        t={t}
        locale={locale}
        adjustDialog={adjustDialog}
        ledgerWallet={ledgerWallet}
        onCloseLedger={() => setLedgerWallet(null)}
      />
    </>
  );
}

function AdminWalletCard({
  t,
  table,
  wallets,
  filters,
  locale,
  onOpenLedger,
  onOpenAdjust,
  onFiltersChange,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  table: ReturnType<typeof useTable>;
  wallets: ReturnType<typeof useAdminWallets>;
  filters: typeof DEFAULT_ADMIN_WALLET_FILTERS;
  locale: string;
  onOpenLedger: React.Dispatch<React.SetStateAction<AdminWallet | null>>;
  onOpenAdjust: (wallet: AdminWallet) => void;
  onFiltersChange: (filters: typeof DEFAULT_ADMIN_WALLET_FILTERS) => void;
}) {
  return (
    <Card>
      <AdminWalletFiltersToolbar t={t} filters={filters} onChange={onFiltersChange} />
      <AdminWalletTable
        t={t}
        locale={locale}
        rows={wallets.data?.items ?? []}
        total={wallets.data?.total ?? 0}
        loading={wallets.isLoading}
        page={table.page}
        rowsPerPage={table.rowsPerPage}
        onOpenLedger={onOpenLedger}
        onOpenAdjust={onOpenAdjust}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </Card>
  );
}

function AdminWalletDialogs({
  t,
  locale,
  adjustDialog,
  ledgerWallet,
  onCloseLedger,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  adjustDialog: ReturnType<typeof useAdjustDialog>;
  ledgerWallet: AdminWallet | null;
  onCloseLedger: VoidFunction;
}) {
  return (
    <>
      <AdminWalletLedgerDialog
        t={t}
        locale={locale}
        wallet={ledgerWallet}
        onClose={onCloseLedger}
      />
      <AdminWalletAdjustDialog
        t={t}
        open={Boolean(adjustDialog.wallet)}
        wallet={adjustDialog.wallet}
        submitting={adjustDialog.submitting}
        onClose={adjustDialog.close}
        onSubmit={adjustDialog.submit}
      />
    </>
  );
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
