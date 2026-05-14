import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import {
  billingMultiplierAndTokenUsage,
  disabledTokenRejected,
  disabledUserRejected,
  tokenQuotaRejected,
  walletLedgerCharged,
  walletQuotaRejected,
} from './lib/billing_access_access_scenarios.mjs';
import {
  applyRecordingSettings,
  ensureRequestSnapshotColumns,
  restoreSystemSettings,
  systemSettingsSnapshot,
} from './lib/billing_access_db_control.mjs';
import { loadBillingAccessContext } from './lib/billing_access_env.mjs';
import {
  clearFixtureRequestRows,
  deactivateBillingAccessFixtures,
  modelNames,
  seedBillingAccessFixtures,
} from './lib/billing_access_fixtures.mjs';
import { makeTokenValues } from './lib/billing_access_ids.mjs';
import { modelProbeEvidence, probeRuntimeModels } from './lib/billing_access_model_probe.mjs';
import {
  cacheAffinityColdStartRandomness,
  cacheAffinityWarmHit,
  ekan8MappedRequest,
  loadBalanceEqualPriority,
  providerTimeoutFailover,
  retryAndProviderFailover,
} from './lib/billing_access_routing_scenarios.mjs';
import { clearProxyCaches } from './lib/billing_access_client.mjs';
import { ensureBackendForTest, startSlowUpstream } from './lib/billing_access_runtime.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const results = [];
const ctx = loadBillingAccessContext();
const db = new Db(ctx.db);
const redis = new RedisClient(ctx.redis);
const tokenValues = makeTokenValues();
const settings = systemSettingsSnapshot(db);
let backend = null;
let slowUpstream = null;

async function main() {
  try {
    const runtime = await requiredStep('upstream model probe', () => probeRuntimeModels(ctx));
    slowUpstream = await startSlowUpstream();
    const fixtureRuntime = { ...runtime, slowBaseUrl: slowUpstream.baseUrl };
    ensureRequestSnapshotColumns(db);
    seedBillingAccessFixtures(ctx, db, tokenValues, fixtureRuntime);
    applyRecordingSettings(db);
    clearFixtureRequestRows(db);
    await clearProxyCaches(redis, ctx.redis.prefix);
    backend = await ensureBackendForTest(ctx.serverBaseUrl);
    await runScenarios(runtime);
    failIfNeeded();
  } finally {
    await cleanup();
    writeResults();
  }
}

async function runScenarios(runtime) {
  await step('upstream model evidence', () => modelProbeEvidence(runtime));
  const state = scenarioState();
  await step('disabled token is rejected', () => disabledTokenRejected(state));
  await step('disabled user is rejected', () => disabledUserRejected(state));
  await step('token quota exhausted is rejected like New API', () => tokenQuotaRejected(state));
  await step('wallet quota exhausted is rejected like New API', () => walletQuotaRejected(state));
  await step('billing multiplier and token used_quota are settled', () => billingMultiplierAndTokenUsage(state));
  await step('wallet balance and consume ledger are settled', () => walletLedgerCharged(state));
  await step('provider retry and failover are effective', () => retryAndProviderFailover(state));
  await step('provider timeout and failover are effective', () => providerTimeoutFailover(state));
  await step('same-priority load_balance distributes providers', () => loadBalanceEqualPriority(state));
  await step('cache_affinity warm request reuses successful key', () => cacheAffinityWarmHit(state));
  await step('cache_affinity cold start randomizes equal priority', () => cacheAffinityColdStartRandomness(state));
  await step('Ekan8 mapped Gemini request succeeds', () => ekan8MappedRequest(state));
}

function scenarioState() {
  return {
    ctx,
    db,
    redis,
    tokenValues,
    models: modelNames(ctx),
    artifacts: {},
    marker: (value) => `billing-access-real-${Date.now()}-${value}`,
    clearCaches: () => clearProxyCaches(redis, ctx.redis.prefix),
  };
}

async function requiredStep(label, action) {
  const evidence = await step(label, action);
  if (evidence === undefined) throw new Error(`required step failed: ${label}`);
  return evidence;
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence });
    console.log(`scenario passed: ${label}`);
    return evidence;
  } catch (error) {
    const message = error.stack || error.message;
    results.push({ label, ok: false, error: message });
    console.error(`scenario failed: ${label}: ${message}`);
    return undefined;
  }
}

async function cleanup() {
  try {
    deactivateBillingAccessFixtures(db);
    restoreSystemSettings(db, settings);
    await clearProxyCaches(redis, ctx.redis.prefix);
  } finally {
    if (backend) await backend.stop();
    if (slowUpstream) await slowUpstream.stop();
  }
}

function failIfNeeded() {
  const failed = results.filter((item) => !item.ok);
  if (failed.length) throw new Error(`failed scenarios: ${failed.map((item) => item.label).join(', ')}`);
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
