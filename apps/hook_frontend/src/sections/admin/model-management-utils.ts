import type { RoutingProfileId } from 'src/types/routing';
import type {
  PricingTier,
  GlobalModelCreate,
  ModelsDevModelItem,
  GlobalModelResponse,
  TieredPricingConfig,
} from 'src/types/model';

import {
  DEFAULT_PRICING,
  finalPricingConfig,
  pricingFromModelsDev,
  hasOneHourCachePricing,
  normalizePricingConfig,
} from './tiered-pricing-utils';

// ----------------------------------------------------------------------

export type GlobalModelForm = {
  name: string;
  display_name: string;
  default_tiered_pricing: TieredPricingConfig;
  default_price_per_request: string;
  routing_profile_id: RoutingProfileId | '';
  context_limit: string;
  output_limit: string;
  description: string;
  family: string;
  knowledge_cutoff: string;
  release_date: string;
  is_active: boolean;
  supports_vision: boolean;
  supports_function_calling: boolean;
  supports_streaming: boolean;
  supports_extended_thinking: boolean;
  supports_structured_output: boolean;
  supports_temperature: boolean;
  supports_attachment: boolean;
  open_weights: boolean;
  input_modalities: string[];
  output_modalities: string[];
};

export const DEFAULT_FORM: GlobalModelForm = {
  name: '',
  display_name: '',
  default_tiered_pricing: DEFAULT_PRICING,
  default_price_per_request: '',
  routing_profile_id: '',
  context_limit: '',
  output_limit: '',
  description: '',
  family: '',
  knowledge_cutoff: '',
  release_date: '',
  is_active: true,
  supports_vision: false,
  supports_function_calling: false,
  supports_streaming: true,
  supports_extended_thinking: false,
  supports_structured_output: false,
  supports_temperature: false,
  supports_attachment: false,
  open_weights: false,
  input_modalities: [],
  output_modalities: [],
};

export const CAPABILITY_KEYS = [
  'vision',
  'function_calling',
  'streaming',
  'extended_thinking',
  'structured_output',
  'temperature',
  'attachment',
  'open_weights',
] as const;

export function formFromModel(model: GlobalModelResponse): GlobalModelForm {
  const config = model.config ?? {};

  return {
    ...DEFAULT_FORM,
    name: model.name,
    display_name: model.display_name,
    default_tiered_pricing: normalizePricingConfig(model.default_tiered_pricing),
    default_price_per_request: optionalNumberText(model.default_price_per_request),
    routing_profile_id: model.routing_profile_id ?? '',
    context_limit: optionalNumberText(configNumber(config, 'context_limit')),
    output_limit: optionalNumberText(configNumber(config, 'output_limit')),
    description: configString(config, 'description'),
    family: configString(config, 'family'),
    knowledge_cutoff: configString(config, 'knowledge_cutoff'),
    release_date: configString(config, 'release_date'),
    is_active: model.is_active,
    supports_vision: configBool(config, 'vision'),
    supports_function_calling: configBool(config, 'function_calling'),
    supports_streaming: configBool(config, 'streaming', true),
    supports_extended_thinking: configBool(config, 'extended_thinking'),
    supports_structured_output: configBool(config, 'structured_output'),
    supports_temperature: configBool(config, 'temperature'),
    supports_attachment: configBool(config, 'attachment'),
    open_weights: configBool(config, 'open_weights'),
    input_modalities: configStringList(config, 'input_modalities'),
    output_modalities: configStringList(config, 'output_modalities'),
  };
}

export function formFromModelsDev(item: ModelsDevModelItem): GlobalModelForm {
  return {
    ...DEFAULT_FORM,
    name: item.modelId,
    display_name: item.modelName,
    default_tiered_pricing: pricingFromModelsDev(item),
    routing_profile_id: '',
    context_limit: optionalNumberText(item.contextLimit),
    output_limit: optionalNumberText(item.outputLimit),
    family: item.family ?? '',
    knowledge_cutoff: item.knowledgeCutoff ?? '',
    release_date: item.releaseDate ?? '',
    supports_vision: item.supportsVision,
    supports_function_calling: item.supportsToolCall,
    supports_extended_thinking: item.supportsReasoning,
    supports_structured_output: item.supportsStructuredOutput,
    supports_temperature: item.supportsTemperature,
    supports_attachment: item.supportsAttachment,
    open_weights: item.openWeights,
    input_modalities: item.inputModalities,
    output_modalities: item.outputModalities,
  };
}

