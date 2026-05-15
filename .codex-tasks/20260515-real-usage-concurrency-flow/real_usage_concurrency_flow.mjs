import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { startBackend, stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { createAdminToken, deleteAdminToken, signInAdmin } from './lib/real_usage_admin.mjs';
import {
  assertMultiAccountAndKeyCoverage,
  assertRequestRecordSummary,
  openAiChatRequest,
  runConcurrentProxyCalls,
  waitForTerminalRecords,
} from './lib/real_usage_client.mjs';
import { cleanupRealUsageFixtures } from './lib/real_usage_cleanup.mjs';
import { loadRealUsageContext } from './lib/real_usage_env.mjs';
import {
  assertFixtureModel,
  fixtureIds,
  fixtureUserIds,
  groupCode,
  seedRealUsageFixtures,
} from './lib/real_usage_fixtures.mjs';
import { clearSchedulingRedis, clearUsageRedis } from './lib/real_usage_keys.mjs';
import { runUsageRecoveryScenarios } from './lib/real_usage_recovery.mjs';
import { applyRealUsageSettings, restoreSystemSettings, systemSettingsSnapshot } from './lib/real_usage_settings.mjs';
import { resolveUpstreamModels } from './lib/real_usage_upstreams.mjs';
import {
  assertCandidateCoverage,
  assertWalletBilling,
  requestCandidateAggregate,
  usageRedisState,
  waitForUsageFlush,
} from './lib/real_usage_verify.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const results = [];
const createdTokenIds = [];
let ctx = null;
let db = null;
let redis = null;
let backend = null;
let adminToken = '';
let originalSettings = null;

async function main() {
  await step('load runtime context', () => {
    ctx = loadRealUsageContext();
    db = new Db(ctx.db);
    redis = new RedisClient(ctx.redis);
    originalSettings = systemSettingsSnapshot(db);
    return {
      serverBaseUrl: ctx.serverBaseUrl,
      db: { host: ctx.db.host, port: ctx.db.port, name: ctx.db.name },
      redis: { host: ctx.redis.host, port: ctx.redis.port, prefix: ctx.redis.prefix },
      requestCount: ctx.realLoad.requestCount,
    };
  });
  const upstream = await step('fetch real upstream models', () => resolveUpstreamModels(ctx));
  cleanupRealUsageFixtures(db, []);
  await step('seed local DB fixtures', () => {
    seedRealUsageFixtures(ctx, db, upstream);
    applyRealUsageSettings(db);
    assertFixtureModel(db, ctx.realModels.chat);
    return { model: ctx.realModels.chat, groupCode, providers: ['hook', 'ekan8'] };
  });
  await clearUsageRedis(redis, ctx.redis.prefix);
  await clearSchedulingRedis(redis, ctx.redis.prefix);
  backend = await startBackend(ctx.serverBaseUrl);
  adminToken = await signInAdmin(ctx);
  try {
    await runScenarios();
    assert(results.every((item) => item.ok), failedSummary());
    console.log('real usage concurrency flow: all scenarios passed');
  } finally {
    await cleanup();
    writeResults();
  }
}

async function runScenarios() {
  const tokens = await step('create customer tokens through admin API', () => createCustomerTokens());
  const load = await step('high-concurrency real proxy traffic', () => runHighConcurrency(tokens));
  await step('usage flush and billing verification', () => verifyUsageAndBilling(load, tokens));
  await step('usage processing recovery after restart', () =>
    runUsageRecoveryScenarios({
      db,
      redis,
      ctx,
      tokenIds: tokens.map((token) => token.id),
      modelId: fixtureIds.model,
      stopBackend: stopHarnessBackend,
      startBackend: startHarnessBackend,
    }),
  );
}

async function stopHarnessBackend() {
  await stopBackend(backend, ctx.serverBaseUrl);
  backend = null;
}

async function startHarnessBackend() {
  backend = await startBackend(ctx.serverBaseUrl);
  adminToken = await signInAdmin(ctx);
}

async function createCustomerTokens() {
  const created = [];
  for (const [index, userId] of fixtureUserIds.entries()) {
    const data = await createAdminToken(ctx, adminToken, {
      name: `Real usage customer token ${index + 1}`,
      token_type: 'user',
      user_id: userId,
      group_code: groupCode,
      model_access_mode: 'all',
      allowed_model_ids: [],
      rate_limit_rpm: 0,
    });
    createdTokenIds.push(data.token.id);
    created.push({
      id: data.token.id,
      userId,
      value: data.raw_token,
      prefix: data.token.token_prefix,
    });
  }
  await clearUsageRedis(
    redis,
    ctx.redis.prefix,
    created.map((token) => token.id),
  );
  return created;
}

async function runHighConcurrency(tokens) {
  const requests = Array.from({ length: ctx.realLoad.requestCount }, (_, index) => {
    const token = tokens[index % tokens.length];
    const marker = `real-usage-concurrency-${index}-${Date.now()}`;
    return {
      token: token.value,
      request: openAiChatRequest(ctx.realModels.chat, marker, false),
    };
  });
  const results = await runConcurrentProxyCalls(ctx, db, requests, ctx.realLoad.requestTimeoutMs);
  await waitForTerminalRecords(
    db,
    results.map((item) => item.requestId),
  );
  assertMultiAccountAndKeyCoverage(results, tokens.length, 4);
  const requestIds = results.map((item) => item.requestId);
  const recordSummary = assertRequestRecordSummary(db, requestIds);
  const candidateAggregate = requestCandidateAggregate(db, requestIds);
  assertCandidateCoverage(candidateAggregate);
  return {
    requestIds,
    recordSummary,
    candidateAggregate,
    sample: results.slice(0, 5).map((item) => ({
      requestId: item.requestId,
      status: item.status,
      trace: item.trace.map((row) => ({
        providerName: row.providerName,
        keyName: row.keyName,
        providerApiFormat: row.providerApiFormat,
        status: row.status,
        totalCost: row.totalCost,
        totalTokens: row.totalTokens,
      })),
    })),
  };
}

async function verifyUsageAndBilling(load, tokens) {
  const tokenIds = tokens.map((token) => token.id);
  const usage = await waitForUsageFlush(db, redis, ctx.redis.prefix, tokenIds, load.requestIds.length, fixtureIds.model);
  const wallets = assertWalletBilling(db, fixtureUserIds, load.requestIds.length);
  const redisState = await usageRedisState(redis, ctx.redis.prefix);
  return { usage, wallets, redisState };
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence: redactEvidence(evidence) });
    console.log(`scenario passed: ${label}`);
    return evidence;
  } catch (error) {
    const message = redactText(error.stack || error.message);
    results.push({ label, ok: false, error: message });
    console.error(`scenario failed: ${label}: ${message}`);
    throw error;
  }
}

