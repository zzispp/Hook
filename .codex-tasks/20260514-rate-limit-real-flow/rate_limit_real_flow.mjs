import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert, assertEqual, assertIncludes } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { Db } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { startBackend, stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { contextWithExistingModels } from '../20260514-real-request-record-flow/lib/request_record_real_support.mjs';
import { providerNames, restoreRouteFixtures, seedRouteDatabase } from '../20260513-real-route-scheduler-flow/lib/route_fixtures.mjs';
import {
  assertSingleSuccessAttempt,
  expectProxyFailure,
  openAiChatRequest,
  proxyCall,
  proxyStatus,
  successRow,
} from '../20260513-user-access-real-flow/lib/user_access_client.mjs';
import {
  clearProxyState,
  deactivateRateLimitFixtures,
  makeTokenValues,
  openAiKeyIds,
  rateLimitTokenIds,
  rateLimitUserIds,
  resetRateLimitFixtures,
  restoreSystemSettings,
  seedRateLimitFixtures,
  setProviderKeyRateLimit,
  setTokenRateLimit,
  setUserRateLimit,
  systemSettingsSnapshot,
} from './lib/rate_limit_fixtures.mjs';

const baseCtx = loadContext();
const db = new Db(baseCtx.db);
const ctx = contextWithExistingModels(baseCtx, db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const tokenValues = makeTokenValues();
const originalSettings = systemSettingsSnapshot(db);
const results = [];
const RATE_LIMIT_WINDOW_MS = 60_000;
const WINDOW_SETTLE_MS = 1_200;
const MIN_REMAINING_SECONDS_STANDARD = 40;
const MIN_REMAINING_SECONDS_PROVIDER_HARD = 45;
const requestPlan = {
  userToken: null,
  dualKey: null,
};
let backend = null;

async function main() {
  seedRouteDatabase(ctx, db);
  seedRateLimitFixtures(db, tokenValues);
  await resetAll();
  backend = await startBackend(ctx.serverBaseUrl);
  try {
    await resolveRequestPlan();
    await runScenarios();
    assert(results.every((item) => item.ok), failedSummary());
    console.log('rate limit real flow: all scenarios passed');
  } finally {
    await cleanup();
    writeResults();
  }
}

async function runScenarios() {
  await step('current code inspection shows no preexisting runtime enforcement path', inspectCurrentCodePath);
  await step('user limit blocks second request when token is unlimited', userLimitScenario);
  await step('token limit blocks second request when user is unlimited', tokenLimitScenario);
  await step('combined user and token limits use the tighter user cap', combinedLimitScenario);
  await step('single user shared limit spans multiple tokens', sharedUserScenario);
  await step('provider key limit fails over to secondary key', providerKeyFailoverScenario);
  await step('provider key limit returns 429 when all keys are exhausted', providerKeyHardLimitScenario);
}

async function inspectCurrentCodePath() {
  return {
    userRuntimePath: 'apps/hook_backend/src/llm_proxy/rate_limit.rs::enforce_request_limits',
    tokenRuntimePath: 'apps/hook_backend/src/llm_proxy/rate_limit.rs::token_scope',
    providerKeyRuntimePath: 'apps/hook_backend/src/llm_proxy/proxy/executor.rs::attempt_once',
    note: '本轮代码检查前确认旧链路缺少运行时执行点；当前场景用于验证修复后真实生效。',
  };
}

async function userLimitScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_STANDARD, 'user-limit scenario');
  await resetAll();
  setUserRateLimit(db, rateLimitUserIds.userLimited, 1);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    tokenValues.userLimited,
    'user-limit first',
    scenarioRequest(requestPlan.userToken, 'user-limit-1'),
  );
  assertSingleSuccessAttempt(first, requestPlan.userToken.providerName);

  const blocked = await expectProxyFailure(
    ctx,
    db,
    tokenValues.userLimited,
    'user-limit second blocked',
    scenarioRequest(requestPlan.userToken, 'user-limit-2'),
    429,
    'user rate limit exceeded',
  );
  return { first: first.requestId, blocked };
}

async function tokenLimitScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_STANDARD, 'token-limit scenario');
  await resetAll();
  setTokenRateLimit(db, rateLimitTokenIds.tokenLimited, 1);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    tokenValues.tokenLimited,
    'token-limit first',
    scenarioRequest(requestPlan.userToken, 'token-limit-1'),
  );
  assertSingleSuccessAttempt(first, requestPlan.userToken.providerName);

  const blocked = await expectProxyFailure(
    ctx,
    db,
    tokenValues.tokenLimited,
    'token-limit second blocked',
    scenarioRequest(requestPlan.userToken, 'token-limit-2'),
    429,
    'token rate limit exceeded',
  );
  return { first: first.requestId, blocked };
}

