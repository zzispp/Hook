import type { Provider, ProviderApiKey } from 'src/types/provider';

import { updateProvider, updateProviderApiKeyPriorities } from 'src/actions/providers';

export type PriorityKind = 'provider' | 'key';

export type PriorityItem = {
  id: string;
  providerId?: string;
  name: string;
  providerName?: string;
  is_active: boolean;
  apiFormats?: string[];
  priority: number;
  priorityText: string;
};

export type KeyPrioritySource = 'internal' | 'global';

export function orderProviders(providers: Provider[]): PriorityItem[] {
  return [...providers]
    .sort((left, right) => left.priority - right.priority || left.name.localeCompare(right.name))
    .map((provider) => ({
      id: provider.id,
      name: provider.name,
      is_active: provider.is_active,
      priority: provider.priority,
      priorityText: String(provider.priority),
    }));
}

export function orderKeys(
  providers: Provider[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  prioritySource: KeyPrioritySource
): PriorityItem[] {
  return [...providers]
    .sort((left, right) => left.priority - right.priority || left.name.localeCompare(right.name))
    .flatMap((provider) =>
      [...(keysByProvider[provider.id] ?? [])]
        .sort((left, right) => keyPriority(left, prioritySource) - keyPriority(right, prioritySource) || left.name.localeCompare(right.name))
        .map((key) => keyPriorityItem(provider, key, prioritySource))
    );
}

function keyPriorityItem(provider: Provider, key: ProviderApiKey, prioritySource: KeyPrioritySource): PriorityItem {
  const priority = keyPriority(key, prioritySource);
  return {
    id: key.id,
    providerId: provider.id,
    name: key.name,
    providerName: provider.name,
    is_active: key.is_active && provider.is_active,
    apiFormats: key.api_formats,
    priority,
    priorityText: String(priority),
  };
}

function keyPriority(key: ProviderApiKey, prioritySource: KeyPrioritySource) {
  return prioritySource === 'internal' ? key.internal_priority : key.global_priority;
}

export function changeItemPriority(items: PriorityItem[], id: string, value: string) {
  return items.map((item) => (item.id === id ? { ...item, priorityText: value } : item));
}

export function movePriorityItem(items: PriorityItem[], sourceId: string, targetId: string) {
  const sourceIndex = items.findIndex((item) => item.id === sourceId);
  const targetIndex = items.findIndex((item) => item.id === targetId);
  if (sourceIndex < 0 || targetIndex < 0) return items;

  const nextItems = [...items];
  const [source] = nextItems.splice(sourceIndex, 1);
  nextItems.splice(targetIndex, 0, source);
  return nextItems.map((item, index) => ({ ...item, priorityText: String(index + 1) }));
}

export function parsePriorities(items: PriorityItem[]) {
  const parsed = new Map<string, number>();
  for (const item of items) {
    const priorityText = item.priorityText.trim();
    if (!priorityText) return null;
    const priority = Number(priorityText);
    if (!Number.isInteger(priority)) return null;
    parsed.set(item.id, priority);
  }
  return parsed;
}

export async function savePriorityChanges(
  kind: PriorityKind,
  items: PriorityItem[],
  providers: Provider[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  priorities: Map<string, number>
) {
  if (kind === 'key') {
    await saveKeyPriorityChanges(items, keysByProvider, priorities);
    return;
  }
  const original = new Map(providers.map((provider) => [provider.id, provider.priority]));
  const changed = items.filter((item) => original.get(item.id) !== priorities.get(item.id));
  await Promise.all(changed.map((item) => updateProvider(item.id, { priority: priorities.get(item.id)! })));
}

async function saveKeyPriorityChanges(
  items: PriorityItem[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  priorities: Map<string, number>
) {
  const original = new Map(
    Object.values(keysByProvider)
      .flat()
      .map((key) => [key.id, key.global_priority])
  );
  const updates = items
    .filter((item) => item.providerId && original.get(item.id) !== priorities.get(item.id))
    .map((item) => ({
      provider_id: item.providerId!,
      key_id: item.id,
      global_priority: priorities.get(item.id)!,
    }));
  if (updates.length > 0) await updateProviderApiKeyPriorities({ updates });
}
