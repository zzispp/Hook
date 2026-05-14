import { randomBytes } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert, assertEqual } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { ensureBackend } from '../20260512-real-proxy-cache-flow/lib/backend.mjs';
import { encryptProviderKey, sha256 } from '../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { openAiChatRequest, proxyCall } from '../20260514-real-request-record-flow/lib/request_record_real_client.mjs';
import {
  applyFullRecordingSettings,
  clearAuth,
  clearScheduling,
  restoreSystemSettings,
  systemSettingsSnapshot,
  waitForRecordTerminal,
} from '../20260514-real-request-record-flow/lib/request_record_real_support.mjs';
import { stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';

import { DockerDb, q } from './lib/docker_db.mjs';

const IDS = Object.freeze({
  group: '00000000-0000-7000-9400-000000000001',
  providerHook: '00000000-0000-7000-9400-000000000101',
  providerEkan8: '00000000-0000-7000-9400-000000000102',
  endpointHook: '00000000-0000-7000-9400-000000000201',
  endpointEkan8: '00000000-0000-7000-9400-000000000202',
  keyHook: '00000000-0000-7000-9400-000000000301',
  keyEkan8: '00000000-0000-7000-9400-000000000302',
  modelDirect: '00000000-0000-7000-9400-000000000401',
  modelMapped: '00000000-0000-7000-9400-000000000402',
  bindingDirect: '00000000-0000-7000-9400-000000000501',
  bindingMapped: '00000000-0000-7000-9400-000000000502',
  token: '00000000-0000-7000-9400-000000000601',
});

const ctx = loadContext();
const db = new DockerDb(ctx.db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const tokenValue = `sk-usage-real-${randomBytes(18).toString('hex')}`;
const originalSettings = systemSettingsSnapshot(db);

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

async function main() {
  const upstream = await resolveUpstream();
  const backend = await startOrReuseBackend();
  try {
    applyFullRecordingSettings(db);
    seedFixtures(upstream);
    clearTestRows();
    await clearCaches();
    const before = usageSnapshot();
    const direct = await runScenario('direct', ctx.modelNames.direct, before.direct_usage_count);
    const mapped = await runScenario('mapped', ctx.modelNames.mapped, before.mapped_usage_count);
    const after = usageSnapshot();
    const results = buildResults(upstream, before, after, direct, mapped);
    assertEqual(results.direct.delta, 1, 'direct model usage_count should increment by 1');
    assertEqual(results.mapped.delta, 1, 'mapped model usage_count should increment by 1');
    assertEqual(results.token.request_count_delta, 2, 'test token request_count should increment by 2');
    writeResults(results);
    console.log(JSON.stringify(results, null, 2));
  } finally {
    restoreSystemSettings(db, originalSettings);
    await clearCaches();
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

function loadContext() {
  return Object.freeze({
    serverBaseUrl: process.env.HOOK_BACKEND_URL || 'http://127.0.0.1:5555',
    providerSecret: process.env.HOOK_PROVIDER_KEY_SECRET || 'hook-local-development-provider-key-secret-change-before-deploy',
    hookBaseUrl: 'https://www.hook.rs',
    ekan8BaseUrl: 'https://www.ekan8.com',
    hookKey: requiredEnv('HOOK_RS_KEY'),
    ekan8Key: requiredEnv('EKAN8_KEY'),
    adminUserId: '00000000-0000-7000-8000-000000000000',
    groupCode: 'usage_real_20260514',
    modelNames: Object.freeze({
      direct: 'usage-real-openai-direct',
      mapped: 'usage-real-openai-mapped',
    }),
    db: Object.freeze({ container: process.env.HOOK_PG_CONTAINER || 'hook-postgres', user: 'postgres', name: 'postgres' }),
    redis: Object.freeze({ host: '127.0.0.1', port: 6380, prefix: 'hook' }),
  });
}

async function resolveUpstream() {
  const hookModels = extractOpenAiModels(await fetchJsonLoose(`${ctx.hookBaseUrl}/v1/models`, bearer(ctx.hookKey)));
  const ekan8Models = extractOpenAiModels(await fetchJsonLoose(`${ctx.ekan8BaseUrl}/v1/models`, bearer(ctx.ekan8Key)));
  const directModel = chooseModel(hookModels, ['gpt-5.5', 'gpt-5.4', 'gpt-5.4-mini']);
  const mappedModel = chooseModel(ekan8Models, ['R-claude-opus-4-7', 'ccmax-claude-opus-4-7', '[满血]gemini-3.1-pro-preview', '[满血]gemini-3-pro-preview']);
  assert(directModel, `Hook.rs direct model not found, available: ${hookModels.slice(0, 20).join(', ')}`);
  assert(mappedModel, `Ekan8 mapped model not found, available: ${ekan8Models.slice(0, 40).join(', ')}`);
  await probeUpstream(ctx.hookBaseUrl, ctx.hookKey, directModel, 'Hook.rs direct model probe');
  await probeUpstream(ctx.ekan8BaseUrl, ctx.ekan8Key, mappedModel, 'Ekan8 mapped model probe');
  return { directModel, mappedModel, hookModels: hookModels.slice(0, 20), ekan8Models: ekan8Models.slice(0, 40) };
}

async function startOrReuseBackend() {
  if (await healthOk()) return null;
  return ensureBackend(ctx.serverBaseUrl);
}

function seedFixtures(upstream) {
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }));
  const hookKey = q(encryptProviderKey(ctx.providerSecret, ctx.hookKey));
  const ekan8Key = q(encryptProviderKey(ctx.providerSecret, ctx.ekan8Key));
  const mapped = q(JSON.stringify({ name: upstream.mappedModel, reasoning_effort: 'high' }));
  db.exec(`
insert into global_models (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(IDS.modelDirect)}, ${q(ctx.modelNames.direct)}, ${q(ctx.modelNames.direct)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(IDS.modelMapped)}, ${q(ctx.modelNames.mapped)}, ${q(ctx.modelNames.mapped)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (id) do update set name = excluded.name, display_name = excluded.display_name, is_active = true, updated_at = now();
insert into billing_groups (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values (${q(IDS.group)}, ${q(ctx.groupCode)}, 'Usage Real Test', null, 1, true, false, 0, now(), now())
on conflict (code) do update set is_active = true, updated_at = now();
insert into providers (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(IDS.providerHook)}, 'Usage Real Hook.rs', 'openai', 0, 60, 60, 10, false, false, true, now(), now()),
  (${q(IDS.providerEkan8)}, 'Usage Real Ekan8', 'openai', 0, 60, 60, 20, false, false, true, now(), now())
on conflict (id) do update set is_active = true, updated_at = now();
delete from provider_endpoints where id in (${q(IDS.endpointHook)}, ${q(IDS.endpointEkan8)});
insert into provider_endpoints (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(IDS.endpointHook)}, ${q(IDS.providerHook)}, 'openai_chat', ${q(ctx.hookBaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(IDS.endpointEkan8)}, ${q(IDS.providerEkan8)}, 'openai_chat', ${q(ctx.ekan8BaseUrl)}, null, 0, true, null, null, null, now(), now());
delete from provider_api_keys where id in (${q(IDS.keyHook)}, ${q(IDS.keyEkan8)});
insert into provider_api_keys (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit, cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(IDS.keyHook)}, ${q(IDS.providerHook)}, 'Usage Real Hook.rs Key', ${hookKey}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(IDS.keyEkan8)}, ${q(IDS.providerEkan8)}, 'Usage Real Ekan8 Key', ${ekan8Key}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now());
delete from provider_models where id in (${q(IDS.bindingDirect)}, ${q(IDS.bindingMapped)});
insert into provider_models (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(IDS.bindingDirect)}, ${q(IDS.providerHook)}, ${q(IDS.modelDirect)}, ${q(upstream.directModel)}, null, true, null, null, null, now(), now()),
  (${q(IDS.bindingMapped)}, ${q(IDS.providerEkan8)}, ${q(IDS.modelMapped)}, 'mapping-placeholder', ${mapped}, true, null, null, null, now(), now());
delete from billing_group_providers where group_code = ${q(ctx.groupCode)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9400-000000000701', ${q(ctx.groupCode)}, ${q(IDS.providerHook)}, now(), now()),
  ('00000000-0000-7000-9400-000000000702', ${q(ctx.groupCode)}, ${q(IDS.providerEkan8)}, now(), now());
delete from billing_group_models where group_code = ${q(ctx.groupCode)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9400-000000000711', ${q(ctx.groupCode)}, ${q(IDS.modelDirect)}, now(), now()),
  ('00000000-0000-7000-9400-000000000712', ${q(ctx.groupCode)}, ${q(IDS.modelMapped)}, now(), now());
delete from api_tokens where id = ${q(IDS.token)};
insert into api_tokens (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at, model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count, is_active, last_used_at, created_at, updated_at)
values (${q(IDS.token)}, ${q(ctx.adminUserId)}, 'independent', 'Usage Real Token', ${q(tokenValue)}, ${q(sha256(tokenValue))}, ${q(tokenValue.slice(0, 10))}, ${q(ctx.groupCode)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());`);
}

function clearTestRows() {
  db.exec(`
delete from request_records where request_id in (select distinct request_id from request_candidates where token_id = ${q(IDS.token)});
delete from request_candidates where token_id = ${q(IDS.token)};`);
}

async function clearCaches() {
  await clearScheduling(redis, ctx.redis.prefix);
  await clearAuth(redis, ctx.redis.prefix);
  const keys = await redis.keys(`${ctx.redis.prefix}:llm_proxy:affinity:${IDS.token}:*`);
  await redis.del(...keys);
}

function usageSnapshot() {
  const [[directUsage], [mappedUsage], [requestCount]] = [
    db.rows(`select usage_count::text from global_models where id = ${q(IDS.modelDirect)} limit 1;`),
    db.rows(`select usage_count::text from global_models where id = ${q(IDS.modelMapped)} limit 1;`),
    db.rows(`select request_count::text from api_tokens where id = ${q(IDS.token)} limit 1;`),
  ];
  return { direct_usage_count: Number(directUsage), mapped_usage_count: Number(mappedUsage), token_request_count: Number(requestCount) };
}

async function runScenario(kind, modelName, beforeUsageCount) {
  const marker = `${kind}-${Date.now()}`;
  const result = await proxyCall(ctx, db, tokenValue, `usage ${kind}`, openAiChatRequest(ctx, modelName, marker));
  await waitForRecordTerminal(db, result.requestId);
  const candidate = successfulCandidate(result.requestId);
  const record = requestRecord(result.requestId);
  const currentUsage = Number(db.scalar(`select usage_count::text from global_models where name = ${q(modelName)} limit 1;`));
  return {
    requestId: result.requestId,
    marker,
    modelName,
    beforeUsageCount,
    afterUsageCount: currentUsage,
    delta: currentUsage - beforeUsageCount,
    providerRequestModel: candidate.provider_request_body?.model ?? null,
    providerResponseModel: candidate.provider_response_body?.model ?? null,
    clientResponseModel: record.client_response_body?.model ?? null,
    trace: result.trace,
  };
}

function successfulCandidate(requestId) {
  const [row] = db.rows(`
select coalesce(provider_request_body::text, '{}'), coalesce(provider_response_body::text, '{}')
from request_candidates where request_id = ${q(requestId)} and status = 'success'
order by candidate_index, retry_index limit 1;`);
  assert(row, `successful request candidate should exist: ${requestId}`);
  return { provider_request_body: JSON.parse(row[0]), provider_response_body: JSON.parse(row[1]) };
}

function requestRecord(requestId) {
  const [row] = db.rows(`
select status, billing_status, coalesce(client_response_body::text, '{}')
from request_records where request_id = ${q(requestId)} limit 1;`);
  assert(row, `request record should exist: ${requestId}`);
  assertEqual(row[0], 'success', 'request record should be successful');
  assertEqual(row[1], 'settled', 'request record billing should be settled');
  return { client_response_body: JSON.parse(row[2]) };
}

function buildResults(upstream, before, after, direct, mapped) {
  return {
    executed_at: new Date().toISOString(),
    fixtures: { group_code: ctx.groupCode, direct_model_name: ctx.modelNames.direct, mapped_model_name: ctx.modelNames.mapped },
    upstream,
    before,
    after,
    direct,
    mapped,
    token: {
      request_count_before: before.token_request_count,
      request_count_after: after.token_request_count,
      request_count_delta: after.token_request_count - before.token_request_count,
    },
  };
}

function writeResults(results) {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

async function fetchJsonLoose(url, headers = {}) {
  const response = await fetch(url, { headers });
  const text = await response.text();
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`invalid JSON from ${url}: ${response.status} ${text.slice(0, 400)} :: ${error.message}`);
  }
}

function extractOpenAiModels(body) {
  const items = Array.isArray(body?.data) ? body.data : [];
  return items.map((item) => item?.id).filter(Boolean);
}

function chooseModel(models, preferred) {
  return preferred.find((name) => models.includes(name)) || models[0] || '';
}

async function probeUpstream(baseUrl, apiKey, model, label) {
  const response = await fetch(`${baseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', authorization: `Bearer ${apiKey}` },
    body: JSON.stringify({ model, messages: [{ role: 'user', content: `${label} ping` }], max_tokens: 8, temperature: 0 }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`${label} failed ${response.status}: ${text.slice(0, 500)}`);
}

function bearer(apiKey) {
  return { Authorization: `Bearer ${apiKey}` };
}

async function healthOk() {
  try {
    const response = await fetch(`${ctx.serverBaseUrl}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) throw new Error(`missing required env: ${name}`);
  return value;
}
