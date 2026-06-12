export type ProviderKeyGroupMember = {
  provider_key_id: string;
  priority: number;
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

export type ProviderKeyGroupListResponse = {
  groups: ProviderKeyGroup[];
  total: number;
};

export type ProviderKeyGroupCreate = {
  name: string;
  description?: string | null;
  sort_order?: number;
  provider_key_members?: ProviderKeyGroupMember[];
};

export type ProviderKeyGroupUpdate = Partial<ProviderKeyGroupCreate>;
