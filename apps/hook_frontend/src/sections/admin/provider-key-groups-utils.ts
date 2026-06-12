import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderKeyGroup } from 'src/types/provider-key-group';

export type ProviderKeyGroupFormMember = {
  id: string;
  priority: string;
};

export type ProviderKeyGroupForm = {
  name: string;
  description: string;
  sort_order: string;
  members: ProviderKeyGroupFormMember[];
};

export type MemberOption = {
  id: string;
  label: string;
  secondary?: string;
};

export const DEFAULT_PROVIDER_KEY_GROUP_FORM: ProviderKeyGroupForm = {
  name: '',
  description: '',
  sort_order: '0',
  members: [],
};

export function formFromProviderKeyGroup(group: ProviderKeyGroup): ProviderKeyGroupForm {
  return {
    name: group.name,
    description: group.description ?? '',
    sort_order: String(group.sort_order),
    members: group.provider_key_members.map((member) => ({
      id: member.provider_key_id,
      priority: String(member.priority),
    })),
  };
}

export function providerKeyGroupPayload(form: ProviderKeyGroupForm) {
  return {
    name: form.name.trim(),
    description: form.description.trim() || null,
    sort_order: Number(form.sort_order || 0),
    provider_key_members: form.members.map((member) => ({
      provider_key_id: member.id,
      priority: Number(member.priority || 0),
    })),
  };
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

export function formMemberIds(form: ProviderKeyGroupForm) {
  return form.members.map((member) => member.id);
}

export function updateSelectedMembers(
  members: ProviderKeyGroupFormMember[],
  selectedIds: string[],
  defaultPriorityForId: (id: string) => number
) {
  const current = new Map(members.map((member) => [member.id, member]));
  return selectedIds.map(
    (id) => current.get(id) ?? { id, priority: String(defaultPriorityForId(id)) }
  );
}

export function updateMemberPriority(
  members: ProviderKeyGroupFormMember[],
  memberId: string,
  priority: string
) {
  return members.map((member) => (member.id === memberId ? { ...member, priority } : member));
}

export function defaultProviderKeyMemberPriority(
  keysByProvider: Record<string, ProviderApiKey[]>,
  keyId: string
) {
  for (const keys of Object.values(keysByProvider)) {
    const key = keys.find((item) => item.id === keyId);
    if (key) return key.internal_priority;
  }
  return 0;
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

export function providerKeyGroupMemberIds(group: ProviderKeyGroup) {
  return group.provider_key_members.map((member) => member.provider_key_id);
}

export function memberLabels(ids: string[], options: MemberOption[]) {
  const labels = new Map(options.map((option) => [option.id, option.label]));
  return ids.map((id) => labels.get(id) ?? id);
}

export function selectedGroupLabel(
  ids: string[],
  groups: Pick<ProviderKeyGroup, 'id' | 'name'>[],
  emptyText: string,
  countText: (count: number) => string
) {
  if (ids.length === 0) return emptyText;
  if (ids.length > 2) return countText(ids.length);
  const labels = new Map(groups.map((group) => [group.id, group.name]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

export function providerKeyGroupIdsForKey(groups: ProviderKeyGroup[], keyId: string) {
  return groups
    .filter((group) =>
      group.provider_key_members.some((member) => member.provider_key_id === keyId)
    )
    .map((group) => group.id);
}

export function providerKeyGroupNamesByKey(groups: ProviderKeyGroup[]) {
  const mapping = new Map<string, string[]>();
  for (const group of groups) {
    for (const member of group.provider_key_members) {
      mapping.set(member.provider_key_id, [
        ...(mapping.get(member.provider_key_id) ?? []),
        group.name,
      ]);
    }
  }
  return mapping;
}
