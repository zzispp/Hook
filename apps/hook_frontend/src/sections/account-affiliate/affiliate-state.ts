'use client';

import type { useTranslate } from 'src/locales/use-locales';

import { useMemo, useState, useCallback } from 'react';

import {
  useAffiliateSummary,
  useAffiliateReferrals,
  useAffiliateCommissions,
  downloadAffiliateCommissionCsv,
} from 'src/actions/account-affiliate';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import {
  toReferralFilters,
  toCommissionFilters,
  DEFAULT_REFERRAL_FILTERS,
  DEFAULT_COMMISSION_FILTERS,
} from './affiliate-filters';

export type AffiliateCenterTab = 'referrals' | 'commissions';

export function useAffiliateCenterState(t: ReturnType<typeof useTranslate>['t']) {
  const referralTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'referred_at' });
  const commissionTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const [referralFilters, setReferralFilters] = useState(DEFAULT_REFERRAL_FILTERS);
  const [commissionFilters, setCommissionFilters] = useState(DEFAULT_COMMISSION_FILTERS);
  const summary = useAffiliateSummary();
  const referralQuery = useMemo(() => toReferralFilters(referralFilters), [referralFilters]);
  const commissionQuery = useMemo(() => toCommissionFilters(commissionFilters), [commissionFilters]);
  const referrals = useAffiliateReferrals(referralTable.page, referralTable.rowsPerPage, referralQuery);
  const commissions = useAffiliateCommissions(
    commissionTable.page,
    commissionTable.rowsPerPage,
    commissionQuery
  );
  const refresh = useCallback(
    (tab: AffiliateCenterTab) => {
      void summary.refresh();
      if (tab === 'referrals') void referrals.refresh();
      if (tab === 'commissions') void commissions.refresh();
    },
    [commissions, referrals, summary]
  );

  return {
    summary,
    referrals,
    commissions,
    referralTable,
    commissionTable,
    referralFilters,
    commissionFilters,
    refresh,
    changeReferralFilters: filterHandler(referralTable, setReferralFilters),
    changeCommissionFilters: filterHandler(commissionTable, setCommissionFilters),
    exportCommissions: exportHandler(t, commissionQuery),
    errorMessage: summary.error?.message ?? referrals.error?.message ?? commissions.error?.message,
  };
}

function filterHandler<T>(table: ReturnType<typeof useTable>, setFilters: React.Dispatch<React.SetStateAction<T>>) {
  return (next: T) => {
    table.onResetPage();
    setFilters(next);
  };
}

function exportHandler(t: ReturnType<typeof useTranslate>['t'], filters: ReturnType<typeof toCommissionFilters>) {
  return async () => {
    try {
      await downloadAffiliateCommissionCsv('affiliate-commissions.csv', filters);
      toast.success(t('affiliateCenter.messages.exportStarted'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };
}
