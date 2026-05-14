import { assert, assertEqual, assertIncludes } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { routeIds } from '../../20260513-user-access-real-flow/lib/user_access_fixtures.mjs';
import { assertStreamSuccess, openAiChatRequest, proxyCall, successRow } from './request_record_real_client.mjs';
import {
  fetchProviderUpstreamModels,
  listProviderEndpoints,
  listProviderKeys,
  listProviderModelBindings,
  updateProviderEndpoint,
  updateProviderKey,
  updateProviderModelBinding,
} from './request_record_real_provider_admin.mjs';
import { getRequestRecord } from './request_record_real_support.mjs';

const FIRST_MAPPING = Object.freeze({ name: 'R-claude-opus-4-7', reasoning_effort: 'high' });
const SECOND_MAPPING = Object.freeze({ name: 'ccmax-claude-opus-4-7', reasoning_effort: 'minimal' });
const OPENAI_PROVIDER_ID = routeIds.providerOpenAI;
const OPENAI_ENDPOINT_IDS = Object.freeze([routeIds.endpointOpenAIChat, routeIds.endpointOpenAIResponses, routeIds.endpointOpenAICompact]);
const OPENAI_KEY_IDS = Object.freeze([routeIds.keyOpenAIPrimary, routeIds.keyOpenAISecondary]);

export async function upstreamModelFetch(state) {
  const { ctx } = state;
  const openaiModels = await fetchProviderUpstreamModels(ctx, state.adminToken(), routeIds.providerOpenAI);
  const claudeModels = await fetchProviderUpstreamModels(ctx, state.adminToken(), routeIds.providerClaude);
  const geminiModels = await fetchProviderUpstreamModels(ctx, state.adminToken(), routeIds.providerGemini);
  assert(openaiModels.models.includes('gpt-5.5'), 'Hook Pool upstream models should include gpt-5.5');
  assert(openaiModels.models.includes('gpt-5.4-mini'), 'Hook Pool upstream models should include gpt-5.4-mini');
  assert(hasClaude47Variant(claudeModels.models), 'Claude upstream models should include a 4.7 variant');
  assert(geminiModels.models.includes('gemini-3.1-pro-preview'), 'Ekan8 Gemini upstream models should include gemini-3.1-pro-preview');
  return {
    openai: selectSample(openaiModels.models, ['gpt-5.5', 'gpt-5.4-mini']),
    claude: selectSample(claudeModels.models, ['claude-opus-4-7', 'R-claude-opus-4-7', 'ccmax-claude-opus-4-7']),
    gemini: selectSample(geminiModels.models, ['gemini-3.1-pro-preview', '[满血]gemini-3.1-pro-preview']),
  };
}

export async function mappedOpenAiRuntime(state, modelIds) {
  const { ctx } = state;
  const original = await currentOpenAiFixtureState(ctx, state.adminToken(), modelIds);
  const evidence = {};
  try {
    await pointOpenAiFixtureToEkan8(ctx, state.adminToken());

    const fetched = await fetchProviderUpstreamModels(ctx, state.adminToken(), OPENAI_PROVIDER_ID);
    assert(fetched.models.includes(FIRST_MAPPING.name), 'mapped Ekan8 OpenAI model list should include first alias');
    assert(fetched.models.includes(SECOND_MAPPING.name), 'mapped Ekan8 OpenAI model list should include second alias');
    evidence.fetch = selectSample(fetched.models, [FIRST_MAPPING.name, SECOND_MAPPING.name, 'claude-opus-4-7']);

    await updateProviderModelBinding(ctx, state.adminToken(), OPENAI_PROVIDER_ID, original.binding.id, {
      provider_model_mapping: FIRST_MAPPING,
    });
    const nonStream = await proxyCall(
      ctx,
      state.db,
      state.tokenValues.openaiOnly,
      'mapped openai non-stream',
      openAiChatRequest(ctx, ctx.models.openai, state.marker('mapped-openai-nonstream')),
    );
    evidence.nonStream = await assertMappedNonStream(state, nonStream, FIRST_MAPPING);

    await updateProviderModelBinding(ctx, state.adminToken(), OPENAI_PROVIDER_ID, original.binding.id, {
      provider_model_mapping: SECOND_MAPPING,
    });
    const stream = await proxyCall(
      ctx,
      state.db,
      state.tokenValues.openaiOnly,
      'mapped openai stream',
      openAiChatRequest(ctx, ctx.models.openai, state.marker('mapped-openai-stream'), true),
    );
    evidence.stream = await assertMappedStream(state, stream, SECOND_MAPPING);
    return evidence;
  } finally {
    await restoreOpenAiFixture(ctx, state.adminToken(), original);
  }
}

