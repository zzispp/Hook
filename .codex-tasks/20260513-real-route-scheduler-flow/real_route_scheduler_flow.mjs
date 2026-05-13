import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { ensureBackend } from '../20260512-real-proxy-cache-flow/lib/backend.mjs';
import { assert, assertEqual } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import {
  ids,
  providerNames,
  seedRouteDatabase,
  setBrokenProviderActive,
  setOpenAIChatBaseUrl,
  setOpenAIKeyPriorities,
  setClaudePrimaryKey,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
} from './lib/route_fixtures.mjs';
import {
  proxyCall,
  successRow,
  tracesSince,
  geminiRequest,
  assertNoAvailableRows,
  openAiChatRequest,
  assertStreamSuccess,
  claudeMessagesRequest,
  openAiResponsesRequest,
  assertSingleSuccessAttempt,
} from './lib/route_client.mjs';

const ctx = loadContext();
const db = new Db(ctx.db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const results = [];
const originalMode = db.scalar("select scheduling_mode from system_settings where id = 'global'") || 'fixed_order';

async function main() {
  const modelIds = seedRouteDatabase(ctx, db);
  await resetAll(modelIds);
  const server = await ensureBackend(ctx.serverBaseUrl);
  try {
    await runScenarios(modelIds);
    assert(results.every((item) => item.ok), failedSummary());
    console.log('real route scheduler flow: all scenarios passed');
  } finally {
    await cleanup(modelIds, server);
    writeResults();
  }
}

async function runScenarios(modelIds) {
  await step('fixed order exact endpoints', () => fixedOrderExact(modelIds));
  await step('route key failover', () => routeKeyFailover(modelIds));
  await step('route endpoint fallback conversion', () => routeEndpointFallback(modelIds));
  await step('provider failover', () => providerFailover(modelIds));
  await step('cache affinity', () => cacheAffinity(modelIds));
  await step('load balance', () => loadBalance(modelIds));
  await step('format conversion matrix', () => formatConversionMatrix(modelIds));
  await step('high concurrency load balance', () => highConcurrency(modelIds));
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence });
    console.log(`scenario passed: ${label}`);
  } catch (error) {
    results.push({ label, ok: false, error: error.message });
    console.error(`scenario failed: ${label}: ${error.stack || error.message}`);
  }
}

async function fixedOrderExact(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const chat = await proxyCall(ctx, db, 'fixed openai chat', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-chat')));
  assertSingleSuccessAttempt(chat, providerNames.openai, 'Route Hook primary');
  const stream = await proxyCall(ctx, db, 'fixed openai stream', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-stream'), true));
  assertSingleSuccessAttempt(stream, providerNames.openai, 'Route Hook primary');
  assertStreamSuccess(stream, false);
  const responses = await proxyCall(ctx, db, 'fixed openai responses', openAiResponsesRequest(ctx, ctx.models.openai, marker('fixed-responses')));
  assertSingleSuccessAttempt(responses, providerNames.openai, 'Route Hook primary');
  const compact = await proxyCall(ctx, db, 'fixed openai compact', openAiResponsesRequest(ctx, ctx.models.openai, marker('fixed-compact'), true));
  assertSingleSuccessAttempt(compact, providerNames.openai, 'Route Hook primary');
  return requestIds(chat, stream, responses, compact);
}

async function routeKeyFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setClaudePrimaryKey(db, ctx, 'sk-route-real-invalid');
  await clearScheduling();
  const result = await proxyCall(ctx, db, 'route key failover', openAiChatRequest(ctx, ctx.models.claude, marker('key-failover')));
  const success = successRow(result.trace);
  assertEqual(result.trace[0].key_name, 'Route Claude primary', 'first attempt should use primary key');
  assertEqual(result.trace[0].status, 'failed', 'invalid primary key should fail visibly');
  assertEqual(success.key_name, 'Route Claude secondary', 'second route key should succeed');
  assertEqual(success.retry_index, '1', 'secondary key should be retry index 1');
  assertEqual(success.needs_conversion, 'true', 'key failover request should still convert OpenAI to Claude');
  assertNoAvailableRows(result.trace);
  setClaudePrimaryKey(db, ctx, ctx.secrets.claudeKey);
  return { requestId: result.requestId, attempts: result.trace.length };
}

async function routeEndpointFallback(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setOpenAIChatBaseUrl(db, 'http://127.0.0.1:9');
  await clearScheduling();
  const result = await proxyCall(ctx, db, 'route endpoint fallback', openAiChatRequest(ctx, ctx.models.openai, marker('endpoint-fallback')));
  const success = successRow(result.trace);
  assert(Number(success.retry_index) >= 2, 'converted endpoint should be reached after exact endpoint key attempts');
  assertEqual(success.needs_conversion, 'true', 'endpoint fallback should convert OpenAI chat to Responses');
  assert(['openai_cli', 'openai_compact'].includes(success.provider_api_format), 'fallback should use an OpenAI Responses endpoint');
  assert(result.trace.length <= 6, 'route fallback should not exceed endpoint x key real attempts');
  assertNoAvailableRows(result.trace);
  setOpenAIChatBaseUrl(db, ctx.upstreams.openaiBaseUrl);
  return { requestId: result.requestId, attempts: result.trace.length, endpoint: success.provider_api_format };
}

async function providerFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setBrokenProviderActive(db, true);
  await clearScheduling();
  const result = await proxyCall(ctx, db, 'provider failover', openAiChatRequest(ctx, ctx.models.openai, marker('provider-failover')));
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
  await redis.setex(affinityKey(modelIds.openai, 'openai_chat'), 300, ids.keyOpenAISecondary);
  await clearScheduling();
  const result = await proxyCall(ctx, db, 'cache affinity', openAiChatRequest(ctx, ctx.models.openai, marker('affinity')));
  assertEqual(successRow(result.trace).key_name, 'Route Hook secondary', 'affinity key should be attempted first');
  return { requestId: result.requestId, key: successRow(result.trace).key_name };
}

