export type BillingGroup = {
  id: string;
  code: string;
  name: string;
  description?: string | null;
  billing_multiplier: number;
  allowed_model_ids: string[];
  allowed_provider_ids: string[];
  visible_user_group_codes: string[];
  is_active: boolean;
  is_system: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
};

export type BillingGroupCreate = {
  code: string;
  name: string;
  description?: string | null;
  billing_multiplier: number;
  allowed_model_ids?: string[];
  allowed_provider_ids?: string[];
  visible_user_group_codes?: string[];
  is_active?: boolean;
  sort_order?: number;
};

export type BillingGroupUpdate = {
  name?: string;
  description?: string | null;
  billing_multiplier?: number;
  allowed_model_ids?: string[];
  allowed_provider_ids?: string[];
  visible_user_group_codes?: string[];
  is_active?: boolean;
  sort_order?: number;
};

export type BillingGroupListResponse = {
  groups: BillingGroup[];
  total: number;
};
