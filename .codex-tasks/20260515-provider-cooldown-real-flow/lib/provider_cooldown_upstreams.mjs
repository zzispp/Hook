import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';

export async function resolveProviderCooldownUpstreams(ctx) {
  const msutoolsModels = await fetchOpenAiModels(ctx.upstreams.msutoolsBaseUrl, ctx.cooldownSecrets.msutoolsKey);
  const ekan8Models = await fetchGeminiModels(ctx.upstreams.ekan8BaseUrl, ctx.cooldownSecrets.ekan8Key);
  const msutoolsModel = chooseModel(msutoolsModels, [
    process.env.HOOK_COOLDOWN_MSUTOOLS_MODEL,
    'gpt-5.4-mini',
    'gpt-5.4',
    'gpt-4o-mini',
  ]);
  const ekan8Model = chooseModel(ekan8Models, [
    process.env.HOOK_COOLDOWN_EKAN8_MODEL,
    'gemini-3.1-pro-preview',
    '[满血]gemini-3.1-pro-preview',
    'gemini-2.5-flash',
  ]);
  assert(msutoolsModel, `msutools model list did not contain a usable model: ${msutoolsModels.slice(0, 20).join(', ')}`);
  assert(ekan8Model, `Ekan8 model list did not contain a usable Gemini model: ${ekan8Models.slice(0, 20).join(', ')}`);
  return {
    msutoolsModel,
    ekan8Model,
    msutoolsModelSample: sampleModels(msutoolsModels, msutoolsModel),
    ekan8ModelSample: sampleModels(ekan8Models, ekan8Model),
  };
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
  const body = text.trim() ? JSON.parse(text) : null;
  return extractModelNames(body, apiFormat);
}

function extractModelNames(value, apiFormat) {
  const names = new Set();
  for (const item of modelItems(value)) {
    const name = modelName(item, apiFormat);
    if (name) names.add(name);
  }
  return [...names].sort();
}

function modelItems(value) {
  if (Array.isArray(value)) return value;
  if (!value || typeof value !== 'object') return [];
  if (Array.isArray(value.data)) return value.data;
  if (Array.isArray(value.models)) return value.models;
  return [];
}

function modelName(item, apiFormat) {
  const raw = typeof item === 'string' ? item : item?.id || item?.name;
  if (!raw || typeof raw !== 'string') return '';
  const trimmed = raw.trim();
  if (!trimmed) return '';
  return apiFormat === 'gemini_chat' ? trimmed.split('/').pop() : trimmed;
}

function chooseModel(models, preferred) {
  for (const name of preferred.filter(Boolean)) {
    if (models.includes(name)) return name;
  }
  return models.find((name) => /gpt|gemini|claude/i.test(name)) || models[0] || '';
}

function sampleModels(models, selected) {
  return [...new Set([selected, ...models.slice(0, 8)].filter(Boolean))];
}