async function combinedLimitScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_STANDARD, 'combined-limit scenario');
  await resetAll();
  setUserRateLimit(db, rateLimitUserIds.combined, 1);
  setTokenRateLimit(db, rateLimitTokenIds.combined, 2);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    tokenValues.combined,
    'combined-limit first',
    scenarioRequest(requestPlan.userToken, 'combined-limit-1'),
  );
  assertSingleSuccessAttempt(first, requestPlan.userToken.providerName);

  const blocked = await expectProxyFailure(
    ctx,
    db,
    tokenValues.combined,
    'combined-limit second blocked',
    scenarioRequest(requestPlan.userToken, 'combined-limit-2'),
    429,
    'user rate limit exceeded',
  );
  return { first: first.requestId, blocked };
}

async function sharedUserScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_STANDARD, 'shared-user scenario');
  await resetAll();
  setUserRateLimit(db, rateLimitUserIds.shared, 1);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    tokenValues.sharedA,
    'shared-user token A first',
    scenarioRequest(requestPlan.userToken, 'shared-user-a1'),
  );
  assertSingleSuccessAttempt(first, requestPlan.userToken.providerName);

  const blocked = await expectProxyFailure(
    ctx,
    db,
    tokenValues.sharedB,
    'shared-user token B blocked',
    scenarioRequest(requestPlan.userToken, 'shared-user-b1'),
    429,
    'user rate limit exceeded',
  );

  return { first: first.requestId, blocked };
}

async function providerKeyFailoverScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_STANDARD, 'provider-key failover scenario');
  await resetAll();
  const keys = openAiKeyIds();
  setProviderKeyRateLimit(db, keys.primary, 1);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    ctx.secrets.systemToken,
    'provider-key first',
    scenarioRequest(requestPlan.dualKey, 'provider-key-1'),
  );
  assertSingleSuccessAttempt(first, requestPlan.dualKey.providerName, requestPlan.dualKey.primaryKeyName);

  const second = await proxyCall(
    ctx,
    db,
    ctx.secrets.systemToken,
    'provider-key second failover',
    scenarioRequest(requestPlan.dualKey, 'provider-key-2'),
  );
  assertEqual(second.status, 200, 'secondary request should still succeed');
  const failed = second.trace.find((row) => row.error_type === 'provider_key_rate_limit');
  const success = successRow(second.trace);
  assert(failed, 'second request should record a primary-key limit rejection');
  assertIncludes(failed.error_message, 'provider key rate limit exceeded', 'limit rejection should expose provider key reason');
  assertEqual(success.key_name, requestPlan.dualKey.secondaryKeyName, 'secondary key should take over after primary is limited');
  return { first: first.requestId, second: second.requestId };
}

async function providerKeyHardLimitScenario() {
  await ensureFreshRateLimitWindow(MIN_REMAINING_SECONDS_PROVIDER_HARD, 'provider-key hard-limit scenario');
  await resetAll();
  const keys = openAiKeyIds();
  setProviderKeyRateLimit(db, keys.primary, 1);
  setProviderKeyRateLimit(db, keys.secondary, 1);
  await refreshRuntimeState();

  const first = await proxyCall(
    ctx,
    db,
    ctx.secrets.systemToken,
    'provider-hard-limit first',
    scenarioRequest(requestPlan.dualKey, 'provider-hard-1'),
  );
  const second = await proxyCall(
    ctx,
    db,
    ctx.secrets.systemToken,
    'provider-hard-limit second',
    scenarioRequest(requestPlan.dualKey, 'provider-hard-2'),
  );
  assertSingleSuccessAttempt(first, requestPlan.dualKey.providerName, requestPlan.dualKey.primaryKeyName);
  assert(
    second.trace.some((row) => row.key_name === requestPlan.dualKey.secondaryKeyName),
    'second request should reach secondary key',
  );

  const blocked = await proxyCall(
    ctx,
    db,
    ctx.secrets.systemToken,
    'provider-hard-limit third blocked',
    scenarioRequest(requestPlan.dualKey, 'provider-hard-3'),
    { expectOk: false },
  );
  assertEqual(blocked.status, 429, 'third request should return 429 after both keys are capped');
  assert(blocked.trace.length > 0, 'provider-key 429 should still produce candidate traces');
  assert(blocked.trace.every((row) => row.error_type === 'provider_key_rate_limit'), 'all attempts should be rejected by provider-key limit');
  assert(blocked.trace.every((row) => row.status === 'failed'), 'all provider-key limit attempts should stay failed');
  return { first: first.requestId, second: second.requestId, blocked: blocked.requestId };
}

