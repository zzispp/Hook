export type AdminAffiliateUserSummary = {
  id: string;
  username: string;
  email: string;
  affiliate_code: string;
};

export type AdminAffiliateOverviewResponse = {
  total_referred_users: number;
  active_referrer_count: number;
  total_commission_amount: number;
  today_commission_amount: number;
  month_commission_amount: number;
  affiliate_commission_percent: number;
};

export type AdminAffiliateRelation = {
  user: AdminAffiliateUserSummary;
  referrer: AdminAffiliateUserSummary | null;
  referred_at: string | null;
  referred_recharge_amount: number;
  commission_amount: number;
  last_commission_at: string | null;
};

export type AdminAffiliateRelationListResponse = {
  items: AdminAffiliateRelation[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminAffiliateRelationChange = {
  id: string;
  user: AdminAffiliateUserSummary;
  old_referrer: AdminAffiliateUserSummary | null;
  new_referrer: AdminAffiliateUserSummary | null;
  operator: AdminAffiliateUserSummary | null;
  operator_user_id: string | null;
  reason: string;
  created_at: string;
};

export type AdminAffiliateRelationChangeListResponse = {
  items: AdminAffiliateRelationChange[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminAffiliateRelationUpdateInput = {
  referrer_aff_code?: string;
  clear_referrer?: boolean;
  reason: string;
};

export type AdminAffiliateCommission = {
  id: string;
  referrer: AdminAffiliateUserSummary;
  referred: AdminAffiliateUserSummary;
  recharge_order_id: string;
  recharge_order_no: string;
  payable_amount: number;
  commission_percent: number;
  commission_amount: number;
  wallet_transaction_id: string | null;
  status: string;
  failure_reason: string | null;
  created_at: string;
};

export type AdminAffiliateCommissionListResponse = {
  items: AdminAffiliateCommission[];
  total: number;
  page: number;
  page_size: number;
};

export type AdminAffiliateDailyReportItem = {
  date: string;
  commission_order_count: number;
  referred_payer_count: number;
  payable_amount: number;
  commission_amount: number;
};

export type AdminAffiliateReferrerReportItem = {
  referrer: AdminAffiliateUserSummary;
  referred_user_count: number;
  commission_order_count: number;
  payable_amount: number;
  commission_amount: number;
};

export type AdminAffiliateReportResponse = {
  daily_items: AdminAffiliateDailyReportItem[];
  referrer_items: AdminAffiliateReferrerReportItem[];
  referrer_total: number;
  page: number;
  page_size: number;
};
