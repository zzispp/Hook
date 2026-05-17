import { createServer } from 'node:http';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { ensureBackend, stopBackend } from '../20260517-real-aether-billing-flow/lib/backend.mjs';
import { encryptProviderKey, randomToken, sha256 } from '../20260517-real-aether-billing-flow/lib/crypto.mjs';
import { DockerDb, q } from '../20260517-real-aether-billing-flow/lib/db.mjs';
import { RedisClient } from '../20260517-real-aether-billing-flow/lib/redis.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const SENSITIVE_HOST = 'api.86gamestore.com';
const SENSITIVE_KEY = 'sk-upstream-secret-real-test';
const MODEL_NAME = 'sanitize-real-upstream-failure-chat';
const IDS = Object.freeze({
  user: '00000000-0000-7000-9600-000000000001',
  wallet: '00000000-0000-7000-9600-000000000002',
  group: 'sanitize_real_20260517',
  provider: '00000000-0000-7000-9600-000000000101',
  endpoint: '00000000-0000-7000-9600-000000000201',
  key: '00000000-0000-7000-9600-000000000301',
  model: '00000000-0000-7000-9600-000000000401',
  binding: '00000000-0000-7000-9600-000000000501',
  token: '00000000-0000-7000-9600-000000000701',
});

const ctx = Object.freeze({
  serverBaseUrl: env('HOOK_BACKEND_URL', 'http://127.0.0.1:5555'),
  providerSecret: env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy'),
  db: Object.freeze({
    container: env('HOOK_PG_CONTAINER', 'hook-postgres'),
    user: env('HOOK_PG_USER', 'postgres'),
    name: env('HOOK_PG_DB', 'postgres'),
  }),
  redis: Object.freeze({
    host: env('HOOK_REDIS_HOST', '127.0.0.1'),
    port: Number(env('HOOK_REDIS_PORT', '6380')),
    prefix: env('HOOK_REDIS_PREFIX', 'hook'),
  }),
});

const db = new DockerDb(ctx.db);
const redis = new RedisClient(ctx.redis);
const tokenValue = randomToken('sk-sanitize-real');

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

async function main() {
  const upstream = await startSensitiveUpstream();
  const settings = settingsSnapshot();
  seedFixtures(upstream.baseUrl);
  await clearCaches();
  const backend = await ensureBackend(ctx.serverBaseUrl);
  try {
    const response = await callProxy();
    const requestId = await waitForRequestId();
    const record = await waitForRecord(requestId);
    const candidate = candidateRecord(requestId);
    const evidence = validateEvidence({ response, requestId, record, candidate, upstream });
    writeResults(evidence);
    console.log(JSON.stringify(evidence, null, 2));
  } finally {
    restoreSettings(settings);
    cleanupFixtures();
    await clearCaches();
    await stopBackend(backend, ctx.serverBaseUrl);
    await upstream.stop();
  }
}

