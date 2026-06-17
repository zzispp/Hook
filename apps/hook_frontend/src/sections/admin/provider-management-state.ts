'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { useTranslate } from 'src/locales/use-locales';
import type { Provider, ProviderApiKey } from 'src/types/provider';

import { useState, useCallback } from 'react';

import {
  createProvider,
  deleteProvider,
  updateProvider,
  createProviderModel,
  createProviderApiKey,
  deleteProviderApiKey,
  updateProviderApiKey,
  getProviderQuickImportSyncSettings,
  updateProviderQuickImportSyncSettings,
} from 'src/actions/providers';

import { toast } from 'src/components/snackbar';

import { missingImageGenerationFormat } from './provider-api-key-format-fields';
import {
  validSyncSettings,
  syncSettingsPayload,
  syncSettingsFormFromResponse,
  DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM,
} from './provider-quick-import-utils';
import {
  apiKeyPayload,
  providerPayload,
  apiKeyFormFromKey,
  apiKeyUpdatePayload,
  providerModelPayload,
  DEFAULT_API_KEY_FORM,
  providerUpdatePayload,
  DEFAULT_PROVIDER_FORM,
  providerFormFromProvider,
  DEFAULT_PROVIDER_MODEL_FORM,
} from './provider-management-utils';

type ProviderDialogOptions = {
  t: ReturnType<typeof useTranslate>['t'];
};

export function useProviderDialog({ t }: ProviderDialogOptions) {
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
      if (editing) {
        await updateProvider(editing.id, providerUpdatePayload(form));
      } else {
        await createProvider(providerPayload(form));
      }
      toast.success(editing ? t('messages.providerUpdated') : t('messages.providerCreated'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  return {
    closeDialog,
    creating,
    editing,
    form,
    open: creating || !!editing,
    openCreate,
    openEdit,
    setForm,
    submit,
    submitting,
  };
}

export function useProviderQuickImportSyncDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [provider, setProvider] = useState<Provider | null>(null);
  const [form, setForm] = useState({ ...DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM });
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const close = useCallback(() => {
    setProvider(null);
    setForm({ ...DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM });
  }, []);

  const open = useCallback((target: Provider) => {
    setProvider(target);
    setForm({ ...DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM });
    void load(target.id, setForm, setLoading, t);
  }, [t]);

  const submit = useCallback(async () => {
    if (!provider) return;
    setSubmitting(true);
    try {
      await updateProviderQuickImportSyncSettings(provider.id, syncSettingsPayload(form));
      toast.success(t('messages.providerUpdated'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, form, provider, t]);

  return {
    close,
    form,
    loading,
    open,
    provider,
    setForm,
    submit,
    submitting,
    valid: validSyncSettings(form),
  };
}

async function load(
  providerId: string,
  setForm: Dispatch<SetStateAction<typeof DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM>>,
  setLoading: (value: boolean) => void,
  t: ReturnType<typeof useTranslate>['t']
) {
  setLoading(true);
  try {
    const response = await getProviderQuickImportSyncSettings(providerId);
    setForm(syncSettingsFormFromResponse(response));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
  } finally {
    setLoading(false);
  }
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
  const [editingApiKey, setEditingApiKey] = useState<ProviderApiKey | null>(null);
  const [deletingApiKey, setDeletingApiKey] = useState<ProviderApiKey | null>(null);
  const [modelOpen, setModelOpen] = useState(false);
  const [apiKeyForm, setApiKeyForm] = useState({ ...DEFAULT_API_KEY_FORM });
  const [modelForm, setModelForm] = useState({ ...DEFAULT_PROVIDER_MODEL_FORM });
  const [submitting, setSubmitting] = useState(false);

  const closeEndpoint = useCallback(() => {
    setEndpointOpen(false);
  }, []);

  const closeApiKey = useCallback(() => {
    setApiKeyOpen(false);
    setEditingApiKey(null);
    setApiKeyForm({ ...DEFAULT_API_KEY_FORM });
  }, []);

  const openCreateApiKey = useCallback(() => {
    setEditingApiKey(null);
    setApiKeyForm({ ...DEFAULT_API_KEY_FORM });
    setApiKeyOpen(true);
  }, []);

  const openEditApiKey = useCallback((apiKey: ProviderApiKey) => {
    setEditingApiKey(apiKey);
    setApiKeyForm(apiKeyFormFromKey(apiKey));
    setApiKeyOpen(true);
  }, []);

  const closeModel = useCallback(() => {
    setModelOpen(false);
    setModelForm({ ...DEFAULT_PROVIDER_MODEL_FORM });
  }, []);

  const submitApiKey = useCallback(async () => {
    if (!providerId) return;
    if (missingImageGenerationFormat(apiKeyForm.api_formats)) {
      toast.error(t('providers.imageEditRequiresImageGeneration'));
      return;
    }
    setSubmitting(true);
    try {
      if (editingApiKey) {
        await updateProviderApiKey(providerId, editingApiKey.id, apiKeyUpdatePayload(apiKeyForm));
      } else {
        await createProviderApiKey(providerId, apiKeyPayload(apiKeyForm));
      }
      toast.success(editingApiKey ? t('messages.providerKeyUpdated') : t('messages.providerKeyCreated'));
      closeApiKey();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [apiKeyForm, closeApiKey, editingApiKey, providerId, t]);

  const toggleApiKey = useCallback(async (apiKey: ProviderApiKey) => {
    if (!providerId) return;
    try {
      await updateProviderApiKey(providerId, apiKey.id, { is_active: !apiKey.is_active });
      toast.success(apiKey.is_active ? t('messages.providerKeyDisabled') : t('messages.providerKeyEnabled'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [providerId, t]);

  const confirmDeleteApiKey = useCallback(async () => {
    if (!providerId || !deletingApiKey) return;
    try {
      await deleteProviderApiKey(providerId, deletingApiKey.id);
      toast.success(t('messages.providerKeyDeleted'));
      setDeletingApiKey(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deletingApiKey, providerId, t]);

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
    confirmDeleteApiKey,
    deletingApiKey,
    endpointOpen,
    editingApiKey,
    modelForm,
    modelOpen,
    openCreateApiKey,
    openEditApiKey,
    setApiKeyForm,
    setApiKeyOpen,
    setDeletingApiKey,
    setEndpointOpen,
    setModelForm,
    setModelOpen,
    submitApiKey,
    submitModel,
    submitting,
    toggleApiKey,
  };
}
