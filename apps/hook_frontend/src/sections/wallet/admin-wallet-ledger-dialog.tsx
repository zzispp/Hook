'use client';

import type { TFunction } from 'i18next';
import type { AdminWallet, WalletTransaction } from 'src/types/wallet';

import { useState } from 'react';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import {
  useAdminWalletDailyModelUsage,
  useAdminWalletLedgerEntriesForWallet,
} from 'src/actions/wallet';

import { useTable } from 'src/components/table';
import { Iconify } from 'src/components/iconify';

import { adminWalletOwner } from './wallet-display';
import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import { useWalletLedgerExpansion } from './wallet-ledger-expansion';
import { WalletLedgerEntriesTable } from './wallet-ledger-entries-table';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  wallet: AdminWallet | null;
  onClose: VoidFunction;
};

export function AdminWalletLedgerDialog({ t, locale, wallet, onClose }: Props) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const entries = useAdminWalletLedgerEntriesForWallet(wallet?.id ?? null, table.page, table.rowsPerPage);
  const [currentTransaction, setCurrentTransaction] = useState<WalletTransaction | null>(null);
  const expansion = useWalletLedgerExpansion();
  const detail = useAdminWalletDailyModelUsage(wallet?.id ?? null, expansion.date, expansion.page, expansion.pageSize);
  const currentWallet = entries.data?.wallet ?? wallet ?? undefined;

  return (
    <>
      <Dialog fullWidth maxWidth="lg" open={Boolean(wallet)} onClose={onClose}>
        <DialogTitle>
          <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
            <Stack spacing={0.5}>
              <Typography variant="h6">{t('adminWallets.ledger.title')}</Typography>
              <Typography variant="caption" color="text.secondary">
                {wallet ? adminWalletOwner(wallet) : ''}
              </Typography>
            </Stack>
            <IconButton onClick={onClose}>
              <Iconify icon="solar:close-circle-bold" />
            </IconButton>
          </Stack>
        </DialogTitle>
        <DialogContent sx={{ pb: 2 }}>
          <WalletLedgerEntriesTable
            t={t}
            wallet={currentWallet}
            locale={locale}
            loading={entries.isLoading}
            items={entries.data?.items ?? []}
            total={entries.data?.total ?? 0}
            loadedCount={entries.data?.items.length ?? 0}
            page={table.page}
            rowsPerPage={table.rowsPerPage}
            expansion={expansion.expansionState(detail)}
            onOpen={setCurrentTransaction}
            onToggleDailyUsage={expansion.toggle}
            onDailyUsagePageChange={expansion.changePage}
            onPageChange={table.onChangePage}
            onRowsPerPageChange={table.onChangeRowsPerPage}
          />
        </DialogContent>
      </Dialog>
      <WalletTransactionDetailDialog
        t={t}
        locale={locale}
        wallet={currentWallet}
        transaction={currentTransaction}
        open={Boolean(currentTransaction)}
        onClose={() => setCurrentTransaction(null)}
      />
    </>
  );
}
