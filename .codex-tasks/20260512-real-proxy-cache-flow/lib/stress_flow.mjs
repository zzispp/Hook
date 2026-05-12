import { assert, assertEqual } from './assertions.mjs';
import { setBrokenProviderActive, setOpenAIKeyPriorities } from './fixtures.mjs';
import { affinityKey, clearAffinity, schedulingLockKey, schedulingSnapshotKey } from './cache_keys.mjs';
import { patchSchedulingMode } from './api.mjs';
import { proxyCall, requestForOpenAiGpt, successRow, tracesSince } from './proxy_client.mjs';

export async function runConcurrentRebuildStress(env) {
  setBrokenProviderActive(env.db, false);
  setOpenAIKeyPriorities(env.db, 0, 0);
  await patchSchedulingMode(env.ctx, env.adminToken, 'load_balance');
  const marker = `cache-stress-${Date.now()}`;
  const before = new Date(Date.now() - 1000).toISOString();
  const requests = stressRequests(env.ctx, marker);
  const flipper = flipSchedulingDuringRequests(env);
  const results = await Promise.all([...requests.map((request) => proxyCall(env.ctx, env.db, request.matchText, request)), flipper]);
  const responses = results.slice(0, requests.length);
  assert(responses.every((item) => item.ok), 'all stress requests should succeed while cache rebuilds');
  const traces = tracesSince(env.db, before, marker);
  assertEqual(traces.length, requests.length, 'stress test should record every request id');
  assertStressTraces(traces);
  assert(await env.redis.get(schedulingSnapshotKey(env.ctx)), 'scheduling snapshot should exist after stress rebuilds');
  assertEqual(await env.redis.get(schedulingLockKey(env.ctx)), null, 'scheduling rebuild lock should be released');
  await clearAffinity(env.ctx, env.redis, env.modelIds);
}

function stressRequests(ctx, marker) {
  const requests = [];
  for (let index = 0; index < ctx.stress.nonStream; index += 1) {
    requests.push(requestForOpenAiGpt(ctx, `${marker}-nonstream-${index}`));
  }
  for (let index = 0; index < ctx.stress.stream; index += 1) {
    requests.push(requestForOpenAiGpt(ctx, `${marker}-stream-${index}`, true));
  }
  return requests;
}

async function flipSchedulingDuringRequests(env) {
  const modes = ['fixed_order', 'cache_affinity', 'load_balance'];
  for (let index = 0; index < env.ctx.stress.rebuildPatches; index += 1) {
    await patchSchedulingMode(env.ctx, env.adminToken, modes[index % modes.length]);
    await sleep(120);
  }
  await patchSchedulingMode(env.ctx, env.adminToken, 'load_balance');
}

function assertStressTraces(traces) {
  const successes = traces.map(({ trace }) => successRow(trace));
  assertEqual(successes.length, traces.length, 'every stress trace should have success');
  const keys = new Set(successes.map((row) => row.key_name));
  assert(keys.has('Hook primary') && keys.has('Hook secondary'), 'stress load balance should use both keys');
  const failed = traces.flatMap(({ trace }) => trace.filter((row) => row.status === 'failed'));
  assertEqual(failed.length, 0, 'stress test should not produce failed attempts');
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
