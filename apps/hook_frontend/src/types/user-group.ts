import type { SystemUser, PageResponse } from './rbac';

export type UserGroup = {
  id: string;
  code: string;
  name: string;
  description?: string | null;
  is_active: boolean;
  is_system: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
};

export type UserGroupCreate = {
  code: string;
  name: string;
  description?: string | null;
  is_active?: boolean;
  sort_order?: number;
};

export type UserGroupUpdate = {
  name?: string;
  description?: string | null;
  is_active?: boolean;
  sort_order?: number;
};

export type UserGroupFilters = {
  search?: string;
  is_active?: boolean;
};

export type UserGroupPageResponse = PageResponse<UserGroup>;

export type UserGroupMembersResponse = PageResponse<SystemUser>;
