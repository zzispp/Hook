import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import {
  tokenIds,
  makeTokenValues,
  seedRouteDatabase,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
  seedUserAccessFixtures,
  resetUserAccessFixtures,
  deactivateUserAccessFixtures,
} from '../20260513-user-access-real-flow/lib/user_access_fixtures.mjs';
import { restartBackend, startBackend, stopBackend } from './lib/backend_session.mjs';
import { adminSignIn } from './lib/request_record_real_client.mjs';
import { mappedOpenAiRuntime, upstreamModelFetch } from './lib/request_record_real_mapping_scenarios.mjs';
import {
  cacheAffinity,
  fixedOrderRoutes,
  formatConversionMatrix,
  highConcurrency,
  loadBalance,
  providerFailover,
  routeEndpointFallback,
  routeKeyFailover,
  userAllowMatrix,
  userDenyMatrix,
} from './lib/request_record_real_routing_scenarios.mjs';
import { cancelledStream, compressionRetention, requestRecordVisibility, staleSweep, structuredUpstreamError } from './lib/request_record_real_record_scenarios.mjs';
import {
  applyFullRecordingSettings,
  clearAffinity,
  clearAuth,
  clearScheduling,
  clearTestRequestRows,
  contextWithExistingModels,
  getRequestRecord,
  listRequestRecords,
  restoreSystemSettings,
  systemSettingsSnapshot,
} from './lib/request_record_real_support.mjs';

const baseCtx = loadContext();
const db = new Db(baseCtx.db);
const ctx = contextWithExistingModels(baseCtx, db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const tokenValues = makeTokenValues();
const results = [];
const originalSettings = systemSettingsSnapshot(db);
const artifacts = {};
let backend = null;
let adminToken = '';
const state = {
  ctx,
  db,
  redis,
  tokenValues,
  artifacts,
  marker: (value) => `real-request-record-${value}`,
  adminToken: () => adminToken,
  directSchedulingChange,
  restartBackend: async () => {
    backend = await restartBackend(backend, ctx.serverBaseUrl);
    adminToken = await adminSignIn(ctx);
    return adminToken;
  },
};

async function main() {
  const modelIds = seedRouteDatabase(ctx, db);
  seedUserAccessFixtures(db, tokenValues, modelIds);
  applyFullRecordingSettings(db);
  await resetAll(modelIds);
  clearTestRequestRows(db, tokenIds);
  backend = await startBackend(ctx.serverBaseUrl);
  adminToken = await adminSignIn(ctx);
  try {
    await runScenarios(modelIds);
    assert(results.every((item) => item.ok), failedSummary());
    console.log('real request record flow: all scenarios passed');
  } finally {
    await cleanup(modelIds);
    writeResults();
  }
}

async function runScenarios(modelIds) {
  await step('fixed order exact non-stream and stream', () => fixedOrderRoutes(state, modelIds));
  await step('user provider and model allow matrix', () => userAllowMatrix(state, modelIds));
  await step('user provider and model deny matrix', () => userDenyMatrix(state, modelIds));
  await step('route key failover and skipped candidates', () => routeKeyFailover(state, modelIds));
  await step('route endpoint fallback conversion', () => routeEndpointFallback(state, modelIds));
  await step('provider failover', () => providerFailover(state, modelIds));
  await step('cache affinity', () => cacheAffinity(state, modelIds));
  await step('load balance', () => loadBalance(state, modelIds));
  await step('format conversion matrix', () => formatConversionMatrix(state, modelIds));
  await step('structured upstream error capture', () => structuredUpstreamError(state, modelIds));
  await step('client cancelled stream recording', () => cancelledStream(state, modelIds));
  await step('payload compression retention', () => compressionRetention(state));
  await step('stale pending and streaming sweep', () => staleSweep(state));
  await step('100 concurrent mixed requests', () => highConcurrency(state, modelIds));
  await step('request record list and detail visibility', () => requestRecordVisibility(state, modelIds));
  await step('provider upstream model fetch api', () => upstreamModelFetch(state, modelIds));
  await step('mapped upstream runtime and cache rebuild', () => mappedOpenAiRuntime(state, modelIds));
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

async function directSchedulingChange(modelIds, mode) {
  setSchedulingModeDb(db, mode);
  await clearScheduling(redis, ctx.redis.prefix);
  await clearAuth(redis, ctx.redis.prefix);
  await clearAffinity(redis, ctx.redis.prefix, tokenIds, modelIds);
}

async function resetAll(modelIds) {
  restoreRouteFixtures(ctx, db, originalSettings.scheduling_mode);
  resetUserAccessFixtures(db, tokenValues, modelIds);
  await clearScheduling(redis, ctx.redis.prefix);
  await clearAuth(redis, ctx.redis.prefix);
  await clearAffinity(redis, ctx.redis.prefix, tokenIds, modelIds);
}

async function cleanup(modelIds) {
  try {
    restoreRouteFixtures(ctx, db, originalSettings.scheduling_mode);
    deactivateUserAccessFixtures(db);
    deactivateRouteFixtures(db);
    restoreSystemSettings(db, originalSettings);
    await clearScheduling(redis, ctx.redis.prefix);
    await clearAuth(redis, ctx.redis.prefix);
    await clearAffinity(redis, ctx.redis.prefix, tokenIds, modelIds);
  } finally {
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

function failedSummary() {
  return `failed scenarios: ${results.filter((item) => !item.ok).map((item) => item.label).join(', ')}`;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
