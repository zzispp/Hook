import { Db } from './lib/db.mjs';
import { RedisClient } from './lib/redis.mjs';
import { loadContext } from './lib/env.mjs';
import { ensureBackend } from './lib/backend.mjs';
import { signIn, patchSchedulingMode, createTransientAdminToken, deleteAdminToken } from './lib/api.mjs';
import { assert, assertEqual, assertIncludes } from './lib/assertions.mjs';
import {
  ids,
  providerNames,
  seedDatabase,
  setBrokenProviderActive,
  setOpenAIKeyPriorities,
  setSchedulingModeDb,
} from './lib/fixtures.mjs';
import {
  affinityKey,
  authTokenKey,
  authVersion,
  clearAffinity,
  clearAuthCache,
  clearSchedulingSnapshot,
  schedulingSnapshot,
} from './lib/cache_keys.mjs';
import { proxyCall, requestForOpenAiGpt, successRow } from './lib/proxy_client.mjs';
import { runFormatConversion as runFormatConversionFlow } from './lib/conversion_flow.mjs';
import { runConcurrentRebuildStress as runConcurrentRebuildStressFlow } from './lib/stress_flow.mjs';

const ctx = loadContext();
const db = new Db(ctx.db);
const redis = new RedisClient(ctx.redis);
const originalSchedulingMode = db.scalar("select scheduling_mode from system_settings where id = 'global'") || 'fixed_order';

async function main() {
  const modelIds = seedDatabase(ctx, db);
  await clearProxyCaches(modelIds);
  const server = await ensureBackend(ctx.serverBaseUrl);
  try {
    const adminToken = await signIn(ctx);
    const failures = [];
    await runStep(failures, modelIds, 'settings cache hook', () => runSchedulingHook(adminToken));
    await runStep(failures, modelIds, 'fixed order', () => runFixedOrder(modelIds));
    await runStep(failures, modelIds, 'failover', () => runFailover(modelIds));
    await runStep(failures, modelIds, 'cache affinity', () => runCacheAffinity(modelIds));
    await runStep(failures, modelIds, 'load balance', () => runLoadBalance(modelIds));
    await runStep(failures, modelIds, 'format conversion', () => runFormatConversion(modelIds));
    await runStep(failures, modelIds, 'auth cache hook', () => runAuthHook(adminToken));
    await runStep(failures, modelIds, 'concurrent rebuild stress', () => runConcurrentRebuildStress(adminToken, modelIds));
    if (failures.length > 0) {
      throw new Error(`real proxy cache flow completed with ${failures.length} failed scenario(s): ${failures.map((item) => item.label).join(', ')}`);
    }
    console.log('real proxy cache flow: all scenarios passed');
  } finally {
    await cleanup(modelIds, server);
  }
}

async function runStep(failures, modelIds, label, action) {
  console.log(`scenario: ${label}`);
  try {
    await action();
    console.log(`scenario passed: ${label}`);
  } catch (error) {
    failures.push({ label, error });
    console.error(`scenario failed: ${label}: ${error.message}`);
  } finally {
    await resetScenario(modelIds);
  }
}

async function runSchedulingHook(adminToken) {
  await patchSchedulingMode(ctx, adminToken, 'cache_affinity');
  const snapshot = await schedulingSnapshot(ctx, redis);
  assertEqual(snapshot?.scheduling_mode, 'cache_affinity', 'settings API should refresh scheduling snapshot');
  await patchSchedulingMode(ctx, adminToken, originalSchedulingMode);
}

async function runFixedOrder(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setBrokenProviderActive(db, false);
  setOpenAIKeyPriorities(db, 0, 1);
  await clearSchedulingSnapshot(ctx, redis);
  const result = await proxyCall(ctx, db, 'fixed order openai exact', requestForOpenAiGpt(ctx, 'hook-fixed'));
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerNames.openai, 'fixed order should use OpenAI provider');
  assertEqual(success.key_name, 'Hook primary', 'fixed order should use primary key first');
  assertEqual(success.needs_conversion, 'false', 'OpenAI exact request should not convert');
  assertExactBeforeConverted(result.trace);
  await assertFirstCallCaches(modelIds);
  const stream = await proxyCall(ctx, db, 'fixed order openai stream exact', requestForOpenAiGpt(ctx, 'hook-stream', true));
  assertStreamSuccess(stream.trace, false);
  assertIncludes(stream.text, 'data:', 'OpenAI stream should return SSE');
}

async function runFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setBrokenProviderActive(db, true);
  await clearSchedulingSnapshot(ctx, redis);
  const result = await proxyCall(ctx, db, 'retry and failover', requestForOpenAiGpt(ctx, 'hook-failover'));
  const brokenFailures = result.trace.filter((row) => row.provider_name === providerNames.broken && row.status === 'failed');
  assert(brokenFailures.length >= 1, 'failover should record failed broken provider attempts');
  assertEqual(successRow(result.trace).provider_name, providerNames.openai, 'failover should continue to OpenAI provider');
  setBrokenProviderActive(db, false);
  await clearSchedulingSnapshot(ctx, redis);
}

