'use client';

import { useState } from 'react';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useCaptchaConfig } from 'src/actions/captcha';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useWalletDailyModelUsage } from 'src/actions/wallet';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import { DASHBOARD_MENU_CODES, DASHBOARD_SECTION_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import {
  useUserRechargeOrders,
  useUserPaymentChannels,
  useUserRechargePackages,
} from 'src/actions/recharge';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { WalletSummaryCards } from './wallet-summary-cards';
import { useWalletCenterState } from './wallet-center-state';
import { WalletLedgerSection } from './wallet-ledger-section';
import { WalletRechargeDialog } from './wallet-recharge-panel';
import { WalletCardCodeDialog } from './wallet-card-code-panel';
import { useWalletLedgerExpansion } from './wallet-ledger-expansion';
import { useWalletDepositActions } from './use-wallet-deposit-actions';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';

export function WalletCenterView() {
  const { t, currentLang } = useTranslate('admin');
  const [rechargeOpen, setRechargeOpen] = useState(false);
  const [cardCodeOpen, setCardCodeOpen] = useState(false);
  const state = useWalletCenterState(t);
  const packages = useUserRechargePackages();
  const orders = useUserRechargeOrders();
  const paymentChannels = useUserPaymentChannels();
  const captcha = useCaptchaConfig();
  const locale = currentLang.numberFormat.code;
  const rechargeEnabled = packages.data?.recharge_enabled === true;
  const entryExpansion = useWalletLedgerExpansion();
  const detail = useWalletDailyModelUsage(entryExpansion.date, entryExpansion.page, entryExpansion.pageSize);
  const deposit = useWalletDepositActions({
    t,
    rechargeOpen: rechargeOpen && rechargeEnabled,
    refreshWallet: state.refresh,
    refreshOrders: orders.refresh,
  });
  const refreshRecharge = () => {
    void packages.refresh();
    void orders.refresh();
  };
  const redeemCardCode = async () => {
    const redeemed = await deposit.submitRedeemCode();
    if (redeemed) {
      setCardCodeOpen(false);
    }
  };

  return (
    <DashboardContent maxWidth="xl">
      <WalletBreadcrumbs
        t={t}
        loading={state.validating}
        rechargeEnabled={rechargeEnabled}
        onRefresh={() => {
          void state.refresh();
        }}
        onRecharge={() => setRechargeOpen(true)}
        onRedeemCardCode={() => setCardCodeOpen(true)}
      />

      {state.errorMessage ? <WalletErrorAlert message={state.errorMessage} /> : null}
      {packages.error ? <WalletErrorAlert message={packages.error.message} /> : null}
      {orders.error ? <WalletErrorAlert message={orders.error.message} /> : null}
      {paymentChannels.error ? <WalletErrorAlert message={paymentChannels.error.message} /> : null}
      {captcha.error ? <WalletErrorAlert message={captcha.error.message} /> : null}

      <WalletSummaryCards t={t} wallet={state.wallet} />

      <WalletRechargeDialog
        t={t}
        open={rechargeOpen && rechargeEnabled}
        loading={packages.isLoading}
        enabled={rechargeEnabled}
        ratio={packages.data?.arrival_ratio ?? 1}
        minAmount={packages.data?.min_amount ?? 0.01}
        maxAmount={packages.data?.max_amount ?? 3000}
        packages={packages.data?.items ?? []}
        channels={paymentChannels.data ?? []}
        channelsLoading={paymentChannels.isLoading}
        captchaConfig={{
          enabled: captcha.data?.recharge_captcha_enabled,
          loading: captcha.isLoading,
          errorMessage: captcha.error?.message,
        }}
        purchasingId={deposit.purchasingId}
        pendingPaymentOrderNo={deposit.pendingPaymentOrderNo}
        checkingPayment={deposit.checkingPayment}
        onClose={() => setRechargeOpen(false)}
        onPurchaseAmount={(amount, channelCode, methodCode, captchaToken) =>
          void deposit.purchaseAmount(amount, channelCode, methodCode, captchaToken)
        }
        onPurchasePackage={(item, channelCode, methodCode, captchaToken) =>
          void deposit.purchasePackage(item.id, channelCode, methodCode, captchaToken)
        }
        onRefresh={refreshRecharge}
        onCheckPayment={() => void deposit.checkPendingPayment()}
      />

      <WalletCardCodeDialog
        t={t}
        open={cardCodeOpen}
        code={deposit.redeemCode}
        redeeming={deposit.redeeming}
        onClose={() => setCardCodeOpen(false)}
        onCodeChange={deposit.setRedeemCode}
        onRedeem={() => void redeemCardCode()}
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
  rechargeEnabled,
  onRefresh,
  onRecharge,
  onRedeemCardCode,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  loading: boolean;
  rechargeEnabled: boolean;
  onRefresh: VoidFunction;
  onRecharge: VoidFunction;
  onRedeemCardCode: VoidFunction;
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
        <Stack direction="row" spacing={1}>
          {rechargeEnabled ? (
            <Button
              variant="contained"
              startIcon={<Iconify icon="solar:wad-of-money-bold" />}
              onClick={onRecharge}
            >
              {t('wallet.recharge.entry')}
            </Button>
          ) : null}
          <Button
            variant="outlined"
            startIcon={<Iconify icon="solar:bill-list-bold" />}
            onClick={onRedeemCardCode}
          >
            {t('wallet.cardCode.redeem')}
          </Button>
          <Button
            variant="outlined"
            loading={loading}
            startIcon={<Iconify icon="solar:restart-bold" />}
            onClick={onRefresh}
          >
            {t('models.refresh')}
          </Button>
        </Stack>
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
