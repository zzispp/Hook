'use client';

import type { Provider } from 'src/types/provider';
import type { useTranslate } from 'src/locales/use-locales';

import { useState, useCallback } from 'react';

import {
  createProvider,
  deleteProvider,
  updateProvider,
  createProviderModel,
  createProviderApiKey,
} from 'src/actions/providers';

import { toast } from 'src/components/snackbar';

import {
  apiKeyPayload,
  providerPayload,
  providerModelPayload,
  DEFAULT_API_KEY_FORM,
  DEFAULT_PROVIDER_FORM,
  providerFormFromProvider,
  DEFAULT_PROVIDER_MODEL_FORM,
} from './provider-management-utils';

export function useProviderDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [form, setForm] = useState({ ...DEFAULT_PROVIDER_FORM });
  const [editing, setEditing] = useState<Provider | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const closeDialog = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setForm({ ...DEFAULT_PROVIDER_FORM });
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_PROVIDER_FORM });
  }, []);

  const openEdit = useCallback((provider: Provider) => {
    setCreating(false);
    setEditing(provider);
    setForm(providerFormFromProvider(provider));
  }, []);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      const payload = providerPayload(form);
      if (editing) {
        await updateProvider(editing.id, payload);
      } else {
        await createProvider(payload);
      }
      toast.success(editing ? t('messages.providerUpdated') : t('messages.providerCreated'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  return { closeDialog, creating, editing, form, open: creating || !!editing, openCreate, openEdit, setForm, submit, submitting };
}

export function useDeleteProviderDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [deleteTarget, setDeleteTarget] = useState<Provider | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteProvider(deleteTarget.id);
      toast.success(t('messages.providerDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return { confirmDelete, deleteTarget, setDeleteTarget };
}

export function useProviderChildDialogs(t: ReturnType<typeof useTranslate>['t'], providerId?: string) {
  const [endpointOpen, setEndpointOpen] = useState(false);
  const [apiKeyOpen, setApiKeyOpen] = useState(false);
  const [modelOpen, setModelOpen] = useState(false);
  const [apiKeyForm, setApiKeyForm] = useState({ ...DEFAULT_API_KEY_FORM });
  const [modelForm, setModelForm] = useState({ ...DEFAULT_PROVIDER_MODEL_FORM });
  const [submitting, setSubmitting] = useState(false);

  const closeEndpoint = useCallback(() => {
    setEndpointOpen(false);
  }, []);

  const closeApiKey = useCallback(() => {
    setApiKeyOpen(false);
    setApiKeyForm({ ...DEFAULT_API_KEY_FORM });
  }, []);

  const closeModel = useCallback(() => {
    setModelOpen(false);
    setModelForm({ ...DEFAULT_PROVIDER_MODEL_FORM });
  }, []);

  const submitApiKey = useCallback(async () => {
    if (!providerId) return;
    setSubmitting(true);
    try {
      await createProviderApiKey(providerId, apiKeyPayload(apiKeyForm));
      toast.success(t('messages.providerKeyCreated'));
      closeApiKey();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [apiKeyForm, closeApiKey, providerId, t]);

  const submitModel = useCallback(async () => {
    if (!providerId) return;
    setSubmitting(true);
    try {
      await createProviderModel(providerId, providerModelPayload(modelForm));
      toast.success(t('messages.providerModelCreated'));
      closeModel();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeModel, modelForm, providerId, t]);

  return {
    apiKeyForm,
    apiKeyOpen,
    closeApiKey,
    closeEndpoint,
    closeModel,
    endpointOpen,
    modelForm,
    modelOpen,
    setApiKeyForm,
    setApiKeyOpen,
    setEndpointOpen,
    setModelForm,
    setModelOpen,
    submitApiKey,
    submitModel,
    submitting,
  };
}
