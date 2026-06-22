'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { TokenAccessibleModelListResponse } from 'src/types/api-token';

import { ACCOUNTING_CURRENCY } from 'src/utils/money-boundary';

import { endpointUrl } from './api-token-endpoints-utils';

export const CC_SWITCH_USAGE_AUTO_INTERVAL_MINUTES = 30;
export const CC_SWITCH_FORMATS = ['claude', 'openai', 'gemini'] as const;

export type CcSwitchFormat = (typeof CC_SWITCH_FORMATS)[number];

export type CcSwitchModelOption = {
  id: string;
  label: string;
  format: CcSwitchFormat;
};

type BuildCcSwitchDeepLinkArgs = {
  format: CcSwitchFormat;
  modelId: string;
  rawToken: string;
  tokenName: string;
  baseUrl: string;
  siteName?: string | null;
  homepage: string;
};

const CC_SWITCH_APP_BY_FORMAT: Record<CcSwitchFormat, string> = {
  claude: 'claude',
  // CC Switch currently uses `codex` as the OpenAI-compatible app key.
  openai: 'codex',
  gemini: 'gemini',
};

const CC_SWITCH_ENDPOINT_PATH_BY_FORMAT: Record<CcSwitchFormat, string> = {
  claude: '/v1',
  openai: '/v1',
  gemini: '/v1beta',
};

export async function fetchTokenModelIds(rawToken: string, baseUrl: string) {
  const response = await fetch(endpointUrl(baseUrl, '/v1/models'), {
    headers: {
      Authorization: `Bearer ${rawToken}`,
      Accept: 'application/json',
    },
  });
  if (!response.ok) {
    throw new Error(await responseErrorMessage(response, 'Failed to load token models'));
  }
  const payload = (await response.json()) as TokenAccessibleModelListResponse;
  return Array.from(new Set((payload.data ?? []).map((model) => model.id).filter(Boolean)));
}

export function tokenModelOptions(modelIds: string[], catalog: GlobalModelResponse[]) {
  const labels = catalogLabels(catalog);
  return Array.from(new Set(modelIds))
    .map((id) => ({
      id,
      label: labels.get(id) ?? id,
      format: ccSwitchFormatForModel(id),
    }))
    .sort((left, right) => left.label.localeCompare(right.label));
}

export function modelsForCcSwitchFormat(models: CcSwitchModelOption[], format: CcSwitchFormat) {
  return models.filter((model) => model.format === format);
}

export function buildCcSwitchDeepLink(args: BuildCcSwitchDeepLinkArgs) {
  const endpoint = endpointUrl(args.baseUrl, CC_SWITCH_ENDPOINT_PATH_BY_FORMAT[args.format]);
  const usageScript = window.btoa(ccSwitchUsageScript());
  const params = new URLSearchParams({
    resource: 'provider',
    app: CC_SWITCH_APP_BY_FORMAT[args.format],
    name: providerName(args.siteName, args.tokenName),
    homepage: args.homepage,
    endpoint,
    apiKey: args.rawToken,
    ...ccSwitchModelParams(args.format, args.modelId),
    configFormat: 'json',
    usageEnabled: 'true',
    usageScript,
    usageApiKey: args.rawToken,
    usageBaseUrl: endpointUrl(args.baseUrl, '/v1'),
    usageAutoInterval: String(CC_SWITCH_USAGE_AUTO_INTERVAL_MINUTES),
  });
  return `ccswitch://v1/import?${params.toString()}`;
}

function ccSwitchFormatForModel(modelId: string): CcSwitchFormat {
  const normalized = modelId.trim().toLowerCase();
  if (normalized.includes('claude')) return 'claude';
  if (normalized.includes('gemini')) return 'gemini';
  return 'openai';
}

function ccSwitchModelParams(format: CcSwitchFormat, modelId: string): Record<string, string> {
  if (format !== 'claude') {
    return { model: modelId };
  }
  return {
    model: modelId,
    haikuModel: modelId,
    sonnetModel: modelId,
    opusModel: modelId,
  };
}

function ccSwitchUsageScript() {
  return `({
  request: {
    url: "{{baseUrl}}/usage",
    method: "GET",
    headers: { "Authorization": "Bearer {{apiKey}}" }
  },
  extractor: function(response) {
    return {
      isValid: true,
      remaining: response?.remaining_quota ?? null,
      used: response?.used_quota ?? null,
      total: response?.quota_limit ?? null,
      unit: "${ACCOUNTING_CURRENCY}"
    };
  }
})`;
}

function providerName(siteName: string | null | undefined, tokenName: string) {
  const prefix = siteName?.trim() || 'Hook';
  return `${prefix} - ${tokenName}`;
}

function catalogLabels(catalog: GlobalModelResponse[]) {
  const labels = new Map<string, string>();
  for (const model of catalog) {
    const label = model.display_name?.trim() && model.display_name !== model.name
      ? `${model.display_name} (${model.name})`
      : model.name;
    labels.set(model.name, label);
    labels.set(model.id, label);
  }
  return labels;
}

async function responseErrorMessage(response: Response, fallback: string) {
  const payload = await response.json().catch(() => null);
  if (!payload || typeof payload !== 'object') return `${fallback}: ${response.status}`;
  const error = 'error' in payload ? payload.error : null;
  const message = error && typeof error === 'object' && 'message' in error ? error.message : null;
  return typeof message === 'string' && message.trim()
    ? message
    : `${fallback}: ${response.status}`;
}