async function startSensitiveUpstream() {
  const requests = [];
  const server = createServer((request, response) => {
    const chunks = [];
    request.on('data', (chunk) => chunks.push(chunk));
    request.on('end', () => {
      requests.push({
        method: request.method,
        url: request.url,
        authorization: request.headers.authorization ?? '',
        body: Buffer.concat(chunks).toString('utf8'),
      });
      response.writeHead(429, { 'content-type': 'application/json' });
      response.end(
        JSON.stringify({
          error: {
            message: `quota exhausted at ${SENSITIVE_HOST} using ${SENSITIVE_KEY}`,
            code: SENSITIVE_KEY,
            param: 'model',
          },
        }),
      );
    });
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  return Object.freeze({
    baseUrl: `http://127.0.0.1:${port}`,
    requests,
    stop: () => new Promise((resolve) => server.close(resolve)),
  });
}

async function callProxy() {
  const response = await fetch(`${ctx.serverBaseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { authorization: `Bearer ${tokenValue}`, 'content-type': 'application/json' },
    body: JSON.stringify({
      model: MODEL_NAME,
      messages: [{ role: 'user', content: 'trigger sanitized upstream failure' }],
      max_tokens: 8,
      temperature: 0,
    }),
  });
  return Object.freeze({
    status: response.status,
    contentType: response.headers.get('content-type') ?? '',
    text: await response.text(),
  });
}

function validateEvidence(input) {
  const clientBody = parseJson(input.response.text);
  const recordClientBody = parseJson(input.record.client_response_body);
  const candidateProviderBody = parseJson(input.candidate.provider_response_body);
  assertEqual(input.response.status, 502, 'client status should be sanitized 502');
  assertNoSensitive(input.response.text, 'client response body');
  assertNoSensitive(input.record.client_response_body, 'request_records.client_response_body');
  assertContains(input.candidate.provider_response_body, SENSITIVE_HOST, 'provider_response_body should retain upstream host for audit');
  assertContains(input.candidate.provider_response_body, SENSITIVE_KEY, 'provider_response_body should retain upstream key marker for audit');
  assertContains(input.candidate.error_message, SENSITIVE_HOST, 'candidate error_message should retain upstream message for audit');
  assertEqual(clientBody.error.message, 'The model service is temporarily unavailable. Please retry later.', 'client error message');
  assertEqual(clientBody.error.type, 'server_error', 'client error type');
  assertEqual(clientBody.error.code, 'model_service_unavailable', 'client error code');
  assertEqual(input.record.status, 'failed', 'request record status');
  assertEqual(input.record.billing_status, 'void', 'failed request billing status');
  assertEqual(input.record.client_status_code, '429', 'request record preserves upstream status code');
  assertEqual(input.candidate.status, 'failed', 'candidate status');
  assertEqual(input.candidate.status_code, '429', 'candidate status code');
  assertEqual(recordClientBody.error.code, 'model_service_unavailable', 'record client body code');
  assertEqual(candidateProviderBody.error.code, SENSITIVE_KEY, 'audit provider body code');
  assertEqual(input.upstream.requests.length, 1, 'local upstream should receive one request');
  return Object.freeze({
    executed_at: new Date().toISOString(),
    request_id: input.requestId,
    upstream_request_count: input.upstream.requests.length,
    client: {
      status: input.response.status,
      content_type: input.response.contentType,
      body: clientBody,
      leaked_sensitive_marker: containsSensitive(input.response.text),
    },
    request_record: {
      status: input.record.status,
      billing_status: input.record.billing_status,
      client_status_code: input.record.client_status_code,
      client_error_type: input.record.client_error_type,
      client_error_message: input.record.client_error_message,
      client_response_body: recordClientBody,
      client_response_leaked_sensitive_marker: containsSensitive(input.record.client_response_body),
    },
    candidate: {
      status: input.candidate.status,
      status_code: input.candidate.status_code,
      error_type: input.candidate.error_type,
      error_code: input.candidate.error_code,
      provider_response_retained_sensitive_marker: containsSensitive(input.candidate.provider_response_body),
    },
  });
}

function seedFixtures(baseUrl) {
  seedMenuSections();
  db.exec(`
update system_settings
set scheduling_mode = 'fixed_order',
    request_record_level = 'full',
    record_request_headers = true,
    record_request_body = true,
    record_response_body = true,
    max_request_body_size_kb = 5120,
    max_response_body_size_kb = 5120,
    updated_at = now()
where id = 'global';
insert into roles (code, name, description, enabled, system, sort_order, created_at, updated_at)
values ('admin', 'Admin', 'Sanitize test admin role', true, true, 0, now(), now())
on conflict (code) do update set enabled = true, updated_at = now();
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids,
   created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
  (${q(IDS.user)}, 'sanitize_real_user', ${q(passwordHash())}, 'sanitize-real@example.com', 'admin',
   true, false, '[]', '[]', now(), now(), null, 'local', true, 0, 'wallet')
on conflict (id) do update set is_active = true, is_deleted = false, quota_mode = 'wallet', updated_at = now();
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
   total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
  (${q(IDS.wallet)}, ${q(IDS.user)}, 10, 0, 'CNY', 'active', 'finite', 10, 0, 0, 0, now(), now())
on conflict (user_id) do update set recharge_balance = 10, gift_balance = 0, status = 'active', updated_at = now();
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  ('00000000-0000-7000-9600-000000000003', ${q(IDS.group)}, 'Sanitize Real Test', null, 1, true, false, 0, now(), now())
on conflict (code) do update set billing_multiplier = 1, is_active = true, updated_at = now();
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(IDS.model)}, ${q(MODEL_NAME)}, 'Sanitize Real Upstream Failure Chat', null,
   ${q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }))},
   '["chat"]', null, true, 0, now(), now())