async function loadBalance(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearScheduling();
  const keys = new Set();
  for (let index = 0; index < 18; index += 1) {
    const result = await proxyCall(ctx, db, `load balance ${index}`, openAiChatRequest(ctx, ctx.models.openai, marker(`lb-${index}`)));
    keys.add(successRow(result.trace).key_name);
    assertSingleSuccessAttempt(result, providerNames.openai);
  }
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'load balance should use both OpenAI keys');
  return { keys: [...keys].sort() };
}

async function formatConversionMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const cases = [
    ['openai to claude', openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude')), providerNames.claude, true],
    ['openai stream to claude', openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude-stream'), true), providerNames.claude, true],
    ['openai to gemini', openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini')), providerNames.gemini, true],
    ['openai stream to gemini', openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini-stream'), true), providerNames.gemini, true],
    ['claude to openai', claudeMessagesRequest(ctx.models.openai, marker('claude-openai')), providerNames.openai, true],
    ['gemini exact', geminiRequest(ctx.models.gemini, marker('gemini-exact')), providerNames.gemini, false],
    ['gemini stream exact', geminiRequest(ctx.models.gemini, marker('gemini-stream'), true), providerNames.gemini, false],
  ];
  const evidence = [];
  for (const item of cases) evidence.push(await runConversionCase(...item));
  return evidence;
}

async function runConversionCase(label, request, providerName, converts) {
  const result = await proxyCall(ctx, db, label, request);
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
  const markerRoot = marker(`concurrency-${Date.now()}`);
  const before = new Date(Date.now() - 1000).toISOString();
  const requests = concurrencyRequests(markerRoot);
  const results = await Promise.all(requests.map((request) => proxyCall(ctx, db, request.matchText, request)));
  assert(results.every((item) => item.ok), 'all high concurrency requests should succeed');
  const traces = tracesSince(db, before, markerRoot);
  assertEqual(traces.length, requests.length, 'high concurrency should record every request');
  const successes = traces.map(({ trace }) => successRow(trace));
  const keys = new Set(successes.map((row) => row.key_name));
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'concurrency load balance should use both keys');
  assertEqual(traces.flatMap(({ trace }) => trace.filter((row) => row.status === 'failed')).length, 0, 'concurrency should not fail attempts');
  return { requests: requests.length, keys: [...keys].sort() };
}

function concurrencyRequests(markerRoot) {
  const requests = [];
  for (let index = 0; index < ctx.stress.nonStream; index += 1) {
    requests.push(openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|nonstream|${String(index).padStart(3, '0')}|`));
  }
  for (let index = 0; index < ctx.stress.stream; index += 1) {
    requests.push(openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|stream|${String(index).padStart(3, '0')}|`, true));
  }
  return requests;
}

async function directSchedulingChange(modelIds, mode) {
  setSchedulingModeDb(db, mode);
  await clearScheduling();
  await clearAffinity(modelIds);
}

async function resetAll(modelIds) {
  restoreRouteFixtures(ctx, db, originalMode);
  await clearScheduling();
  await clearAuth();
  await clearAffinity(modelIds);
}

async function cleanup(modelIds, server) {
  try {
    restoreRouteFixtures(ctx, db, originalMode);
    deactivateRouteFixtures(db);
    await clearScheduling();
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
    affinityKey(modelIds.openai, 'openai_chat'),
    affinityKey(modelIds.openai, 'openai_cli'),
    affinityKey(modelIds.claude, 'openai_chat'),
    affinityKey(modelIds.gemini, 'openai_chat'),
    affinityKey(modelIds.gemini, 'gemini_chat'),
  );
}

function affinityKey(modelId, format) {
  return `${ctx.redis.prefix}:llm_proxy:affinity:${ids.token}:${modelId}:${format}`;
}

function schedulingSnapshotKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:snapshot:v2`;
}

function schedulingLockKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:rebuild_lock`;
}

function marker(value) {
  return `route-real-${value}`;
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
