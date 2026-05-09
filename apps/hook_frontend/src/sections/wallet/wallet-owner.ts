import type { AdminWallet, WalletSummary, WalletTransaction, AdminWalletLedgerTransaction } from 'src/types/wallet';

export type WalletOwner = {
  name?: string;
  status?: string | null;
  type: string;
};

export function walletOwner(wallet?: WalletSummary | AdminWallet): WalletOwner {
  if (wallet && 'owner_name' in wallet) {
    return {
      name: wallet.owner_name || wallet.owner_email,
      status: wallet.status,
      type: wallet.owner_type || 'user',
    };
  }

  return { name: undefined, status: wallet?.status, type: 'user' };
}

export function walletFromTransaction(transaction: WalletTransaction): AdminWallet | undefined {
  if (!('owner_name' in transaction)) {
    return undefined;
  }

  const item = transaction as AdminWalletLedgerTransaction;
  return {
    id: item.wallet_id,
    user_id: '',
    owner_name: item.owner_name,
    owner_email: item.owner_email,
    owner_type: item.owner_type,
    balance: item.balance_after,
    recharge_balance: item.recharge_balance_after,
    gift_balance: item.gift_balance_after,
    refundable_balance: item.recharge_balance_after,
    currency: 'CNY',
    status: item.wallet_status,
    limit_mode: 'finite',
    unlimited: false,
    created_at: item.created_at,
    total_recharged: 0,
    total_consumed: 0,
    total_refunded: 0,
    total_adjusted: 0,
    updated_at: item.created_at,
  };
}
