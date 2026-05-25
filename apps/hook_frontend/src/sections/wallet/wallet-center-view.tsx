'use client';

import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useWalletDailyModelUsage } from 'src/actions/wallet';
import { useUserRechargeOrders, useUserRechargePackages } from 'src/actions/recharge';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import { DASHBOARD_MENU_CODES, DASHBOARD_SECTION_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { WalletSummaryCards } from './wallet-summary-cards';
import { useWalletCenterState } from './wallet-center-state';
import { WalletLedgerSection } from './wallet-ledger-section';
import { WalletDepositSection } from './wallet-deposit-section';
import { useWalletLedgerExpansion } from './wallet-ledger-expansion';
import { useWalletDepositActions } from './use-wallet-deposit-actions';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';

export function WalletCenterView() {
  const { t, currentLang } = useTranslate('admin');
  const state = useWalletCenterState(t);
  const packages = useUserRechargePackages();
  const orders = useUserRechargeOrders();
  const locale = currentLang.numberFormat.code;
  const entryExpansion = useWalletLedgerExpansion();
  const detail = useWalletDailyModelUsage(entryExpansion.date, entryExpansion.page, entryExpansion.pageSize);
  const deposit = useWalletDepositActions({ t, refreshWallet: state.refresh, refreshOrders: orders.refresh });
  const refreshRecharge = () => {
    void packages.refresh();
    void orders.refresh();
  };

  return (
    <DashboardContent maxWidth="xl">
      <WalletBreadcrumbs t={t} loading={state.validating} onRefresh={state.refresh} />

      {state.errorMessage ? <WalletErrorAlert message={state.errorMessage} /> : null}
      {packages.error ? <WalletErrorAlert message={packages.error.message} /> : null}
      {orders.error ? <WalletErrorAlert message={orders.error.message} /> : null}

      <WalletSummaryCards t={t} wallet={state.wallet} />

      <WalletDepositSection
        t={t}
        locale={locale}
        rechargeLoading={packages.isLoading}
        rechargeEnabled={packages.data?.recharge_enabled ?? false}
        ratio={packages.data?.arrival_ratio ?? 1}
        packages={packages.data?.items ?? []}
        purchasingId={deposit.purchasingId}
        cardCode={deposit.redeemCode}
        redeeming={deposit.redeeming}
        onPurchase={(item) => void deposit.purchasePackage(item.id)}
        onRefreshRecharge={refreshRecharge}
        onCardCodeChange={deposit.setRedeemCode}
        onRedeemCardCode={() => void deposit.submitRedeemCode()}
      />

      <WalletLedgerSection
        t={t}
        wallet={state.wallet}
        locale={locale}
        rechargeOrders={orders.data?.items ?? []}
        loading={state.loading}
        filters={state.filters}
        hasFilters={state.hasFilters}
        reasonOptions={state.filterOptions.reasons}
        linkTypeOptions={state.filterOptions.linkTypes}
        items={state.filteredItems}
        total={state.entries.data?.total ?? 0}
        loadedCount={state.entries.data?.items.length ?? 0}
        page={state.table.page}
        rowsPerPage={state.table.rowsPerPage}
        expansion={entryExpansion.expansionState(detail)}
        onFilterChange={state.changeFilters}
        onOpen={state.setCurrentTransaction}
        onToggleDailyUsage={entryExpansion.toggle}
        onDailyUsagePageChange={entryExpansion.changePage}
        onPageChange={state.table.onChangePage}
        onRowsPerPageChange={state.table.onChangeRowsPerPage}
      />

      <WalletTransactionDetailDialog
        t={t}
        wallet={state.wallet}
        locale={locale}
        transaction={state.currentTransaction}
        open={Boolean(state.currentTransaction)}
        onClose={() => state.setCurrentTransaction(null)}
      />
    </DashboardContent>
  );
}

function WalletBreadcrumbs({
  t,
  loading,
  onRefresh,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  loading: boolean;
  onRefresh: VoidFunction;
}) {
  const breadcrumbs = useDashboardBreadcrumbs({
    headingCode: DASHBOARD_MENU_CODES.walletCenter,
    sectionCode: DASHBOARD_SECTION_CODES.operations,
  });

  return (
    <CustomBreadcrumbs
      heading={breadcrumbs.heading}
      links={breadcrumbs.links}
      action={
        <Button
          variant="outlined"
          loading={loading}
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
        >
          {t('models.refresh')}
        </Button>
      }
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

function WalletErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}
