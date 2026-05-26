'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  UserGroup,
  UserGroupCreate,
  UserGroupUpdate,
  UserGroupFilters,
  UserGroupPageResponse,
  UserGroupMembersResponse,
} from 'src/types/user-group';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useUserGroups(
  page: number,
  pageSize: number,
  filters: UserGroupFilters = {}
) {
  const disabled = page < 0 || pageSize <= 0;
  const key = disabled ? null : userGroupsKey(page, pageSize, filters);
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<UserGroupPageResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading: disabled ? false : isLoading,
      error,
      isValidating: disabled ? false : isValidating,
      refresh: revalidate,
    };
  }, [data, disabled, error, isLoading, isValidating, revalidate]);
}

export function useUserGroupMembers(
  code: string | undefined,
  page: number,
  pageSize: number,
  filters: UserGroupFilters = {}
) {
  const disabled = !code || page < 0 || pageSize <= 0;
  const key = disabled ? null : userGroupMembersKey(code, page, pageSize, filters);
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<UserGroupMembersResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading: disabled ? false : isLoading,
      error,
      isValidating: disabled ? false : isValidating,
      refresh: revalidate,
    };
  }, [data, disabled, error, isLoading, isValidating, revalidate]);
}

export async function createUserGroup(payload: UserGroupCreate) {
  const group = await requestData<UserGroup>(axios.post(endpoints.adminUserGroups.list, payload));
  await mutateUserGroupResources();
  return group;
}

export async function updateUserGroup(code: string, payload: UserGroupUpdate) {
  const group = await requestData<UserGroup>(
    axios.patch(endpoints.adminUserGroups.byCode(code), payload)
  );
  await mutateUserGroupResources();
  return group;
}

export async function deleteUserGroup(code: string) {
  await requestData<void>(axios.delete(endpoints.adminUserGroups.byCode(code)));
  await mutateUserGroupResources();
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

export async function mutateUserGroupResources() {
  await mutate((key) => isUserGroupKey(key) || isUsersKey(key) || isAdminGroupsKey(key));
  await mutate(endpoints.adminSettings.system);
}

function userGroupsKey(page: number, pageSize: number, filters: UserGroupFilters) {
  return [endpoints.adminUserGroups.list, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
}

function userGroupMembersKey(
  code: string,
  page: number,
  pageSize: number,
  filters: UserGroupFilters
) {
  return [
    endpoints.adminUserGroups.users(code),
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function isUserGroupKey(key: unknown) {
  return Array.isArray(key) && String(key[0]).startsWith(endpoints.adminUserGroups.list);
}

function isUsersKey(key: unknown) {
  return isEndpointKey(key, endpoints.users);
}

function isAdminGroupsKey(key: unknown) {
  return isEndpointKey(key, endpoints.adminGroups.list);
}
