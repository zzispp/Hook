'use client';

import type { NavSectionProps } from 'src/components/nav-section';
import type {
  Role,
  MenuItem,
  UserInput,
  RoleInput,
  SystemUser,
  NavResponse,
  ApiEnvelope,
  MenuSection,
  PageResponse,
  MenuItemInput,
  ApiPermission,
  MenuApiBinding,
  BackendNavItem,
  ApiMenuBinding,
  MenuSectionInput,
  BackendNavSection,
  ApiPermissionInput,
  RolePermissionBinding,
} from 'src/types/rbac';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import { useTranslate } from 'src/locales/use-locales';
import axios, { fetcher, endpoints } from 'src/lib/axios';

// ----------------------------------------------------------------------

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export const pageQuery = (page: number, pageSize: number) => ({
  page: page + 1,
  page_size: pageSize,
});

export type RbacListFilters = {
  enabled?: boolean;
  is_active?: boolean;
  group_code?: string;
  role?: string;
  search?: string;
};

export function requireApiData<T>(payload: ApiEnvelope<T>): T {
  if (!payload.success) {
    throw new Error(payload.message || 'Request failed');
  }

  if (payload.data === undefined || payload.data === null) {
    throw new Error('Response data not found');
  }

  return payload.data;
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

function pageKey(endpoint: string, page: number, pageSize: number, filters: RbacListFilters = {}) {
  return [endpoint, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
}

function usePagedResource<T>(
  endpoint: string,
  page: number,
  pageSize: number,
  filters: RbacListFilters = {}
) {
  const disabled = page < 0 || pageSize <= 0;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<ApiEnvelope<PageResponse<T>>>(
    disabled ? null : pageKey(endpoint, page, pageSize, filters),
    fetcher,
    swrOptions
  );

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

export function useRoles(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<Role>(endpoints.rbac.roles, page, pageSize, filters);
}

export function useApis(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<ApiPermission>(endpoints.rbac.apis, page, pageSize, filters);
}

export function useUnboundApis(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<ApiPermission>(endpoints.rbac.unboundApis, page, pageSize, filters);
}

export function useMenuSections(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<MenuSection>(endpoints.rbac.menuSections, page, pageSize, filters);
}

export function useMenuItems(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<MenuItem>(endpoints.rbac.menuItems, page, pageSize, filters);
}

export function useUsers(page: number, pageSize: number, filters?: RbacListFilters) {
  return usePagedResource<SystemUser>(endpoints.users, page, pageSize, filters);
}

export function useNavbar() {
  const { t } = useTranslate('admin');
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<ApiEnvelope<NavResponse>>(
    endpoints.navbar,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const nav = data ? requireApiData(data) : undefined;

    return {
      data: toNavSections(nav?.nav_items ?? [], (code) => String(t(`nav.${code}`))),
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate, t]);
}

export async function createRole(payload: RoleInput) {
  const role = await requestData<Role>(axios.post(endpoints.rbac.roles, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
  return role;
}

export async function updateRole(code: string, payload: RoleInput) {
  const role = await requestData<Role>(axios.put(endpoints.rbac.role(code), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
  return role;
}

export async function deleteRole(code: string) {
  await requestData<void>(axios.delete(endpoints.rbac.role(code)));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
}

export async function getRolePermissions(code: string) {
  return requestData<RolePermissionBinding>(axios.get(endpoints.rbac.rolePermissions(code)));
}

export async function updateRolePermissions(
  code: string,
  menuItemIds: string[],
  apiPermissionIds: string[]
) {
  await requestData<void>(
    axios.put(endpoints.rbac.rolePermissions(code), {
      menu_item_ids: menuItemIds,
      api_permission_ids: apiPermissionIds,
    })
  );
  await mutate(endpoints.navbar);
}

export async function getMenuApis(id: string) {
  return requestData<MenuApiBinding>(axios.get(endpoints.rbac.menuItemApis(id)));
}

export async function updateMenuApis(id: string, apiPermissionIds: string[]) {
  await requestData<void>(
    axios.put(endpoints.rbac.menuItemApis(id), { api_permission_ids: apiPermissionIds })
  );
  await mutate(endpoints.navbar);
}

export async function getApiMenus(id: string) {
  return requestData<ApiMenuBinding>(axios.get(endpoints.rbac.apiMenus(id)));
}

export async function updateApiMenus(id: string, menuItemIds: string[]) {
  await requestData<void>(axios.put(endpoints.rbac.apiMenus(id), { menu_item_ids: menuItemIds }));
  await mutate(endpoints.navbar);
}

export async function createApi(payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.post(endpoints.rbac.apis, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
  await mutate(endpoints.navbar);
  return api;
}

export async function updateApi(id: string, payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.put(endpoints.rbac.api(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
  await mutate(endpoints.navbar);
  return api;
}

export async function deleteApi(id: string) {
  await requestData<void>(axios.delete(endpoints.rbac.api(id)));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
}

export async function createMenuSection(payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.post(endpoints.rbac.menuSections, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
  return section;
}

export async function updateMenuSection(id: string, payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.put(endpoints.rbac.menuSection(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
  return section;
}

export async function deleteMenuSection(id: string) {
  await requestData<void>(axios.delete(endpoints.rbac.menuSection(id)));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
}

export async function createMenuItem(payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.post(endpoints.rbac.menuItems, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
  return item;
}

export async function updateMenuItem(id: string, payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.put(endpoints.rbac.menuItem(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
  return item;
}

export async function deleteMenuItem(id: string) {
  await requestData<void>(axios.delete(endpoints.rbac.menuItem(id)));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
}

export async function createUser(payload: UserInput) {
  const user = await requestData<SystemUser>(axios.post(endpoints.users, payload));
  await mutate((key) => isEndpointKey(key, endpoints.users));
  return user;
}

export async function updateUser(id: string, payload: UserInput) {
  const user = await requestData<SystemUser>(axios.put(endpoints.user(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.users));
  return user;
}

export async function deleteUser(id: string) {
  await requestData<void>(axios.delete(endpoints.user(id)));
  await mutate((key) => isEndpointKey(key, endpoints.users));
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function toNavSections(
  sections: BackendNavSection[],
  titleForCode: (code: string) => string
): NavSectionProps['data'] {
  return sections.map((section) => ({
    code: section.code,
    subheader: titleForCode(section.code),
    items: section.items.map((item) => toNavItem(item, titleForCode)),
  }));
}

function toNavItem(
  item: BackendNavItem,
  titleForCode: (code: string) => string
): NavSectionProps['data'][number]['items'][number] {
  return {
    code: item.code,
    title: titleForCode(item.code),
    path: item.path,
    icon: item.icon ?? undefined,
    caption: item.caption ?? undefined,
    deepMatch: item.deep_match,
    children: item.children.length ? item.children.map((child) => toNavItem(child, titleForCode)) : undefined,
  };
}
