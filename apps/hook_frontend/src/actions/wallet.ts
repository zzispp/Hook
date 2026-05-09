'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  WalletBalanceResponse,
  AdminWalletListResponse,
  AdminWalletLedgerResponse,
  AdminWalletAdjustmentInput,
  WalletTransactionsResponse,
  AdminWalletAdjustmentResponse,
  AdminWalletTransactionsResponse,
} from 'src/types/wallet';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useWalletBalance() {
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<WalletBalanceResponse>
  >(endpoints.wallet.balance, fetcher, swrOptions);

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    const balance = data?.success ? requireApiData(data) : undefined;
    return {
      data: balance,
      wallet: balance?.wallet,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

export function useWalletTransactions(page: number, pageSize: number) {
  const key = [
    endpoints.wallet.transactions,
    { params: { page: page + 1, page_size: pageSize } },
  ] as const;

  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<WalletTransactionsResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    const transactions = data?.success ? requireApiData(data) : undefined;
    return {
      data: transactions,
      items: transactions?.items ?? [],
      total: transactions?.total ?? 0,
      wallet: transactions?.wallet,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

export type AdminWalletFilters = {
  search?: string;
  status?: string;
};

export type AdminWalletLedgerFilters = {
  category?: string;
  reason_code?: string;
  owner_type?: string;
};

export function useAdminWallets(page: number, pageSize: number, filters: AdminWalletFilters = {}) {
  const key = [endpoints.adminWallets.list, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useWalletResource<AdminWalletListResponse>(key);
}

export function useAdminWalletLedger(page: number, pageSize: number, filters: AdminWalletLedgerFilters = {}) {
  const key = [endpoints.adminWallets.ledger, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useWalletResource<AdminWalletLedgerResponse>(key);
}

export function useAdminWalletTransactions(walletId: string | null, page: number, pageSize: number) {
  const key = walletId
    ? ([endpoints.adminWallets.transactions(walletId), { params: pageQuery(page, pageSize) }] as const)
    : null;
  return useWalletResource<AdminWalletTransactionsResponse>(key);
}

export async function adjustAdminWallet(walletId: string, payload: AdminWalletAdjustmentInput) {
  const response = await axios.post<ApiEnvelope<AdminWalletAdjustmentResponse>>(
    endpoints.adminWallets.adjust(walletId),
    payload
  );
  return requireApiData(response.data);
}

function useWalletResource<T>(key: Key) {
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
