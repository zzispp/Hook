'use client';

import type { SystemUser } from 'src/types/rbac';
import type { UserGroup } from 'src/types/user-group';

import { useState, useEffect, useCallback } from 'react';

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
  targetGroupCodes: string[];
  setSubmitting: (value: boolean) => void;
};

const USER_SEARCH_PAGE_SIZE = 20;

export function useUserGroupAssignmentDialogState(input: AssignmentStateInput) {
  const { t } = useTranslate('admin');
  const [search, setSearch] = useState('');
  const [user, setUser] = useState<SystemUser | null>(null);
  const [targetCodes, setTargetCodes] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const usersPage = input.initialGroup ? 0 : -1;
  const users = useUsers(usersPage, USER_SEARCH_PAGE_SIZE, { search: search.trim() || undefined });
  const targetGroups = mergeGroups(withInitialGroup(input.groups, input.initialGroup), groupsForUser(input.displayGroups, user));
  const visibleGroups = mergeGroups(input.displayGroups, targetGroups);
  const targetGroupCodes = targetCodes;
  const canSubmit = Boolean(user && targetGroupCodes.length > 0 && !sameCodes(user.group_codes, targetGroupCodes));

  useEffect(() => {
    setTargetCodes(defaultTargetCodes(user, input.initialGroup));
  }, [input.initialGroup, user]);

  const close = useCallback(() => {
    setSearch('');
    setUser(null);
    setTargetCodes([]);
    input.onClose();
  }, [input]);

  const selectUser = useCallback((nextUser: SystemUser | null) => {
    setUser(nextUser);
  }, []);

  const submit = useAssignmentSubmit({
    t,
    user,
    close,
    canSubmit,
    targetGroupCodes,
    setSubmitting,
    onAssigned: input.onAssigned,
  });

  return {
    t,
    user,
    search,
    close,
    submit,
    setUser: selectUser,
    canSubmit,
    submitting,
    setSearch,
    targetGroups,
    visibleGroups,
    usersLoading: users.isLoading,
    users: assignableUsers(users.items),
    setTargetCodes,
    targetGroupCodes,
  };
}

function useAssignmentSubmit({
  t,
  user,
  close,
  canSubmit,
  onAssigned,
  targetGroupCodes,
  setSubmitting,
}: AssignmentSubmitInput) {
  return useCallback(async () => {
    if (!user || !canSubmit) return;
    setSubmitting(true);
    try {
      await updateUser(user.id, userGroupPayload(user, targetGroupCodes));
      await mutateUserGroupResources();
      toast.success(t('messages.userGroupAssignmentUpdated'));
      onAssigned();
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [canSubmit, close, onAssigned, setSubmitting, t, targetGroupCodes, user]);
}

function userGroupPayload(user: SystemUser, targetGroupCodes: string[]) {
  return formToPayload({
    ...formFromUser(user),
    group_codes: targetGroupCodes,
  });
}

function defaultTargetCodes(user: SystemUser | null, initialGroup: UserGroup | null) {
  const codes = user?.group_codes ?? [];
  if (!initialGroup) {
    return codes;
  }
  return uniqueCodes([...codes, initialGroup.code]);
}

function groupsForUser(groups: UserGroup[], user: SystemUser | null) {
  const codes = new Set(user?.group_codes ?? []);
  return groups.filter((group) => codes.has(group.code));
}

function sameCodes(left: string[], right: string[]) {
  return left.length === right.length && left.every((code) => right.includes(code));
}

function uniqueCodes(codes: string[]) {
  return [...new Set(codes)];
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
