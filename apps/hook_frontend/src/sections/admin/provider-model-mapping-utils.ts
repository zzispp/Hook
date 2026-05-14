import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderModelBinding } from 'src/types/provider';

export function sortedBindingOptions(items: ProviderModelBinding[], models: GlobalModelResponse[]) {
  return items
    .map((item) => ({ id: item.id, label: bindingOptionLabel(item, models) }))
    .sort((left, right) => left.label.localeCompare(right.label));
}

export function bindingOptionLabel(item: ProviderModelBinding, models: GlobalModelResponse[]) {
  const model = models.find((entry) => entry.id === item.global_model_id);
  return `${model?.display_name ?? item.provider_model_name} · ${item.provider_model_name}`;
}

export function bindingDisplayLabel(item: ProviderModelBinding, models: GlobalModelResponse[]) {
  const model = models.find((entry) => entry.id === item.global_model_id);
  return model?.display_name ?? item.provider_model_name;
}

export function mappingName(binding: ProviderModelBinding | null) {
  return binding?.provider_model_mapping?.name ?? '';
}

export function mappingReasoningEffort(binding: ProviderModelBinding | null) {
  return binding?.provider_model_mapping?.reasoning_effort ?? '';
}

export function matchesQuery(query: string) {
  const normalized = query.trim().toLowerCase();
  return (value: string) => !normalized || value.toLowerCase().includes(normalized);
}

export function allowCustomName(query: string, selectedName: string, upstreamSet: Set<string>) {
  const normalized = query.trim();
  return !!normalized && normalized !== selectedName && !upstreamSet.has(normalized);
}

export function toggleName(current: string, name: string) {
  return current === name ? '' : name;
}
