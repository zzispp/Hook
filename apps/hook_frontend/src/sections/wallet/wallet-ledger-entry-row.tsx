'use client';

import type { TFunction } from 'i18next';
import type { AdminWallet, WalletSummary, WalletLedgerEntry, WalletTransaction } from 'src/types/wallet';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { tableStickyActionCellSx } from 'src/components/table';

import { walletOwner, walletFromTransaction } from './wallet-owner';
import {
  dailyUsageDate,
  entryDescription,
  entryAsTransaction,
  isDailyModelUsageEntry,
} from './wallet-ledger-entry-utils';
import {
  formatWalletDate,
  walletStatusLabel,
  formatSignedAmount,
  formatBalanceChange,
  formatWalletDateTime,
  formatBalanceBreakdown,
  walletTransactionColor,
  walletTransactionReasonLabel,
  walletTransactionCategoryLabel,
} from './wallet-display';

type Props = {
  t: TFunction<'admin'>;
  entry: WalletLedgerEntry;
  wallet?: WalletSummary | AdminWallet;
  locale: string;
  expanded: boolean;
  onOpen: (transaction: WalletTransaction) => void;
  onToggleDailyUsage: (entry: WalletLedgerEntry) => void;
};

export function WalletLedgerEntryRow({
  t,
  entry,
  wallet,
  onOpen,
  locale,
  expanded,
  onToggleDailyUsage,
}: Props) {
  const rowWallet = wallet ?? walletFromTransaction(entry);
  const isDaily = isDailyModelUsageEntry(entry);

  return (
    <TableRow hover sx={{ cursor: isDaily ? 'default' : 'pointer' }} onClick={() => !isDaily && onOpen(entryAsTransaction(entry))}>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {isDaily ? formatWalletDate(dailyUsageDate(entry), locale) : formatWalletDateTime(entry.created_at, locale)}
      </TableCell>
      <TableCell>
        <OwnerCell t={t} wallet={rowWallet} />
      </TableCell>
      <TableCell>
        <TransactionTypeCell t={t} entry={entry} />
      </TableCell>
      <AmountCell amount={entry.amount} />
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        <BalanceChangeCell t={t} entry={entry} />
      </TableCell>
      <DescriptionCell description={entryDescription(t, entry, locale) || t('wallet.emptyValue')} />
      <DetailActionCell t={t} entry={entry} expanded={expanded} onOpen={onOpen} onToggleDailyUsage={onToggleDailyUsage} />
    </TableRow>
  );
}

function DetailActionCell({
  t,
  entry,
  onOpen,
  expanded,
  onToggleDailyUsage,
}: {
  t: TFunction<'admin'>;
  entry: WalletLedgerEntry;
  expanded: boolean;
  onOpen: (transaction: WalletTransaction) => void;
  onToggleDailyUsage: (entry: WalletLedgerEntry) => void;
}) {
  if (isDailyModelUsageEntry(entry)) {
    return (
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Button
          size="small"
          variant="outlined"
          startIcon={
            <Iconify
              icon={
                expanded
                  ? 'solar:double-alt-arrow-up-bold-duotone'
                  : 'solar:double-alt-arrow-down-bold-duotone'
              }
            />
          }
          sx={{ minWidth: 88, whiteSpace: 'nowrap' }}
          onClick={(event) => {
            event.stopPropagation();
            onToggleDailyUsage(entry);
          }}
        >
          {t('wallet.actions.details')}
        </Button>
      </TableCell>
    );
  }

  return (
    <TableCell align="left" sx={tableStickyActionCellSx}>
      <IconButton
        onClick={(event) => {
          event.stopPropagation();
          onOpen(entryAsTransaction(entry));
        }}
      >
        <Iconify icon="solar:eye-bold" />
      </IconButton>
    </TableCell>
  );
}

function OwnerCell({ t, wallet }: { t: TFunction<'admin'>; wallet?: WalletSummary | AdminWallet }) {
  const owner = walletOwner(wallet);

  return (
    <Stack spacing={0.5}>
      <Typography variant="body2" sx={{ fontWeight: 600 }}>
        {owner.name || t('wallet.emptyValue')}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('wallet.ownerSummary', {
          type: t(`wallet.ownerTypes.${owner.type}`),
          status: walletStatusLabel(t, owner.status),
        })}
      </Typography>
    </Stack>
  );
}

function TransactionTypeCell({ t, entry }: { t: TFunction<'admin'>; entry: WalletLedgerEntry }) {
  return (
    <Stack spacing={0.5} alignItems="flex-start">
      <Label color={walletTransactionColor(entry.category)} variant="soft">
        {walletTransactionCategoryLabel(t, entry.category)}
      </Label>
      <Typography variant="caption" color="text.secondary">
        {walletTransactionReasonLabel(t, entry.reason_code)}
      </Typography>
    </Stack>
  );
}

function AmountCell({ amount }: { amount: number }) {
  return (
    <TableCell sx={{ color: amount >= 0 ? 'success.main' : 'error.main', fontWeight: 700 }}>
      {formatSignedAmount(amount)}
    </TableCell>
  );
}

function BalanceChangeCell({ t, entry }: { t: TFunction<'admin'>; entry: WalletLedgerEntry }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
        {formatBalanceChange(entry.balance_before, entry.balance_after)}
      </Typography>
      <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
        {formatBalanceBreakdown(t, entry)}
      </Typography>
    </Stack>
  );
}

function DescriptionCell({ description }: { description: string }) {
  return (
    <TableCell sx={{ color: 'text.secondary', width: 240, maxWidth: 240 }}>
      <Typography variant="body2" noWrap sx={{ maxWidth: 240 }}>
        {description}
      </Typography>
    </TableCell>
  );
}
