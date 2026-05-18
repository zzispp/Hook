import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderEndpoint, ProviderModelBinding } from 'src/types/provider';

import { formatApiFormat } from './provider-management-utils';

export type SelectOption = {
  value: string;
  label: string;
  description?: string;
};

export function providerModelOptions(
  models: GlobalModelResponse[],
  bindings: ProviderModelBinding[]
): SelectOption[] {
  const labels = modelLabels(models);
  const options = new Map<string, SelectOption>();

  for (const binding of bindings) {
    if (options.has(binding.global_model_id)) continue;
    const label = labels.get(binding.global_model_id) ?? binding.global_model_id;
    options.set(binding.global_model_id, {
      value: binding.global_model_id,
      label,
      description: providerModelDescription(binding, label),
    });
  }

  return Array.from(options.values());
}

export function providerEndpointFormatOptions(endpoints: ProviderEndpoint[]): SelectOption[] {
  const options = new Map<string, SelectOption>();

  for (const endpoint of endpoints) {
    if (options.has(endpoint.api_format)) continue;
    options.set(endpoint.api_format, {
      value: endpoint.api_format,
      label: formatApiFormat(endpoint.api_format),
      description: endpoint.base_url,
    });
  }

  return Array.from(options.values());
}

export function selectedOptions(value: string[], options: SelectOption[]) {
  return value.map(
    (id) => options.find((option) => option.value === id) ?? { value: id, label: id }
  );
}

function modelLabels(models: GlobalModelResponse[]) {
  return new Map(models.map((model) => [model.id, model.display_name || model.name]));
}

function providerModelDescription(binding: ProviderModelBinding, label: string) {
  const upstream = binding.provider_model_mapping?.name ?? binding.provider_model_name;
  return upstream === label ? binding.global_model_id : `${upstream} · ${binding.global_model_id}`;
}
