'use client';

import type { ApiToken } from 'src/types/api-token';
import type {
  TokenForm,
  TokenScope,
  TokenFormErrors,
  BillingGroupOption,
} from './api-token-management-types';

import { useState, useCallback } from 'react';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import {
  createApiToken,
  deleteApiToken,
  updateApiToken,
  getApiTokenSecret,
  createAdminApiToken,
  deleteAdminApiToken,
  updateAdminApiToken,
  getAdminApiTokenSecret,
} from 'src/actions/api-tokens';

import { toast } from 'src/components/snackbar';

import {
  formFromToken,
  defaultGroupCode,
  defaultCreateForm,
  tokenUpdatePayload,
  userTokenCreatePayload,
  adminTokenCreatePayload,
} from './api-token-management-utils';

const USER_REQUIRED_ERROR = 'validation.userRequired';

export function useTokenDialog(
  scope: TokenScope,
  t: (key: string, options?: Record<string, unknown>) => string,
  groups: BillingGroupOption[],
  defaultUserId = ''
) {
  const [form, setForm] = useState(defaultCreateForm('', defaultUserId));
  const [errors, setErrors] = useState<TokenFormErrors>({});
  const [editing, setEditing] = useState<ApiToken | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [createdToken, setCreatedToken] = useState<string | null>(null);

  const closeDialog = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setErrors({});
    setForm(defaultCreateForm('', defaultUserId));
  }, [defaultUserId]);

  const openCreate = useCallback((defaultGroup: string) => {
    setEditing(null);
    setCreating(true);
    setErrors({});
    setForm(defaultCreateForm(defaultGroup || defaultGroupCode(groups), defaultUserId));
  }, [defaultUserId, groups]);

  const openEdit = useCallback((token: ApiToken) => {
    setCreating(false);
    setEditing(token);
    setErrors({});
    setForm(formFromToken(token));
  }, []);

  const clearError = useCallback((field: keyof TokenForm) => {
    setErrors((current) => withoutError(current, field));
  }, []);

  const submit = useCallback(async () => {
    const nextErrors = validateTokenForm(scope, form, editing);
    if (hasFormErrors(nextErrors)) {
      setErrors(nextErrors);
      return;
    }

    setErrors({});
    setSubmitting(true);
    try {
      if (editing) {
        await saveUpdate(scope, editing.id, form);
        toast.success(t('messages.apiTokenUpdated'));
      } else {
        const created = await saveCreate(scope, form);
        setCreatedToken(created.raw_token);
      }
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, scope, t]);

  return {
    clearError,
    closeCreatedToken: () => setCreatedToken(null),
    closeDialog,
    createdToken,
    creating,
    editing,
    errors,
    form,
    open: creating || !!editing,
    openCreate,
    openEdit,
    setForm,
    submit,
    submitting,
  };
}

function validateTokenForm(
  scope: TokenScope,
  form: TokenForm,
  editing: ApiToken | null
): TokenFormErrors {
  if (editing || scope !== 'admin' || form.token_type !== 'user') {
    return {};
  }

  return form.user_id.trim() ? {} : { user_id: USER_REQUIRED_ERROR };
}

function hasFormErrors(errors: TokenFormErrors) {
  return Object.keys(errors).length > 0;
}

function withoutError(errors: TokenFormErrors, field: keyof TokenForm) {
  return Object.fromEntries(Object.entries(errors).filter(([key]) => key !== field)) as TokenFormErrors;
}

export function useDeleteDialog(scope: TokenScope, t: (key: string) => string) {
  const [deleteTarget, setDeleteTarget] = useState<ApiToken | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteByScope(scope, deleteTarget.id);
      toast.success(t('messages.apiTokenDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, scope, t]);

  return { deleteTarget, setDeleteTarget, confirmDelete };
}

export function useCopyToken(scope: TokenScope, t: (key: string) => string) {
  const { copy } = useCopyToClipboard();

  return useCallback(async (token: ApiToken) => {
    try {
      const secret = scope === 'admin' ? await getAdminApiTokenSecret(token.id) : await getApiTokenSecret(token.id);
      copy(secret.raw_token);
      toast.success(t('messages.apiKeyCopied'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.copyFailed'));
    }
  }, [copy, scope, t]);
}

export async function toggleToken(scope: TokenScope, token: ApiToken) {
  const payload = { is_active: !token.is_active };
  return scope === 'admin' ? updateAdminApiToken(token.id, payload) : updateApiToken(token.id, payload);
}

export async function toggleTokenAndNotify(
  scope: TokenScope,
  token: ApiToken,
  t: (key: string) => string
) {
  try {
    await toggleToken(scope, token);
    toast.success(t('messages.apiTokenUpdated'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
}

async function saveCreate(scope: TokenScope, form: TokenForm) {
  return scope === 'admin'
    ? createAdminApiToken(adminTokenCreatePayload(form))
    : createApiToken(userTokenCreatePayload(form));
}

async function saveUpdate(scope: TokenScope, id: string, form: TokenForm) {
  return scope === 'admin'
    ? updateAdminApiToken(id, tokenUpdatePayload(form))
    : updateApiToken(id, tokenUpdatePayload(form));
}

async function deleteByScope(scope: TokenScope, id: string) {
  return scope === 'admin' ? deleteAdminApiToken(id) : deleteApiToken(id);
}
