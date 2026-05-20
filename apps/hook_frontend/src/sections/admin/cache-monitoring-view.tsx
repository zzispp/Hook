'use client';

import type { CacheAffinityItem } from 'src/types/cache-monitoring';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useCacheAffinities, deleteCacheAffinity, clearCacheAffinities } from 'src/actions/cache-monitoring';

import { useTable } from 'src/components/table';
import { toast } from 'src/components/snackbar';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { CacheMonitoringTable } from './cache-monitoring-table';

const DEFAULT_ROWS_PER_PAGE = 20;

export function CacheMonitoringView() {
  const { t } = useTranslate('admin');
  const table = useTable({
    defaultRowsPerPage: DEFAULT_ROWS_PER_PAGE,
    defaultOrderBy: 'ttl_seconds',
  });
  const [search, setSearch] = useState('');
  const [deletingItem, setDeletingItem] = useState<CacheAffinityItem | null>(null);
  const [clearAllOpen, setClearAllOpen] = useState(false);
  const [submitting, setSubmitting] = useState<'delete' | 'clear' | null>(null);
  const records = useCacheAffinities(table.page, table.rowsPerPage, search.trim());

  const handleSearchChange = useCallback(
    (value: string) => {
      table.onResetPage();
      setSearch(value);
    },
    [table]
  );

  const handleDelete = useCallback(async () => {
    if (!deletingItem) return;
    setSubmitting('delete');
    try {
      await deleteCacheAffinity(deletingItem);
      table.onUpdatePageDeleteRow(records.items.length);
      setDeletingItem(null);
      toast.success(t('messages.cacheAffinityDeleted'));
      await records.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    } finally {
      setSubmitting(null);
    }
  }, [deletingItem, records, t, table]);

  const handleClearAll = useCallback(async () => {
    setSubmitting('clear');
    try {
      await clearCacheAffinities();
      table.onResetPage();
      setClearAllOpen(false);
      toast.success(t('messages.cacheAffinitiesCleared'));
      await records.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    } finally {
      setSubmitting(null);
    }
  }, [records, t, table]);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.cacheMonitoring}
        action={
          <Stack direction="row" spacing={1}>
            <Button
              color="error"
              variant="outlined"
              disabled={records.total === 0 || records.isLoading}
              onClick={() => setClearAllOpen(true)}
            >
              {t('cacheMonitoring.actions.clearAll')}
            </Button>
            <RefreshButton loading={records.isValidating} onClick={() => void records.refresh()} />
          </Stack>
        }
      />

      <Stack spacing={3}>
        {records.error ? <Alert severity="error">{records.error.message}</Alert> : null}
        <Card>
          <Stack
            direction={{ xs: 'column', md: 'row' }}
            spacing={2}
            alignItems={{ xs: 'stretch', md: 'center' }}
            sx={{ p: 2.5 }}
          >
            <TextField
              fullWidth
              size="small"
              value={search}
              placeholder={t('cacheMonitoring.searchPlaceholder')}
              onChange={(event) => handleSearchChange(event.target.value)}
            />
            <Button color="inherit" variant="outlined" onClick={() => handleSearchChange('')}>
              {t('common.clear')}
            </Button>
          </Stack>
          <CacheMonitoringTable
            loading={records.isLoading}
            rows={records.items}
            total={records.total}
            table={table}
            onDelete={setDeletingItem}
          />
        </Card>
      </Stack>

      <ConfirmDialog
        open={Boolean(deletingItem)}
        title={t('cacheMonitoring.dialogs.deleteTitle')}
        content={t('cacheMonitoring.dialogs.deleteContent', {
          key: deletingItem?.token_prefix || deletingItem?.token_name || deletingItem?.affinity_key || '',
        })}
        cancelText={t('common.cancel')}
        onClose={() => setDeletingItem(null)}
        action={
          <Button
            variant="contained"
            color="error"
            loading={submitting === 'delete'}
            onClick={() => void handleDelete()}
          >
            {t('common.delete')}
          </Button>
        }
      />

      <ConfirmDialog
        open={clearAllOpen}
        title={t('cacheMonitoring.dialogs.clearAllTitle')}
        content={t('cacheMonitoring.dialogs.clearAllContent')}
        cancelText={t('common.cancel')}
        onClose={() => setClearAllOpen(false)}
        action={
          <Button
            variant="contained"
            color="error"
            loading={submitting === 'clear'}
            onClick={() => void handleClearAll()}
          >
            {t('cacheMonitoring.actions.clearAll')}
          </Button>
        }
      />
    </DashboardContent>
  );
}
