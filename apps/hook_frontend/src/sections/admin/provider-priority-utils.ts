import type { Provider, ProviderApiKey } from 'src/types/provider';

import { updateProvider, updateProviderApiKeyPriorities } from 'src/actions/providers';

export type PriorityKind = 'provider' | 'key';

export type PriorityItem = {
  id: string;
  providerId?: string;
  apiFormat?: string;
  name: string;
  providerName?: string;
  is_active: boolean;
  apiFormats?: string[];
  priority: number;
  priorityText: string;
};

export type KeyPrioritySource = 'internal' | 'global';

export type PriorityItemsByFormat = Record<string, PriorityItem[]>;

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
  prioritySource: KeyPrioritySource,
  apiFormat: string
): PriorityItem[] {
  return [...providers]
    .sort((left, right) => left.priority - right.priority || left.name.localeCompare(right.name))
    .flatMap((provider) =>
      [...(keysByProvider[provider.id] ?? [])]
        .filter((key) => key.api_formats.includes(apiFormat))
        .sort(
          (left, right) =>
            keyPriority(left, prioritySource, apiFormat) -
              keyPriority(right, prioritySource, apiFormat) || left.name.localeCompare(right.name)
        )
        .map((key) => keyPriorityItem(provider, key, prioritySource, apiFormat))
    );
}

export function orderKeysByFormat(
  providers: Provider[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  prioritySource: KeyPrioritySource
): PriorityItemsByFormat {
  return Object.fromEntries(
    priorityFormats(keysByProvider).map((format) => [
      format,
      orderKeys(providers, keysByProvider, prioritySource, format),
    ])
  );
}

function keyPriorityItem(
  provider: Provider,
  key: ProviderApiKey,
  prioritySource: KeyPrioritySource,
  apiFormat: string
): PriorityItem {
  const priority = keyPriority(key, prioritySource, apiFormat);
  return {
    id: key.id,
    providerId: provider.id,
    apiFormat,
    name: key.name,
    providerName: provider.name,
    is_active: key.is_active && provider.is_active,
    apiFormats: key.api_formats,
    priority,
    priorityText: String(priority),
  };
}

function keyPriority(key: ProviderApiKey, prioritySource: KeyPrioritySource, apiFormat: string) {
  if (prioritySource === 'internal') return key.internal_priority;
  return key.global_priority_by_format[apiFormat] ?? key.internal_priority;
}

export function priorityFormats(keysByProvider: Record<string, ProviderApiKey[]>) {
  return [
    ...new Set(
      Object.values(keysByProvider)
        .flat()
        .flatMap((key) => key.api_formats)
    ),
  ].sort((left, right) => left.localeCompare(right));
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
    parsed.set(priorityItemKey(item), priority);
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
  const changed = items.filter((item) => original.get(item.id) !== priorities.get(priorityItemKey(item)));
  await Promise.all(
    changed.map((item) => updateProvider(item.id, { priority: priorities.get(priorityItemKey(item))! }))
  );
}

async function saveKeyPriorityChanges(
  items: PriorityItem[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  priorities: Map<string, number>
) {
  const original = new Map(
    Object.values(keysByProvider)
      .flat()
      .map((key) => [key.id, key.global_priority_by_format])
  );
  const updatesByKey = new Map<
    string,
    { provider_id: string; key_id: string; global_priority_by_format: Record<string, number> }
  >();
  for (const item of items) {
    if (!item.providerId || !item.apiFormat || !originalPriorityChanged(item, original, priorities)) {
      continue;
    }
    const update = updatesByKey.get(item.id) ?? {
      provider_id: item.providerId,
      key_id: item.id,
      global_priority_by_format: { ...(original.get(item.id) ?? {}) },
    };
    update.global_priority_by_format[item.apiFormat] = priorities.get(priorityItemKey(item))!;
    updatesByKey.set(item.id, update);
  }
  const updates = [...updatesByKey.values()];
  if (updates.length > 0) await updateProviderApiKeyPriorities({ updates });
}

function originalPriorityChanged(
  item: PriorityItem,
  original: Map<string, Record<string, number>>,
  priorities: Map<string, number>
) {
  const apiFormat = item.apiFormat;
  if (!apiFormat) return false;
  return original.get(item.id)?.[apiFormat] !== priorities.get(priorityItemKey(item));
}

function priorityItemKey(item: PriorityItem) {
  return item.apiFormat ? `${item.id}:${item.apiFormat}` : item.id;
}
