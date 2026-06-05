'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { toReportFilters } from './admin-affiliate-filters';

import { downloadAdminAffiliateCsv } from 'src/actions/affiliate';

import { toast } from 'src/components/snackbar';

export function exportDetailsHandler(
  t: ReturnType<typeof useTranslate>['t'],
  filters: ReturnType<typeof toReportFilters>
) {
  return async () => {
    try {
      await downloadAdminAffiliateCsv('affiliate-commissions.csv', {
        referrer_search: filters.referrer_search,
        referred_search: filters.referred_search,
        start_at: filters.start_date,
        end_at: filters.end_date,
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };
}

export function exportReportHandler(
  t: ReturnType<typeof useTranslate>['t'],
  filters: ReturnType<typeof toReportFilters>,
  exportType: 'daily' | 'referrers'
) {
  return async () => {
    try {
      await downloadAdminAffiliateCsv(`affiliate-${exportType}.csv`, {
        ...filters,
        export_type: exportType,
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };
}
