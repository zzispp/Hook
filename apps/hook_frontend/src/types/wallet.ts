export type WalletSummary = {
  id: string;
  user_id: string;
  balance: number;
  recharge_balance: number;
  gift_balance: number;
  refundable_balance: number;
  currency: string;
  status: string;
  limit_mode: string;
  unlimited: boolean;
  created_at: string;
  total_recharged: number;
  total_consumed: number;
  total_refunded: number;
  total_adjusted: number;
  updated_at: string;
};

export type AdminWallet = WalletSummary & {
  owner_name: string;
  owner_email: string;
  owner_type: string;
};

export type WalletBalanceResponse = {
  wallet: WalletSummary;
  unlimited: boolean;
  limit_mode: string;
  balance: number;
  recharge_balance: number;
  gift_balance: number;
  refundable_balance: number;
  currency: string;
};

export type WalletTransaction = {
  id: string;
  wallet_id: string;
  category: string;
  reason_code: string;
  amount: number;
  balance_before: number;
  balance_after: number;
  recharge_balance_before: number;
  recharge_balance_after: number;
  gift_balance_before: number;
  gift_balance_after: number;
  link_type: string | null;
  link_id: string | null;
  operator_id: string | null;
  description: string | null;
  created_at: string;
};

export type AdminWalletLedgerTransaction = WalletTransaction & {
  owner_name: string;
  owner_email: string;
  owner_type: string;
  wallet_status: string;
};

export type WalletTransactionsResponse = {
  wallet: WalletSummary;
  items: WalletTransaction[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminWalletLedgerResponse = {
  items: AdminWalletLedgerTransaction[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminWalletListResponse = {
  items: AdminWallet[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminWalletTransactionsResponse = {
  wallet: AdminWallet;
  items: WalletTransaction[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminWalletAdjustmentInput = {
  amount: number;
  balance_type: 'recharge' | 'gift';
  adjustment_type: 'increase' | 'deduct';
  description?: string;
};

export type AdminWalletAdjustmentResponse = {
  transaction: WalletTransaction;
};
