'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  RoutingProfile,
  RoutingMetricWindow,
  RoutingRankingsQuery,
  RoutingProfileUpsert,
  RoutingPreviewRequest,
  RoutingPreviewResponse,
  RoutingRankingResponse,
  RoutingDecisionResponse,
  RoutingProfilesResponse,
} from 'src/types/routing';

import { useMemo } from 'react';
import useSWR, { mutate as globalMutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useRoutingProfiles() {
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<RoutingProfilesResponse>
  >(endpoints.routing.profiles, fetcher, swrOptions);

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    const response = data?.success ? requireApiData(data) : undefined;
    return {
      items: response?.profiles ?? [],
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

export async function updateRoutingProfile(id: string, payload: RoutingProfileUpsert) {
  const response = await axios.put<ApiEnvelope<RoutingProfile>>(
    endpoints.routing.profile(id),
    payload
  );
  const profile = requireApiData(response.data);
  await globalMutate(endpoints.routing.profiles);
  return profile;
}

export function useRoutingRankings(query: RoutingRankingsQuery | null, autoRefresh: boolean) {
  const url = query ? rankingUrl(query) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<RoutingRankingResponse>
  >(url, fetcher, {
    ...swrOptions,
    refreshInterval: autoRefresh ? 5000 : 0,
  });

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

export function useRoutingDecision(requestId: string | null) {
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<RoutingDecisionResponse>
  >(requestId ? endpoints.routing.decision(requestId) : null, fetcher, swrOptions);

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

export function useRoutingWindowRankings(
  query: RoutingRankingsQuery | null,
  windows: RoutingMetricWindow[],
  autoRefresh: boolean
) {
  const key = query ? ['routing-window-rankings', query, windows] : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR(key, fetchWindowRankings, {
    ...swrOptions,
    refreshInterval: autoRefresh ? 5000 : 0,
  });

  return useMemo(
    () => ({
      data,
      isLoading,
      error,
      isValidating,
      refresh: mutate,
    }),
    [data, error, isLoading, isValidating, mutate]
  );
}

export async function previewRouting(payload: RoutingPreviewRequest) {
  const response = await axios.post<ApiEnvelope<RoutingPreviewResponse>>(
    endpoints.routing.preview,
    payload
  );
  return requireApiData(response.data);
}

async function fetchWindowRankings([
  ,
  query,
  windows,
]: [
  string,
  RoutingRankingsQuery,
  RoutingMetricWindow[],
]) {
  const entries = await Promise.all(
    windows.map(async (window) => {
      const response = await axios.get<ApiEnvelope<RoutingRankingResponse>>(
        rankingUrl({ ...query, window })
      );
      return [window, requireApiData(response.data)] as const;
    })
  );
  return Object.fromEntries(entries) as Partial<Record<RoutingMetricWindow, RoutingRankingResponse>>;
}

function rankingUrl(query: RoutingRankingsQuery) {
  const params = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    params.set(key, String(value));
  });
  return `${endpoints.routing.rankings}?${params.toString()}`;
}
