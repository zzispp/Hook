'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { AdminAffiliateRelation } from 'src/types/affiliate';

import { useMemo, useState, useCallback } from 'react';

import {
  useAdminAffiliateReports,
  useAdminAffiliateOverview,
  useAdminAffiliateRelations,
  useAdminAffiliateCommissions,
  updateAdminAffiliateRelation,
  useAdminAffiliateRelationChanges,
} from 'src/actions/affiliate';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import { exportReportHandler, exportDetailsHandler } from './admin-affiliate-export-actions';
import {
  toReportFilters,
  toRelationFilters,
  toCommissionFilters,
  DEFAULT_REPORT_FILTERS,
  DEFAULT_RELATION_FILTERS,
  DEFAULT_COMMISSION_FILTERS,
} from './admin-affiliate-filters';

export type AffiliateTab = 'relations' | 'relationChanges' | 'commissions' | 'reports';
type DialogMode = 'rebind' | 'clear';

export type RelationDialogState = {
  mode: DialogMode;
  relation: AdminAffiliateRelation;
  referrerAffCode: string;
  reason: string;
};

export function useAdminAffiliateManagementState(t: ReturnType<typeof useTranslate>['t']) {
  const tables = useAffiliateTables();
  const filters = useAffiliateFilterState();
  const data = useAffiliateData(tables, filters);
  const [dialog, setDialog] = useState<RelationDialogState | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const closeDialog = useCallback(() => setDialog(null), []);
  const refresh = useCallback(
    (tab: AffiliateTab) =>
      refreshTab(
        tab,
        data.overview.refresh,
        data.relations.refresh,
        data.relationChanges.refresh,
        data.commissions.refresh,
        data.reports.refresh
      ),
    [
      data.commissions.refresh,
      data.overview.refresh,
      data.relationChanges.refresh,
      data.relations.refresh,
      data.reports.refresh,
    ]
  );

  return {
    overview: data.overview,
    relations: data.relations,
    relationChanges: data.relationChanges,
    commissions: data.commissions,
    reports: data.reports,
    relationTable: tables.relationTable,
    relationChangeTable: tables.relationChangeTable,
    commissionTable: tables.commissionTable,
    reportTable: tables.reportTable,
    relationFilters: filters.relationFilters,
    commissionFilters: filters.commissionFilters,
    reportFilters: filters.reportFilters,
    dialog,
    submitting,
    closeDialog,
    refresh,
    openRebind: openRelationDialog(setDialog, 'rebind'),
    openClear: openRelationDialog(setDialog, 'clear'),
    changeRelationFilters: filterHandler(tables.relationTable, filters.setRelationFilters),
    changeCommissionFilters: filterHandler(tables.commissionTable, filters.setCommissionFilters),
    changeReportFilters: filterHandler(tables.reportTable, filters.setReportFilters),
    changeDialog: dialogPatchHandler(dialog, setDialog),
    submitDialog: submitDialogHandler(
      t,
      dialog,
      setSubmitting,
      closeDialog,
      data.relations.refresh,
      data.overview.refresh
    ),
    exportDetails: exportDetailsHandler(t, data.reportQuery),
    exportDaily: exportReportHandler(t, data.reportQuery, 'daily'),
    exportReferrers: exportReportHandler(t, data.reportQuery, 'referrers'),
    errorMessage: affiliateErrorMessage(data),
  };
}

function useAffiliateTables() {
  return {
    relationTable: useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' }),
    relationChangeTable: useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' }),
    commissionTable: useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' }),
    reportTable: useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'commission_amount' }),
  };
}

function useAffiliateFilterState() {
  const [relationFilters, setRelationFilters] = useState(DEFAULT_RELATION_FILTERS);
  const [commissionFilters, setCommissionFilters] = useState(DEFAULT_COMMISSION_FILTERS);
  const [reportFilters, setReportFilters] = useState(DEFAULT_REPORT_FILTERS);
  return {
    relationFilters,
    commissionFilters,
    reportFilters,
    setRelationFilters,
    setCommissionFilters,
    setReportFilters,
  };
}

