import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { Db, q } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { ensureBackend } from '../20260512-real-proxy-cache-flow/lib/backend.mjs';
import { assert, assertEqual } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import {
  groupCode,
  routeIds,
  tokenIds,
  userProfile,
  providerNames,
  makeTokenValues,
  seedRouteDatabase,
  setClaudePrimaryKey,
  setOpenAIChatBaseUrl,
  setBrokenProviderActive,
  setOpenAIKeyPriorities,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
  seedUserAccessFixtures,
  resetUserAccessFixtures,
  deactivateUserAccessFixtures,
} from './lib/user_access_fixtures.mjs';
import {
  proxyCall,
  proxyStatus,
  successRow,
  adminSignIn,
  geminiRequest,
  assertNoAvailableRows,
  openAiChatRequest,
  assertStreamSuccess,
  expectProxyFailure,
  claudeMessagesRequest,
  openAiResponsesRequest,
  replaceUserViaApi,
  tracesSinceByTokenIds,
  assertSingleSuccessAttempt,
} from './lib/user_access_client.mjs';

const baseCtx = loadContext();
const db = new Db(baseCtx.db);
const ctx = contextWithExistingModels(baseCtx, db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const tokenValues = makeTokenValues();
const results = [];
const originalMode = db.scalar("select scheduling_mode from system_settings where id = 'global'") || 'fixed_order';

async function main() {
  const modelIds = seedRouteDatabase(ctx, db);
  seedUserAccessFixtures(db, tokenValues, modelIds);
  await resetAll(modelIds);
  const server = await ensureBackend(ctx.serverBaseUrl);
  try {
    await runScenarios(modelIds);
    assert(results.every((item) => item.ok), failedSummary());
    console.log('user access real flow: all scenarios passed');
  } finally {
    await cleanup(modelIds, server);
    writeResults();
  }
}

async function runScenarios(modelIds) {
  await step('fixed order unrestricted user exact routes', () => fixedOrderUnrestricted(modelIds));
  await step('user provider and model allow matrix', () => userAllowMatrix(modelIds));
  await step('user model and provider deny matrix', () => userDenyMatrix(modelIds));
  await step('edit user API refreshes proxy scheduling snapshot', () => apiUpdateRefreshesSnapshot(modelIds));
  await step('route key failover under user restrictions', () => routeKeyFailover(modelIds));
  await step('route endpoint fallback conversion under user restrictions', () => routeEndpointFallback(modelIds));
  await step('provider failover with unrestricted user', () => providerFailover(modelIds));
  await step('cache affinity under user restrictions', () => cacheAffinity(modelIds));
  await step('load balance under user restrictions', () => loadBalance(modelIds));
  await step('format conversion matrix under user restrictions', () => formatConversionMatrix(modelIds));
  await step('100 concurrent mixed real requests', () => highConcurrency(modelIds));
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence });
    console.log(`scenario passed: ${label}`);
  } catch (error) {
    results.push({ label, ok: false, error: error.stack || error.message });
    console.error(`scenario failed: ${label}: ${error.stack || error.message}`);
  }
}

async function fixedOrderUnrestricted(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const token = tokenValues.unrestricted;
  const chat = await proxyCall(ctx, db, token, 'unrestricted openai chat', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-chat')));
  assertSingleSuccessAttempt(chat, providerNames.openai, 'Route Hook primary');
  const stream = await proxyCall(ctx, db, token, 'unrestricted openai stream', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-stream'), true));
  assertSingleSuccessAttempt(stream, providerNames.openai, 'Route Hook primary');
  assertStreamSuccess(stream, false);
  const responses = await proxyCall(ctx, db, token, 'unrestricted responses', openAiResponsesRequest(ctx.models.openai, marker('fixed-responses')));
  assertSingleSuccessAttempt(responses, providerNames.openai, 'Route Hook primary');
  const compact = await proxyCall(ctx, db, token, 'unrestricted compact', openAiResponsesRequest(ctx.models.openai, marker('fixed-compact'), true));
  assertSingleSuccessAttempt(compact, providerNames.openai, 'Route Hook primary');
  return requestIds(chat, stream, responses, compact);
}

async function userAllowMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const openai = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai user allows openai',
    openAiChatRequest(ctx, ctx.models.openai, marker('allow-openai')),
  );
  assertSingleSuccessAttempt(openai, providerNames.openai, 'Route Hook primary');
  const claude = await proxyCall(
    ctx,
    db,
    tokenValues.claudeOnly,
    'claude user allows claude conversion',
    openAiChatRequest(ctx, ctx.models.claude, marker('allow-claude')),
  );
  assertEqual(successRow(claude.trace).provider_name, providerNames.claude, 'claude user should use Claude provider');
  assertEqual(successRow(claude.trace).needs_conversion, 'true', 'claude user should convert OpenAI request');
  const gemini = await proxyCall(
    ctx,
    db,
    tokenValues.geminiOnly,
    'gemini user allows gemini conversion',
    openAiChatRequest(ctx, ctx.models.gemini, marker('allow-gemini')),
  );
  assertEqual(successRow(gemini.trace).provider_name, providerNames.gemini, 'gemini user should use Gemini provider');
  assertEqual(successRow(gemini.trace).needs_conversion, 'true', 'gemini user should convert OpenAI request');
  return requestIds(openai, claude, gemini);
}