on conflict (id) do update set name = excluded.name, is_active = true, updated_at = now();
delete from billing_group_models where group_code = ${q(IDS.group)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values ('00000000-0000-7000-9600-000000000801', ${q(IDS.group)}, ${q(IDS.model)}, now(), now());
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds,
   priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(IDS.provider)}, 'Sanitize Sensitive Upstream Provider', 'openai', 0, 10, 10, -100, false, true, true, now(), now())
on conflict (id) do update set priority = -100, max_retries = 0, is_active = true, updated_at = now();
delete from provider_endpoints where id = ${q(IDS.endpoint)};
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active,
   format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(IDS.endpoint)}, ${q(IDS.provider)}, 'openai_chat', ${q(baseUrl)}, null, 0, true, null, null, null, now(), now());
delete from provider_api_keys where id = ${q(IDS.key)};
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end,
   health_by_format, circuit_breaker_by_format, is_active, api_formats, allowed_model_ids, created_at, updated_at)
values
  (${q(IDS.key)}, ${q(IDS.provider)}, 'Sanitize sensitive key', ${q(encryptProviderKey(ctx.providerSecret, SENSITIVE_KEY))}, null, 0, null, null,
   0, 0, false, null, null, null, null, true, '["openai_chat"]', '[]', now(), now());
delete from provider_models where id = ${q(IDS.binding)};
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active,
   price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(IDS.binding)}, ${q(IDS.provider)}, ${q(IDS.model)}, 'sensitive-upstream-model', null, true, null, null, null, now(), now());
