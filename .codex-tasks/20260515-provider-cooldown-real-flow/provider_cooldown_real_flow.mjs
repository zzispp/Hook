import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert, assertEqual } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { startBackend, stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { createAdminToken, deleteAdminToken, signInAdmin } from '../20260515-real-usage-concurrency-flow/lib/real_usage_admin.mjs';
import {
  applyProviderCooldownSettings,
  cleanupProviderCooldownFixtures,
  ensureAdminProviderCooldownApis,
  fixtureIds,
  prepareProviderCooldownSchema,
  restoreSystemSettings,
  seedProviderCooldownFixtures,
  systemSettingsSnapshot,
  testTokenPayload,
  uniqueRunId,
} from './lib/provider_cooldown_fixtures.mjs';
import { loadProviderCooldownContext } from './lib/provider_cooldown_env.mjs';
import { resolveProviderCooldownUpstreams } from './lib/provider_cooldown_upstreams.mjs';
import { clearProviderCooldownRedis, providerCooldownKey, providerFailureKey } from './lib/provider_cooldown_redis.mjs';
import { adminJson, proxyCall } from './lib/provider_cooldown_api.mjs';
import { activeCooldownRow } from './lib/provider_cooldown_queries.mjs';
import { failedSummary, redactEvidence, redactText, writeResults } from './lib/provider_cooldown_report.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const results = [];
const createdTokenIds = [];
let ctx = null;
let db = null;
let redis = null;
let backend = null;
let adminToken = '';
let originalSettings = null;
let customerToken = null;

async function main() {
  await step('load context and prepare schema', () => {
    ctx = loadProviderCooldownContext();
    db = new Db(ctx.db);
    redis = new RedisClient(ctx.redis);
    prepareProviderCooldownSchema(db);
    ensureAdminProviderCooldownApis(db);
    originalSettings = systemSettingsSnapshot(db);
    return publicContext();
  });

  const upstream = await step('fetch real upstream model names', () => resolveProviderCooldownUpstreams(ctx));
  await step('seed local DB fixtures', () => {
    cleanupProviderCooldownFixtures(db);
    seedProviderCooldownFixtures(ctx, db, upstream);
    applyProviderCooldownSettings(db, ctx);
    return fixtureSummary(upstream);
  });

  await clearProviderCooldownRedis(redis, ctx.redis.prefix, []);
  backend = await startBackend(ctx.serverBaseUrl);
  adminToken = await signInAdmin(ctx);

  try {
    await runScenarios();
    assert(results.every((item) => item.ok), failedSummary(results));
    console.log('provider cooldown real flow: all scenarios passed');
  } finally {
    await cleanup();
    writeResults(taskDir, results);
  }
}

async function runScenarios() {
  await step('create customer token through admin API', async () => {
    const created = await createAdminToken(ctx, adminToken, testTokenPayload());
    customerToken = created;
    createdTokenIds.push(created.token.id);
    return {
      id: created.token.id,
      prefix: created.token.token_prefix,
      groupCode: created.token.group_code,
      tokenType: created.token.token_type,
    };
  });

  const first = await step('trigger 404 cooldown on first provider', () => triggerCooldown());
  await step('verify DB and Redis cooldown state', () => verifyCooldownState(first));
  const second = await step('cooled provider is skipped and second provider succeeds', () => verifySchedulingSkip());
  await step('manual release clears DB list and Redis key', () => releaseCooldown(second));
}

async function triggerCooldown() {
  const result = await callProxy(`cooldown-trigger-${uniqueRunId()}`, false);
  const failed = result.trace.find((row) => row.provider_id === fixtureIds.providerMsutools && row.status === 'failed');
  assert(failed, 'first request should record failed msutools candidate');
  assertEqual(failed.status_code, String(ctx.cooldown.statusCode), 'failed candidate status should match cooldown rule');
  return result;
}

async function verifyCooldownState(first) {
  const cooldown = activeCooldownRow(db, fixtureIds.providerMsutools);
  assert(cooldown, 'active cooldown row should exist for msutools provider');
  assertEqual(cooldown.status_code, String(ctx.cooldown.statusCode), 'cooldown DB status should match');
  assertEqual(cooldown.request_id, first.requestId, 'cooldown DB should reference triggering request');
  assertEqual(cooldown.error_type, 'upstream_status', 'cooldown DB should preserve error type');
  const redisValue = await redis.get(providerCooldownKey(ctx.redis.prefix, fixtureIds.providerMsutools));
  assertEqual(redisValue, '1', 'Redis cooldown key should exist');
  const failureCount = await redis.command('ZCARD', providerFailureKey(ctx.redis.prefix, fixtureIds.providerMsutools, ctx.cooldown.statusCode));
  assert(Number(failureCount) >= 1, 'Redis fixed-window failure zset should contain event');
  const apiList = await callAdmin('GET', '/api/admin/provider-cooldowns');
  assert(apiList.cooldowns.some((item) => item.provider_id === fixtureIds.providerMsutools), 'cooldown list API should include msutools');
  return { cooldown, redisValue, failureCount, apiTotal: apiList.total };
}

async function verifySchedulingSkip() {
  const result = await callProxy(`cooldown-skip-${uniqueRunId()}`, true);
  assert(result.ok, `second request should succeed through Ekan8, got ${result.status}: ${JSON.stringify(result.body)}`);
  assert(!result.trace.some((row) => row.provider_id === fixtureIds.providerMsutools), 'second request should not schedule cooled msutools provider');
  const success = result.trace.find((row) => row.provider_id === fixtureIds.providerEkan8 && row.status === 'success');
  assert(success, 'second request should succeed on Ekan8 provider');
  return result;
}

async function releaseCooldown(second) {
  const released = await callAdmin('POST', `/api/admin/provider-cooldowns/${fixtureIds.providerMsutools}/release`);
  assertEqual(released.provider_id, fixtureIds.providerMsutools, 'release API should return msutools cooldown');
  const active = activeCooldownRow(db, fixtureIds.providerMsutools);
  assert(!active, 'released cooldown should disappear from active DB query');
  const redisValue = await redis.get(providerCooldownKey(ctx.redis.prefix, fixtureIds.providerMsutools));
  assertEqual(redisValue, null, 'release should delete Redis cooldown key');
  const apiList = await callAdmin('GET', '/api/admin/provider-cooldowns');
  assert(!apiList.cooldowns.some((item) => item.provider_id === fixtureIds.providerMsutools), 'cooldown list API should omit released provider');
  return {
    releasedProviderId: released.provider_id,
    secondRequestId: second.requestId,
    remainingTotal: apiList.total,
  };
}

function callProxy(marker, expectOk) {
  return proxyCall({
    ctx,
    db,
    marker,
    expectOk,
    customerToken,
    modelName: ctx.cooldown.model,
    modelId: fixtureIds.model,
    timeoutMs: ctx.cooldown.timeoutMs,
  });
}

function callAdmin(method, path, payload) {
  return adminJson(ctx, adminToken, method, path, payload);
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence: redactEvidence(evidence, ctx) });
    console.log(`scenario passed: ${label}`);
    return evidence;
  } catch (error) {
    const message = redactText(error.stack || error.message, ctx);
    results.push({ label, ok: false, error: message });
    console.error(`scenario failed: ${label}: ${message}`);
    throw error;
  }
}

