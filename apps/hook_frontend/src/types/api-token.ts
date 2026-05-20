export type ModelAccessMode = 'all' | 'limited';
export type ApiTokenType = 'user' | 'independent';

export type ApiToken = {
  id: string;
  user_id?: string | null;
  owner?: ApiTokenOwner | null;
  token_type: ApiTokenType;
  name: string;
  token_prefix: string;
  group_code: string;
  expires_at?: string | null;
  model_access_mode: ModelAccessMode;
  allowed_model_ids: string[];
  rate_limit_rpm?: number | null;
  quota_limit?: number | null;
  used_quota: number;
  request_count: number;
  is_active: boolean;
  last_used_at?: string | null;
  created_at: string;
  updated_at: string;
};

export type ApiTokenOwner = {
  username: string;
  email: string;
};

export type ApiTokenCreate = {
  name: string;
  group_code: string;
  expires_at?: string | null;
  model_access_mode?: ModelAccessMode;
  allowed_model_ids?: string[];
  rate_limit_rpm?: number | null;
  quota_limit?: number | null;
};

export type AdminApiTokenCreate = ApiTokenCreate & {
  token_type: ApiTokenType;
  user_id?: string | null;
};

export type ApiTokenUpdate = {
  name?: string;
  group_code?: string;
  expires_at?: string | null;
  model_access_mode?: ModelAccessMode;
  allowed_model_ids?: string[] | null;
  rate_limit_rpm?: number | null;
  quota_limit?: number | null;
  is_active?: boolean;
};

export type ApiTokenCreateResponse = {
  token: ApiToken;
  raw_token: string;
};

export type ApiTokenSecretResponse = {
  raw_token: string;
};

export type TokenAccessibleModel = {
  id: string;
  object: 'model';
  created: number;
  owned_by: string;
};

export type TokenAccessibleModelListResponse = {
  object: 'list';
  data: TokenAccessibleModel[];
};

export type ApiTokenListResponse = {
  tokens: ApiToken[];
  total: number;
};