async function runCacheAffinity(modelIds) {
  await directSchedulingChange(modelIds, 'cache_affinity');
  setOpenAIKeyPriorities(db, 0, 1);
  await redis.setex(affinityKey(ctx, modelIds.gpt, 'openai_chat'), 300, ids.keyOpenAISecondary);
  await clearSchedulingSnapshot(ctx, redis);
  const result = await proxyCall(ctx, db, 'cache affinity', requestForOpenAiGpt(ctx, 'hook-affinity'));
  assertEqual(successRow(result.trace).key_name, 'Hook secondary', 'cache affinity should promote cached key');
  await clearAffinity(ctx, redis, modelIds);
}

async function runLoadBalance(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearSchedulingSnapshot(ctx, redis);
  const keys = new Set();
  for (let index = 0; index < 12; index += 1) {
    const result = await proxyCall(ctx, db, `load balance ${index + 1}`, requestForOpenAiGpt(ctx, `hook-lb-${index}`));
    const success = successRow(result.trace);
    keys.add(success.key_name);
    assertEqual(success.provider_api_format, 'openai_chat', 'load balance should keep exact endpoint first');
  }
  assert(keys.has('Hook primary') && keys.has('Hook secondary'), 'load balance should distribute over both OpenAI keys');
  setOpenAIKeyPriorities(db, 0, 1);
}

async function runFormatConversion(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  await clearAffinity(ctx, redis, modelIds);
  await runFormatConversionFlow({ ctx, db, assertStreamSuccess });
}

async function runAuthHook(adminToken) {
  const before = await authVersion(ctx, redis);
  const tokenId = await createTransientAdminToken(ctx, adminToken);
  assertEqual(await authVersion(ctx, redis), before + 1, 'admin token create should bump auth cache version');
  await deleteAdminToken(ctx, adminToken, tokenId);
  assertEqual(await authVersion(ctx, redis), before + 2, 'admin token delete should bump auth cache version');
}

async function runConcurrentRebuildStress(adminToken, modelIds) {
  await runConcurrentRebuildStressFlow({ ctx, db, redis, adminToken, modelIds });
}

async function assertFirstCallCaches(modelIds) {
  const snapshot = await schedulingSnapshot(ctx, redis);
  assert(snapshot, 'first proxy request should rebuild scheduling snapshot');
  assert(snapshot.providers.some((item) => item.id === ids.providerOpenAI), 'snapshot should contain OpenAI provider');
  assert(snapshot.providers.some((item) => item.keys?.some((key) => key.id === ids.keyOpenAIPrimary)), 'snapshot should contain provider keys');
  assert(snapshot.models.some((item) => item.id === modelIds.gpt), 'snapshot should contain global models');
  const version = await authVersion(ctx, redis);
  const cachedToken = await redis.get(authTokenKey(ctx, version));
  assert(cachedToken && JSON.parse(cachedToken).id === ids.token, 'first proxy request should cache API token');
}

function assertExactBeforeConverted(trace) {
  const exact = trace.find((row) => row.needs_conversion === 'false');
  const converted = trace.find((row) => row.needs_conversion === 'true');
  assert(exact && converted, 'candidate list should contain exact and converted candidates');
  assert(Number(exact.candidate_index) < Number(converted.candidate_index), 'converted candidates should be after exact candidates');
}

function assertStreamSuccess(trace, shouldConvert) {
  const success = successRow(trace);
  assertEqual(success.is_stream, 'true', 'stream request should be recorded as stream');
  assertEqual(success.needs_conversion, String(shouldConvert), 'stream conversion flag should match expectation');
  assert(success.first_byte_time_ms !== '', 'stream request should record first byte time');
  assert(success.latency_ms !== '', 'stream request should record total latency after completion');
}

async function directSchedulingChange(modelIds, mode) {
  setSchedulingModeDb(db, mode);
  await clearSchedulingSnapshot(ctx, redis);
  await clearAffinity(ctx, redis, modelIds);
}

async function clearProxyCaches(modelIds) {
  await clearSchedulingSnapshot(ctx, redis);
  await clearAuthCache(ctx, redis);
  await clearAffinity(ctx, redis, modelIds);
}

async function cleanup(modelIds, server) {
  try {
    setBrokenProviderActive(db, false);
    setOpenAIKeyPriorities(db, 0, 1);
    setSchedulingModeDb(db, originalSchedulingMode);
    await clearProxyCaches(modelIds);
  } finally {
    if (server) {
      server.kill('SIGTERM');
    }
  }
}

async function resetScenario(modelIds) {
  setBrokenProviderActive(db, false);
  setOpenAIKeyPriorities(db, 0, 1);
  await clearSchedulingSnapshot(ctx, redis);
  await clearAffinity(ctx, redis, modelIds);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
