'use client';

import type {
  TranslationTab} from './translation-management-state';

import { useMemo, useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useTranslationEntries, useTranslationLanguages } from 'src/actions/i18n';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';

import { AdminBreadcrumbs } from './shared';
import { RefreshAddActions } from './admin-page-actions';
import { translationRows } from './translation-management-utils';
import { TranslationValuesTable } from './translation-values-table';
import { TranslationLanguagesTable } from './translation-languages-table';
import { TranslationValueFormDialog } from './translation-value-form-dialog';
import { TranslationLanguageFormDialog } from './translation-language-form-dialog';
import { useTranslationManagementActions } from './translation-management-actions';
import { toEnabledFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';
import {
  DeleteTranslationValueDialog,
  DeleteTranslationLanguageDialog,
} from './translation-delete-dialogs';
import {
  useTranslationValueForm,
  useTranslationDeleteState,
  useTranslationLanguageForm,
} from './translation-management-state';

export function TranslationManagementView() {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<TranslationTab>('values');
  const [valueFilters, setValueFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const [languageFilters, setLanguageFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const data = useTranslationManagementData({ languageFilters, valueFilters });
  const valueForm = useTranslationValueForm(data.allLanguages.items);
  const languageForm = useTranslationLanguageForm();
  const deleteState = useTranslationDeleteState();
  const actions = useTranslationManagementActions({ deleteState, languageForm, valueForm });
  const filters = tab === 'values' ? valueFilters : languageFilters;
  const activeResource = tab === 'values' ? data.entries : data.languages;
  const handleFiltersChange = useTranslationFiltersChange({
    data,
    setLanguageFilters,
    setValueFilters,
    tab,
  });

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.translationManagement}
        action={
          <RefreshAddActions
            loading={activeResource.isLoading}
            addLabel={t(tab === 'values' ? 'translations.actions.addValue' : 'translations.actions.addLanguage')}
            onAdd={tab === 'values' ? valueForm.openCreate : languageForm.openCreate}
            onRefresh={() => void activeResource.refresh()}
          />
        }
      />

      <Card>
        <Tabs value={tab} onChange={(_event, value: TranslationTab) => setTab(value)} sx={{ px: 2.5 }}>
          <Tab value="values" label={t('translations.tabs.values')} />
          <Tab value="languages" label={t('translations.tabs.languages')} />
        </Tabs>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t(
            tab === 'values' ? 'translations.filters.searchValues' : 'translations.filters.searchLanguages'
          )}
          onChange={handleFiltersChange}
        />
        {tab === 'values' ? (
          <TranslationValuesTable
            languages={data.allLanguages.items}
            loading={data.entries.isLoading}
            rows={data.valueRows}
            table={data.valueTable}
            total={data.entries.total}
            onDelete={deleteState.setValueTarget}
            onEdit={valueForm.openEdit}
          />
        ) : (
          <TranslationLanguagesTable
            loading={data.languages.isLoading}
            rows={data.languages.items}
            table={data.languageTable}
            total={data.languages.total}
            onDelete={deleteState.setLanguageTarget}
            onEdit={languageForm.openEdit}
          />
        )}
      </Card>

      <TranslationValueFormDialog
        editing={valueForm.editing}
        form={valueForm.form}
        languages={data.allLanguages.items}
        open={valueForm.open}
        submitting={actions.submitting}
        onClose={valueForm.close}
        onFormChange={valueForm.setForm}
        onSubmit={actions.submitValue}
      />
      <TranslationLanguageFormDialog
        editing={languageForm.editing}
        form={languageForm.form}
        open={languageForm.open}
        submitting={actions.submitting}
        onClose={languageForm.close}
        onFormChange={languageForm.setForm}
        onSubmit={actions.submitLanguage}
      />
      <DeleteTranslationValueDialog
        target={deleteState.valueTarget}
        onClose={() => deleteState.setValueTarget(null)}
        onConfirm={actions.confirmDeleteValue}
      />
      <DeleteTranslationLanguageDialog
        target={deleteState.languageTarget}
        onClose={() => deleteState.setLanguageTarget(null)}
        onConfirm={actions.confirmDeleteLanguage}
      />
    </DashboardContent>
  );
}

function useTranslationManagementData({
  languageFilters,
  valueFilters,
}: {
  languageFilters: typeof DEFAULT_ADMIN_FILTERS;
  valueFilters: typeof DEFAULT_ADMIN_FILTERS;
}) {
  const languageTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const valueTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'group_key' });
  const languages = useTranslationLanguages(
    languageTable.page,
    languageTable.rowsPerPage,
    toEnabledFilters(languageFilters)
  );
  const allLanguages = useTranslationLanguages(0, 100, { enabled: true });
  const entries = useTranslationEntries(valueTable.page, valueTable.rowsPerPage, {
    ...toEnabledFilters(valueFilters),
    namespace: 'admin',
  });
  const valueRows = useMemo(() => translationRows(entries.items), [entries.items]);

  return { allLanguages, entries, languages, languageTable, valueRows, valueTable };
}

function useTranslationFiltersChange({
  data,
  setLanguageFilters,
  setValueFilters,
  tab,
}: {
  data: ReturnType<typeof useTranslationManagementData>;
  setLanguageFilters: React.Dispatch<React.SetStateAction<typeof DEFAULT_ADMIN_FILTERS>>;
  setValueFilters: React.Dispatch<React.SetStateAction<typeof DEFAULT_ADMIN_FILTERS>>;
  tab: TranslationTab;
}) {
  return useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      if (tab === 'values') {
        data.valueTable.onResetPage();
        setValueFilters(nextFilters);
        return;
      }
      data.languageTable.onResetPage();
      setLanguageFilters(nextFilters);
    },
    [data.languageTable, data.valueTable, setLanguageFilters, setValueFilters, tab]
  );
}
