import { assert } from './assertions.mjs';

export async function resolveUpstreams(ctx) {
  const game86Models = extractOpenAiModels(await fetchJson(`${ctx.baseUrls.game86}/v1/models`, bearer(ctx.keys.game86)));
  const ekan8Models = extractOpenAiModels(await fetchJson(`${ctx.baseUrls.ekan8}/v1/models`, bearer(ctx.keys.ekan8)));
  const game86Model = chooseModel(game86Models, ['gpt-5.4-mini', 'gpt-5.4', 'gpt-4o-mini', 'gpt-4o']);
  const ekan8OpenAiModel = chooseEkan8MappedModel(ekan8Models);
  assert(game86Model, `86gamestore model not found, available: ${game86Models.slice(0, 40).join(', ')}`);
  assert(ekan8OpenAiModel, `Ekan8 OpenAI-compatible model not found, available: ${ekan8Models.slice(0, 60).join(', ')}`);
  await probeChat(ctx.baseUrls.game86, ctx.keys.game86, game86Model, '86gamestore');
  await probeChat(ctx.baseUrls.ekan8, ctx.keys.ekan8, ekan8OpenAiModel, 'Ekan8');
  return Object.freeze({
    game86Model,
    ekan8OpenAiModel,
    game86Models: game86Models.slice(0, 40),
    ekan8Models: ekan8Models.slice(0, 60),
  });
}

export async function fetchJson(url, headers) {
  const response = await fetch(url, { headers });
  const text = await response.text();
  let body;
  try {
    body = JSON.parse(text);
  } catch (error) {
    throw new Error(`invalid JSON from ${url}: ${response.status} ${text.slice(0, 400)} :: ${error.message}`);
  }
  if (!response.ok) {
    throw new Error(`upstream JSON request failed ${response.status} from ${url}: ${text.slice(0, 500)}`);
  }
  return body;
}

function extractOpenAiModels(body) {
  const items = Array.isArray(body?.data) ? body.data : [];
  return items.map((item) => item?.id).filter(Boolean);
}

function chooseModel(models, preferred) {
  return preferred.find((name) => models.includes(name)) || models.find(isOpenAiChatModel) || models[0] || '';
}

function chooseEkan8MappedModel(models) {
  const preferred = [
    'R-claude-opus-4-7',
    'ccmax-claude-opus-4-7',
    'claude-opus-4-5',
    'claude-sonnet-4-5',
    '[满血]gemini-3.1-pro-preview',
    '[满血]gemini-3-pro-preview',
    'gemini-3.1-pro-preview',
    'gemini-3-pro-preview',
  ];
  return preferred.find((name) => models.includes(name)) || models.find(isClaudeOrGeminiModel) || chooseModel(models, []);
}

function isOpenAiChatModel(name) {
  return /^gpt-|^chatgpt-|^o[134]-|^o[134]-/.test(name);
}

function isClaudeOrGeminiModel(name) {
  return name.toLowerCase().includes('claude') || name.toLowerCase().includes('gemini');
}

async function probeChat(baseUrl, key, model, label) {
  const response = await fetch(`${baseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { ...bearer(key), 'content-type': 'application/json' },
    body: JSON.stringify({
      model,
      messages: [{ role: 'user', content: `${label} Hook billing probe` }],
      max_tokens: 8,
      temperature: 0,
    }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${label} probe failed ${response.status}: ${text.slice(0, 500)}`);
  }
}

function bearer(apiKey) {
  return { Authorization: `Bearer ${apiKey}` };
}
