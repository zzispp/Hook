import type { UserGroup } from 'src/types/user-group';

export const DEFAULT_USER_GROUP_CODE = 'default';
export const USER_GROUP_MAX_PAGE_SIZE = 100;

export function enabledUserGroupOptions(groups: UserGroup[]) {
  return groups.filter((group) => group.is_active);
}

export function displayUserGroup(code: string, groups: UserGroup[]) {
  const group = groups.find((item) => item.code === code);
  return group?.name ?? code;
}

export function defaultUserGroupCode(groups: UserGroup[]) {
  return (
    groups.find((group) => group.code === DEFAULT_USER_GROUP_CODE)?.code ??
    groups[0]?.code ??
    DEFAULT_USER_GROUP_CODE
  );
}

export function userGroupSelectionLabel(
  codes: string[],
  groups: UserGroup[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (codes.length === 0) {
    return t('billingGroups.noVisibleUserGroups');
  }
  if (codes.length > 2) {
    return t('billingGroups.selectedUserGroupCount', { count: codes.length });
  }
  return codes.map((code) => displayUserGroup(code, groups)).join(', ');
}
