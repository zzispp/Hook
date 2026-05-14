import { assert, assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { providerNames, routeIds, tokenIds, setBrokenProviderActive, setClaudePrimaryKey, setOpenAIChatBaseUrl, setOpenAIKeyPriorities } from '../../20260513-user-access-real-flow/lib/user_access_fixtures.mjs';
import {
  assertNoOpenRows,
  assertSingleSuccessAttempt,
  assertStreamSuccess,
  claudeMessagesRequest,
  expectProxyFailure,
  geminiRequest,
  openAiChatRequest,
  openAiResponsesRequest,
  proxyCall,
  proxyStatus,
  successRow,
  tracesSinceByTokenIds,
} from './request_record_real_client.mjs';
import { getRequestRecord, waitForRequestRecords } from './request_record_real_support.mjs';

export async function fixedOrderRoutes(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues, artifacts } = state;
  const token = tokenValues.unrestricted;
  const chat = await proxyCall(ctx, db, token, 'fixed chat', openAiChatRequest(ctx, ctx.models.openai, state.marker('fixed-chat')));
  const stream = await proxyCall(ctx, db, token, 'fixed stream', openAiChatRequest(ctx, ctx.models.openai, state.marker('fixed-stream'), true));
  const responses = await proxyCall(ctx, db, token, 'fixed responses', openAiResponsesRequest(ctx.models.openai, state.marker('fixed-responses')));
  const compact = await proxyCall(ctx, db, token, 'fixed compact', openAiResponsesRequest(ctx.models.openai, state.marker('fixed-compact'), true));
  assertSingleSuccessAttempt(chat, providerNames.openai, 'Route Hook primary');
  assertSingleSuccessAttempt(stream, providerNames.openai, 'Route Hook primary');
  assertSingleSuccessAttempt(responses, providerNames.openai, 'Route Hook primary');
  assertSingleSuccessAttempt(compact, providerNames.openai, 'Route Hook primary');
  assertStreamSuccess(stream, false);
  await waitForRequestRecords(db, [chat.requestId, stream.requestId, responses.requestId, compact.requestId]);
  artifacts.successForCompression = chat.requestId;
  artifacts.successStreamForSweep = stream.requestId;
  return [chat.requestId, stream.requestId, responses.requestId, compact.requestId];
}

export async function userAllowMatrix(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues, artifacts } = state;
  const openai = await proxyCall(ctx, db, tokenValues.openaiOnly, 'allow openai', openAiChatRequest(ctx, ctx.models.openai, state.marker('allow-openai')));
  const claude = await proxyCall(ctx, db, tokenValues.claudeOnly, 'allow claude', openAiChatRequest(ctx, ctx.models.claude, state.marker('allow-claude')), { expectOk: false });
  const gemini = await proxyCall(ctx, db, tokenValues.geminiOnly, 'allow gemini', openAiChatRequest(ctx, ctx.models.gemini, state.marker('allow-gemini')));
  assertSingleSuccessAttempt(openai, providerNames.openai, 'Route Hook primary');
  assertEqual(successRow(gemini.trace).provider_name, providerNames.gemini, 'gemini user should use Gemini provider');
  artifacts.claudeProviderUnavailable = claude.status !== 200;
  if (!artifacts.claudeProviderUnavailable) {
    assertEqual(successRow(claude.trace).provider_name, providerNames.claude, 'claude user should use Claude provider');
  } else {
    const claudeFailure = claude.trace.at(-1);
    assertEqual(claude.status, 503, 'unavailable claude provider should surface upstream 503');
    assertEqual(claudeFailure?.error_code, 'model_not_found', 'claude provider unavailability should preserve upstream code');
  }
  return { openai: openai.requestId, claude: claude.requestId, gemini: gemini.requestId, claudeUnavailable: artifacts.claudeProviderUnavailable };
}

export async function userDenyMatrix(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues } = state;
  const modelDenied = await expectProxyFailure(ctx, db, tokenValues.openaiOnly, 'deny model', openAiChatRequest(ctx, ctx.models.claude, state.marker('deny-model')), 403, 'model is not allowed by user');
  const providerDenied = await expectProxyFailure(ctx, db, tokenValues.providerMismatch, 'deny provider', openAiChatRequest(ctx, ctx.models.claude, state.marker('deny-provider')), 404, 'no active provider candidate');
  const modelOnlyDenied = await expectProxyFailure(ctx, db, tokenValues.modelOpenaiOnly, 'deny model-only', openAiChatRequest(ctx, ctx.models.claude, state.marker('deny-model-only')), 403, 'model is not allowed by user');
  return { modelDenied, providerDenied, modelOnlyDenied };
}