export function toGlobalModelPayload(form: GlobalModelForm): GlobalModelCreate {
  const pricing = finalPricingConfig(form.default_tiered_pricing);

  return {
    name: form.name.trim(),
    display_name: form.display_name.trim(),
    default_price_per_request: optionalNumber(form.default_price_per_request),
    default_tiered_pricing: pricing,
    supported_capabilities: capabilitiesForPayload(form, pricing),
    config: configFromForm(form),
    routing_profile_id: form.routing_profile_id || null,
    is_active: form.is_active,
  };
}

export function capabilitiesFromForm(form: GlobalModelForm) {
  return CAPABILITY_KEYS.filter((capability) => configValue(form, capability));
}

function capabilitiesForPayload(form: GlobalModelForm, pricing: TieredPricingConfig) {
  const capabilities = new Set<string>(capabilitiesFromForm(form));
  if (hasOneHourCachePricing(pricing)) capabilities.add('cache_1h');
  return Array.from(capabilities);
}

export function firstTier(model: GlobalModelResponse): PricingTier | undefined {
  return model.default_tiered_pricing.tiers[0];
}

function configFromForm(form: GlobalModelForm): Record<string, unknown> {
  return cleanConfig({
    description: optionalString(form.description),
    family: optionalString(form.family),
    knowledge_cutoff: optionalString(form.knowledge_cutoff),
    release_date: optionalString(form.release_date),
    context_limit: optionalNumber(form.context_limit),
    output_limit: optionalNumber(form.output_limit),
    vision: form.supports_vision,
    function_calling: form.supports_function_calling,
    streaming: form.supports_streaming,
    extended_thinking: form.supports_extended_thinking,
    structured_output: form.supports_structured_output,
    temperature: form.supports_temperature,
    attachment: form.supports_attachment,
    open_weights: form.open_weights,
    input_modalities: form.input_modalities,
    output_modalities: form.output_modalities,
  });
}

function cleanConfig(config: Record<string, unknown>) {
  return Object.fromEntries(
    Object.entries(config).filter(([, value]) => {
      if (value === undefined || value === '') return false;
      if (Array.isArray(value)) return value.length > 0;
      return true;
    })
  );
}

function configValue(form: GlobalModelForm, capability: (typeof CAPABILITY_KEYS)[number]) {
  if (capability === 'vision') return form.supports_vision;
  if (capability === 'function_calling') return form.supports_function_calling;
  if (capability === 'streaming') return form.supports_streaming;
  if (capability === 'extended_thinking') return form.supports_extended_thinking;
  if (capability === 'structured_output') return form.supports_structured_output;
  if (capability === 'temperature') return form.supports_temperature;
  if (capability === 'attachment') return form.supports_attachment;
  return form.open_weights;
}

function optionalNumber(value: string | number | null | undefined) {
  if (value === null || value === undefined || value === '') return undefined;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) throw new Error('Invalid numeric field');
  return parsed;
}

function optionalString(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : undefined;
}

function optionalNumberText(value: number | null | undefined) {
  return value === null || value === undefined ? '' : numberToText(value);
}

function numberToText(value: number) {
  return Number.isFinite(value) ? String(value) : '';
}

function configBool(config: Record<string, unknown>, key: string, defaultValue = false) {
  return typeof config[key] === 'boolean' ? Boolean(config[key]) : defaultValue;
}

function configNumber(config: Record<string, unknown>, key: string) {
  return typeof config[key] === 'number' ? Number(config[key]) : undefined;
}

function configString(config: Record<string, unknown>, key: string) {
  return typeof config[key] === 'string' ? String(config[key]) : '';
}

function configStringList(config: Record<string, unknown>, key: string) {
  const value = config[key];
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
}