delete from billing_group_providers where group_code = ${q(IDS.group)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values ('00000000-0000-7000-9600-000000000811', ${q(IDS.group)}, ${q(IDS.provider)}, now(), now());
delete from api_tokens where id = ${q(IDS.token)};
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
  (${q(IDS.token)}, ${q(IDS.user)}, 'user', 'Sanitize real token', ${q(tokenValue)}, ${q(sha256(tokenValue))},
   ${q(tokenValue.slice(0, 10))}, ${q(IDS.group)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());
delete from request_records where request_id in (select request_id from request_candidates where token_id = ${q(IDS.token)});
delete from request_candidates where token_id = ${q(IDS.token)};`);
}

function cleanupFixtures() {
  db.exec(`
delete from request_records where request_id in (select request_id from request_candidates where token_id = ${q(IDS.token)});
delete from request_candidates where token_id = ${q(IDS.token)};
delete from api_tokens where id = ${q(IDS.token)};
delete from provider_models where id = ${q(IDS.binding)};
delete from provider_api_keys where id = ${q(IDS.key)};
delete from provider_endpoints where id = ${q(IDS.endpoint)};
delete from billing_group_providers where group_code = ${q(IDS.group)};
delete from providers where id = ${q(IDS.provider)};
delete from billing_group_models where group_code = ${q(IDS.group)};
delete from global_models where id = ${q(IDS.model)};
delete from billing_groups where code = ${q(IDS.group)};
delete from wallet_transactions where wallet_id = ${q(IDS.wallet)};
delete from wallets where id = ${q(IDS.wallet)};
update users set is_active = false, is_deleted = true, updated_at = now() where id = ${q(IDS.user)};`);
}

function seedMenuSections() {
  db.exec(`
insert into menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000101', 'overview', '概览', -10, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000102', 'operations', '运营管理', -5, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000103', 'system_management', '系统管理', 0, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00')
on conflict (id) do update set code = excluded.code, subheader = excluded.subheader, sort_order = excluded.sort_order, enabled = excluded.enabled;`);
}

function settingsSnapshot() {
  const [row] = db.rows(`
select request_record_level, record_request_headers::text, record_request_body::text, record_response_body::text,
  max_request_body_size_kb::text, max_response_body_size_kb::text, scheduling_mode
from system_settings
where id = 'global';`);
  assert(row, 'system_settings global row should exist');
  return Object.freeze({
    request_record_level: row[0],
    record_request_headers: row[1] === 't',
    record_request_body: row[2] === 't',
    record_response_body: row[3] === 't',
    max_request_body_size_kb: row[4],
    max_response_body_size_kb: row[5],
    scheduling_mode: row[6],
  });
}

function restoreSettings(snapshot) {
  db.exec(`
update system_settings
set request_record_level = ${q(snapshot.request_record_level)},
    record_request_headers = ${snapshot.record_request_headers ? 'true' : 'false'},
    record_request_body = ${snapshot.record_request_body ? 'true' : 'false'},
    record_response_body = ${snapshot.record_response_body ? 'true' : 'false'},
    max_request_body_size_kb = ${q(snapshot.max_request_body_size_kb)},
    max_response_body_size_kb = ${q(snapshot.max_response_body_size_kb)},
    scheduling_mode = ${q(snapshot.scheduling_mode)},
    updated_at = now()
where id = 'global';`);
}

async function clearCaches() {
  await delPattern(`${ctx.redis.prefix}:llm_proxy:scheduling_snapshot`);
  await delPattern(`${ctx.redis.prefix}:llm_proxy:token:*`);
  await delPattern(`${ctx.redis.prefix}:llm_proxy:token_usage:*`);
  await delPattern(`${ctx.redis.prefix}:llm_proxy:affinity:*`);
  await delPattern(`${ctx.redis.prefix}:llm_proxy:provider_cooldown:*`);
}

async function delPattern(pattern) {
  const keys = await redis.keys(pattern);
  if (keys.length > 0) {
    await redis.del(...keys);
  }
}

async function waitForRequestId() {
  const tokenId = IDS.token;
  const started = Date.now();
  while (Date.now() - started < 12_000) {
    const value = db.scalar(`
select request_id
from request_candidates
where token_id = ${q(tokenId)}
order by created_at desc
limit 1;`);
    if (value) {
      return value;
    }
    await sleep(250);
  }
  throw new Error('request candidate row was not created');
}

async function waitForRecord(requestId) {
  const started = Date.now();
  while (Date.now() - started < 12_000) {
    const [row] = db.rows(`
select request_id, status, billing_status, coalesce(client_status_code::text, ''), coalesce(client_error_type, ''),
  coalesce(client_error_message, ''), coalesce(client_response_body, '')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
    if (row && !['pending', 'streaming'].includes(row[1])) {
      return Object.freeze({
        request_id: row[0],
        status: row[1],
        billing_status: row[2],
        client_status_code: row[3],
        client_error_type: row[4],
        client_error_message: row[5],
        client_response_body: row[6],
      });
    }
    await sleep(250);
  }
  throw new Error(`request record did not finish: ${requestId}`);
}

function candidateRecord(requestId) {
  const [row] = db.rows(`
select status, coalesce(status_code::text, ''), coalesce(error_type, ''), coalesce(error_message, ''),
  coalesce(error_code, ''), coalesce(error_param, ''), coalesce(provider_response_body, '')
from request_candidates
where request_id = ${q(requestId)}
order by candidate_index, retry_index
limit 1;`);
  assert(row, `candidate record should exist: ${requestId}`);
  return Object.freeze({
    status: row[0],
    status_code: row[1],
    error_type: row[2],
    error_message: row[3],
    error_code: row[4],
    error_param: row[5],
    provider_response_body: row[6],
  });
}

function writeResults(evidence) {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'sanitize-results.json'), `${JSON.stringify(evidence, null, 2)}\n`);
}

function parseJson(value) {
  return JSON.parse(value);
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertContains(value, needle, label) {
  assert(value.includes(needle), `${label} should contain ${needle}`);
}

function assertNoSensitive(value, label) {
  assert(!containsSensitive(value), `${label} leaked sensitive upstream marker`);
}

function containsSensitive(value) {
  return String(value).includes(SENSITIVE_HOST) || String(value).includes(SENSITIVE_KEY);
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}

function env(name, fallback) {
  return process.env[name] || fallback;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
