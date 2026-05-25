export type RechargePackageStatus = 'active' | 'disabled';
export type RechargeOrderStatus = 'pending' | 'expired' | 'paid' | 'cancelled' | 'failed';

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
  expires_at: string;
  created_at: string;
  updated_at: string;
};

export type PaymentChannel = {
  code: string;
  name: string;
  enabled: boolean;
  registered_at: string;
  updated_at: string;
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
};

export type UserRechargePackage = Omit<
  RechargePackage,
  'status' | 'created_at' | 'total_arrival_amount'
> & {
  total_arrival_amount: number;
  estimated_payable_amount: number;
};

export type RechargeOrderCreateInput = {
  package_id: string;
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