async function cleanup() {
  for (const tokenId of createdTokenIds) {
    try {
      if (adminToken) {
        await deleteAdminToken(ctx, adminToken, tokenId);
      }
    } catch (error) {
      results.push({ label: `cleanup token ${tokenId}`, ok: false, error: redactText(error.stack || error.message) });
    }
  }
  try {
    if (db) {
      cleanupRealUsageFixtures(db, createdTokenIds);
      restoreSystemSettings(db, originalSettings);
    }
    if (redis && ctx) {
      await clearUsageRedis(redis, ctx.redis.prefix, createdTokenIds);
      await clearSchedulingRedis(redis, ctx.redis.prefix);
    }
  } finally {
    if (ctx) {
      await stopBackend(backend, ctx.serverBaseUrl);
    }
  }
}

function redactEvidence(value) {
  return JSON.parse(
    JSON.stringify(value, (key, item) => {
      if (key.toLowerCase().includes('key') && typeof item === 'string') {
        return item.startsWith('Real Usage') ? item : '[redacted]';
      }
      if (key === 'value' || key === 'raw_token') {
        return '[redacted]';
      }
      if (typeof item === 'string' && secretValues().includes(item)) {
        return redactText(item);
      }
      return item;
    }),
  );
}

function redactText(value) {
  return secretValues().reduce((text, secret) => text.replaceAll(secret, '[redacted]'), String(value));
}

function secretValues() {
  if (!ctx?.realSecrets) {
    return [];
  }
  return [ctx.realSecrets.provider1Key, ctx.realSecrets.provider2Key, ...ctx.realSecrets.provider1Keys, ...ctx.realSecrets.provider2Keys].filter(Boolean);
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
  writeResults();
  process.exit(1);
});