function useAffiliateData(
  tables: ReturnType<typeof useAffiliateTables>,
  filters: ReturnType<typeof useAffiliateFilterState>
) {
  const overview = useAdminAffiliateOverview();
  const relationQuery = useMemo(
    () => toRelationFilters(filters.relationFilters),
    [filters.relationFilters]
  );
  const commissionQuery = useMemo(
    () => toCommissionFilters(filters.commissionFilters),
    [filters.commissionFilters]
  );
  const reportQuery = useMemo(
    () => toReportFilters(filters.reportFilters),
    [filters.reportFilters]
  );
  const relations = useAdminAffiliateRelations(
    tables.relationTable.page,
    tables.relationTable.rowsPerPage,
    relationQuery
  );
  const relationChanges = useAdminAffiliateRelationChanges(
    tables.relationChangeTable.page,
    tables.relationChangeTable.rowsPerPage,
    {}
  );
  const commissions = useAdminAffiliateCommissions(
    tables.commissionTable.page,
    tables.commissionTable.rowsPerPage,
    commissionQuery
  );
  const reports = useAdminAffiliateReports(
    tables.reportTable.page,
    tables.reportTable.rowsPerPage,
    reportQuery
  );
  return { overview, relations, relationChanges, commissions, reports, reportQuery };
}

function affiliateErrorMessage(data: ReturnType<typeof useAffiliateData>) {
  return (
    data.overview.error?.message ??
    data.relations.error?.message ??
    data.relationChanges.error?.message ??
    data.commissions.error?.message ??
    data.reports.error?.message
  );
}

function refreshTab(
  tab: AffiliateTab,
  overview: VoidFunction,
  relations: VoidFunction,
  relationChanges: VoidFunction,
  commissions: VoidFunction,
  reports: VoidFunction
) {
  void overview();
  if (tab === 'relations') void relations();
  if (tab === 'relationChanges') void relationChanges();
  if (tab === 'commissions') void commissions();
  if (tab === 'reports') void reports();
}

function openRelationDialog(
  setDialog: React.Dispatch<React.SetStateAction<RelationDialogState | null>>,
  mode: DialogMode
) {
  return (relation: AdminAffiliateRelation) => {
    setDialog({
      mode,
      relation,
      referrerAffCode: relation.referrer?.affiliate_code ?? '',
      reason: '',
    });
  };
}

function filterHandler<T>(
  table: ReturnType<typeof useTable>,
  setFilters: React.Dispatch<React.SetStateAction<T>>
) {
  return (next: T) => {
    table.onResetPage();
    setFilters(next);
  };
}

function dialogPatchHandler(
  dialog: RelationDialogState | null,
  setDialog: React.Dispatch<React.SetStateAction<RelationDialogState | null>>
) {
  return (patch: Partial<Pick<RelationDialogState, 'referrerAffCode' | 'reason'>>) => {
    if (!dialog) return;
    setDialog({ ...dialog, ...patch });
  };
}

function submitDialogHandler(
  t: ReturnType<typeof useTranslate>['t'],
  dialog: RelationDialogState | null,
  setSubmitting: (value: boolean) => void,
  closeDialog: VoidFunction,
  refreshRelations: VoidFunction,
  refreshOverview: VoidFunction
) {
  return async () => {
    if (!dialog || !validateDialog(dialog, t)) return;
    setSubmitting(true);
    try {
      await updateAdminAffiliateRelation(dialog.relation.user.id, relationPayload(dialog));
      toast.success(t('adminAffiliates.messages.relationUpdated'));
      refreshRelations();
      refreshOverview();
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

function validateDialog(dialog: RelationDialogState, t: ReturnType<typeof useTranslate>['t']) {
  if (!dialog.reason.trim()) {
    toast.error(t('adminAffiliates.messages.reasonRequired'));
    return false;
  }
  if (dialog.mode === 'rebind' && !dialog.referrerAffCode.trim()) {
    toast.error(t('adminAffiliates.messages.referrerCodeRequired'));
    return false;
  }
  return true;
}

function relationPayload(dialog: RelationDialogState) {
  return {
    referrer_aff_code: dialog.mode === 'rebind' ? dialog.referrerAffCode.trim() : undefined,
    clear_referrer: dialog.mode === 'clear',
    reason: dialog.reason.trim(),
  };
}
