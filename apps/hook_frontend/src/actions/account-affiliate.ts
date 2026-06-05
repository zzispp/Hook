'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  AffiliateSummary,
  AffiliateReferralListResponse,
  AffiliateCommissionListResponse,
} from 'src/types/account-affiliate';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type AffiliateReferralFilters = {
  search?: string;
  referred_start?: string;
  referred_end?: string;
};

export type AffiliateCommissionFilters = {
  referred_search?: string;
  recharge_order_no?: string;
  start_at?: string;
  end_at?: string;
  min_commission_amount?: number;
  max_commission_amount?: number;
};

export function useAffiliateSummary() {
  return useAffiliateResource<AffiliateSummary>(endpoints.account.affiliateSummary);
}

export function useAffiliateReferrals(page: number, pageSize: number, filters: AffiliateReferralFilters) {
  const key = [
    endpoints.account.affiliateReferrals,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AffiliateReferralListResponse>(key);
}

export function useAffiliateCommissions(
  page: number,
  pageSize: number,
  filters: AffiliateCommissionFilters
) {
  const key = [
    endpoints.account.affiliateCommissions,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AffiliateCommissionListResponse>(key);
}

export async function downloadAffiliateCommissionCsv(
  filename: string,
  params: AffiliateCommissionFilters
) {
  const response = await axios.get(endpoints.account.affiliateCommissionsExport, {
    params: { page: 1, page_size: 1, ...params },
    responseType: 'blob',
  });
  downloadBlob(filename, response.data, 'text/csv');
}

function useAffiliateResource<T>(key: Key) {
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<T>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    return {
      data: data?.success ? requireApiData(data) : undefined,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

function downloadBlob(filename: string, data: BlobPart, contentType: string) {
  const blob = new Blob([data], { type: contentType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}
