'use client';

import type {
  TokenForm,
  TokenScope,
  TokenDialogState,
  BillingGroupOption,
} from './api-token-management-types';

import { useMemo, useEffect } from 'react';

import { useBillingGroups, useAvailableBillingGroups } from 'src/actions/groups';

import { defaultGroupCode } from './api-token-management-utils';

type TokenOwnerBillingGroupsOptions = {
  dialog: TokenDialogState;
  disabled: boolean;
  scope: TokenScope;
};

export function useTokenOwnerBillingGroups({
  dialog,
  disabled,
  scope,
}: TokenOwnerBillingGroupsOptions) {
  const availableGroups = useAvailableBillingGroups(scope !== 'admin' && !disabled);
  const adminGroups = useBillingGroups(
    scope === 'admin' && !disabled ? 0 : -1,
    scope === 'admin' && !disabled ? 1000 : 0,
    { is_active: true }
  );
  const groupSource = scope === 'admin' ? adminGroups : availableGroups;
  const items = groupSource.items;

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
