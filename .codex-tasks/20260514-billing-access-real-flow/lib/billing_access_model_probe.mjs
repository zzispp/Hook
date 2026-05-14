import { fetchUpstreamModels } from './billing_access_client.mjs';

export async function probeRuntimeModels(ctx) {
  const [primaryModels, ekan8Models] = await Promise.all([
    fetchUpstreamModels(ctx.upstreams.primaryBaseUrl, ctx.upstreams.primaryKey),
    fetchUpstreamModels(ctx.upstreams.ekan8BaseUrl, ctx.upstreams.ekan8Key),
  ]);
  return Object.freeze({
    primaryModels,
    ekan8Models,
    primaryProviderModel: selectPrimaryModel(ctx, primaryModels),
    ekan8ProviderModel: selectEkan8Model(ctx, ekan8Models),
  });
}

export function modelProbeEvidence(runtime) {
  return {
    primaryProviderModel: runtime.primaryProviderModel,
    ekan8ProviderModel: runtime.ekan8ProviderModel,
    primaryModelCount: runtime.primaryModels.length,
    ekan8ModelCount: runtime.ekan8Models.length,
    primarySamples: runtime.primaryModels.slice(0, 12),
    ekan8Samples: runtime.ekan8Models.slice(0, 12),
  };
}

function selectPrimaryModel(ctx, models) {
  if (ctx.upstreams.primaryModel) return requireModel(models, ctx.upstreams.primaryModel, 'primary');
  return preferredModel(models, [
    (name) => name === 'gpt-5.4-mini',
    (name) => name === 'gpt-5.5',
    (name) => name.startsWith('gpt-'),
  ], 'primary gpt model');
}

function selectEkan8Model(ctx, models) {
  if (ctx.upstreams.ekan8Model) return requireModel(models, ctx.upstreams.ekan8Model, 'Ekan8');
  return preferredModel(models, [
    (name) => name.includes('gemini-3.1-pro-preview'),
    (name) => name.includes('gemini') && name.includes('pro'),
    (name) => name.includes('gemini'),
  ], 'Ekan8 gemini model');
}

function preferredModel(models, predicates, label) {
  for (const predicate of predicates) {
    const match = models.find(predicate);
    if (match) return match;
  }
  throw new Error(`missing ${label}; available models: ${models.slice(0, 30).join(', ')}`);
}

function requireModel(models, model, label) {
  if (models.includes(model)) return model;
  throw new Error(`${label} configured model not found upstream: ${model}`);
}
