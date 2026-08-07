import type { ModelsDevData, ModelsDevModel, ModelsDevProvider } from 'src/types/model';

const MODELS_DEV_URL = 'https://models.dev/api.json';

const OFFICIAL_PROVIDER_IDS: ReadonlySet<string> = new Set([
  'anthropic',
  'openai',
  'google',
  'google-vertex',
  'azure',
  'amazon-bedrock',
  'xai',
  'meta',
  'deepseek',
  'mistral',
  'cohere',
  'zhipuai',
  'alibaba',
  'minimax',
  'moonshot',
  'baichuan',
  'ai21',
]);

export type ModelsDevRequest = (
  input: RequestInfo | URL,
  init?: RequestInit
) => Promise<Response>;

export async function fetchModelsDevData(
  request: ModelsDevRequest = globalThis.fetch
): Promise<ModelsDevData> {
  const response = await request(MODELS_DEV_URL, {
    headers: { Accept: 'application/json' },
  });
  if (!response.ok) {
    throw new Error(`models.dev request failed: HTTP ${response.status}`);
  }

  return parseModelsDevData(await response.json());
}

function parseModelsDevData(value: unknown): ModelsDevData {
  if (!isRecord(value)) {
    throw new Error('models.dev response must be a provider object');
  }

  return Object.fromEntries(
    Object.entries(value).map(([providerId, provider]) => [
      providerId,
      parseProvider(providerId, provider),
    ])
  );
}

function parseProvider(providerId: string, value: unknown): ModelsDevProvider {
  if (!isRecord(value)) {
    throw new Error(`models.dev provider "${providerId}" must be an object`);
  }

  return {
    ...value,
    models: parseModels(providerId, value.models),
    official: OFFICIAL_PROVIDER_IDS.has(providerId),
  };
}

function parseModels(
  providerId: string,
  value: unknown
): Record<string, ModelsDevModel> | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) {
    throw new Error(`models.dev provider "${providerId}" models must be an object`);
  }

  return Object.fromEntries(
    Object.entries(value).map(([modelId, model]) => [
      modelId,
      parseModel(providerId, modelId, model),
    ])
  );
}

function parseModel(providerId: string, modelId: string, value: unknown): ModelsDevModel {
  if (!isRecord(value)) {
    throw new Error(`models.dev model "${providerId}/${modelId}" must be an object`);
  }
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
