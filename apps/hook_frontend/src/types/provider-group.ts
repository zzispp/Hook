export type ProviderGroupMember = {
  provider_id: string;
  priority: number;
};

export type ProviderKeyGroupMember = {
  provider_key_id: string;
  priority: number;
};

export type ProviderGroup = {
  id: string;
  name: string;
  description?: string | null;
  sort_order: number;
  provider_members: ProviderGroupMember[];
  created_at: string;
  updated_at: string;
};

export type ProviderKeyGroup = {
  id: string;
  name: string;
  description?: string | null;
  sort_order: number;
  provider_key_members: ProviderKeyGroupMember[];
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
  provider_members?: ProviderGroupMember[];
};

export type ProviderKeyGroupCreate = {
  name: string;
  description?: string | null;
  sort_order?: number;
  provider_key_members?: ProviderKeyGroupMember[];
};

export type ProviderGroupUpdate = Partial<ProviderGroupCreate>;

export type ProviderKeyGroupUpdate = Partial<ProviderKeyGroupCreate>;
