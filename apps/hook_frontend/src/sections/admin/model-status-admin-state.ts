'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusCheckFormState } from './model-status-check-form';
import type { ModelStatusRun, ModelStatusListFilters, ModelStatusRunListFilters, ModelStatusBatchUpdateRequest } from 'src/types/model-status';

import { useMemo, useState } from 'react';

import { useGlobalModels } from 'src/actions/models';
import { useAdminApiTokens } from 'src/actions/api-tokens';
import {
  updateModelStatusCheck,
  useAdminModelStatusRuns,
  useAdminModelStatusChecks,
  batchCreateModelStatusChecks,
  batchDeleteModelStatusChecks,
  batchUpdateModelStatusChecks,
} from 'src/actions/model-status';

import { useTable } from 'src/components/table';
import { toast } from 'src/components/snackbar';

import { modelStatusCheckPayload, modelStatusCheckBatchCreatePayload } from './model-status-check-form';

export function useModelStatusAdminState(t: TFunction<'admin'>) {
  const checkTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const runTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'checked_at' });
  const [form, setForm] = useState<ModelStatusCheckFormState | null>(null);
  const [deletingIds, setDeletingIds] = useState<string[]>([]);
  const [batchFixOpen, setBatchFixOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [detailRun, setDetailRun] = useState<ModelStatusRun | null>(null);
  const [checkFilters, setCheckFilters] = useState<ModelStatusListFilters>({ preset: 'today' });
  const [runFilters, setRunFilters] = useState<ModelStatusRunListFilters>({});
  const checks = useAdminModelStatusChecks(checkFilters);
  const runs = useAdminModelStatusRuns(runTable.page, runTable.rowsPerPage, compactRunFilters(runFilters));
  const models = useGlobalModels(0, 200, { is_active: true });
  const tokens = useAdminApiTokens(0, 200, { token_type: 'independent', is_active: true });
  const errorMessage = useMemo(() => firstError([checks.error, runs.error, models.error, tokens.error]), [checks.error, models.error, runs.error, tokens.error]);

  return {
    checkTable,
    runTable,
    form,
    checks,
    runs,
    models,
    tokens,
    submitting,
    detailRun,
    deletingIds,
    batchFixOpen,
    checkFilters,
    runFilters,
    errorMessage,
    setForm,
    setDetailRun,
    setDeletingIds,
    setBatchFixOpen,
    submitForm: submitHandler({ checks, form, setForm, setSubmitting, t }),
    confirmDelete: deleteHandler({ checks, checkTable, deletingIds, setDeletingIds, setSubmitting, t }),
    confirmBatchFix: batchFixHandler({ checks, checkTable, setBatchFixOpen, setSubmitting, t }),
    changeCheckSearch: checkFilterHandler(checkTable, checkFilters, setCheckFilters, 'search'),
    changeCheckApiFormat: checkFilterHandler(checkTable, checkFilters, setCheckFilters, 'api_format'),
    changeRunSearch: runFilterHandler(runTable, runFilters, setRunFilters, 'search'),
    changeRunApiFormat: runFilterHandler(runTable, runFilters, setRunFilters, 'api_format'),
    changeRunStatus: runFilterHandler(runTable, runFilters, setRunFilters, 'status'),
  };
}

function submitHandler(options: {
  checks: ReturnType<typeof useAdminModelStatusChecks>;
  form: ModelStatusCheckFormState | null;
  setForm: (form: ModelStatusCheckFormState | null) => void;
  setSubmitting: (value: boolean) => void;
  t: TFunction<'admin'>;
}) {
  return async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!options.form) return;
    options.setSubmitting(true);
    try {
      if (options.form.id) await updateModelStatusCheck(options.form.id, modelStatusCheckPayload(options.form));
      else await createChecks(options.form, options.t);
      toast.success(options.t(options.form.id ? 'modelStatusChecks.messages.updated' : 'modelStatusChecks.messages.created'));
      options.setForm(null);
      await options.checks.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : options.t('messages.saveFailed'));
    } finally {
      options.setSubmitting(false);
    }
  };
}

function deleteHandler(options: {
  checks: ReturnType<typeof useAdminModelStatusChecks>;
  checkTable: ReturnType<typeof useTable>;
  deletingIds: string[];
  setDeletingIds: (ids: string[]) => void;
  setSubmitting: (value: boolean) => void;
  t: TFunction<'admin'>;
}) {
  return async () => {
    if (options.deletingIds.length === 0) return;
    options.setSubmitting(true);
    try {
      const result = await batchDeleteModelStatusChecks(options.deletingIds);
      toast.success(options.t('modelStatusChecks.messages.batchDeleted', { count: result.success_count }));
      if (result.failed.length) toast.error(result.failed.map((item) => item.error).join('\n'));
      options.setDeletingIds([]);
      options.checkTable.setSelected([]);
      await options.checks.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : options.t('messages.deleteFailed'));
    } finally {
      options.setSubmitting(false);
    }
  };
}

function batchFixHandler(options: {
  checks: ReturnType<typeof useAdminModelStatusChecks>;
  checkTable: ReturnType<typeof useTable>;
  setBatchFixOpen: (value: boolean) => void;
  setSubmitting: (value: boolean) => void;
  t: TFunction<'admin'>;
}) {
  return async (patch: Omit<ModelStatusBatchUpdateRequest, 'ids'>) => {
    const ids = options.checkTable.selected;
    if (ids.length === 0) return;
    options.setSubmitting(true);
    try {
      const result = await batchUpdateModelStatusChecks({ ids, ...patch });
      toast.success(options.t('modelStatusChecks.messages.batchFixed', { count: result.success_count }));
      if (result.failed.length) toast.error(result.failed.map((item) => item.error).join('\n'));
      options.setBatchFixOpen(false);
      options.checkTable.setSelected([]);
      await options.checks.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : options.t('messages.saveFailed'));
    } finally {
      options.setSubmitting(false);
    }
  };
}

async function createChecks(form: ModelStatusCheckFormState, t: TFunction<'admin'>) {
  const result = await batchCreateModelStatusChecks(modelStatusCheckBatchCreatePayload(form, t));
  if (result.failed.length) toast.error(result.failed.map((item) => item.error).join('\n'));
}

function checkFilterHandler(
  table: ReturnType<typeof useTable>,
  filters: ModelStatusListFilters,
  setFilters: (filters: ModelStatusListFilters) => void,
  key: 'search' | 'api_format'
) {
  return (value: string) => {
    table.onResetPage();
    table.setSelected([]);
    setFilters({ ...filters, [key]: value || undefined });
  };
}

function runFilterHandler(
  table: ReturnType<typeof useTable>,
  filters: ModelStatusRunListFilters,
  setFilters: (filters: ModelStatusRunListFilters) => void,
  key: keyof ModelStatusRunListFilters
) {
  return (value: string) => {
    table.onResetPage();
    setFilters({ ...filters, [key]: value || undefined });
  };
}

function compactRunFilters(filters: ModelStatusRunListFilters): ModelStatusRunListFilters {
  return Object.fromEntries(Object.entries(filters).filter(([, value]) => value !== undefined && value !== ''));
}

function firstError(errors: unknown[]) {
  const error = errors.find(Boolean);
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return undefined;
}
