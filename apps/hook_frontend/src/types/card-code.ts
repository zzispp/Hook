export type CardCodeStatus = 'active' | 'disabled' | 'used' | 'expired';
export type CardCodeBalanceType = 'recharge' | 'gift';

export type CardCodeType = {
  id: string;
  name: string;
  balance_type: CardCodeBalanceType;
  status: CardCodeStatus;
  remark: string | null;
  created_at: string;
  updated_at: string;
};

export type CardCode = {
  id: string;
  code: string;
  batch_no: string;
  type_id: string;
  type_name: string;
  recharge_amount: number;
  gift_amount: number;
  status: CardCodeStatus;
  remark: string | null;
  expires_at: string | null;
  created_at: string;
  updated_at: string;
  created_by_user_id: string | null;
  created_by_username: string | null;
  created_ip: string | null;
  used_by_user_id: string | null;
  used_by_username: string | null;
  used_ip: string | null;
  used_at: string | null;
  wallet_id: string | null;
  wallet_transaction_id: string | null;
};

export type CardCodeTypeInput = {
  name: string;
  balance_type: CardCodeBalanceType;
  status: 'active' | 'disabled';
  remark?: string;
};

export type CardCodeGenerateInput = {
  type_id: string;
  quantity: number;
  code_length: number;
  status?: 'active' | 'disabled';
  remark?: string;
  expires_at?: string;
  amount: number;
};

export type CardCodeBatchStatusInput = {
  ids: string[];
  status: 'active' | 'disabled';
};

export type CardCodeRedeemInput = {
  code: string;
};

export type CardCodeTypeListResponse = {
  items: CardCodeType[];
  total: number;
  page: number;
  page_size: number;
};

export type CardCodeListResponse = {
  items: CardCode[];
  total: number;
  page: number;
  page_size: number;
};

export type CardCodeGenerateResponse = {
  items: CardCode[];
  total: number;
  batch_no: string | null;
};

export type CardCodeBatchStatusResponse = {
  updated_count: number;
};
