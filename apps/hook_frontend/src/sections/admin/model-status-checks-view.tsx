'use client';

import type { TFunction } from 'i18next';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ConfirmDialog } from 'src/components/custom-dialog';

import { ModelStatusRunsTable } from './model-status-runs-table';
import { ModelStatusChecksTable } from './model-status-checks-table';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { useModelStatusAdminState } from './model-status-admin-state';
import { RunsToolbar, ChecksToolbar } from './model-status-admin-toolbar';
import { ModelStatusBatchFixDialog } from './model-status-batch-fix-dialog';
import { ModelStatusCheckDialog, emptyModelStatusCheckForm } from './model-status-check-form';

type ModelStatusAdminTab = 'checks' | 'runs';

export function ModelStatusChecksView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<ModelStatusAdminTab>('checks');
  const state = useModelStatusAdminState(t);
  const locale = currentLang.numberFormat.code;

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.modelStatusChecks}
        action={<PageActions tab={tab} state={state} t={t} />}
      />
      {state.errorMessage ? <Alert severity="error" sx={{ mb: 3 }}>{state.errorMessage}</Alert> : null}
      <Tabs value={tab} onChange={(_event, value: ModelStatusAdminTab) => setTab(value)} sx={{ mb: 3 }}>
        <Tab value="checks" label={t('modelStatusChecks.tabs.checks')} />
        <Tab value="runs" label={t('modelStatusChecks.tabs.runs')} />
      </Tabs>
      {tab === 'checks' ? <ChecksPanel state={state} t={t} /> : null}
      {tab === 'runs' ? <RunsPanel state={state} locale={locale} t={t} /> : null}
      <ModelStatusCheckDialog
        form={state.form}
        t={t}
        models={state.models.items}
        tokens={state.tokens.items}
        submitting={state.submitting}
        onChange={state.setForm}
        onClose={() => state.setForm(null)}
        onSubmit={state.submitForm}
      />
      <ModelStatusBatchFixDialog
        open={state.batchFixOpen}
        selectedCount={state.checkTable.selected.length}
        tokens={state.tokens.items}
        submitting={state.submitting}
        t={t}
        onClose={() => state.setBatchFixOpen(false)}
        onSubmit={state.confirmBatchFix}
      />
      <DeleteChecksDialog state={state} t={t} />
    </DashboardContent>
  );
}

function ChecksPanel({ state, t }: { state: ReturnType<typeof useModelStatusAdminState>; t: TFunction<'admin'> }) {
  return (
    <Card>
      <ChecksToolbar state={state} t={t} />
      <ModelStatusChecksTable
        rows={state.checks.items}
        loading={state.checks.isLoading}
        selected={state.checkTable.selected}
        t={t}
        onEdit={state.setForm}
        onDelete={(row) => state.setDeletingIds([row.id])}
        onSelectRow={state.checkTable.onSelectRow}
        onSelectAllRows={state.checkTable.onSelectAllRows}
      />
    </Card>
  );
}

function RunsPanel({
  state,
  locale,
  t,
}: {
  state: ReturnType<typeof useModelStatusAdminState>;
  locale: string;
  t: TFunction<'admin'>;
}) {
  return (
    <Card>
      <RunsToolbar state={state} t={t} />
      <ModelStatusRunsTable
        rows={state.runs.items}
        total={state.runs.total}
        loading={state.runs.isLoading}
        page={state.runTable.page}
        rowsPerPage={state.runTable.rowsPerPage}
        locale={locale}
        detail={state.detailRun}
        t={t}
        onDetail={state.setDetailRun}
        onPageChange={state.runTable.onChangePage}
        onRowsPerPageChange={state.runTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function PageActions({
  tab,
  state,
  t,
}: {
  tab: ModelStatusAdminTab;
  state: ReturnType<typeof useModelStatusAdminState>;
  t: TFunction<'admin'>;
}) {
  const loading = tab === 'checks' ? state.checks.isValidating : state.runs.isValidating;
  const refresh = tab === 'checks' ? state.checks.refresh : state.runs.refresh;
  return (
    <Stack direction="row" spacing={1}>
      <RefreshButton loading={loading} onClick={() => void refresh()} />
      {tab === 'checks' ? <AddButton onClick={() => state.setForm(emptyModelStatusCheckForm())}>{t('modelStatusChecks.create')}</AddButton> : null}
    </Stack>
  );
}

function DeleteChecksDialog({ state, t }: { state: ReturnType<typeof useModelStatusAdminState>; t: TFunction<'admin'> }) {
  const count = state.deletingIds.length;
  return (
    <ConfirmDialog
      open={count > 0}
      title={t(count > 1 ? 'modelStatusChecks.batchDeleteTitle' : 'modelStatusChecks.deleteTitle')}
      content={t(count > 1 ? 'modelStatusChecks.batchDeleteContent' : 'modelStatusChecks.deleteContent', { count })}
      cancelText={t('common.cancel')}
      onClose={() => state.setDeletingIds([])}
      action={
        <Button variant="contained" color="error" loading={state.submitting} onClick={() => void state.confirmDelete()}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}
