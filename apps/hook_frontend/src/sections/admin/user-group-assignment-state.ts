'use client';

import type { SystemUser } from 'src/types/rbac';
import type { UserGroup } from 'src/types/user-group';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { useUsers, updateUser } from 'src/actions/rbac';
import { mutateUserGroupResources } from 'src/actions/user-groups';

import { toast } from 'src/components/snackbar';

import { formFromUser, formToPayload } from './user-management-utils';

type AssignmentStateInput = {
  initialGroup: UserGroup | null;
  displayGroups: UserGroup[];
  groups: UserGroup[];
  onClose: () => void;
  onAssigned: () => void;
};

type AssignmentSubmitInput = {
  t: ReturnType<typeof useTranslate>['t'];
  user: SystemUser | null;
  close: VoidFunction;
  canSubmit: boolean;
  onAssigned: VoidFunction;
  targetGroupCode: string;
  setSubmitting: (value: boolean) => void;
};

const USER_SEARCH_PAGE_SIZE = 20;

export function useUserGroupAssignmentDialogState(input: AssignmentStateInput) {
  const { t } = useTranslate('admin');
  const [search, setSearch] = useState('');
  const [user, setUser] = useState<SystemUser | null>(null);
  const [targetCode, setTargetCode] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const usersPage = input.initialGroup ? 0 : -1;
  const users = useUsers(usersPage, USER_SEARCH_PAGE_SIZE, { search: search.trim() || undefined });
  const targetGroups = withInitialGroup(input.groups, input.initialGroup);
  const visibleGroups = mergeGroups(input.displayGroups, targetGroups);
  const targetGroupCode = targetCode || input.initialGroup?.code || '';
  const canSubmit = Boolean(user && targetGroupCode && user.group_code !== targetGroupCode);

  const close = useCallback(() => {
    setSearch('');
    setUser(null);
    setTargetCode('');
    input.onClose();
  }, [input]);

  const submit = useAssignmentSubmit({
    t,
    user,
    close,
    canSubmit,
    targetGroupCode,
    setSubmitting,
    onAssigned: input.onAssigned,
  });

  return {
    t,
    user,
    search,
    close,
    submit,
    setUser,
    canSubmit,
    submitting,
    setSearch,
    targetGroups,
    visibleGroups,
    usersLoading: users.isLoading,
    users: assignableUsers(users.items),
    setTargetCode,
    targetGroupCode,
  };
}

function useAssignmentSubmit({
  t,
  user,
  close,
  canSubmit,
  onAssigned,
  targetGroupCode,
  setSubmitting,
}: AssignmentSubmitInput) {
  return useCallback(async () => {
    if (!user || !canSubmit) return;
    setSubmitting(true);
    try {
      await updateUser(user.id, userGroupPayload(user, targetGroupCode));
      await mutateUserGroupResources();
      toast.success(t('messages.userGroupAssignmentUpdated'));
      onAssigned();
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [canSubmit, close, onAssigned, setSubmitting, t, targetGroupCode, user]);
}

function userGroupPayload(user: SystemUser, targetGroupCode: string) {
  return formToPayload({ ...formFromUser(user), group_code: targetGroupCode });
}

function withInitialGroup(groups: UserGroup[], initialGroup: UserGroup | null) {
  if (!initialGroup?.is_active || groups.some((group) => group.code === initialGroup.code)) {
    return groups;
  }
  return [...groups, initialGroup];
}

function mergeGroups(left: UserGroup[], right: UserGroup[]) {
  const byCode = new Map(left.map((group) => [group.code, group]));
  right.forEach((group) => byCode.set(group.code, group));
  return [...byCode.values()];
}

function assignableUsers(users: SystemUser[]) {
  return users.filter((user) => !user.system);
}