async function currentOpenAiFixtureState(ctx, adminToken, modelIds) {
  const [bindings, endpoints, keys] = await Promise.all([
    listProviderModelBindings(ctx, adminToken, OPENAI_PROVIDER_ID),
    listProviderEndpoints(ctx, adminToken, OPENAI_PROVIDER_ID),
    listProviderKeys(ctx, adminToken, OPENAI_PROVIDER_ID),
  ]);
  const binding = bindings.find((item) => item.global_model_id === modelIds.openai);
  assert(binding, 'OpenAI fixture binding should exist');
  const endpointBaseUrls = Object.fromEntries(
    endpoints.filter((item) => OPENAI_ENDPOINT_IDS.includes(item.id)).map((item) => [item.id, item.base_url]),
  );
  for (const id of OPENAI_ENDPOINT_IDS) assert(endpointBaseUrls[id], `OpenAI fixture endpoint should exist: ${id}`);
  const keyNames = Object.fromEntries(keys.filter((item) => OPENAI_KEY_IDS.includes(item.id)).map((item) => [item.id, item.name]));
  for (const id of OPENAI_KEY_IDS) assert(keyNames[id], `OpenAI fixture key should exist: ${id}`);
  return { binding, endpointBaseUrls };
}

async function pointOpenAiFixtureToEkan8(ctx, adminToken) {
  for (const endpointId of OPENAI_ENDPOINT_IDS) {
    await updateProviderEndpoint(ctx, adminToken, OPENAI_PROVIDER_ID, endpointId, { base_url: ctx.upstreams.geminiBaseUrl });
  }
  for (const keyId of OPENAI_KEY_IDS) {
    await updateProviderKey(ctx, adminToken, OPENAI_PROVIDER_ID, keyId, { api_key: ctx.secrets.geminiKey });
  }
}

async function restoreOpenAiFixture(ctx, adminToken, original) {
  for (const endpointId of OPENAI_ENDPOINT_IDS) {
    await updateProviderEndpoint(ctx, adminToken, OPENAI_PROVIDER_ID, endpointId, { base_url: original.endpointBaseUrls[endpointId] });
  }
  for (const keyId of OPENAI_KEY_IDS) {
    await updateProviderKey(ctx, adminToken, OPENAI_PROVIDER_ID, keyId, { api_key: ctx.secrets.hookPoolKey });
  }
  await updateProviderModelBinding(ctx, adminToken, OPENAI_PROVIDER_ID, original.binding.id, {
    provider_model_mapping: original.binding.provider_model_mapping ?? null,
  });
}

async function assertMappedNonStream(state, result, mapping) {
  const detail = await getRequestRecord(state.ctx, state.adminToken(), result.requestId);
  const candidate = detail.candidates.find((item) => item.status === 'success');
  assert(candidate, 'mapped non-stream request should have successful candidate');
  assertEqual(candidate.provider_request_body?.model, mapping.name, 'provider request should use mapped upstream model');
  assertEqual(candidate.provider_request_body?.reasoning_effort, mapping.reasoning_effort, 'provider request should inject reasoning_effort');
  assertEqual(candidate.provider_response_body?.model, 'claude-opus-4-7', 'real upstream should expose its own response model');
  assertEqual(detail.client_response_body?.model, state.ctx.models.openai, 'client response should be rewritten to requested model');
  assertEqual(successRow(result.trace).provider_name, 'Route Test Hook Pool', 'mapped request should stay on OpenAI fixture provider');
  return {
    requestId: result.requestId,
    upstreamModel: candidate.provider_request_body.model,
    responseModel: detail.client_response_body.model,
  };
}

async function assertMappedStream(state, result, mapping) {
  assertStreamSuccess(result, false);
  assertIncludes(result.text, `"model":"${state.ctx.models.openai}"`, 'stream chunks should expose requested model');
  assertStreamModels(result.text, state.ctx.models.openai);
  const detail = await getRequestRecord(state.ctx, state.adminToken(), result.requestId);
  const candidate = detail.candidates.find((item) => item.status === 'success');
  assert(candidate, 'mapped stream request should have successful candidate');
  assertEqual(candidate.provider_request_body?.model, mapping.name, 'stream provider request should use updated mapped model');
  assertEqual(candidate.provider_request_body?.reasoning_effort, mapping.reasoning_effort, 'stream provider request should inject updated reasoning_effort');
  assertEqual(detail.record.model_name, state.ctx.models.openai, 'record model name should stay as client model');
  return {
    requestId: result.requestId,
    upstreamModel: candidate.provider_request_body.model,
    traceModel: state.ctx.models.openai,
  };
}

function selectSample(models, preferred) {
  return preferred.filter((name) => models.includes(name));
}

function assertStreamModels(text, expectedModel) {
  const matches = [...text.matchAll(/"model":"([^"]+)"/g)].map(([, value]) => value);
  assert(matches.length > 0, 'stream response should contain model fields');
  assert(matches.every((value) => value === expectedModel), `stream response should only expose requested model: ${matches.join(', ')}`);
}

function hasClaude47Variant(models) {
  return models.some((name) => name.includes('claude-opus-4-7'));
}
