import type { GlobalModelResponse } from 'src/types/model';
import type { QuickImportSyncConfigForm } from './provider-quick-import-sync-utils';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
} from 'src/types/provider-quick-import';

import { providerPayload, DEFAULT_PROVIDER_FORM } from './provider-management-utils';
import {
  validSyncConfig,
  syncConfigPayload,
  defaultQuickImportSyncConfigForm,
} from './provider-quick-import-sync-utils';

export {
  validSyncConfig,
  validSyncSettings,
  syncSettingsPayload,
  syncSettingsFormFromResponse,
  DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM,
} from './provider-quick-import-sync-utils';
export type {
  QuickImportSyncConfigForm,
  QuickImportSyncSettingsForm,
} from './provider-quick-import-sync-utils';

export type QuickImportFormState = {
  providerName: string;
  baseUrl: string;
  systemAccessToken: string;
  userId: string;
  rechargeMultiplier: string;
  provider_group_id: string;
  max_retries: string;
  request_timeout_seconds: string;
  stream_first_byte_timeout_seconds: string;
  stream_idle_timeout_seconds: string;
  priority: string;
  keep_priority_on_conversion: boolean;
  enable_format_conversion: boolean;
  is_active: boolean;
  sync: QuickImportSyncConfigForm;
};

export type QuickImportTokenDraft = {
  selected: boolean;
  name: string;
  endpointFormats: string[];
  costMultiplier: string;
};

export const DEFAULT_QUICK_IMPORT_FORM: QuickImportFormState = {
  providerName: '',
  baseUrl: '',
  systemAccessToken: '',
  userId: '',
  rechargeMultiplier: '1',
  provider_group_id: DEFAULT_PROVIDER_FORM.provider_group_id,
  max_retries: DEFAULT_PROVIDER_FORM.max_retries,
  request_timeout_seconds: DEFAULT_PROVIDER_FORM.request_timeout_seconds,
  stream_first_byte_timeout_seconds: DEFAULT_PROVIDER_FORM.stream_first_byte_timeout_seconds,
  stream_idle_timeout_seconds: DEFAULT_PROVIDER_FORM.stream_idle_timeout_seconds,
  priority: DEFAULT_PROVIDER_FORM.priority,
  keep_priority_on_conversion: DEFAULT_PROVIDER_FORM.keep_priority_on_conversion,
  enable_format_conversion: DEFAULT_PROVIDER_FORM.enable_format_conversion,
  is_active: DEFAULT_PROVIDER_FORM.is_active,
  sync: defaultQuickImportSyncConfigForm(),
};

export function previewPayload(form: QuickImportFormState) {
  return {
    source_kind: 'newapi' as const,
    source: {
      kind: 'newapi' as const,
      base_url: trimmedBaseUrl(form.baseUrl),
      system_access_token: form.systemAccessToken.trim(),
      user_id: form.userId.trim(),
    },
    provider_name: form.providerName.trim(),
    provider_config: providerConfigPayload(form),
    recharge_multiplier: Number(form.rechargeMultiplier),
  };
}

export function commitPayload(
  form: QuickImportFormState,
  selected: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappings: Record<string, string>,
  models: GlobalModelResponse[]
) {
  const selectedModelIds = selectedMappedUpstreamModels(selected, mappings);

  return {
    ...previewPayload(form),
    selected_tokens: selected.map((token) => ({
      upstream_token_id: token.upstream_token_id,
      name: (tokens[token.upstream_token_id]?.name ?? token.name).trim(),
      endpoint_formats: tokens[token.upstream_token_id]?.endpointFormats ?? [],
      effective_cost_multiplier: Number(tokens[token.upstream_token_id]?.costMultiplier),
    })),
    selected_model_ids: selectedModelIds,
    model_mappings: mappingInputs(selectedModelIds, mappings, models),
    sync_config: syncConfigPayload(form.sync),
  };
}

export function appendCommitPayload(
  selected: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappings: Record<string, string>,
  models: GlobalModelResponse[]
) {
  const selectedModelIds = selectedMappedUpstreamModels(selected, mappings);

  return {
    selected_tokens: selected.map((token) => ({
      upstream_token_id: token.upstream_token_id,
      name: (tokens[token.upstream_token_id]?.name ?? token.name).trim(),
      endpoint_formats: tokens[token.upstream_token_id]?.endpointFormats ?? [],
      effective_cost_multiplier: Number(tokens[token.upstream_token_id]?.costMultiplier),
    })),
    selected_model_ids: selectedModelIds,
    model_mappings: mappingInputs(selectedModelIds, mappings, models),
  };
}

