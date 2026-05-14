'use client';

import type { CardCodeType } from 'src/types/card-code';
import type { useTranslate } from 'src/locales/use-locales';

import { useMemo, useState, useCallback } from 'react';

import {
  useCardCodes,
  useCardCodeTypes,
  generateCardCodes,
  createCardCodeType,
  updateCardCodeType,
  batchUpdateCardCodes,
} from 'src/actions/card-code';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import { CARD_CODE_MAX_PAGE_SIZE } from './card-code-constants';
import { exportableCardCodes, downloadCardCodesCsv, downloadCardCodesTxt } from './card-code-export';
import {
  toCardCodeFilters,
  toCardCodeTypeFilters,
  DEFAULT_CARD_CODE_FILTERS,
  DEFAULT_CARD_CODE_TYPE_FILTERS,
} from './card-code-filters';

export type CardCodeTab = 'codes' | 'types';

export function useCardCodeManagementState(t: ReturnType<typeof useTranslate>['t']) {
  const codeTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const typeTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'updated_at' });
  const [codeFilters, setCodeFilters] = useState(DEFAULT_CARD_CODE_FILTERS);
  const [typeFilters, setTypeFilters] = useState(DEFAULT_CARD_CODE_TYPE_FILTERS);
  const [generateOpen, setGenerateOpen] = useState(false);
  const [typeDialogOpen, setTypeDialogOpen] = useState(false);
  const [editingType, setEditingType] = useState<CardCodeType | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const cardCodeFilters = useMemo(() => toCardCodeFilters(codeFilters), [codeFilters]);
  const cardCodeTypeFilters = useMemo(() => toCardCodeTypeFilters(typeFilters), [typeFilters]);
  const codes = useCardCodes(codeTable.page, codeTable.rowsPerPage, cardCodeFilters);
  const types = useCardCodeTypes(typeTable.page, typeTable.rowsPerPage, cardCodeTypeFilters);
  const options = useCardCodeTypes(0, CARD_CODE_MAX_PAGE_SIZE);
  const selectedCodes = useMemo(
    () => (codes.data?.items ?? []).filter((item) => codeTable.selected.includes(item.id)),
    [codeTable.selected, codes.data?.items]
  );

  const refresh = useCallback(
    (tab: CardCodeTab) => {
      void (tab === 'codes' ? codes.refresh() : types.refresh());
    },
    [codes, types]
  );

  return {
    codeTable,
    typeTable,
    codeFilters,
    typeFilters,
    codes,
    types,
    submitting,
    generateOpen,
    editingType,
    typeDialogOpen,
    typeOptions: options.data?.items ?? [],
    activeTypes: (options.data?.items ?? []).filter((item) => item.status === 'active'),
    errorMessage: codes.error?.message ?? types.error?.message ?? options.error?.message,
    setGenerateOpen,
    refresh,
    changeCodeFilters: codeFilterHandler(codeTable, setCodeFilters),
    changeTypeFilters: typeFilterHandler(typeTable, setTypeFilters),
    openCreateType: () => {
      setEditingType(null);
      setTypeDialogOpen(true);
    },
    openEditType: (item: CardCodeType) => {
      setEditingType(item);
      setTypeDialogOpen(true);
    },
    closeTypeDialog: () => setTypeDialogOpen(false),
    submitType: submitTypeHandler(t, editingType, setSubmitting, setTypeDialogOpen, types.refresh, options.refresh),
    submitGenerate: submitGenerateHandler(t, setSubmitting, setGenerateOpen, codes.refresh, codeTable.setSelected),
    batchStatus: batchStatusHandler(t, codeTable.selected, setSubmitting, codes.refresh, codeTable.setSelected),
    exportCodes: exportHandler(t, selectedCodes, cardCodeFilters, setSubmitting),
  };
}

function codeFilterHandler(
  table: ReturnType<typeof useTable>,
  setFilters: (value: typeof DEFAULT_CARD_CODE_FILTERS) => void
) {
  return (next: typeof DEFAULT_CARD_CODE_FILTERS) => {
    table.onResetPage();
    table.setSelected([]);
    setFilters(next);
  };
}

function typeFilterHandler(
  table: ReturnType<typeof useTable>,
  setFilters: (value: typeof DEFAULT_CARD_CODE_TYPE_FILTERS) => void
) {
  return (next: typeof DEFAULT_CARD_CODE_TYPE_FILTERS) => {
    table.onResetPage();
    setFilters(next);
  };
}

function submitTypeHandler(
  t: ReturnType<typeof useTranslate>['t'],
  editingType: CardCodeType | null,
  setSubmitting: (value: boolean) => void,
  setOpen: (value: boolean) => void,
  refreshTypes: VoidFunction,
  refreshOptions: VoidFunction
) {
  return async (input: Parameters<typeof createCardCodeType>[0]) => {
    setSubmitting(true);
    try {
      if (editingType) {
        await updateCardCodeType(editingType.id, input);
      } else {
        await createCardCodeType(input);
      }
      toast.success(t('adminCardCodes.messages.typeSaved'));
      refreshTypes();
      refreshOptions();
      setOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

function submitGenerateHandler(
  t: ReturnType<typeof useTranslate>['t'],
  setSubmitting: (value: boolean) => void,
  setOpen: (value: boolean) => void,
  refresh: VoidFunction,
  setSelected: (value: string[]) => void
) {
  return async (input: Parameters<typeof generateCardCodes>[0]) => {
    setSubmitting(true);
    try {
      const result = await generateCardCodes(input);
      toast.success(t('adminCardCodes.messages.generated', { count: result.total }));
      refresh();
      setSelected([]);
      setOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

function batchStatusHandler(
  t: ReturnType<typeof useTranslate>['t'],
  ids: string[],
  setSubmitting: (value: boolean) => void,
  refresh: VoidFunction,
  setSelected: (value: string[]) => void
) {
  return async (status: 'active' | 'disabled') => {
    setSubmitting(true);
    try {
      const result = await batchUpdateCardCodes({ ids, status });
      toast.success(t('adminCardCodes.messages.batchUpdated', { count: result.updated_count }));
      refresh();
      setSelected([]);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

function exportHandler(
  t: ReturnType<typeof useTranslate>['t'],
  selected: Parameters<typeof exportableCardCodes>[0],
  filters: Parameters<typeof exportableCardCodes>[1],
  setSubmitting: (value: boolean) => void
) {
  return async (format: 'csv' | 'txt') => {
    setSubmitting(true);
    try {
      const items = await exportableCardCodes(selected, filters);
      if (format === 'csv') {
        downloadCardCodesCsv(t, items);
      } else {
        downloadCardCodesTxt(items);
      }
      toast.success(t('adminCardCodes.messages.exported', { count: items.length }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}
