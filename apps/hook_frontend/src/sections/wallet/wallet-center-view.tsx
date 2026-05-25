'use client';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { redeemCardCode } from 'src/actions/card-code';
import { DashboardContent } from 'src/layouts/dashboard';
import { useWalletDailyModelUsage } from 'src/actions/wallet';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import {
  DASHBOARD_MENU_CODES,
  DASHBOARD_SECTION_CODES,
} from 'src/layouts/dashboard/dashboard-menu-values';
import {
  useUserRechargeOrders,
  createUserRechargeOrder,
  useUserRechargePackages,
} from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { WalletSummaryCards } from './wallet-summary-cards';
import { useWalletCenterState } from './wallet-center-state';
import { WalletRechargePanel } from './wallet-recharge-panel';
import { WalletLedgerFilters } from './wallet-ledger-filters';
import { useWalletLedgerExpansion } from './wallet-ledger-expansion';
import { WalletLedgerEntriesTable } from './wallet-ledger-entries-table';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';

type WalletTab = 'recharge' | 'ledger';

export function WalletCenterView() {
  const { t, currentLang } = useTranslate('admin');
  const [activeTab, setActiveTab] = useState<WalletTab>('recharge');
  const [redeemCode, setRedeemCode] = useState('');
  const [redeeming, setRedeeming] = useState(false);
  const [purchasingId, setPurchasingId] = useState<string | null>(null);
  const state = useWalletCenterState(t);
  const packages = useUserRechargePackages();
  const orders = useUserRechargeOrders();
  const locale = currentLang.numberFormat.code;
  const entryExpansion = useWalletLedgerExpansion();
  const detail = useWalletDailyModelUsage(entryExpansion.date, entryExpansion.page, entryExpansion.pageSize);
  const submitRedeemCode = async () => {
    if (!redeemCode.trim()) return;
    setRedeeming(true);
    try {
      await redeemCardCode({ code: redeemCode.trim() });
      toast.success(t('wallet.cardCode.redeemed'));
      setRedeemCode('');
      state.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setRedeeming(false);
    }
  };
  const purchasePackage = async (packageId: string) => {
    setPurchasingId(packageId);
    try {
      await createUserRechargeOrder({ package_id: packageId });
      toast.success(t('wallet.recharge.orderCreated'));
      await orders.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setPurchasingId(null);
    }
  };
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

      <Card sx={{ mb: 3, p: 2.5 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextField
            fullWidth
            label={t('wallet.cardCode.code')}
            value={redeemCode}
            onChange={(event) => setRedeemCode(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                void submitRedeemCode();
              }
            }}
          />
          <Button
            variant="contained"
            loading={redeeming}
            disabled={!redeemCode.trim()}
            startIcon={<Iconify icon="solar:bill-list-bold" />}
            onClick={() => void submitRedeemCode()}
            sx={{ minWidth: 140 }}
          >
            {t('wallet.cardCode.redeem')}
          </Button>
        </Stack>
      </Card>

      <Card>
        <WalletTabs t={t} activeTab={activeTab} onChange={setActiveTab} />
        {activeTab === 'recharge' ? (
          <WalletRechargePanel
            t={t}
            locale={locale}
            loading={packages.isLoading}
            enabled={packages.data?.recharge_enabled ?? false}
            ratio={packages.data?.arrival_ratio ?? 1}
            packages={packages.data?.items ?? []}
            orders={orders.data?.items ?? []}
            purchasingId={purchasingId}
            onPurchase={(item) => void purchasePackage(item.id)}
            onRefresh={refreshRecharge}
          />
        ) : null}
        {activeTab === 'ledger' ? (
          <>
            <WalletLedgerFilters
              t={t}
              filters={state.filters}
              hasFilters={state.hasFilters}
              reasonOptions={state.filterOptions.reasons}
              linkTypeOptions={state.filterOptions.linkTypes}
              onChange={state.changeFilters}
            />
            <WalletLedgerEntriesTable
              t={t}
              wallet={state.wallet}
              locale={locale}
              loading={state.loading}
              items={state.filteredItems}
              total={state.entries.data?.total ?? 0}
              loadedCount={state.entries.data?.items.length ?? 0}
              page={state.table.page}
              rowsPerPage={state.table.rowsPerPage}
              expansion={entryExpansion.expansionState(detail)}
              onOpen={state.setCurrentTransaction}
              onToggleDailyUsage={entryExpansion.toggle}
              onDailyUsagePageChange={entryExpansion.changePage}
              onPageChange={state.table.onChangePage}
              onRowsPerPageChange={state.table.onChangeRowsPerPage}
            />
          </>
        ) : null}
      </Card>

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

function WalletTabs({
  t,
  activeTab,
  onChange,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  activeTab: WalletTab;
  onChange: (tab: WalletTab) => void;
}) {
  return (
    <Stack spacing={1.5} sx={{ px: 2.5, pt: 2.5, pb: 2 }}>
      <Typography variant="h6">{t('wallet.title')}</Typography>
      <Tabs value={activeTab} onChange={(_, value: WalletTab) => onChange(value)}>
        <Tab value="recharge" label={t('wallet.tabs.recharge')} />
        <Tab value="ledger" label={t('wallet.tabs.ledger')} />
      </Tabs>
    </Stack>
  );
}
