'use client';

import type { GlobalModelForm } from './model-management-utils';
import type { ModelsDevModelItem, GlobalModelResponse } from 'src/types/model';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';
import {
  useGlobalModels,
  getModelsDevList,
  createGlobalModel,
  deleteGlobalModel,
  updateGlobalModel,
} from 'src/actions/models';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { ModelDevPicker } from './model-dev-picker';
import { GlobalModelTable } from './global-model-table';
import { GlobalModelFormDialog } from './global-model-form-dialog';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { toModelFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';
import {
  DEFAULT_FORM,
  formFromModel,
  formFromModelsDev,
  toGlobalModelPayload,
} from './model-management-utils';

// ----------------------------------------------------------------------

export function ModelManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const models = useGlobalModels(table.page, table.rowsPerPage, toModelFilters(filters));
  const dialog = useModelDialog(t);
  const deleteDialog = useDeleteDialog(t);
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        heading={DASHBOARD_MENU_TITLES.modelManagement}
        action={
          <Stack direction="row" spacing={1}>
            <RefreshButton loading={models.isLoading} onClick={() => void models.refresh()} />
            <AddButton onClick={dialog.openCreate}>{t('actions.addModel')}</AddButton>
          </Stack>
        }
      />

      <Card>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchModels')}
          onChange={handleFiltersChange}
        />
        <GlobalModelTable
          rows={models.items}
          total={models.total}
          loading={models.isLoading}
          table={table}
          onEdit={dialog.openEdit}
          onDelete={deleteDialog.setDeleteTarget}
        />
      </Card>

      <GlobalModelFormDialog
        open={dialog.open}
        title={dialog.editing ? t('dialogs.editModel') : t('dialogs.createModel')}
        form={dialog.form}
        isEdit={!!dialog.editing}
        submitting={dialog.submitting}
        picker={dialog.creating ? <PickerPanel dialog={dialog} /> : undefined}
        onClose={dialog.closeDialog}
        onSubmit={dialog.submitModel}
        onChange={dialog.setForm}
      />

      <ConfirmDialog
        open={!!deleteDialog.deleteTarget}
        onClose={() => deleteDialog.setDeleteTarget(null)}
        title={t('dialogs.deleteModel')}
        content={t('dialogs.deleteContent', {
          name: deleteDialog.deleteTarget?.display_name ?? '',
        })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

function useModelDialog(t: ReturnType<typeof useTranslate>['t']) {
  const state = useModelDialogState();
  const loadModelDevModels = useModelsDevLoader(
    t,
    state.setModelDevItems,
    state.setModelDevLoading,
    state.setModelDevError
  );
  const actions = useModelDialogActions(t, state, loadModelDevModels);

  return {
    ...state,
    open: state.creating || !!state.editing,
    ...actions,
  };
}

function useModelDialogState() {
  const [form, setForm] = useState<GlobalModelForm>(DEFAULT_FORM);
  const [editing, setEditing] = useState<GlobalModelResponse | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [modelDevItems, setModelDevItems] = useState<ModelsDevModelItem[]>([]);
  const [modelDevQuery, setModelDevQuery] = useState('');
  const [modelDevLoading, setModelDevLoading] = useState(false);
  const [modelDevError, setModelDevError] = useState<Error | undefined>();
  const [selectedModel, setSelectedModel] = useState<ModelsDevModelItem | null>(null);

  return {
    form,
    setForm,
    editing,
    setEditing,
    creating,
    setCreating,
    submitting,
    setSubmitting,
    modelDevItems,
    setModelDevItems,
    modelDevQuery,
    setModelDevQuery,
    modelDevLoading,
    setModelDevLoading,
    modelDevError,
    setModelDevError,
    selectedModel,
    setSelectedModel,
  };
}

function useModelDialogActions(
  t: ReturnType<typeof useTranslate>['t'],
  state: ReturnType<typeof useModelDialogState>,
  loadModelDevModels: () => Promise<void>
) {
  const openEdit = useCallback((model: GlobalModelResponse) => {
    state.setCreating(false);
    state.setEditing(model);
    state.setForm(formFromModel(model));
  }, [state]);

  const closeDialog = useCallback(() => {
    state.setCreating(false);
    state.setEditing(null);
    state.setSelectedModel(null);
    state.setForm({ ...DEFAULT_FORM });
  }, [state]);

  const selectModelDevModel = useCallback((item: ModelsDevModelItem) => {
    state.setSelectedModel(item);
    state.setForm(formFromModelsDev(item));
  }, [state]);

  const openCreate = useCallback(() => {
    state.setEditing(null);
    state.setCreating(true);
    state.setForm({ ...DEFAULT_FORM });
    void loadModelDevModels();
  }, [loadModelDevModels, state]);

  const submitModel = useCallback(async () => {
    state.setSubmitting(true);
    try {
      await saveModelForm(state.form, state.editing);
      toast.success(state.editing ? t('messages.modelUpdated') : t('messages.modelCreated'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeDialog, state, t]);

  return { closeDialog, loadModelDevModels, openCreate, openEdit, selectModelDevModel, submitModel };
}

function useModelsDevLoader(
  t: ReturnType<typeof useTranslate>['t'],
  setItems: React.Dispatch<React.SetStateAction<ModelsDevModelItem[]>>,
  setLoading: React.Dispatch<React.SetStateAction<boolean>>,
  setError: React.Dispatch<React.SetStateAction<Error | undefined>>
) {
  return useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      setItems(await getModelsDevList(false));
    } catch (error) {
      setError(error instanceof Error ? error : new Error(t('messages.loadModelsFailed')));
    } finally {
      setLoading(false);
    }
  }, [setError, setItems, setLoading, t]);
}

async function saveModelForm(form: GlobalModelForm, editing: GlobalModelResponse | null) {
  const payload = toGlobalModelPayload(form);
  if (!editing) {
    await createGlobalModel(payload);
    return;
  }

  await updateGlobalModel(editing.id, {
    display_name: payload.display_name,
    default_price_per_request: payload.default_price_per_request,
    default_tiered_pricing: payload.default_tiered_pricing,
    supported_capabilities: payload.supported_capabilities,
    config: payload.config,
    is_active: payload.is_active,
  });
}

function useDeleteDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [deleteTarget, setDeleteTarget] = useState<GlobalModelResponse | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;

    try {
      await deleteGlobalModel(deleteTarget.id);
      toast.success(t('messages.modelDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return { deleteTarget, setDeleteTarget, confirmDelete };
}

function PickerPanel({ dialog }: { dialog: ReturnType<typeof useModelDialog> }) {
  return (
    <ModelDevPicker
      items={dialog.modelDevItems}
      loading={dialog.modelDevLoading}
      error={dialog.modelDevError}
      query={dialog.modelDevQuery}
      selected={dialog.selectedModel}
      onQueryChange={dialog.setModelDevQuery}
      onSelect={dialog.selectModelDevModel}
      onRetry={dialog.loadModelDevModels}
    />
  );
}