async function cleanup() {
  try {
    for (const tokenId of createdTokenIds) {
      if (adminToken) {
        await deleteAdminToken(ctx, adminToken, tokenId).catch((error) => {
          results.push({ label: `cleanup token ${tokenId}`, ok: false, error: redactText(error.stack || error.message, ctx) });
        });
      }
    }
    if (db) {
      cleanupProviderCooldownFixtures(db, createdTokenIds);
      restoreSystemSettings(db, originalSettings);
    }
    if (redis && ctx) {
      await clearProviderCooldownRedis(redis, ctx.redis.prefix, createdTokenIds);
    }
  } finally {
    if (ctx) {
      await stopBackend(backend, ctx.serverBaseUrl);
    }
  }
}

function publicContext() {
  return {
    serverBaseUrl: ctx.serverBaseUrl,
    db: { host: ctx.db.host, port: ctx.db.port, name: ctx.db.name },
    redis: { host: ctx.redis.host, port: ctx.redis.port, prefix: ctx.redis.prefix },
    cooldown: ctx.cooldown,
  };
}

function fixtureSummary(upstream) {
  return {
    providers: ['Cooldown Real Msutools', 'Cooldown Real Ekan8'],
    model: ctx.cooldown.model,
    upstream,
    policy: {
      statusCode: ctx.cooldown.statusCode,
      thresholdCount: ctx.cooldown.thresholdCount,
      windowSeconds: ctx.cooldown.windowSeconds,
      cooldownSeconds: ctx.cooldown.cooldownSeconds,
    },
  };
}

main().catch((error) => {
  console.error(redactText(error.stack || error.message, ctx));
  process.exit(1);
});
