import { ids, providerNames } from './billing_access_ids.mjs';
import { setAccessProviders, setPrimaryKeyPriorities, setSchedulingMode } from './billing_access_db_control.mjs';
import { affinityKeyValue, openAiChatRequest, proxyExchange, successCandidate } from './billing_access_client.mjs';
import {
  expect,
  expectEqual,
  expectSuccess,
  providerAttempts,
} from './billing_access_assertions.mjs';

const LOAD_BALANCE_REQUESTS = 12;
const COLD_AFFINITY_REQUESTS = 6;

export async function retryAndProviderFailover(state) {
  setSchedulingMode(state.db, 'fixed_order');
  setAccessProviders(state.db, { broken: true, primaryA: true, brokenRetries: 1 });
  await state.clearCaches();
  const result = await routingExchange(state, 'retry-failover');
  expectSuccess(result, `retry failover should finish on healthy provider ${brief(result)}`);
  const broken = providerAttempts(result.trace, providerNames.broken);
  expectEqual(broken.length, 2, `broken provider should be retried once ${brief(result)}`);
  expect(broken.every((row) => row.status === 'failed'), 'broken retry attempts should be failed');
  expectEqual(successCandidate(result.trace)?.provider_name, providerNames.primaryA, 'primary provider should succeed after broken retries');
  return result;
}

export async function providerTimeoutFailover(state) {
  setSchedulingMode(state.db, 'fixed_order');
  setAccessProviders(state.db, { slow: true, primaryA: true, slowTimeout: 0.2 });
  await state.clearCaches();
  const result = await routingExchange(state, 'timeout-failover');
  expectSuccess(result, `timeout failover should finish on healthy provider ${brief(result)}`);
  const slow = providerAttempts(result.trace, providerNames.slow);
  expectEqual(slow.length, 1, `slow provider should be attempted once ${brief(result)}`);
  expectEqual(slow[0].error_type, 'upstream_timeout', 'slow provider should record upstream_timeout');
  expectEqual(successCandidate(result.trace)?.provider_name, providerNames.primaryA, 'primary provider should succeed after timeout');
  return result;
}

export async function loadBalanceEqualPriority(state) {
  setSchedulingMode(state.db, 'load_balance');
  setAccessProviders(state.db, { primaryA: true, primaryB: true });
  setPrimaryKeyPriorities(state.db, 0, 0);
  await state.clearCaches();
  const results = [];
  for (let index = 0; index < LOAD_BALANCE_REQUESTS; index += 1) {
    const result = await routingExchange(state, `load-balance-${index}`);
    expectSuccess(result, `load balance request ${index} should succeed ${brief(result)}`);
    results.push(result);
  }
  const providers = new Set(results.map((item) => successCandidate(item.trace)?.provider_name));
  expect(providers.has(providerNames.primaryA) && providers.has(providerNames.primaryB), `load_balance should use both providers: ${[...providers].join(', ')}`);
  return { providers: [...providers].sort(), requestIds: results.map((item) => item.requestId) };
}

export async function cacheAffinityWarmHit(state) {
  await prepareCacheAffinity(state);
  const first = await routingExchange(state, 'affinity-warm-first');
  expectSuccess(first, `first affinity request should succeed ${brief(first)}`);
  const cachedKey = await affinityKeyValue(state.redis, state.ctx.redis.prefix);
  expectEqual(cachedKey, successCandidate(first.trace)?.key_id ?? cachedKey, 'affinity key should be remembered');
  const second = await routingExchange(state, 'affinity-warm-second');
  expectSuccess(second, `second affinity request should succeed ${brief(second)}`);
  expectEqual(successCandidate(second.trace)?.key_name, successCandidate(first.trace)?.key_name, 'warm affinity should keep the first successful key');
  return { first: first.requestId, second: second.requestId, cachedKey };
}

export async function cacheAffinityColdStartRandomness(state) {
  const firstKeys = [];
  for (let index = 0; index < COLD_AFFINITY_REQUESTS; index += 1) {
    await prepareCacheAffinity(state);
    const result = await routingExchange(state, `affinity-cold-${index}`);
    expectSuccess(result, `cold affinity request ${index} should succeed ${brief(result)}`);
    firstKeys.push(successCandidate(result.trace)?.key_name ?? '');
  }
  const unique = [...new Set(firstKeys)];
  expect(unique.length > 1, `cold cache_affinity with equal priority should randomize first pick, got ${unique.join(', ')}`);
  return { firstKeys, unique };
}

export async function ekan8MappedRequest(state) {
  setSchedulingMode(state.db, 'fixed_order');
  setAccessProviders(state.db, { primaryA: false, ekan8: true });
  await state.clearCaches();
  const request = openAiChatRequest(state.models.ekan8, state.marker('ekan8-mapped'));
  const result = await proxyExchange(state.ctx, state.db, ids.tokenRouting, state.tokenValues.routing, 'ekan8-mapped', request);
  expectSuccess(result, `Ekan8 mapped request should succeed ${brief(result)}`);
  expectEqual(successCandidate(result.trace)?.provider_name, providerNames.ekan8, 'Ekan8 provider should serve mapped model');
  return result;
}

async function prepareCacheAffinity(state) {
  setSchedulingMode(state.db, 'cache_affinity');
  setAccessProviders(state.db, { primaryA: true, primaryB: true });
  setPrimaryKeyPriorities(state.db, 0, 0);
  await state.clearCaches();
}

async function routingExchange(state, marker) {
  return proxyExchange(
    state.ctx,
    state.db,
    ids.tokenRouting,
    state.tokenValues.routing,
    marker,
    openAiChatRequest(state.models.openai, state.marker(marker)),
  );
}

function brief(result) {
  return JSON.stringify({
    status: result.status,
    requestId: result.requestId,
    trace: result.trace.map((row) => ({
      provider: row.provider_name,
      key: row.key_name,
      status: row.status,
      retry: row.retry_index,
      error: row.error_type || row.error_code,
    })),
    body: result.body,
  });
}
