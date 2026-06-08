import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';

export type ProviderGroupKind = 'provider' | 'key';

export type ProviderGroupForm = {
  name: string;
  description: string;
  sort_order: string;
  member_ids: string[];
};

export type MemberOption = {
  id: string;
  label: string;
  secondary?: string;
};

export const DEFAULT_PROVIDER_GROUP_FORM: ProviderGroupForm = {
  name: '',
  description: '',
  sort_order: '0',
  member_ids: [],
};

export function formFromProviderGroup(group: ProviderGroup): ProviderGroupForm {
  return {
    name: group.name,
    description: group.description ?? '',
    sort_order: String(group.sort_order),
    member_ids: group.provider_ids,
  };
}

export function formFromProviderKeyGroup(group: ProviderKeyGroup): ProviderGroupForm {
  return {
    name: group.name,
    description: group.description ?? '',
    sort_order: String(group.sort_order),
    member_ids: group.provider_key_ids,
  };
}

export function providerGroupPayload(form: ProviderGroupForm) {
  return {
    name: form.name.trim(),
    description: form.description.trim() || null,
    sort_order: Number(form.sort_order || 0),
    provider_ids: form.member_ids,
  };
}

export function providerKeyGroupPayload(form: ProviderGroupForm) {
  return {
    name: form.name.trim(),
    description: form.description.trim() || null,
    sort_order: Number(form.sort_order || 0),
    provider_key_ids: form.member_ids,
  };
}

export function providerMemberOptions(providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[]) {
  return providers.map((provider) => ({
    id: provider.id,
    label: provider.name,
    secondary: provider.provider_type,
  }));
}

export function providerKeyMemberOptions(
  providers: Pick<Provider, 'id' | 'name'>[],
  keysByProvider: Record<string, ProviderApiKey[]>
) {
  return providers.flatMap((provider) =>
    (keysByProvider[provider.id] ?? []).map((key) => ({
      id: key.id,
      label: key.name,
      secondary: provider.name,
    }))
  );
}

export function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

export function selectedMemberLabel(
  ids: string[],
  options: MemberOption[],
  emptyText: string,
  countText: (count: number) => string
) {
  if (ids.length === 0) return emptyText;
  if (ids.length > 2) return countText(ids.length);
  const labels = new Map(options.map((option) => [option.id, option.label]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

export function groupMemberIds(group: ProviderGroup | ProviderKeyGroup, kind: ProviderGroupKind) {
  return kind === 'provider'
    ? (group as ProviderGroup).provider_ids
    : (group as ProviderKeyGroup).provider_key_ids;
}

export function memberLabels(ids: string[], options: MemberOption[]) {
  const labels = new Map(options.map((option) => [option.id, option.label]));
  return ids.map((id) => labels.get(id) ?? id);
}

export function selectedGroupLabel(
  ids: string[],
  groups: Pick<ProviderGroup | ProviderKeyGroup, 'id' | 'name'>[],
  emptyText: string,
  countText: (count: number) => string
) {
  if (ids.length === 0) return emptyText;
  if (ids.length > 2) return countText(ids.length);
  const labels = new Map(groups.map((group) => [group.id, group.name]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

export function providerGroupIdsForProvider(groups: ProviderGroup[], providerId: string) {
  return groups.filter((group) => group.provider_ids.includes(providerId)).map((group) => group.id);
}

export function providerKeyGroupIdsForKey(groups: ProviderKeyGroup[], keyId: string) {
  return groups.filter((group) => group.provider_key_ids.includes(keyId)).map((group) => group.id);
}

export function providerKeyGroupNamesByKey(groups: ProviderKeyGroup[]) {
  const mapping = new Map<string, string[]>();
  for (const group of groups) {
    for (const keyId of group.provider_key_ids) {
      mapping.set(keyId, [...(mapping.get(keyId) ?? []), group.name]);
    }
  }
  return mapping;
}