async function resolveRequestPlan() {
  requestPlan.userToken = await resolveFirstAvailableRoute([
    {
      name: 'openai',
      providerName: providerNames.openai,
      primaryKeyName: 'Route Hook primary',
      secondaryKeyName: 'Route Hook secondary',
      build: (label) => openAiChatRequest(ctx, ctx.models.openai, marker(label)),
    },
    {
      name: 'claude',
      providerName: providerNames.claude,
      primaryKeyName: 'Route Claude primary',
      secondaryKeyName: 'Route Claude secondary',
      build: (label) => openAiChatRequest(ctx, ctx.models.claude, marker(label)),
    },
    {
      name: 'gemini',
      providerName: providerNames.gemini,
      primaryKeyName: 'Route Gemini primary',
      secondaryKeyName: null,
      build: (label) => openAiChatRequest(ctx, ctx.models.gemini, marker(label)),
    },
  ]);
  requestPlan.dualKey = await resolveFirstAvailableRoute(
    [
      {
        name: 'openai',
        providerName: providerNames.openai,
        primaryKeyName: 'Route Hook primary',
        secondaryKeyName: 'Route Hook secondary',
        build: (label) => openAiChatRequest(ctx, ctx.models.openai, marker(label)),
      },
      {
        name: 'claude',
        providerName: providerNames.claude,
        primaryKeyName: 'Route Claude primary',
        secondaryKeyName: 'Route Claude secondary',
        build: (label) => openAiChatRequest(ctx, ctx.models.claude, marker(label)),
      },
    ],
    true,
  );
}

async function resolveFirstAvailableRoute(candidates, requireSecondaryKey = false) {
  const failures = [];
  for (const candidate of candidates) {
    if (requireSecondaryKey && !candidate.secondaryKeyName) {
      continue;
    }
    const probe = await proxyStatus(ctx, ctx.secrets.systemToken, candidate.build(`probe-${candidate.name}`));
    if (probe.ok) {
      return candidate;
    }
    failures.push(`${candidate.name}:${probe.status}:${probe.text.slice(0, 160)}`);
  }
  throw new Error(`no real upstream route available for rate-limit validation: ${failures.join(' | ')}`);
}

function scenarioRequest(route, label) {
  return route.build(label);
}

async function resetAll() {
  restoreRouteFixtures(ctx, db, originalSettings.scheduling_mode);
  resetRateLimitFixtures(db, tokenValues);
  await clearProxyState(redis, ctx.redis.prefix);
}

async function refreshRuntimeState() {
  await clearProxyState(redis, ctx.redis.prefix);
}

async function cleanup() {
  try {
    restoreRouteFixtures(ctx, db, originalSettings.scheduling_mode);
    deactivateRateLimitFixtures(db);
    restoreSystemSettings(db, originalSettings);
    await clearProxyState(redis, ctx.redis.prefix);
  } finally {
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

async function ensureFreshRateLimitWindow(minRemainingSeconds, label) {
  const remainingSeconds = secondsRemainingInMinute();
  if (remainingSeconds >= minRemainingSeconds) {
    return;
  }
  const waitMilliseconds = millisecondsUntilNextMinute() + WINDOW_SETTLE_MS;
  console.log(`waiting ${waitMilliseconds}ms for fresh minute window before ${label}`);
  await sleep(waitMilliseconds);
  const refreshedRemaining = secondsRemainingInMinute();
  assert(
    refreshedRemaining >= minRemainingSeconds,
    `${label} should start with at least ${minRemainingSeconds}s left, got ${refreshedRemaining}s`,
  );
}

function secondsRemainingInMinute(now = Date.now()) {
  const elapsed = Math.floor((now % RATE_LIMIT_WINDOW_MS) / 1000);
  return 60 - elapsed;
}

function millisecondsUntilNextMinute(now = Date.now()) {
  return RATE_LIMIT_WINDOW_MS - (now % RATE_LIMIT_WINDOW_MS);
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
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

function marker(value) {
  return `rate-limit-real-${value}`;
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
