export type RechargePackageStatus = 'active' | 'disabled';
export type RechargeOrderStatus = 'pending' | 'expired' | 'paid' | 'cancelled' | 'failed';
export type RechargeOrderDatePreset = 'all' | 'today' | 'last7days' | 'last30days' | 'custom';
export type PaymentCallbackStatus = 'received' | 'processed' | 'ignored' | 'failed';

export type RechargePackage = {
  id: string;
  name: string;
  description: string | null;
  recharge_amount: number;
  gift_amount: number;
  total_arrival_amount: number;
  status: RechargePackageStatus;
  sort_order: number;
  created_at: string;
  updated_at: string;
};

export type RechargeOrder = {
  id: string;
  order_no: string;
  user_id: string;
  username: string;
  user_email: string;
  package_id: string | null;
  package_name: string;
  recharge_amount: number;
  gift_amount: number;
  total_arrival_amount: number;
  payable_amount: number;
  status: RechargeOrderStatus;
  payment_channel_code: string | null;
  payment_channel_name: string | null;
  payment_method: string | null;
  provider_trade_no: string | null;
  refund_status: string | null;
  refund_amount: number | null;
  paid_at: string | null;
  refunded_at: string | null;
  expires_at: string;
  created_at: string;
  updated_at: string;
};

export type PaymentChannel = {
  code: string;
  name: string;
  enabled: boolean;
  config: Record<string, unknown>;
  secret_set: boolean;
  config_schema: PaymentChannelConfigSchema | null;
  registered_at: string;
  updated_at: string;
};

export type PublicPaymentChannel = {
  code: string;
  name: string;
  methods: PaymentMethodOption[];
};

export type PaymentCallbackRecord = {
  id: string;
  payment_channel_code: string;
  callback_kind: 'notify' | 'return';
  http_method: string;
  order_no: string | null;
  provider_trade_no: string | null;
  payment_method: string | null;
  trade_status: string | null;
  status: PaymentCallbackStatus;
  settled: boolean;
  error_message: string | null;
  raw_params: Record<string, unknown>;
  received_at: string;
  processed_at: string | null;
};

export type PaymentChannelConfigSchema = {
  fields: PaymentChannelConfigField[];
  methods: PaymentMethodOption[];
};

export type PaymentChannelConfigField = {
  key: string;
  label: string;
  secret: boolean;
  required: boolean;
};

export type PaymentMethodOption = {
  code: string;
  name: string;
};

export type RechargePackageInput = {
  name: string;
  description?: string;
  recharge_amount: number;
  gift_amount: number;
  status: RechargePackageStatus;
  sort_order: number;
};

export type PaymentChannelUpdateInput = {
  enabled: boolean;
  config?: Record<string, unknown>;
  api_key?: string;
};

export type UserRechargePackage = Omit<
  RechargePackage,
  'status' | 'created_at' | 'total_arrival_amount'
> & {
  total_arrival_amount: number;
  estimated_payable_amount: number;
};

export type RechargeOrderCreateInput = {
  package_id?: string;
  recharge_amount?: number;
  payment_channel_code: string;
  payment_method: string;
  captcha_token?: string;
};

export type PaymentOrderAction = {
  kind: 'form_post';
  action: string;
  method: 'POST';
  fields: Record<string, string>;
};

export type RechargeOrderCreateResponse = {
  order: RechargeOrder;
  payment: PaymentOrderAction;
};

export type RechargePackageListResponse = {
  items: RechargePackage[];
  total: number;
  page: number;
  page_size: number;
};

export type UserRechargePackageListResponse = {
  recharge_enabled: boolean;
  arrival_ratio: number;
  min_amount: number;
  max_amount: number;
  items: UserRechargePackage[];
  total: number;
  page: number;
  page_size: number;
};

export type RechargeOrderListResponse = {
  items: RechargeOrder[];
  total: number;
  page: number;
  page_size: number;
};

export type RechargeOrderSummary = {
  total_payable_amount: number;
  order_count: number;
  user_count: number;
};

export type RechargeOrderUserSummary = {
  user_id: string;
  username: string;
  user_email: string;
  order_count: number;
  total_payable_amount: number;
  last_paid_at: string | null;
};

export type RechargeOrderSummaryResponse = {
  summary: RechargeOrderSummary;
  items: RechargeOrderUserSummary[];
  total: number;
  page: number;
  page_size: number;
};

export type PaymentCallbackRecordListResponse = {
  items: PaymentCallbackRecord[];
  total: number;
  page: number;
  page_size: number;
};