export async function routeKeyFailover(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues } = state;
  setClaudePrimaryKey(db, ctx, 'sk-real-record-invalid');
  const result = await proxyCall(ctx, db, tokenValues.claudeOnly, 'key failover', openAiChatRequest(ctx, ctx.models.claude, state.marker('key-failover')), { expectOk: false });
  setClaudePrimaryKey(db, ctx, ctx.secrets.claudeKey);
  assertEqual(result.trace[0].status, 'failed', 'invalid primary key should fail');
  assertEqual(result.trace[0].status_code, '401', 'invalid primary key should expose auth failure');
  if (!state.artifacts.claudeProviderUnavailable) {
    assertEqual(successRow(result.trace).key_name, 'Route Claude secondary', 'secondary key should succeed');
    assertNoOpenRows(result.trace);
    return { requestId: result.requestId, attempts: result.trace.length, claudeUnavailable: false };
  }
  const secondary = result.trace.at(-1);
  assertEqual(result.status, 503, 'secondary key should surface provider unavailability');
  assertEqual(secondary?.error_code, 'model_not_found', 'secondary key should preserve provider unavailability code');
  return { requestId: result.requestId, attempts: result.trace.length, claudeUnavailable: true };
}

export async function routeEndpointFallback(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues } = state;
  setOpenAIChatBaseUrl(db, 'http://127.0.0.1:9');
  const result = await proxyCall(ctx, db, tokenValues.openaiOnly, 'endpoint fallback', openAiChatRequest(ctx, ctx.models.openai, state.marker('endpoint-fallback')));
  setOpenAIChatBaseUrl(db, ctx.upstreams.openaiBaseUrl);
  const success = successRow(result.trace);
  assert(Number(success.retry_index) >= 2, 'fallback should reach a converted endpoint after exact endpoint failures');
  assertEqual(success.needs_conversion, 'true', 'endpoint fallback should convert to responses');
  assertNoOpenRows(result.trace);
  return { requestId: result.requestId, endpoint: success.provider_api_format, attempts: result.trace.length };
}

export async function providerFailover(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues } = state;
  setBrokenProviderActive(db, true);
  const result = await proxyCall(ctx, db, tokenValues.unrestricted, 'provider failover', openAiChatRequest(ctx, ctx.models.openai, state.marker('provider-failover')));
  setBrokenProviderActive(db, false);
  assert(result.trace.some((row) => row.provider_name === providerNames.broken && row.status === 'failed'), 'broken provider should fail first');
  assertEqual(successRow(result.trace).provider_name, providerNames.openai, 'healthy provider should take over');
  return { requestId: result.requestId, attempts: result.trace.length };
}

export async function cacheAffinity(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'cache_affinity');
  const { ctx, redis, db, tokenValues } = state;
  setOpenAIKeyPriorities(db, 0, 1);
  await redis.setex(`${ctx.redis.prefix}:llm_proxy:affinity:${tokenIds.openaiOnly}:${modelIds.openai}:openai_chat`, 300, routeIds.keyOpenAISecondary);
  const result = await proxyCall(ctx, db, tokenValues.openaiOnly, 'cache affinity', openAiChatRequest(ctx, ctx.models.openai, state.marker('affinity')));
  assertEqual(successRow(result.trace).key_name, 'Route Hook secondary', 'affinity should pin the secondary key');
  return { requestId: result.requestId, key: successRow(result.trace).key_name };
}

export async function loadBalance(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'load_balance');
  const { ctx, db, tokenValues } = state;
  setOpenAIKeyPriorities(db, 0, 0);
  const keys = new Set();
  for (let index = 0; index < 20; index += 1) {
    const result = await proxyCall(ctx, db, tokenValues.openaiOnly, `load balance ${index}`, openAiChatRequest(ctx, ctx.models.openai, state.marker(`lb-${index}`)), { printTrace: index < 4 });
    keys.add(successRow(result.trace).key_name);
  }
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'load balance should use both OpenAI keys');
  return { keys: [...keys].sort() };
}

