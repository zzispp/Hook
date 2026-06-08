export type ProviderGroup = {
  id: string;
  name: string;
  description?: string | null;
  sort_order: number;
  provider_ids: string[];
  created_at: string;
  updated_at: string;
};

export type ProviderKeyGroup = {
  id: string;
  name: string;
  description?: string | null;
  sort_order: number;
  provider_key_ids: string[];
  created_at: string;
  updated_at: string;
};

export type ProviderGroupListResponse = {
  groups: ProviderGroup[];
  total: number;
};

export type ProviderKeyGroupListResponse = {
  groups: ProviderKeyGroup[];
  total: number;
};

export type ProviderGroupCreate = {
  name: string;
  description?: string | null;
  sort_order?: number;
  provider_ids?: string[];
};

export type ProviderKeyGroupCreate = {
  name: string;
  description?: string | null;
  sort_order?: number;
  provider_key_ids?: string[];
};

export type ProviderGroupUpdate = Partial<ProviderGroupCreate>;

export type ProviderKeyGroupUpdate = Partial<ProviderKeyGroupCreate>;
