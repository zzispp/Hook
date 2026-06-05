export type AffiliateSummary = {
  affiliate_code: string;
  affiliate_link: string;
  affiliate_enabled: boolean;
  referred_user_count: number;
  total_referred_recharge_amount: number;
  total_commission_amount: number;
  today_commission_amount: number;
  month_commission_amount: number;
  affiliate_commission_percent: number;
  last_commission_at: string | null;
};

export type AffiliateReferral = {
  referred_user_id: string;
  username: string;
  masked_email: string;
  referred_at: string | null;
  referred_recharge_amount: number;
  commission_amount: number;
  last_commission_at: string | null;
};

export type AffiliateReferralListResponse = {
  items: AffiliateReferral[];
  total: number;
  page: number;
  page_size: number;
};

export type AffiliateReferredUser = {
  referred_user_id: string;
  username: string;
  masked_email: string;
};

export type AffiliateCommission = {
  id: string;
  referred: AffiliateReferredUser;
  recharge_order_no: string;
  payable_amount: number;
  commission_percent: number;
  commission_amount: number;
  wallet_transaction_id: string | null;
  status: string;
  failure_reason: string | null;
  created_at: string;
};

export type AffiliateCommissionListResponse = {
  items: AffiliateCommission[];
  total: number;
  page: number;
  page_size: number;
};
