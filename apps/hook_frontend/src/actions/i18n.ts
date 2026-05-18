'use client';

import type {
  I18nApiEnvelope,
  TranslationEntry,
  TranslationLanguage,
  I18nResourceResponse,
  TranslationEntryInput,
  TranslationLanguageInput,
  TranslationBundleResponse,
  TranslationEntryListResponse,
  TranslationLanguageListResponse,
} from 'src/types/i18n';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

type TranslationFilters = {
  search?: string;
  enabled?: boolean;
  namespace?: string;
  group_key?: string;
  lang_code?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useAdminI18nResource(lang: string, namespace = 'admin') {
  const key = [endpoints.i18n.resources, { params: { lang, namespace } }] as const;
  const { data, error, isLoading, isValidating, mutate: refresh } = useSWR<
    I18nApiEnvelope<I18nResourceResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const resource = data ? requireApiData(data) : undefined;
    return { data: resource, error, isLoading, isValidating, refresh };
  }, [data, error, isLoading, isValidating, refresh]);
}

export function useI18nResource(lang: string, namespace: string) {
  return useAdminI18nResource(lang, namespace);
}

export function useTranslationLanguages(page: number, pageSize: number, filters: TranslationFilters = {}) {
  const key = [endpoints.adminI18n.languages, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  const { data, error, isLoading, isValidating, mutate: refresh } = useSWR<
    I18nApiEnvelope<TranslationLanguageListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.languages ?? [],
      total: pageData?.total ?? 0,
      error,
      isLoading,
      isValidating,
      refresh,
    };
  }, [data, error, isLoading, isValidating, refresh]);
}

export function useTranslationEntries(page: number, pageSize: number, filters: TranslationFilters = {}) {
  const key = [endpoints.adminI18n.translations, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  const { data, error, isLoading, isValidating, mutate: refresh } = useSWR<
    I18nApiEnvelope<TranslationEntryListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.translations ?? [],
      total: pageData?.total ?? 0,
      error,
      isLoading,
      isValidating,
      refresh,
    };
  }, [data, error, isLoading, isValidating, refresh]);
}

export async function createTranslationLanguage(payload: TranslationLanguageInput) {
  const language = await requestData<TranslationLanguage>(axios.post(endpoints.adminI18n.languages, payload));
  await mutateI18n();
  return language;
}

export async function updateTranslationLanguage(code: string, payload: Partial<TranslationLanguageInput>) {
  const language = await requestData<TranslationLanguage>(
    axios.patch(endpoints.adminI18n.language(code), payload)
  );
  await mutateI18n();
  return language;
}

export async function deleteTranslationLanguage(code: string) {
  await requestData<void>(axios.delete(endpoints.adminI18n.language(code)));
  await mutateI18n();
}

export async function createTranslationEntry(payload: TranslationEntryInput) {
  const entry = await requestData<TranslationEntry>(axios.post(endpoints.adminI18n.translations, payload));
  await mutateI18n();
  return entry;
}

export async function updateTranslationEntry(id: string, payload: Partial<TranslationEntryInput>) {
  const entry = await requestData<TranslationEntry>(
    axios.patch(endpoints.adminI18n.translation(id), payload)
  );
  await mutateI18n();
  return entry;
}

export async function deleteTranslationEntry(id: string) {
  await requestData<void>(axios.delete(endpoints.adminI18n.translation(id)));
  await mutateI18n();
}

export async function upsertTranslationBundle(
  namespace: string,
  groupKey: string,
  itemKey: string,
  values: Record<string, string>,
  enabled: boolean
) {
  const bundle = await requestData<TranslationBundleResponse>(
    axios.put(endpoints.adminI18n.translationBundle(namespace, groupKey, itemKey), {
      values,
      enabled,
    })
  );
  await mutateI18n();
  return bundle;
}

async function requestData<T>(request: Promise<{ data: I18nApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function mutateI18n() {
  await mutate((key) => Array.isArray(key) && typeof key[0] === 'string' && key[0].includes('/i18n/'));
}
