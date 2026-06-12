'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderKeyGroup } from 'src/types/provider-key-group';
import type { ProviderKeyGroupForm } from './provider-key-groups-utils';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import {
  createProviderKeyGroup,
  deleteProviderKeyGroup,
  updateProviderKeyGroup,
} from 'src/actions/provider-key-groups';

import { toast } from 'src/components/snackbar';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { AddButton } from './shared';
import { ProviderKeyGroupsTable } from './provider-key-groups-table';
import { ProviderKeyGroupDialog } from './provider-key-groups-dialog';
import {
  providerKeyGroupPayload,
  formFromProviderKeyGroup,
  DEFAULT_PROVIDER_KEY_GROUP_FORM,
} from './provider-key-groups-utils';

export function ProviderKeyGroupsCard({
  providerKeyGroups,
  providers,
  keysByProvider,
}: {
  providerKeyGroups: {
    items: ProviderKeyGroup[];
    isLoading: boolean;
    refresh: () => Promise<unknown>;
  };
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
}) {
  const { t } = useTranslate('admin');
  const state = useProviderKeyGroupDialogs(t);

  return (
    <Card>
      <Stack direction="row" alignItems="center" justifyContent="flex-end" sx={{ p: 2 }}>
        <AddButton onClick={state.openCreate}>{t('actions.addProviderKeyGroup')}</AddButton>
      </Stack>
      <ProviderKeyGroupsTable
        rows={providerKeyGroups.items}
        loading={providerKeyGroups.isLoading}
        providers={providers}
        keysByProvider={keysByProvider}
        onEdit={state.openEdit}
        onDelete={state.setDeleting}
      />
      <ProviderKeyGroupDialog
        open={state.open}
        editing={!!state.editing}
        form={state.form}
        providers={providers}
        keysByProvider={keysByProvider}
        submitting={state.submitting}
        onClose={state.close}
        onSubmit={state.submit}
        onFormChange={state.setForm}
      />
      <DeleteKeyGroupDialog
        target={state.deleting}
        onClose={() => state.setDeleting(null)}
        onConfirm={state.confirmDelete}
      />
    </Card>
  );
}

function useProviderKeyGroupDialogs(t: (key: string) => string) {
  const [form, setForm] = useState<ProviderKeyGroupForm>({ ...DEFAULT_PROVIDER_KEY_GROUP_FORM });
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<ProviderKeyGroup | null>(null);
  const [deleting, setDeleting] = useState<ProviderKeyGroup | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const close = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setForm({ ...DEFAULT_PROVIDER_KEY_GROUP_FORM });
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_PROVIDER_KEY_GROUP_FORM });
  }, []);

  const openEdit = useCallback((group: ProviderKeyGroup) => {
    setCreating(false);
    setEditing(group);
    setForm(formFromProviderKeyGroup(group));
  }, []);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveProviderKeyGroup(form, editing);
      toast.success(t(editing ? 'messages.providerKeyGroupUpdated' : 'messages.providerKeyGroupCreated'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, editing, form, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleting) return;
    try {
      await deleteProviderKeyGroup(deleting.id);
      toast.success(t('messages.providerKeyGroupDeleted'));
      setDeleting(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleting, t]);

  return {
    close,
    confirmDelete,
    creating,
    deleting,
    editing,
    form,
    open: creating || !!editing,
    openCreate,
    openEdit,
    setDeleting,
    setForm,
    submit,
    submitting,
  };
}

function DeleteKeyGroupDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: ProviderKeyGroup | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('dialogs.deleteProviderKeyGroup')}
      content={t('dialogs.deleteContent', { name: target?.name ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

async function saveProviderKeyGroup(form: ProviderKeyGroupForm, editing: ProviderKeyGroup | null) {
  const payload = providerKeyGroupPayload(form);
  if (editing) await updateProviderKeyGroup(editing.id, payload);
  else await createProviderKeyGroup(payload);
}
