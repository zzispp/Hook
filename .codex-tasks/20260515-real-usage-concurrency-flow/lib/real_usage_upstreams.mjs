import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';

export async function resolveUpstreamModels(ctx) {
  const hookModels = await fetchOpenAiModels(ctx.upstreams.provider1BaseUrl, ctx.realSecrets.provider1Key);
  const ekan8Models = await fetchGeminiModels(ctx.upstreams.provider2BaseUrl, ctx.realSecrets.provider2Key);
  const hookModel = chooseModel(hookModels, [
    process.env.HOOK_REAL_PROVIDER1_MODEL,
    'gpt-5.4-mini',
    'gpt-5.4',
    'gpt-5.5',
  ]);
  const ekan8Model = ctx.realModels.ekan8 || chooseModel(ekan8Models, ['gemini-3.1-pro-preview', '[满血]gemini-3.1-pro-preview']);
  assert(hookModel, `provider1 model list did not include a usable chat model: ${hookModels.slice(0, 20).join(', ')}`);
  assert(ekan8Model, `Ekan8 model list did not include a usable Gemini model: ${ekan8Models.slice(0, 20).join(', ')}`);
  return Object.freeze({
    provider1KeyCount: ctx.realSecrets.provider1Keys.length,
    provider2KeyCount: ctx.realSecrets.provider2Keys.length,
    hookModel,
    ekan8Model,
    hookModelSample: sampleModels(hookModels, hookModel),
    ekan8ModelSample: sampleModels(ekan8Models, ekan8Model),
  });
}

async function fetchOpenAiModels(baseUrl, key) {
  const response = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/models`, {
    headers: { authorization: `Bearer ${key}` },
  });
  return parseModelResponse(response, 'openai_chat');
}

async function fetchGeminiModels(baseUrl, key) {
  const url = new URL(`${baseUrl.replace(/\/$/, '')}/v1beta/models`);
  url.searchParams.set('key', key);
  const response = await fetch(url);
  return parseModelResponse(response, 'gemini_chat');
}

async function parseModelResponse(response, apiFormat) {
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`upstream model fetch failed ${response.status}: ${text.slice(0, 500)}`);
  }
  const value = text.trim() ? JSON.parse(text) : null;
  return extractModelNames(value, apiFormat);
}

function extractModelNames(value, apiFormat) {
  const names = new Set();
  for (const item of modelItems(value)) {
    const name = modelName(item, apiFormat);
    if (name) {
      names.add(name);
    }
  }
  return [...names].sort();
}

function modelItems(value) {
  if (Array.isArray(value)) {
    return value;
  }
  if (value && typeof value === 'object') {
    return Array.isArray(value.data) ? value.data : Array.isArray(value.models) ? value.models : [];
  }
  return [];
}

function modelName(item, apiFormat) {
  const raw = typeof item === 'string' ? item : item?.id || item?.name;
  if (!raw || typeof raw !== 'string') {
    return '';
  }
  const trimmed = raw.trim();
  if (!trimmed) {
    return '';
  }
  if (apiFormat === 'gemini_chat') {
    return trimmed.split('/').pop();
  }
  return trimmed;
}

function chooseModel(models, preferred) {
  for (const name of preferred.filter(Boolean)) {
    if (models.includes(name)) {
      return name;
    }
  }
  return models.find((name) => /gpt|gemini|claude/i.test(name)) || models[0] || '';
}

function sampleModels(models, selected) {
  return [...new Set([selected, ...models.slice(0, 8)].filter(Boolean))];
}
