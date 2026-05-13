'use client';

import type { RoleFormState, RoleDeleteState, RolePermissionState } from './role-management-state';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { deleteRole, updateRolePermissions } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';

import { saveRole } from './role-management-state';
import { filterRoleAssignableApiIds } from './role-permission-utils';

type ActionOptions = {
  deleteState: RoleDeleteState;
  formState: RoleFormState;
  permissionState: RolePermissionState;
};

export function useRoleManagementActions({ deleteState, formState, permissionState }: ActionOptions) {
  const [submitting, setSubmitting] = useState(false);
  const role = useRoleSaveAction({ formState, setSubmitting });
  const deletion = useRoleDeleteAction({ deleteState });
  const permission = useRolePermissionAction({ permissionState, setSubmitting });

  return { ...role, ...deletion, ...permission, submitting };
}

function useRoleSaveAction({
  formState,
  setSubmitting,
}: Pick<ActionOptions, 'formState'> & SubmitState) {
  const { t } = useTranslate('admin');

  const submitRole = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveRole(formState.editing, formState.form);
      toast.success(t(formState.editing ? 'messages.roleUpdated' : 'messages.roleCreated'));
      formState.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [formState, setSubmitting, t]);

  return { submitRole };
}

function useRoleDeleteAction({ deleteState }: Pick<ActionOptions, 'deleteState'>) {
  const { t } = useTranslate('admin');

  const confirmDelete = useCallback(async () => {
    if (!deleteState.target) return;
    try {
      await deleteRole(deleteState.target.code);
      toast.success(t('messages.roleDeleted'));
      deleteState.setTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteState, t]);

  return { confirmDelete };
}

function useRolePermissionAction({
  permissionState,
  setSubmitting,
}: Pick<ActionOptions, 'permissionState'> & SubmitState) {
  const { t } = useTranslate('admin');

  const savePermissions = useCallback(async () => {
    if (!permissionState.target) return;
    setSubmitting(true);
    try {
      const apiPermissionIds = filterRoleAssignableApiIds(
        permissionState.selectedApis,
        permissionState.readOnlyApis
      );
      await updateRolePermissions(
        permissionState.target.code,
        permissionState.selectedMenus,
        apiPermissionIds
      );
      toast.success(t('messages.rolePermissionsUpdated'));
      permissionState.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [permissionState, setSubmitting, t]);

  return { savePermissions };
}

type SubmitState = {
  setSubmitting: (value: boolean) => void;
};