async function userDenyMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const modelDenied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai user denied claude model',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-model')),
    403,
    'model is not allowed by user',
  );
  const unrestrictedProviderDeniedByModel = await expectProxyFailure(
    ctx,
    db,
    tokenValues.modelOpenaiOnly,
    'model-only user denied claude model',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-model-only')),
    403,
    'model is not allowed by user',
  );
  const providerDenied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.providerMismatch,
    'provider mismatch user has no allowed provider route',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-provider')),
    404,
    'no active provider candidate',
  );
  return { modelDenied, unrestrictedProviderDeniedByModel, providerDenied };
}

async function apiUpdateRefreshesSnapshot(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const before = await proxyCall(
    ctx,
    db,
    tokenValues.apiUpdated,
    'api-updated unrestricted before edit',
    openAiChatRequest(ctx, ctx.models.claude, marker('api-before')),
  );
  assertEqual(successRow(before.trace).provider_name, providerNames.claude, 'api-updated user starts unrestricted');
  const adminToken = await adminSignIn(ctx);
  const profile = userProfile('apiUpdated');
  const updated = await replaceUserViaApi(ctx, adminToken, profile, {
    allowedModelIds: [modelIds.openai],
    allowedProviderIds: [routeIds.providerOpenAI],
  });
  const denied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.apiUpdated,
    'api-updated denied claude after edit',
    openAiChatRequest(ctx, ctx.models.claude, marker('api-after-deny')),
    403,
    'model is not allowed by user',
  );
  const allowed = await proxyCall(
    ctx,
    db,
    tokenValues.apiUpdated,
    'api-updated allows openai after edit',
    openAiChatRequest(ctx, ctx.models.openai, marker('api-after-allow')),
  );
  assertSingleSuccessAttempt(allowed, providerNames.openai);
  return { before: before.requestId, updatedUser: updated.id, denied, allowed: allowed.requestId };
}

async function routeKeyFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setClaudePrimaryKey(db, ctx, 'sk-user-access-invalid');
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.claudeOnly,
    'claude key failover',
    openAiChatRequest(ctx, ctx.models.claude, marker('key-failover')),
  );
  const success = successRow(result.trace);
  assertEqual(result.trace[0].key_name, 'Route Claude primary', 'first attempt should use primary key');
  assertEqual(result.trace[0].status, 'failed', 'invalid primary key should fail visibly');
  assertEqual(success.key_name, 'Route Claude secondary', 'secondary key should succeed');
  assertEqual(success.needs_conversion, 'true', 'key failover request should still convert OpenAI to Claude');
  assertNoAvailableRows(result.trace);
  setClaudePrimaryKey(db, ctx, ctx.secrets.claudeKey);
  return { requestId: result.requestId, attempts: result.trace.length };
}

async function routeEndpointFallback(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setOpenAIChatBaseUrl(db, 'http://127.0.0.1:9');
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai endpoint fallback',
    openAiChatRequest(ctx, ctx.models.openai, marker('endpoint-fallback')),
  );
  const success = successRow(result.trace);
  assert(Number(success.retry_index) >= 2, 'converted endpoint should be reached after exact endpoint key attempts');
  assertEqual(success.needs_conversion, 'true', 'endpoint fallback should convert OpenAI chat to Responses');
  assert(['openai_cli', 'openai_compact'].includes(success.provider_api_format), 'fallback should use an OpenAI Responses endpoint');
  assertNoAvailableRows(result.trace);
  setOpenAIChatBaseUrl(db, ctx.upstreams.openaiBaseUrl);
  return { requestId: result.requestId, attempts: result.trace.length, endpoint: success.provider_api_format };
}

async function providerFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setBrokenProviderActive(db, true);
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.unrestricted,
    'provider failover',
    openAiChatRequest(ctx, ctx.models.openai, marker('provider-failover')),
  );
  const failures = result.trace.filter((row) => row.provider_name === providerNames.broken && row.status === 'failed');
  assert(failures.length >= 1, 'broken provider should fail before provider failover');
  assertEqual(successRow(result.trace).provider_name, providerNames.openai, 'provider failover should reach Hook Pool');
  assertNoAvailableRows(result.trace);
  setBrokenProviderActive(db, false);
  return { requestId: result.requestId, attempts: result.trace.length };
}

