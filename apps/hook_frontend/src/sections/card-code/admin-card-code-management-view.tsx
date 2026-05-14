'use client';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { CardCodeTable } from './card-code-table';
import { CardCodeTypeTable } from './card-code-type-table';
import {
  CardCodeToolbar,
  CardCodeTypeToolbar,
} from './card-code-filters';
import { CardCodeTypeDialog, CardCodeGenerateDialog } from './card-code-dialogs';
import { type CardCodeTab, useCardCodeManagementState } from './admin-card-code-state';

export function AdminCardCodeManagementView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<CardCodeTab>('codes');
  const state = useCardCodeManagementState(t);
  const locale = currentLang.numberFormat.code;
  const loading = tab === 'codes' ? state.codes.isLoading : state.types.isLoading;

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.cardCodeManagement}
        action={<RefreshButton loading={loading} onClick={() => void state.refresh(tab)} />}
      />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      <Tabs value={tab} onChange={(_event, next: CardCodeTab) => setTab(next)} sx={{ mb: 3 }}>
        <Tab value="codes" label={t('adminCardCodes.tabs.codes')} />
        <Tab value="types" label={t('adminCardCodes.tabs.types')} />
      </Tabs>
      {tab === 'codes' ? <CardCodePanel t={t} locale={locale} state={state} /> : null}
      {tab === 'types' ? <CardCodeTypePanel t={t} locale={locale} state={state} /> : null}
      <CardCodeGenerateDialog
        t={t}
        open={state.generateOpen}
        types={state.activeTypes}
        currency={state.systemCurrency}
        submitting={state.submitting}
        onClose={() => state.setGenerateOpen(false)}
        onSubmit={state.submitGenerate}
      />
      <CardCodeTypeDialog
        t={t}
        open={state.typeDialogOpen}
        item={state.editingType}
        submitting={state.submitting}
        onClose={state.closeTypeDialog}
        onSubmit={state.submitType}
      />
    </DashboardContent>
  );
}

function CardCodePanel({
  t,
  locale,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useCardCodeManagementState>;
}) {
  return (
    <Card>
      <CardCodeToolbar
        t={t}
        filters={state.codeFilters}
        types={state.typeOptions}
        selectedCount={state.codeTable.selected.length}
        busy={state.submitting}
        onChange={state.changeCodeFilters}
        onGenerate={() => state.setGenerateOpen(true)}
        onExportCsv={() => void state.exportCodes('csv')}
        onExportTxt={() => void state.exportCodes('txt')}
        onEnable={() => void state.batchStatus('active')}
        onDisable={() => void state.batchStatus('disabled')}
      />
      <CardCodeTable
        t={t}
        locale={locale}
        rows={state.codes.data?.items ?? []}
        total={state.codes.data?.total ?? 0}
        loading={state.codes.isLoading}
        page={state.codeTable.page}
        rowsPerPage={state.codeTable.rowsPerPage}
        selected={state.codeTable.selected}
        onSelectRow={state.codeTable.onSelectRow}
        onSelectAllRows={state.codeTable.onSelectAllRows}
        onPageChange={state.codeTable.onChangePage}
        onRowsPerPageChange={state.codeTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function CardCodeTypePanel({
  t,
  locale,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useCardCodeManagementState>;
}) {
  return (
    <Card>
      <CardCodeTypeToolbar
        t={t}
        filters={state.typeFilters}
        busy={state.submitting}
        onChange={state.changeTypeFilters}
        onCreate={state.openCreateType}
      />
      <CardCodeTypeTable
        t={t}
        locale={locale}
        rows={state.types.data?.items ?? []}
        total={state.types.data?.total ?? 0}
        loading={state.types.isLoading}
        page={state.typeTable.page}
        rowsPerPage={state.typeTable.rowsPerPage}
        onEdit={state.openEditType}
        onPageChange={state.typeTable.onChangePage}
        onRowsPerPageChange={state.typeTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function ErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}
