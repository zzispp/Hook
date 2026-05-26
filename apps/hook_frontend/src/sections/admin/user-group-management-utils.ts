import type { UserGroup, UserGroupCreate, UserGroupUpdate } from 'src/types/user-group';

import { DEFAULT_USER_GROUP_CODE } from './user-group-utils';

export type UserGroupForm = {
  code: string;
  name: string;
  description: string;
  is_active: boolean;
  sort_order: string;
};

export const DEFAULT_USER_GROUP_FORM: UserGroupForm = {
  code: '',
  name: '',
  description: '',
  is_active: true,
  sort_order: '0',
};

export function userGroupFormFromRecord(group: UserGroup): UserGroupForm {
  return {
    code: group.code,
    name: group.name,
    description: group.description ?? '',
    is_active: group.is_active,
    sort_order: String(group.sort_order),
  };
}

export function userGroupCreatePayload(form: UserGroupForm): UserGroupCreate {
  return {
    code: form.code.trim(),
    name: form.name.trim(),
    description: descriptionValue(form.description),
    is_active: form.is_active,
    sort_order: Number(form.sort_order || 0),
  };
}

export function userGroupUpdatePayload(form: UserGroupForm): UserGroupUpdate {
  return {
    name: form.name.trim(),
    description: descriptionValue(form.description),
    is_active: form.is_active,
    sort_order: Number(form.sort_order || 0),
  };
}

export function isDefaultUserGroup(group: Pick<UserGroup, 'code'> | null) {
  return group?.code === DEFAULT_USER_GROUP_CODE;
}

function descriptionValue(value: string) {
  const description = value.trim();
  return description || null;
}
