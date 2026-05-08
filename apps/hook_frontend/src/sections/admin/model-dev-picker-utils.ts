import type { ModelsDevModelItem } from 'src/types/model';

import { useMemo } from 'react';

// ----------------------------------------------------------------------

export type ProviderGroup = {
  providerId: string;
  providerName: string;
  models: ModelsDevModelItem[];
};

export function useProviderGroups(items: ModelsDevModelItem[], query: string) {
  return useMemo(() => groupModelsByProvider(filterVisibleModels(items, query)), [items, query]);
}

export function countModels(groups: ProviderGroup[]) {
  return groups.reduce((total, group) => total + group.models.length, 0);
}

export function toggleProvider(current: string | null, providerId: string) {
  return current === providerId ? null : providerId;
}

export function providerLogoUrl(providerId: string) {
  return `https://models.dev/logos/${providerId}.svg`;
}

function filterVisibleModels(items: ModelsDevModelItem[], query: string) {
  const activeItems = items.filter((item) => !item.deprecated);
  const normalized = query.trim().toLowerCase();

  if (!normalized) {
    return activeItems.filter((item) => item.official);
  }

  return activeItems.filter((item) => matchesQuery(item, normalized));
}

function groupModelsByProvider(items: ModelsDevModelItem[]) {
  const groups = new Map<string, ProviderGroup>();

  for (const item of items) {
    const group = groups.get(item.providerId);
    if (group) {
      group.models.push(item);
      continue;
    }

    groups.set(item.providerId, {
      providerId: item.providerId,
      providerName: item.providerName,
      models: [item],
    });
  }

  return Array.from(groups.values()).sort(compareProviderGroups);
}

function matchesQuery(item: ModelsDevModelItem, query: string) {
  const keywords = query.split(/\s+/).filter(Boolean);
  const text = [item.providerId, item.providerName, item.modelId, item.modelName, item.family ?? '']
    .join(' ')
    .toLowerCase();

  return keywords.every((keyword) => text.includes(keyword));
}

function compareProviderGroups(left: ProviderGroup, right: ProviderGroup) {
  return left.providerName.localeCompare(right.providerName);
}
