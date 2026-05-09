'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type { WalletBalanceResponse, WalletTransactionsResponse } from 'src/types/wallet';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

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
