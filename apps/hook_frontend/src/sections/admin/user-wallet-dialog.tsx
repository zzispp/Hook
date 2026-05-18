'use client';

import type { SystemUser } from 'src/types/rbac';
import type { WalletSummary, WalletTransaction } from 'src/types/wallet';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { useAdminUserWalletBalance, useAdminWalletTransactions } from 'src/actions/wallet';

import { Label } from 'src/components/label';
import { useTable } from 'src/components/table';
import { Iconify } from 'src/components/iconify';

import { WalletLedgerTable } from '../wallet/wallet-ledger-table';
import { DEFAULT_WALLET_ROWS_PER_PAGE } from '../wallet/wallet-constants';
import { formatWalletMoney, walletStatusLabel } from '../wallet/wallet-display';
import { WalletTransactionDetailDialog } from '../wallet/wallet-transaction-detail-dialog';
import {
  ManualRechargePanel,
  useManualRechargeForm,
} from './user-wallet-recharge-panel';

type Props = {
  user: SystemUser | null;
  onClose: VoidFunction;
  onChanged: VoidFunction;
};

type UserWalletTab = 'operation' | 'ledger' | 'refund';

export function UserWalletDialog({ user, onClose, onChanged }: Props) {
  const { t, currentLang } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const balance = useAdminUserWalletBalance(user?.id ?? null);
  const walletId = balance.data?.wallet.id ?? null;
  const ledger = useAdminWalletTransactions(walletId, table.page, table.rowsPerPage);
  const [tab, setTab] = useState<UserWalletTab>('operation');
  const [currentTransaction, setCurrentTransaction] = useState<WalletTransaction | null>(null);
  const form = useManualRechargeForm({
    walletId,
    onChanged,
    refreshBalance: balance.refresh,
    refreshLedger: ledger.refresh,
  });
  const wallet = ledger.data?.wallet ?? balance.data?.wallet;
  const locale = currentLang.numberFormat.code;

  return (
    <>
      <Dialog fullWidth maxWidth="xl" open={Boolean(user)} onClose={onClose}>
        <DialogTitle>
          <DialogHeader t={t} user={user} wallet={wallet} onClose={onClose} />
        </DialogTitle>
        <DialogContent sx={{ pb: 2 }}>
          <UserWalletContent
            tab={tab}
            form={form}
            table={table}
            wallet={wallet}
            ledger={ledger}
            locale={locale}
            onTabChange={setTab}
            onOpenTransaction={setCurrentTransaction}
          />
        </DialogContent>
        <DialogActions>
          <Button variant="outlined" onClick={onClose}>
            {t('common.close')}
          </Button>
        </DialogActions>
      </Dialog>
      <WalletTransactionDetailDialog
        t={t}
        locale={locale}
        wallet={wallet}
        transaction={currentTransaction}
        open={Boolean(currentTransaction)}
        onClose={() => setCurrentTransaction(null)}
      />
    </>
  );
}

function UserWalletContent({
  tab,
  form,
  table,
  wallet,
  ledger,
  locale,
  onTabChange,
  onOpenTransaction,
}: {
  tab: UserWalletTab;
  form: ReturnType<typeof useManualRechargeForm>;
  table: ReturnType<typeof useTable>;
  wallet?: WalletSummary | null;
  ledger: ReturnType<typeof useAdminWalletTransactions>;
  locale: string;
  onTabChange: (tab: UserWalletTab) => void;
  onOpenTransaction: (transaction: WalletTransaction) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={3}>
      <WalletMetrics wallet={wallet} />
      <Tabs value={tab} onChange={(_event, value: UserWalletTab) => onTabChange(value)}>
        <Tab value="operation" label={t('userWallet.tabs.operation')} />
        <Tab value="ledger" label={t('userWallet.tabs.ledger')} />
        <Tab value="refund" label={t('userWallet.tabs.refund')} />
      </Tabs>
      {tab === 'operation' ? <ManualRechargePanel form={form} /> : null}
      {tab === 'ledger' ? <UserWalletLedger table={table} wallet={wallet} ledger={ledger} locale={locale} onOpen={onOpenTransaction} /> : null}
      {tab === 'refund' ? <RefundEmptyState /> : null}
    </Stack>
  );
}

function UserWalletLedger({
  table,
  wallet,
  ledger,
  locale,
  onOpen,
}: {
  table: ReturnType<typeof useTable>;
  wallet?: WalletSummary | null;
  ledger: ReturnType<typeof useAdminWalletTransactions>;
  locale: string;
  onOpen: (transaction: WalletTransaction) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <WalletLedgerTable
      t={t}
      locale={locale}
      wallet={wallet ?? undefined}
      loading={ledger.isLoading}
      items={ledger.data?.items ?? []}
      total={ledger.data?.total ?? 0}
      loadedCount={ledger.data?.items.length ?? 0}
      page={table.page}
      rowsPerPage={table.rowsPerPage}
      onOpen={onOpen}
      onPageChange={table.onChangePage}
      onRowsPerPageChange={table.onChangeRowsPerPage}
    />
  );
}

function DialogHeader({
  t,
  user,
  wallet,
  onClose,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  user: SystemUser | null;
  wallet?: WalletSummary | null;
  onClose: VoidFunction;
}) {
  return (
    <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
      <Stack spacing={0.75}>
        <Typography variant="h6">{t('userWallet.title')}</Typography>
        <Stack direction="row" spacing={1} alignItems="center" useFlexGap flexWrap="wrap">
          <Label color={wallet?.status === 'active' ? 'success' : 'default'} variant="soft">
            {walletStatusLabel(t, wallet?.status)}
          </Label>
          <Typography variant="caption" color="text.secondary">
            {user ? `${user.username} · ${user.email}` : ''}
          </Typography>
        </Stack>
      </Stack>
      <IconButton onClick={onClose}>
        <Iconify icon="solar:close-circle-bold" />
      </IconButton>
    </Stack>
  );
}

function WalletMetrics({ wallet }: { wallet?: WalletSummary | null }) {
  const { t } = useTranslate('admin');
  const metrics = [
    { label: t('wallet.metrics.availableBalance'), value: formatWalletMoney(wallet?.balance) },
    { label: t('wallet.metrics.rechargeBalance'), value: formatWalletMoney(wallet?.recharge_balance) },
    { label: t('wallet.metrics.giftBalance'), value: formatWalletMoney(wallet?.gift_balance) },
    { label: t('wallet.metrics.totalConsumed'), value: formatWalletMoney(wallet?.total_consumed) },
  ];

  return (
    <Box sx={{ display: 'grid', gap: 2, gridTemplateColumns: { xs: '1fr', md: 'repeat(4, 1fr)' } }}>
      {metrics.map((metric) => (
        <Stack key={metric.label} spacing={0.5} sx={{ p: 2, borderRadius: 1, bgcolor: 'background.neutral' }}>
          <Typography variant="caption" color="text.secondary">
            {metric.label}
          </Typography>
          <Typography variant="h6">{metric.value}</Typography>
        </Stack>
      ))}
    </Box>
  );
}

function RefundEmptyState() {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1} sx={{ py: 5, alignItems: 'center', color: 'text.secondary' }}>
      <Iconify icon="solar:shield-check-bold" width={32} />
      <Typography variant="body2">{t('userWallet.emptyRefunds')}</Typography>
    </Stack>
  );
}
