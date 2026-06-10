'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';
import type { ProviderGroupKind, ProviderGroupForm } from './provider-groups-utils';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/locales/use-locales';
import {
  createProviderGroup,
  deleteProviderGroup,
  updateProviderGroup,
  createProviderKeyGroup,
  deleteProviderKeyGroup,
  updateProviderKeyGroup,
} from 'src/actions/provider-groups';

import { toast } from 'src/components/snackbar';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { AddButton } from './shared';
import { ProviderGroupsTable } from './provider-groups-table';
import { ProviderGroupDialog } from './provider-groups-dialog';
import {
  providerGroupPayload,
  formFromProviderGroup,
  providerKeyGroupPayload,
  formFromProviderKeyGroup,
  DEFAULT_PROVIDER_GROUP_FORM,
} from './provider-groups-utils';

type GroupRow = ProviderGroup | ProviderKeyGroup;

export function ProviderGroupsCard({
  providerGroups,
  providerKeyGroups,
  providers,
  keysByProvider,
}: {
  providerGroups: { items: ProviderGroup[]; isLoading: boolean; refresh: () => Promise<unknown> };
  providerKeyGroups: {
    items: ProviderKeyGroup[];
    isLoading: boolean;
    refresh: () => Promise<unknown>;
  };
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
}) {
  const { t } = useTranslate('admin');
  const [kind, setKind] = useState<ProviderGroupKind>('provider');
  const state = useProviderGroupDialogs(kind, t);
  const rows = kind === 'provider' ? providerGroups.items : providerKeyGroups.items;
  const loading = kind === 'provider' ? providerGroups.isLoading : providerKeyGroups.isLoading;

  return (
    <Card>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ p: 2 }}>
        <KindSwitch kind={kind} onChange={setKind} />
        <AddButton onClick={state.openCreate}>
          {t(kind === 'provider' ? 'actions.addProviderGroup' : 'actions.addProviderKeyGroup')}
        </AddButton>
      </Stack>
      <ProviderGroupsTable
        kind={kind}
        rows={rows}
        loading={loading}
        providers={providers}
        keysByProvider={keysByProvider}
        onEdit={state.openEdit}
        onDelete={state.setDeleting}
      />
      <ProviderGroupDialog
        kind={kind}
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
      <DeleteGroupDialog
        target={state.deleting}
        kind={kind}
        onClose={() => state.setDeleting(null)}
        onConfirm={state.confirmDelete}
      />
    </Card>
  );
}

function KindSwitch({
  kind,
  onChange,
}: {
  kind: ProviderGroupKind;
  onChange: (kind: ProviderGroupKind) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ToggleButtonGroup
      exclusive
      size="small"
      value={kind}
      onChange={(_event, next) => next && onChange(next)}
    >
      <ToggleButton value="provider">{t('providers.providerGroups')}</ToggleButton>
      <ToggleButton value="key">{t('providers.providerKeyGroups')}</ToggleButton>
    </ToggleButtonGroup>
  );
}

function useProviderGroupDialogs(kind: ProviderGroupKind, t: (key: string) => string) {
  const [form, setForm] = useState<ProviderGroupForm>({ ...DEFAULT_PROVIDER_GROUP_FORM });
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<GroupRow | null>(null);
  const [deleting, setDeleting] = useState<GroupRow | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const close = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setForm({ ...DEFAULT_PROVIDER_GROUP_FORM });
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_PROVIDER_GROUP_FORM });
  }, []);

  const openEdit = useCallback(
    (group: GroupRow) => {
      setCreating(false);
      setEditing(group);
      setForm(
        kind === 'provider'
          ? formFromProviderGroup(group as ProviderGroup)
          : formFromProviderKeyGroup(group as ProviderKeyGroup)
      );
    },
    [kind]
  );

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveProviderGroup(kind, form, editing);
      toast.success(groupSaveMessage(kind, !!editing, t));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, editing, form, kind, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleting) return;
    try {
      await removeProviderGroup(kind, deleting.id);
      toast.success(
        t(
          kind === 'provider' ? 'messages.providerGroupDeleted' : 'messages.providerKeyGroupDeleted'
        )
      );
      setDeleting(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleting, kind, t]);

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

function DeleteGroupDialog({
  target,
  kind,
  onClose,
  onConfirm,
}: {
  target: GroupRow | null;
  kind: ProviderGroupKind;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t(
        kind === 'provider' ? 'dialogs.deleteProviderGroup' : 'dialogs.deleteProviderKeyGroup'
      )}
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

async function saveProviderGroup(
  kind: ProviderGroupKind,
  form: ProviderGroupForm,
  editing: GroupRow | null
) {
  if (kind === 'provider') {
    await saveProviderGroupRow(form, editing as ProviderGroup | null);
    return;
  }
  await saveProviderKeyGroupRow(form, editing as ProviderKeyGroup | null);
}

async function saveProviderGroupRow(form: ProviderGroupForm, editing: ProviderGroup | null) {
  const payload = providerGroupPayload(form);
  if (editing) await updateProviderGroup(editing.id, payload);
  else await createProviderGroup(payload);
}

async function saveProviderKeyGroupRow(form: ProviderGroupForm, editing: ProviderKeyGroup | null) {
  const payload = providerKeyGroupPayload(form);
  if (editing) await updateProviderKeyGroup(editing.id, payload);
  else await createProviderKeyGroup(payload);
}

async function removeProviderGroup(kind: ProviderGroupKind, id: string) {
  if (kind === 'provider') await deleteProviderGroup(id);
  else await deleteProviderKeyGroup(id);
}

function groupSaveMessage(kind: ProviderGroupKind, editing: boolean, t: (key: string) => string) {
  if (kind === 'provider') {
    return t(editing ? 'messages.providerGroupUpdated' : 'messages.providerGroupCreated');
  }
  return t(editing ? 'messages.providerKeyGroupUpdated' : 'messages.providerKeyGroupCreated');
}
