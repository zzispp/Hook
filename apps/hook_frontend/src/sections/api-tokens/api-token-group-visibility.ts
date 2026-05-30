'use client';

import type {
  TokenForm,
  TokenScope,
  TokenDialogState,
  BillingGroupOption,
} from './api-token-management-types';

import { useMemo, useEffect } from 'react';

import { useBillingGroups, useAvailableBillingGroups } from 'src/actions/groups';

import { useAuthContext } from 'src/auth/hooks';

import { defaultGroupCode } from './api-token-management-utils';

type UserGroupOwner = {
  id: string;
  group_codes: string[];
};

type TokenOwnerBillingGroupsOptions = {
  dialog: TokenDialogState;
  disabled: boolean;
  fixedUserGroupCodes?: string[];
  scope: TokenScope;
  users: UserGroupOwner[];
};

export function useTokenOwnerBillingGroups({
  dialog,
  disabled,
  fixedUserGroupCodes,
  scope,
  users,
}: TokenOwnerBillingGroupsOptions) {
  const { user } = useAuthContext();
  const availableGroups = useAvailableBillingGroups(scope !== 'admin' && !disabled);
  const adminGroups = useBillingGroups(
    scope === 'admin' && !disabled ? 0 : -1,
    scope === 'admin' && !disabled ? 1000 : 0,
    { is_active: true }
  );
  const groupSource = scope === 'admin' ? adminGroups : availableGroups;
  const ownerGroupCodes = tokenOwnerGroupCodes({
    currentUserGroupCodes: authUserGroupCodes(user),
    dialog,
    fixedUserGroupCodes,
    scope,
    users,
  });
  const items = useMemo(
    () => visibleGroupsForOwner({ groups: groupSource.items, ownerGroupCodes, scope }),
    [groupSource.items, ownerGroupCodes, scope]
  );

  useEffect(() => {
    syncGroupSelection({
      editing: Boolean(dialog.editing),
      groupCode: dialog.form.group_code,
      groups: items,
      open: dialog.open,
      setForm: dialog.setForm,
    });
  }, [dialog.editing, dialog.form.group_code, dialog.open, dialog.setForm, items]);

  return useMemo(() => ({ ...groupSource, items }), [groupSource, items]);
}

function visibleGroupsForOwner({
  groups,
  ownerGroupCodes,
  scope,
}: {
  groups: BillingGroupOption[];
  ownerGroupCodes: string[];
  scope: TokenScope;
}) {
  if (scope !== 'admin') {
    return groups;
  }

  if (ownerGroupCodes.length === 0) {
    return [];
  }

  return groups.filter((group) =>
    group.visible_user_group_codes.some((code) => ownerGroupCodes.includes(code))
  );
}

function tokenOwnerGroupCodes({
  currentUserGroupCodes,
  dialog,
  fixedUserGroupCodes,
  scope,
  users,
}: {
  currentUserGroupCodes: string[];
  dialog: TokenDialogState;
  fixedUserGroupCodes?: string[];
  scope: TokenScope;
  users: UserGroupOwner[];
}) {
  if (scope !== 'admin') {
    return [];
  }

  if (dialog.editing?.owner?.group_codes.length) {
    return dialog.editing.owner.group_codes;
  }

  if (fixedUserGroupCodes?.length) {
    return fixedUserGroupCodes;
  }

  if (dialog.form.token_type === 'user') {
    return users.find((item) => item.id === dialog.form.user_id)?.group_codes ?? [];
  }

  return currentUserGroupCodes;
}

function authUserGroupCodes(user: unknown) {
  if (!isRecord(user)) {
    return [];
  }

  return Array.isArray(user.group_codes)
    ? user.group_codes.filter((code): code is string => typeof code === 'string')
    : [];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function syncGroupSelection({
  editing,
  groupCode,
  groups,
  open,
  setForm,
}: {
  editing: boolean;
  groupCode: string;
  groups: BillingGroupOption[];
  open: boolean;
  setForm: TokenDialogState['setForm'];
}) {
  if (!open || groupIsVisible(groupCode, groups)) {
    return;
  }

  const nextGroupCode = editing ? '' : defaultGroupCode(groups);
  if (groupCode === nextGroupCode) {
    return;
  }

  setForm((form) => resetGroupForm({ form, groupCode: nextGroupCode }));
}

function groupIsVisible(groupCode: string, groups: BillingGroupOption[]) {
  return Boolean(groupCode) && groups.some((group) => group.code === groupCode);
}

function resetGroupForm({ form, groupCode }: { form: TokenForm; groupCode: string }): TokenForm {
  return {
    ...form,
    group_code: groupCode,
    allowed_model_ids: [],
  };
}