export async function formatConversionMatrix(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues } = state;
  const requestIds = [];
  if (!state.artifacts.claudeProviderUnavailable) {
    const claude = await proxyCall(ctx, db, tokenValues.claudeOnly, 'openai to claude', openAiChatRequest(ctx, ctx.models.claude, state.marker('openai-claude')));
    const claudeStream = await proxyCall(ctx, db, tokenValues.claudeOnly, 'openai stream to claude', openAiChatRequest(ctx, ctx.models.claude, state.marker('openai-claude-stream'), true));
    assertStreamSuccess(claudeStream, true);
    await assertConvertedDetail(state, claude.requestId, 'claude');
    requestIds.push(claude.requestId, claudeStream.requestId);
  }
  const gemini = await proxyCall(ctx, db, tokenValues.geminiOnly, 'openai to gemini', openAiChatRequest(ctx, ctx.models.gemini, state.marker('openai-gemini')));
  const geminiStream = await proxyCall(ctx, db, tokenValues.geminiOnly, 'openai stream to gemini', openAiChatRequest(ctx, ctx.models.gemini, state.marker('openai-gemini-stream'), true));
  const claudeToOpenAi = await proxyCall(ctx, db, tokenValues.openaiOnly, 'claude to openai', claudeMessagesRequest(ctx.models.openai, state.marker('claude-openai')));
  const geminiExact = await proxyCall(ctx, db, tokenValues.geminiOnly, 'gemini exact', geminiRequest(ctx.models.gemini, state.marker('gemini-exact')));
  const geminiExactStream = await proxyCall(ctx, db, tokenValues.geminiOnly, 'gemini exact stream', geminiRequest(ctx.models.gemini, state.marker('gemini-exact-stream'), true));
  assertStreamSuccess(geminiStream, true);
  assertStreamSuccess(geminiExactStream, false);
  await assertConvertedDetail(state, gemini.requestId, 'gemini');
  requestIds.push(gemini.requestId, geminiStream.requestId, claudeToOpenAi.requestId, geminiExact.requestId, geminiExactStream.requestId);
  return { requestIds, claudeUnavailable: state.artifacts.claudeProviderUnavailable };
}

export async function highConcurrency(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'load_balance');
  const { ctx, db, tokenValues } = state;
  setOpenAIKeyPriorities(db, 0, 0);
  const before = new Date().toISOString();
  const requests = concurrencyRequests(state, ctx);
  const responses = await Promise.all(requests.map((item) => proxyStatus(ctx, item.token, item.request)));
  assert(responses.every((item) => item.ok), `all 100 concurrent requests should succeed: ${JSON.stringify(statusCounts(responses))}`);
  const traces = tracesSinceByTokenIds(db, before, [tokenIds.openaiOnly]);
  assertEqual(traces.length, requests.length, 'concurrency should record every request');
  const keys = new Set(traces.map(({ trace }) => successRow(trace).key_name));
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'concurrency should use both OpenAI keys');
  return { requests: requests.length, keys: [...keys].sort() };
}

async function assertConvertedDetail(state, requestId, providerKind) {
  const detail = await getRequestRecord(state.ctx, state.adminToken(), requestId);
  const success = detail.candidates.find((item) => item.status === 'success');
  assert(success?.needs_conversion === true, 'converted detail should keep conversion flag');
  if (providerKind === 'claude') assert(success.provider_request_body?.model === state.ctx.models.claudeProvider, 'Claude provider request should use provider model');
  if (providerKind === 'gemini') assert(Array.isArray(success.provider_request_body?.contents), 'Gemini provider request should use Gemini payload shape');
}

function concurrencyRequests(state, ctx) {
  const requests = [];
  const markerRoot = state.marker(`concurrency-${Date.now()}`);
  for (let index = 0; index < 30; index += 1) requests.push({ token: state.tokenValues.openaiOnly, request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|nonstream|${String(index).padStart(3, '0')}|`) });
  for (let index = 0; index < 70; index += 1) requests.push({ token: state.tokenValues.openaiOnly, request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|stream|${String(index).padStart(3, '0')}|`, true) });
  return requests;
}

function statusCounts(responses) {
  const counts = {};
  for (const response of responses) counts[String(response.status)] = (counts[String(response.status)] ?? 0) + 1;
  return counts;
}