async function cacheAffinity(modelIds) {
  await directSchedulingChange(modelIds, 'cache_affinity');
  setOpenAIKeyPriorities(db, 0, 1);
  await redis.setex(affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_chat'), 300, routeIds.keyOpenAISecondary);
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'cache affinity',
    openAiChatRequest(ctx, ctx.models.openai, marker('affinity')),
  );
  assertEqual(successRow(result.trace).key_name, 'Route Hook secondary', 'affinity key should be attempted first');
  return { requestId: result.requestId, key: successRow(result.trace).key_name };
}

async function loadBalance(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearScheduling();
  const keys = new Set();
  for (let index = 0; index < 20; index += 1) {
    const result = await proxyCall(
      ctx,
      db,
      tokenValues.openaiOnly,
      `load balance ${index}`,
      openAiChatRequest(ctx, ctx.models.openai, marker(`lb-${index}`)),
      { printTrace: index < 4 },
    );
    keys.add(successRow(result.trace).key_name);
    assertSingleSuccessAttempt(result, providerNames.openai);
  }
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'load balance should use both OpenAI keys');
  return { keys: [...keys].sort() };
}

async function formatConversionMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const cases = [
    ['openai to claude', tokenValues.claudeOnly, openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude')), providerNames.claude, true],
    [
      'openai stream to claude',
      tokenValues.claudeOnly,
      openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude-stream'), true),
      providerNames.claude,
      true,
    ],
    ['openai to gemini', tokenValues.geminiOnly, openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini')), providerNames.gemini, true],
    [
      'openai stream to gemini',
      tokenValues.geminiOnly,
      openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini-stream'), true),
      providerNames.gemini,
      true,
    ],
    ['claude to openai', tokenValues.openaiOnly, claudeMessagesRequest(ctx.models.openai, marker('claude-openai')), providerNames.openai, true],
    ['gemini exact', tokenValues.geminiOnly, geminiRequest(ctx.models.gemini, marker('gemini-exact')), providerNames.gemini, false],
    ['gemini stream exact', tokenValues.geminiOnly, geminiRequest(ctx.models.gemini, marker('gemini-stream'), true), providerNames.gemini, false],
  ];
  const evidence = [];
  for (const item of cases) evidence.push(await runConversionCase(...item));
  return evidence;
}

async function runConversionCase(label, token, request, providerName, converts) {
  const result = await proxyCall(ctx, db, token, label, request);
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerName, `${label} provider should match`);
  assertEqual(success.needs_conversion, String(converts), `${label} conversion flag should match`);
  if (request.body.stream) assertStreamSuccess(result, converts);
  assertNoAvailableRows(result.trace);
  return { label, requestId: result.requestId, provider: success.provider_name, conversion: success.needs_conversion };
}

async function highConcurrency(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearScheduling();
  const markerRoot = marker(`concurrency-${Date.now()}`);
  const before = new Date().toISOString();
  const requests = concurrencyRequests(markerRoot);
  const responses = await Promise.all(requests.map((item) => proxyStatus(ctx, item.token, item.request)));
  assert(
    responses.every((item) => item.ok),
    `all 100 high concurrency requests should succeed: ${JSON.stringify(statusCounts(responses))}`,
  );
  const traces = tracesSinceByTokenIds(db, before, [tokenIds.openaiOnly]);
  assertEqual(traces.length, requests.length, 'high concurrency should record every request');
  const successes = traces.map(({ trace }) => successRow(trace));
  const openaiKeys = new Set(successes.filter((row) => row.provider_name === providerNames.openai).map((row) => row.key_name));
  assert(openaiKeys.has('Route Hook primary') && openaiKeys.has('Route Hook secondary'), 'concurrency load balance should use both OpenAI keys');
  assertEqual(traces.flatMap(({ trace }) => trace.filter((row) => row.status === 'failed')).length, 0, 'concurrency should not fail attempts');
  return {
    requests: requests.length,
    openaiKeys: [...openaiKeys].sort(),
    providers: [...new Set(successes.map((row) => row.provider_name))].sort(),
  };
}

function concurrencyRequests(markerRoot) {
  const requests = [];
  for (let index = 0; index < 30; index += 1) {
    requests.push({
      token: tokenValues.openaiOnly,
      label: `concurrency openai ${index}`,
      request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|openai|${String(index).padStart(3, '0')}|`),
    });
  }
  for (let index = 0; index < 70; index += 1) {
    requests.push({
      token: tokenValues.openaiOnly,
      label: `concurrency openai stream ${index}`,
      request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|openai-stream|${String(index).padStart(3, '0')}|`, true),
    });
  }
  return requests;
}

