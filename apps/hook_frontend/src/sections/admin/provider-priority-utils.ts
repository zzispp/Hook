import type { Provider } from 'src/types/provider';

import { updateProvider } from 'src/actions/providers';

export type PriorityItem = Provider & {
  priorityText: string;
};

export function orderProviders(providers: Provider[]): PriorityItem[] {
  return [...providers]
    .sort((left, right) => left.priority - right.priority || left.name.localeCompare(right.name))
    .map((provider) => ({ ...provider, priorityText: String(provider.priority) }));
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
  items: PriorityItem[],
  providers: Provider[],
  priorities: Map<string, number>
) {
  const original = new Map(providers.map((provider) => [provider.id, provider.priority]));
  const changed = items.filter((item) => original.get(item.id) !== priorities.get(item.id));
  await Promise.all(changed.map((item) => updateProvider(item.id, { priority: priorities.get(item.id)! })));
}
