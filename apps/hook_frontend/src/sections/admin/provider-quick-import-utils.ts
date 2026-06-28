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
import {
  quickImportSourceReady,
  quickImportSourceConfig,
  type QuickImportSourceFields,
  DEFAULT_QUICK_IMPORT_AUTH_TAB,
} from './provider-quick-import-source';

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
  rechargeMultiplier: string;
  max_retries: string;
  request_timeout_seconds: string;
  stream_first_byte_timeout_seconds: string;
  stream_first_output_timeout_seconds: string;
  stream_idle_timeout_seconds: string;
  priority: string;
  keep_priority_on_conversion: boolean;
  enable_format_conversion: boolean;
  upstream_image_native_stream: boolean;
  is_active: boolean;
  sync: QuickImportSyncConfigForm;
} & QuickImportSourceFields;

export type QuickImportTokenDraft = {
  selected: boolean;
  name: string;
  endpointFormats: string[];
  costMultiplier: string;
  localKeyId?: string;
};

export type QuickImportTokenMappings = Record<string, string>;
export type QuickImportMappingsByToken = Record<string, QuickImportTokenMappings>;

export const DEFAULT_QUICK_IMPORT_FORM: QuickImportFormState = {
  providerName: '',
  sourceKind: 'newapi',
  sub2apiAuthTab: DEFAULT_QUICK_IMPORT_AUTH_TAB,
  baseUrl: '',
  systemAccessToken: '',
  userId: '',
  email: '',
  password: '',
  authToken: '',
  refreshToken: '',
  tokenExpiresAt: '',
  rechargeMultiplier: '1',
  max_retries: DEFAULT_PROVIDER_FORM.max_retries,
  request_timeout_seconds: DEFAULT_PROVIDER_FORM.request_timeout_seconds,
  stream_first_byte_timeout_seconds: DEFAULT_PROVIDER_FORM.stream_first_byte_timeout_seconds,
  stream_first_output_timeout_seconds: DEFAULT_PROVIDER_FORM.stream_first_output_timeout_seconds,
  stream_idle_timeout_seconds: DEFAULT_PROVIDER_FORM.stream_idle_timeout_seconds,
  priority: DEFAULT_PROVIDER_FORM.priority,
  keep_priority_on_conversion: DEFAULT_PROVIDER_FORM.keep_priority_on_conversion,
  enable_format_conversion: DEFAULT_PROVIDER_FORM.enable_format_conversion,
  upstream_image_native_stream: false,
  is_active: DEFAULT_PROVIDER_FORM.is_active,
  sync: defaultQuickImportSyncConfigForm(),
};

export function previewPayload(form: QuickImportFormState) {
  return {
    source_kind: requireSourceKind(form.sourceKind),
    source: quickImportSourceConfig(form),
    provider_name: form.providerName.trim(),
    provider_config: providerConfigPayload(form),
    recharge_multiplier: Number(form.rechargeMultiplier),
  };
}

export function commitPayload(
  form: QuickImportFormState,
  selected: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappingsByToken: QuickImportMappingsByToken,
  models: GlobalModelResponse[]
) {
  return {
    ...previewPayload(form),
    selected_tokens: selected.map((token) => ({
      upstream_token_id: token.upstream_token_id,
      name: (tokens[token.upstream_token_id]?.name ?? token.name).trim(),
      endpoint_formats: tokens[token.upstream_token_id]?.endpointFormats ?? [],
      effective_cost_multiplier: Number(tokens[token.upstream_token_id]?.costMultiplier),
      model_mappings: tokenMappingInputs(token, mappingsByToken[token.upstream_token_id] ?? {}, models),
    })),
    sync_config: syncConfigPayload(form.sync),
  };
}

export function appendCommitPayload(
  selected: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappingsByToken: QuickImportMappingsByToken,
  models: GlobalModelResponse[]
) {
  return {
    selected_tokens: selected.map((token) => ({
      upstream_token_id: token.upstream_token_id,
      name: (tokens[token.upstream_token_id]?.name ?? token.name).trim(),
      endpoint_formats: tokens[token.upstream_token_id]?.endpointFormats ?? [],
      effective_cost_multiplier: Number(tokens[token.upstream_token_id]?.costMultiplier),
      model_mappings: tokenMappingInputs(token, mappingsByToken[token.upstream_token_id] ?? {}, models),
    })),
  };
}

export function bindSourcePayload(form: QuickImportFormState) {
  return {
    source_kind: requireSourceKind(form.sourceKind),
    source: quickImportSourceConfig(form),
    recharge_multiplier: Number(form.rechargeMultiplier),
  };
}