function statusCounts(responses) {
  const counts = {};
  for (const response of responses) {
    const key = String(response.status);
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return counts;
}

async function directSchedulingChange(modelIds, mode) {
  setSchedulingModeDb(db, mode);
  await clearScheduling();
  await clearAffinity(modelIds);
}

async function resetAll(modelIds) {
  restoreRouteFixtures(ctx, db, originalMode);
  resetUserAccessFixtures(db, tokenValues, modelIds);
  await clearScheduling();
  await clearAuth();
  await clearAffinity(modelIds);
}

async function cleanup(modelIds, server) {
  try {
    restoreRouteFixtures(ctx, db, originalMode);
    deactivateUserAccessFixtures(db);
    deactivateRouteFixtures(db);
    await clearScheduling();
    await clearAuth();
    await clearAffinity(modelIds);
  } finally {
    if (server) server.kill('SIGTERM');
  }
}

async function clearScheduling() {
  await redis.del(schedulingSnapshotKey(), schedulingLockKey());
}

async function clearAuth() {
  const keys = await redis.keys(`${ctx.redis.prefix}:llm_proxy:auth:v*`);
  await redis.del(...keys, `${ctx.redis.prefix}:llm_proxy:auth:version`);
}

async function clearAffinity(modelIds) {
  await redis.del(
    affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_chat'),
    affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_cli'),
    affinityKey(tokenIds.claudeOnly, modelIds.claude, 'openai_chat'),
    affinityKey(tokenIds.geminiOnly, modelIds.gemini, 'openai_chat'),
    affinityKey(tokenIds.geminiOnly, modelIds.gemini, 'gemini_chat'),
    affinityKey(tokenIds.apiUpdated, modelIds.openai, 'openai_chat'),
    affinityKey(tokenIds.apiUpdated, modelIds.claude, 'openai_chat'),
  );
}

function affinityKey(tokenId, modelId, format) {
  return `${ctx.redis.prefix}:llm_proxy:affinity:${tokenId}:${modelId}:${format}`;
}

function schedulingSnapshotKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:snapshot:v2`;
}

function schedulingLockKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:rebuild_lock`;
}

function contextWithExistingModels(ctx, database) {
  const openai = selectedModel(database, process.env.HOOK_OPENAI_MODEL, (name) => name.startsWith('gpt-'), 'OpenAI');
  const claude = selectedModel(database, process.env.HOOK_CLAUDE_MODEL, (name) => name.startsWith('claude-'), 'Claude');
  const gemini = selectedModel(database, process.env.HOOK_GEMINI_MODEL, (name) => name.includes('gemini'), 'Gemini');
  const models = Object.freeze({
    openai,
    claude,
    gemini,
    openaiProvider: process.env.HOOK_OPENAI_PROVIDER_MODEL || openai,
    claudeProvider: process.env.HOOK_CLAUDE_PROVIDER_MODEL || claude,
    geminiProvider: process.env.HOOK_GEMINI_PROVIDER_MODEL || geminiProviderName(gemini),
  });
  console.log(`models: ${JSON.stringify(models)}`);
  return Object.freeze({ ...ctx, models });
}

function selectedModel(database, configured, matches, label) {
  if (configured && modelExists(database, configured)) {
    return configured;
  }
  if (configured) {
    throw new Error(`${label} model does not exist in global_models: ${configured}`);
  }
  const name = database.scalar(`
select name from global_models
where is_active = true
order by created_at desc, name
limit 1;`);
  const candidates = database.rows("select name from global_models where is_active = true order by created_at desc, name;").map(([value]) => value);
  const matched = candidates.find(matches);
  if (!matched) {
    throw new Error(`${label} model not found in active global_models; available: ${candidates.join(', ') || 'none'}`);
  }
  assert(name || matched, `${label} model selection should have a candidate`);
  return matched;
}

function modelExists(database, name) {
  return database.scalar(`select id from global_models where name = ${q(name)} and is_active = true limit 1;`) !== '';
}

function geminiProviderName(globalName) {
  if (globalName.startsWith('gemini-')) {
    return `[满血]${globalName}`;
  }
  return globalName;
}

function marker(value) {
  return `user-access-real-${value}`;
}

function requestIds(...items) {
  return items.map((item) => item.requestId);
}

function failedSummary() {
  return `failed scenarios: ${results.filter((item) => !item.ok).map((item) => item.label).join(', ')}`;
}

function writeResults() {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