export function mappingInputs(
  upstreamModelIds: string[],
  mappings: Record<string, string>,
  models: GlobalModelResponse[]
) {
  return upstreamModelIds
    .map((upstream_model_id) => [upstream_model_id, mappings[upstream_model_id]] as const)
    .filter(([upstream_model_id, global_model_id]) =>
      global_model_id && mappingNeedsOverride(models, upstream_model_id, global_model_id)
    )
    .map(([upstream_model_id, global_model_id]) => ({ upstream_model_id, global_model_id }));
}

export function sourceReady(form: QuickImportFormState) {
  return Boolean(
    form.providerName.trim() &&
      form.baseUrl.trim() &&
      form.systemAccessToken.trim() &&
      form.userId.trim() &&
      Number(form.rechargeMultiplier) > 0 &&
      validSyncConfig(form.sync)
  );
}

export function defaultTokenDrafts(preview: ProviderQuickImportPreviewResponse) {
  return Object.fromEntries(
    preview.tokens.map((token) => [
      token.upstream_token_id,
      {
        selected: token.importable,
        name: token.linked_key?.name ?? token.name,
        endpointFormats: token.linked_key?.endpoint_formats ?? [],
        costMultiplier: String(token.linked_key?.effective_cost_multiplier ?? token.effective_cost_multiplier),
      },
    ])
  );
}

export function defaultMappings(preview: ProviderQuickImportPreviewResponse) {
  const suggested = preview.model_mappings.map((mapping) => [
      mapping.upstream_model_id,
      mapping.suggested_global_model_id ?? '',
    ] as const);
  const linked = preview.tokens.flatMap((token) =>
    token.linked_key?.model_mappings.map((mapping) => [mapping.upstream_model_id, mapping.global_model_id] as const) ?? []
  );
  return Object.fromEntries([...suggested, ...linked]);
}

export function selectedTokenRows(
  preview: ProviderQuickImportPreviewResponse | null,
  tokens: Record<string, QuickImportTokenDraft>
) {
  return preview?.tokens.filter((token) => tokens[token.upstream_token_id]?.selected) ?? [];
}

export function selectedUpstreamModels(tokens: ProviderQuickImportTokenPreview[]) {
  return [...new Set(tokens.flatMap((token) => token.models.map((model) => model.upstream_model_id)))];
}

export function selectedMappedUpstreamModels(
  tokens: ProviderQuickImportTokenPreview[],
  mappings: Record<string, string>
) {
  const selected = new Set(selectedUpstreamModels(tokens));
  return Object.keys(mappings).filter((id) => selected.has(id));
}

export function globalModelHasCost(models: GlobalModelResponse[], id?: string) {
  const model = models.find((item) => item.id === id);
  return !!model && (model.default_tiered_pricing.tiers.length > 0 || model.default_price_per_request !== null);
}

export function validCostMultiplier(value?: string) {
  return Number(value) > 0;
}

function trimmedBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, '');
}

function providerConfigPayload(form: QuickImportFormState) {
  const payload = providerPayload({
    name: form.providerName,
    provider_type: DEFAULT_PROVIDER_FORM.provider_type,
    provider_group_id: form.provider_group_id,
    max_retries: form.max_retries,
    request_timeout_seconds: form.request_timeout_seconds,
    stream_first_byte_timeout_seconds: form.stream_first_byte_timeout_seconds,
    stream_idle_timeout_seconds: form.stream_idle_timeout_seconds,
    priority: form.priority,
    keep_priority_on_conversion: form.keep_priority_on_conversion,
    enable_format_conversion: form.enable_format_conversion,
    is_active: form.is_active,
  });
  return {
    provider_group_id: payload.provider_group_id,
    max_retries: payload.max_retries,
    request_timeout_seconds: payload.request_timeout_seconds,
    stream_first_byte_timeout_seconds: payload.stream_first_byte_timeout_seconds,
    stream_idle_timeout_seconds: payload.stream_idle_timeout_seconds,
    priority: payload.priority,
    keep_priority_on_conversion: payload.keep_priority_on_conversion,
    enable_format_conversion: payload.enable_format_conversion,
    is_active: payload.is_active,
  };
}

function mappingNeedsOverride(models: GlobalModelResponse[], upstreamModelId: string, globalModelId: string) {
  const model = models.find((item) => item.id === globalModelId);
  return !model || model.name !== upstreamModelId;
}
