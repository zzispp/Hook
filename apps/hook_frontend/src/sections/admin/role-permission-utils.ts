import type { ApiPermission } from 'src/types/rbac';

export function filterRoleAssignableApis(apis: ApiPermission[], readOnlyApis: ApiPermission[]) {
  const readOnlyApiIds = new Set(readOnlyApis.map((api) => api.id));
  return apis.filter((api) => !readOnlyApiIds.has(api.id));
}

export function filterRoleAssignableApiIds(ids: string[], readOnlyApis: ApiPermission[]) {
  const readOnlyApiIds = new Set(readOnlyApis.map((api) => api.id));
  return ids.filter((id) => !readOnlyApiIds.has(id));
}