export function bindCommitPayload(
  form: QuickImportFormState,
  selected: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappingsByToken: QuickImportMappingsByToken,
  models: GlobalModelResponse[]
) {
  return {
    ...bindSourcePayload(form),
    selected_tokens: selected.map((token) => ({
      upstream_token_id: token.upstream_token_id,
      local_key_id: tokens[token.upstream_token_id]?.localKeyId || null,
      name: (tokens[token.upstream_token_id]?.name ?? token.name).trim(),
      endpoint_formats: tokens[token.upstream_token_id]?.endpointFormats ?? [],
      effective_cost_multiplier: Number(tokens[token.upstream_token_id]?.costMultiplier),
      model_mappings: tokenMappingInputs(token, mappingsByToken[token.upstream_token_id] ?? {}, models),
    })),
    sync_config: syncConfigPayload(form.sync),
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

export function tokenMappingInputs(
  token: ProviderQuickImportTokenPreview,
  mappings: QuickImportTokenMappings,
  _models: GlobalModelResponse[]
) {
  return token.models
    .map((model) => [model.upstream_model_id, mappings[model.upstream_model_id]] as const)
    .filter(([, global_model_id]) => !!global_model_id)
    .map(([upstream_model_id, global_model_id]) => ({ upstream_model_id, global_model_id }));
}

export function sourceReady(form: QuickImportFormState) {
  return Boolean(
    form.providerName.trim() &&
      quickImportSourceReady(form) &&
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
  return Object.fromEntries(preview.tokens.map((token) => [token.upstream_token_id, defaultTokenMappings(preview, token)]));
}

export function defaultTokenMappings(preview: ProviderQuickImportPreviewResponse, token: ProviderQuickImportTokenPreview) {
  const suggested = Object.fromEntries(
    token.models.map((model) => {
      const previewMapping = preview.model_mappings.find((mapping) => mapping.upstream_model_id === model.upstream_model_id);
      return [model.upstream_model_id, previewMapping?.suggested_global_model_id ?? ''];
    })
  );
  const linked = Object.fromEntries(
    token.linked_key?.model_mappings.map((mapping) => [mapping.upstream_model_id, mapping.global_model_id] as const) ?? []
  );
  return { ...suggested, ...linked };
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

export function flattenSelectedMappings(
  tokens: ProviderQuickImportTokenPreview[],
  mappingsByToken: QuickImportMappingsByToken
) {
  return tokens.reduce<Record<string, string>>((output, token) => {
    const tokenMappings = mappingsByToken[token.upstream_token_id] ?? {};
    for (const model of token.models) {
      const globalModelId = tokenMappings[model.upstream_model_id];
      if (globalModelId) {
        output[`${token.upstream_token_id}:${model.upstream_model_id}`] = globalModelId;
      }
    }
    return output;
  }, {});
}

export function globalModelHasCost(models: GlobalModelResponse[], id?: string) {
  const model = models.find((item) => item.id === id);
  return !!model && (model.default_tiered_pricing.tiers.length > 0 || model.default_price_per_request !== null);
}

export function validCostMultiplier(value?: string) {
  return Number(value) > 0;
}

function providerConfigPayload(form: QuickImportFormState) {
  const payload = providerPayload({
    name: form.providerName,
    provider_type: DEFAULT_PROVIDER_FORM.provider_type,
    max_retries: form.max_retries,
    request_timeout_seconds: form.request_timeout_seconds,
    stream_first_byte_timeout_seconds: form.stream_first_byte_timeout_seconds,
    stream_first_output_timeout_seconds: form.stream_first_output_timeout_seconds,
    stream_idle_timeout_seconds: form.stream_idle_timeout_seconds,
    priority: form.priority,
    keep_priority_on_conversion: form.keep_priority_on_conversion,
    enable_format_conversion: form.enable_format_conversion,
    is_active: form.is_active,
  });
  return {
    max_retries: payload.max_retries,
    request_timeout_seconds: payload.request_timeout_seconds,
    stream_first_byte_timeout_seconds: payload.stream_first_byte_timeout_seconds,
    stream_first_output_timeout_seconds: payload.stream_first_output_timeout_seconds,
    stream_idle_timeout_seconds: payload.stream_idle_timeout_seconds,
    priority: payload.priority,
    keep_priority_on_conversion: payload.keep_priority_on_conversion,
    enable_format_conversion: payload.enable_format_conversion,
    upstream_image_native_stream: form.upstream_image_native_stream,
    is_active: payload.is_active,
  };
}

function mappingNeedsOverride(models: GlobalModelResponse[], upstreamModelId: string, globalModelId: string) {
  const model = models.find((item) => item.id === globalModelId);
  return !model || model.name !== upstreamModelId;
}

function requireSourceKind(sourceKind: QuickImportFormState['sourceKind']) {
  if (!sourceKind) {
    throw new Error('source kind is not selected');
  }
  return sourceKind;
}
