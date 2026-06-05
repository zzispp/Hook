'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  AdminAffiliateReportResponse,
  AdminAffiliateOverviewResponse,
  AdminAffiliateRelationUpdateInput,
  AdminAffiliateRelationListResponse,
  AdminAffiliateCommissionListResponse,
  AdminAffiliateRelationChangeListResponse,
} from 'src/types/affiliate';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type AdminAffiliateRelationFilters = {
  user_search?: string;
  referrer_search?: string;
  has_referrer?: boolean;
  referred_start?: string;
  referred_end?: string;
};

export type AdminAffiliateCommissionFilters = {
  referrer_search?: string;
  referred_search?: string;
  recharge_order_id?: string;
  start_at?: string;
  end_at?: string;
  min_commission_amount?: number;
  max_commission_amount?: number;
};

export type AdminAffiliateRelationChangeFilters = {
  user_search?: string;
  operator_search?: string;
  start_at?: string;
  end_at?: string;
};

export type AdminAffiliateReportFilters = {
  start_date?: string;
  end_date?: string;
  referrer_search?: string;
  referred_search?: string;
};

export function useAdminAffiliateOverview() {
  return useAffiliateResource<AdminAffiliateOverviewResponse>(endpoints.adminAffiliates.overview);
}

export function useAdminAffiliateRelations(
  page: number,
  pageSize: number,
  filters: AdminAffiliateRelationFilters
) {
  const key = [
    endpoints.adminAffiliates.relations,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AdminAffiliateRelationListResponse>(key);
}

export function useAdminAffiliateCommissions(
  page: number,
  pageSize: number,
  filters: AdminAffiliateCommissionFilters
) {
  const key = [
    endpoints.adminAffiliates.commissions,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AdminAffiliateCommissionListResponse>(key);
}

export function useAdminAffiliateRelationChanges(
  page: number,
  pageSize: number,
  filters: AdminAffiliateRelationChangeFilters
) {
  const key = [
    endpoints.adminAffiliates.relationChanges,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AdminAffiliateRelationChangeListResponse>(key);
}

export function useAdminAffiliateReports(
  page: number,
  pageSize: number,
  filters: AdminAffiliateReportFilters
) {
  const key = [
    endpoints.adminAffiliates.reports,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useAffiliateResource<AdminAffiliateReportResponse>(key);
}

export async function updateAdminAffiliateRelation(
  userId: string,
  payload: AdminAffiliateRelationUpdateInput
) {
  const response = await axios.patch<ApiEnvelope<unknown>>(
    endpoints.adminAffiliates.relation(userId),
    payload
  );
  return requireApiData(response.data);
}

export async function downloadAdminAffiliateCsv(
  filename: string,
  params: AdminAffiliateCommissionFilters & AdminAffiliateReportFilters & { export_type?: string }
) {
  const response = await axios.get(endpoints.adminAffiliates.export, {
    params,
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
    const apiError =
      data && !data.success ? new Error(data.message || 'Request failed') : undefined;
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
