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
  total_recharged: number;
  total_consumed: number;
  total_refunded: number;
  total_adjusted: number;
  updated_at: string;
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

export type WalletTransactionsResponse = {
  wallet: WalletSummary;
  items: WalletTransaction[];
  total: number;
  page: number;
  page_size: number;
};
