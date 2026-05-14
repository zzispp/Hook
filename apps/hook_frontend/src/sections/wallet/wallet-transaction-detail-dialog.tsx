'use client';

import type { TFunction } from 'i18next';
import type { AdminWallet, WalletSummary, WalletTransaction } from 'src/types/wallet';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { walletFromTransaction } from './wallet-owner';
import { DETAIL_DIALOG_MAX_WIDTH } from './wallet-constants';
import {
  walletStatusLabel,
  formatSignedAmount,
  walletLinkTypeLabel,
  formatBalanceChange,
  formatWalletDateTime,
  formatBalanceBreakdown,
  walletTransactionColor,
  walletTransactionReasonLabel,
  walletTransactionCategoryLabel,
} from './wallet-display';

type WalletTransactionDetailDialogProps = {
  open: boolean;
  t: TFunction<'admin'>;
  locale: string;
  wallet?: WalletSummary | AdminWallet;
  transaction: WalletTransaction | null;
  onClose: VoidFunction;
};

export function WalletTransactionDetailDialog({
  open,
  t,
  wallet,
  locale,
  onClose,
  transaction,
}: WalletTransactionDetailDialogProps) {
  if (!transaction) {
    return null;
  }
  const currency = wallet?.currency ?? walletFromTransaction(transaction)?.currency;

  return (
    <Dialog fullWidth maxWidth={DETAIL_DIALOG_MAX_WIDTH} open={open} onClose={onClose}>
      <WalletDetailTitle t={t} onClose={onClose} />
      <DialogContent sx={{ pb: 3 }}>
        <Stack spacing={2.5}>
          <DetailOverview t={t} locale={locale} transaction={transaction} currency={currency} />
          <TransactionAuditGrid t={t} wallet={wallet} transaction={transaction} />
          <Divider />
          <DetailField label={t('wallet.fields.description')} value={transaction.description || t('wallet.emptyValue')} multiline />
          <CloseAction t={t} onClose={onClose} />
        </Stack>
      </DialogContent>
    </Dialog>
  );
}

function WalletDetailTitle({ t, onClose }: { t: TFunction<'admin'>; onClose: VoidFunction }) {
  return (
    <DialogTitle>
      <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
        <Box>
          <Typography variant="h6">{t('wallet.dialog.title')}</Typography>
          <Typography variant="caption" color="text.secondary">
            {t('wallet.dialog.subtitle')}
          </Typography>
        </Box>
        <IconButton onClick={onClose}>
          <Iconify icon="solar:close-circle-bold" />
        </IconButton>
      </Stack>
    </DialogTitle>
  );
}

function DetailOverview({
  t,
  locale,
  transaction,
  currency,
}: {
  t: TFunction<'admin'>;
  locale: string;
  transaction: WalletTransaction;
  currency?: string;
}) {
  const positive = transaction.amount >= 0;

  return (
    <Stack spacing={1.5} sx={{ p: 2, borderRadius: 1, bgcolor: 'background.neutral' }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
        <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
          <Label color={walletTransactionColor(transaction.category)} variant="soft">
            {walletTransactionCategoryLabel(t, transaction.category)}
          </Label>
          <Label color="default" variant="soft">
            {walletTransactionReasonLabel(t, transaction.reason_code)}
          </Label>
        </Stack>
        <Typography
          variant="subtitle1"
          sx={{ color: positive ? 'success.main' : 'error.main', fontWeight: 700 }}
        >
          {formatSignedAmount(transaction.amount, currency)}
        </Typography>
      </Stack>
      <Typography variant="caption" color="text.secondary">
        {formatWalletDateTime(transaction.created_at, locale)}
      </Typography>
    </Stack>
  );
}

function TransactionAuditGrid({
  t,
  wallet,
  transaction,
}: {
  t: TFunction<'admin'>;
  wallet?: WalletSummary | AdminWallet;
  transaction: WalletTransaction;
}) {
  const owner = wallet ?? walletFromTransaction(transaction);

  return (
    <DetailGrid>
      <DetailField
        label={t('wallet.fields.owner')}
        value={walletOwnerName(t, owner)}
        helper={t('wallet.ownerSummary', {
          type: t(`wallet.ownerTypes.${walletOwnerType(owner)}`),
          status: walletStatusLabel(t, owner?.status),
        })}
      />
      <DetailField
        label={t('wallet.fields.balanceChange')}
        value={formatBalanceChange(transaction.balance_before, transaction.balance_after)}
        helper={formatBalanceBreakdown(t, transaction)}
        monospace
      />
      <DetailField label={t('wallet.fields.linkType')} value={walletLinkTypeLabel(t, transaction.link_type)} />
      <DetailField label={t('wallet.fields.transactionId')} value={transaction.id} monospace />
      <DetailField label={t('wallet.fields.linkId')} value={transaction.link_id || t('wallet.emptyValue')} monospace />
      <OperatorField t={t} transaction={transaction} />
    </DetailGrid>
  );
}

function walletOwnerName(t: TFunction<'admin'>, wallet?: WalletSummary | AdminWallet) {
  if (wallet && 'owner_name' in wallet) {
    return wallet.owner_name || wallet.owner_email || t('wallet.emptyValue');
  }

  return t('wallet.currentUser');
}

function walletOwnerType(wallet?: WalletSummary | AdminWallet) {
  return wallet && 'owner_type' in wallet ? wallet.owner_type : 'user';
}

function DetailGrid({ children }: { children: React.ReactNode }) {
  return (
    <Box
      sx={{
        gap: 1.5,
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)' },
      }}
    >
      {children}
    </Box>
  );
}

function CloseAction({ t, onClose }: { t: TFunction<'admin'>; onClose: VoidFunction }) {
  return (
    <Stack direction="row" justifyContent="flex-end">
      <Button variant="contained" onClick={onClose}>
        {t('common.close')}
      </Button>
    </Stack>
  );
}

function DetailField({
  label,
  value,
  helper,
  monospace,
  multiline,
}: {
  label: string;
  value: React.ReactNode;
  helper?: React.ReactNode;
  monospace?: boolean;
  multiline?: boolean;
}) {
  return (
    <Stack spacing={0.75} sx={{ p: 1.5, border: (theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1 }}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography
        variant="body2"
        sx={{
          fontWeight: 600,
          wordBreak: 'break-word',
          whiteSpace: multiline ? 'pre-wrap' : 'normal',
          fontFamily: monospace ? 'monospace' : undefined,
        }}
      >
        {value}
      </Typography>
      {helper ? (
        <Typography variant="caption" color="text.secondary" sx={{ fontFamily: monospace ? 'monospace' : undefined }}>
          {helper}
        </Typography>
      ) : null}
    </Stack>
  );
}

function OperatorField({ t, transaction }: { t: TFunction<'admin'>; transaction: WalletTransaction }) {
  const operatorName = transaction.operator_id ? t('wallet.operatorRecorded') : t('wallet.systemAuto');

  return (
    <DetailField
      label={t('wallet.fields.operator')}
      value={operatorName}
      helper={t('wallet.operatorId', { id: transaction.operator_id || t('wallet.emptyValue') })}
      monospace={Boolean(transaction.operator_id)}
    />
  );
}
