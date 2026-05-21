import type { TFunction } from 'i18next';
import type { WalletLedgerEntry, WalletTransaction } from 'src/types/wallet';

import { formatWalletDate } from './wallet-display';

export function isDailyModelUsageEntry(entry: WalletLedgerEntry) {
  return entry.entry_kind === 'daily_model_usage';
}

export function dailyUsageDate(entry: WalletLedgerEntry) {
  return entry.local_date ?? entry.created_at.slice(0, 10);
}

export function entryDescription(t: TFunction<'admin'>, entry: WalletLedgerEntry, locale: string) {
  if (!isDailyModelUsageEntry(entry)) {
    return entry.description;
  }

  return t('wallet.dailyUsage.description', {
    date: formatWalletDate(dailyUsageDate(entry), locale),
    count: entry.transaction_count,
  });
}

export function entryAsTransaction(entry: WalletLedgerEntry): WalletTransaction {
  return {
    id: entry.id,
    wallet_id: entry.wallet_id,
    category: entry.category,
    reason_code: entry.reason_code,
    amount: entry.amount,
    balance_before: entry.balance_before,
    balance_after: entry.balance_after,
    recharge_balance_before: entry.recharge_balance_before,
    recharge_balance_after: entry.recharge_balance_after,
    gift_balance_before: entry.gift_balance_before,
    gift_balance_after: entry.gift_balance_after,
    link_type: entry.link_type,
    link_id: entry.link_id,
    operator_id: entry.operator_id,
    description: entry.description,
    created_at: entry.created_at,
  };
}
