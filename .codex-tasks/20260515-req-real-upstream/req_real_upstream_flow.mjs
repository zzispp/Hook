import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert, assertEqual, assertIncludes } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { encryptProviderKey } from '../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { Db, q } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { startBackend, stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { adminSignIn } from '../20260514-real-request-record-flow/lib/request_record_real_client.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const ids = Object.freeze({
  providerHook: '00000000-0000-7000-9300-000000000101',
  providerEkan8: '00000000-0000-7000-9300-000000000102',
  keyHook: '00000000-0000-7000-9300-000000000201',
  keyEkan8: '00000000-0000-7000-9300-000000000202',
  endpointHookOpenAi: '00000000-0000-7000-9300-000000000301',
  endpointHookClaude: '00000000-0000-7000-9300-000000000302',
  endpointEkan8Gemini: '00000000-0000-7000-9300-000000000303',
  endpointEkan8OpenAi: '00000000-0000-7000-9300-000000000304',
  modelOpenAi: '00000000-0000-7000-9300-000000000401',
  modelClaude: '00000000-0000-7000-9300-000000000402',
  modelGemini: '00000000-0000-7000-9300-000000000403',
  modelMapped: '00000000-0000-7000-9300-000000000404',
  bindingHookOpenAi: '00000000-0000-7000-9300-000000000501',
  bindingHookClaude: '00000000-0000-7000-9300-000000000502',
  bindingEkan8Gemini: '00000000-0000-7000-9300-000000000503',
  bindingEkan8Mapped: '00000000-0000-7000-9300-000000000504',
  group: '00000000-0000-7000-9300-000000000601',
  groupProviderHook: '00000000-0000-7000-9300-000000000701',
  groupProviderEkan8: '00000000-0000-7000-9300-000000000702',
  groupModelOpenAi: '00000000-0000-7000-9300-000000000801',
  groupModelClaude: '00000000-0000-7000-9300-000000000802',
  groupModelGemini: '00000000-0000-7000-9300-000000000803',
  groupModelMapped: '00000000-0000-7000-9300-000000000804',
});

const groupCode = 'req_real';
const models = Object.freeze({
  openai: 'req-real-openai',
  claude: 'req-real-claude',
  gemini: 'req-real-gemini',
  mapped: 'req-real-ekan8-openai-mapped',
});
const upstream = Object.freeze({
  hookOpenAi: 'gpt-5.4-mini',
  hookClaude: 'claude-haiku-4-5-20251001',
  ekan8Gemini: '[满血]gemini-3.1-pro-preview',
  ekan8Mapped: 'ccmax-claude-opus-4-7',
});

const ctx = loadReqContext();
const db = new Db(ctx.db);
const redis = new RedisClient(ctx.redis);
const results = [];
let backend = null;
let adminToken = '';
let proxyToken = '';
let proxyTokenId = '';
let originalSettings = null;

async function main() {
  originalSettings = systemSettingsSnapshot();
  await probeUpstreams();
  seedFixtures();
  await clearProxyCaches();
  backend = await startBackend(ctx.serverBaseUrl);
  try {
    adminToken = await adminSignIn(ctx);
    proxyToken = await createProxyToken();
    await runScenario('admin upstream model fetch uses req client', upstreamModelFetchScenario);
    await runScenario('hook openai non-stream through proxy', hookOpenAiNonStreamScenario);
    await runScenario('hook openai stream through proxy', hookOpenAiStreamScenario);
    await runScenario('hook claude through proxy', hookClaudeScenario);
    await runScenario('ekan8 gemini direct through proxy', ekan8GeminiScenario);
    await runScenario('ekan8 openai-compatible mapped model through proxy', ekan8MappedScenario);
    assert(results.every((item) => item.ok), failedSummary());
    console.log('req real upstream flow: all scenarios passed');
  } finally {
    await cleanup();
    writeResults();
  }
}

function loadReqContext() {
  return Object.freeze({
    serverBaseUrl: env('HOOK_BACKEND_URL', 'http://127.0.0.1:5555'),
    providerSecret: env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy'),
    adminUserId: env('HOOK_ADMIN_USER_ID', '00000000-0000-7000-8000-000000000000'),
    adminIdentifier: env('HOOK_ADMIN_IDENTIFIER', 'admin'),
    adminPassword: env('HOOK_ADMIN_PASSWORD', '12345678'),
    db: Object.freeze({
      host: env('HOOK_DB_HOST', 'localhost'),
      port: env('HOOK_DB_PORT', '5433'),
      user: env('HOOK_DB_USER', 'postgres'),
      password: env('HOOK_DB_PASSWORD', '123456'),
      name: env('HOOK_DB_NAME', 'postgres'),
      psqlBin: env('PSQL_BIN', 'psql'),
    }),
    redis: Object.freeze({
      prefix: env('HOOK_REDIS_PREFIX', 'hook'),
      host: env('HOOK_REDIS_HOST', '127.0.0.1'),
      port: Number(env('HOOK_REDIS_PORT', '6380')),
    }),
    upstreams: Object.freeze({
      hookBaseUrl: env('REQ_REAL_HOOK_BASE_URL', 'https://www.hook.rs'),
      ekan8BaseUrl: env('REQ_REAL_EKAN8_BASE_URL', 'https://www.ekan8.com'),
    }),
    secrets: Object.freeze({
      hookKey: requiredEnv('REQ_REAL_HOOK_KEY'),
      ekan8Key: requiredEnv('REQ_REAL_EKAN8_KEY'),
    }),
  });
}

async function runScenario(label, action) {
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

async function probeUpstreams() {
  const [hookModels, ekan8Models, geminiModels] = await Promise.all([
    fetchOpenAiModels(ctx.upstreams.hookBaseUrl, ctx.secrets.hookKey),
    fetchOpenAiModels(ctx.upstreams.ekan8BaseUrl, ctx.secrets.ekan8Key),
    fetchGeminiModels(ctx.upstreams.ekan8BaseUrl, ctx.secrets.ekan8Key),
  ]);
  assertIncludes(hookModels.join(','), upstream.hookOpenAi, 'Hook.rs OpenAI models should include selected OpenAI model');
  assertIncludes(hookModels.join(','), upstream.hookClaude, 'Hook.rs models should include selected Claude model');
  assertIncludes(ekan8Models.join(','), upstream.ekan8Mapped, 'Ekan8 OpenAI-compatible models should include mapped Claude model');
  assertIncludes(geminiModels.join(','), upstream.ekan8Gemini, 'Ekan8 Gemini models should include selected Gemini model');
}

async function upstreamModelFetchScenario() {
  const hook = await adminGet(`/api/admin/providers/${ids.providerHook}/upstream-models`);
  const ekan8 = await adminGet(`/api/admin/providers/${ids.providerEkan8}/upstream-models`);
  assert(hook.models.includes(upstream.hookOpenAi), 'backend fetch should include Hook.rs OpenAI model');
  assert(ekan8.models.includes(upstream.ekan8Gemini), 'backend fetch should include Ekan8 Gemini model');
  return {
    hook: pickModels(hook.models, [upstream.hookOpenAi, upstream.hookClaude]),
    ekan8: pickModels(ekan8.models, [upstream.ekan8Gemini, upstream.ekan8Mapped]),
  };
}

async function hookOpenAiNonStreamScenario() {
  const marker = markerText('hook-openai-non-stream');
  const result = await openAiChat(models.openai, marker, false);
  assertEqual(result.status, 200, 'OpenAI non-stream response status');
  const body = parseJson(result.text, 'OpenAI non-stream body');
  assertEqual(body.model, models.openai, 'client response model should be rewritten to global model');
  return assertRecorded(result.requestId, {
    marker,
    providerId: ids.providerHook,
    providerFormat: 'openai_chat',
    clientFormat: 'openai_chat',
    modelName: models.openai,
    upstreamModel: upstream.hookOpenAi,
    stream: false,
    conversion: false,
  });
}

async function hookOpenAiStreamScenario() {
  const marker = markerText('hook-openai-stream');
  const result = await openAiChat(models.openai, marker, true);
  assertEqual(result.status, 200, 'OpenAI stream response status');
  assertIncludes(result.text, 'data:', 'OpenAI stream should return SSE chunks');
  return assertRecorded(result.requestId, {
    marker,
    providerId: ids.providerHook,
    providerFormat: 'openai_chat',
    clientFormat: 'openai_chat',
    modelName: models.openai,
    upstreamModel: upstream.hookOpenAi,
    stream: true,
    conversion: false,
  });
}

async function hookClaudeScenario() {
  const marker = markerText('hook-claude');
  const response = await proxyJson('/v1/messages', {
    model: models.claude,
    max_tokens: 16,
    temperature: 0,
    messages: [{ role: 'user', content: marker }],
  }, marker);
  assertEqual(response.status, 200, 'Claude response status');
  const body = parseJson(response.text, 'Claude body');
  assertEqual(body.model, models.claude, 'Claude response model should be rewritten to global model');
  return assertRecorded(response.requestId, {
    marker,
    providerId: ids.providerHook,
    providerFormat: 'claude_chat',
    clientFormat: 'claude_chat',
    modelName: models.claude,
    upstreamModel: upstream.hookClaude,
    stream: false,
    conversion: false,
  });
}

async function ekan8GeminiScenario() {
  const marker = markerText('ekan8-gemini');
  const path = `/v1beta/models/${encodeURIComponent(models.gemini)}:generateContent`;
  const response = await proxyJson(path, {
    contents: [{ role: 'user', parts: [{ text: marker }] }],
    generationConfig: { maxOutputTokens: 16, temperature: 0 },
  }, marker);
  assertEqual(response.status, 200, 'Gemini response status');
  assert(parseJson(response.text, 'Gemini body').candidates, 'Gemini response should include candidates');
  return assertRecorded(response.requestId, {
    marker,
    providerId: ids.providerEkan8,
    providerFormat: 'gemini_chat',
    clientFormat: 'gemini_chat',
    modelName: models.gemini,
    upstreamModel: null,
    stream: false,
    conversion: false,
  });
}

async function ekan8MappedScenario() {
  const marker = markerText('ekan8-mapped');
  const result = await openAiChat(models.mapped, marker, false);
  assertEqual(result.status, 200, 'Ekan8 mapped response status');
  const body = parseJson(result.text, 'Ekan8 mapped body');
  assertEqual(body.model, models.mapped, 'mapped client response model should be rewritten');
  const evidence = await assertRecorded(result.requestId, {
    marker,
    providerId: ids.providerEkan8,
    providerFormat: 'openai_chat',
    clientFormat: 'openai_chat',
    modelName: models.mapped,
    upstreamModel: upstream.ekan8Mapped,
    stream: false,
    conversion: false,
  });
  assertEqual(evidence.providerRequestBody.model, upstream.ekan8Mapped, 'mapped provider request should use Ekan8 upstream model');
  return evidence;
}

async function openAiChat(model, marker, stream) {
  return proxyJson('/v1/chat/completions', {
    model,
    messages: [{ role: 'user', content: marker }],
    max_tokens: 16,
    temperature: 0,
    stream,
  }, marker);
}

async function proxyJson(path, body, marker) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${proxyToken}`,
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  const requestId = await waitForLatestRequestId(before, marker);
  console.log(`${path}: status=${response.status} request_id=${requestId}`);
  if (!response.ok) {
    throw new Error(`${path} failed ${response.status}: ${text.slice(0, 1200)}`);
  }
  return { status: response.status, text, requestId };
}

async function assertRecorded(requestId, expected) {
  const record = await waitForRecordTerminal(requestId);
  assertEqual(record.status, 'success', 'request record should be success');
  assertEqual(record.client_status_code, '200', 'request record client status should be 200');
  assertEqual(record.provider_id, expected.providerId, 'request record provider id');
  assertEqual(record.client_api_format, expected.clientFormat, 'request record client format');
  assertEqual(record.provider_api_format, expected.providerFormat, 'request record provider format');
  assertEqual(record.is_stream, String(expected.stream), 'request record stream flag');
  assertEqual(record.model_name_snapshot, expected.modelName, 'request record model snapshot');
  const candidate = successCandidate(requestId);
  assertEqual(candidate.provider_id, expected.providerId, 'candidate provider id');
  assertEqual(candidate.provider_api_format, expected.providerFormat, 'candidate provider format');
  assertEqual(candidate.client_api_format, expected.clientFormat, 'candidate client format');
  assertEqual(candidate.is_stream, String(expected.stream), 'candidate stream flag');
  assertEqual(candidate.needs_conversion, String(expected.conversion), 'candidate conversion flag');
  assertEqual(candidate.status_code, '200', 'candidate status code should be 200');
  const providerRequestBody = parseNullableJson(candidate.provider_request_body);
  if (expected.upstreamModel) {
    assertEqual(providerRequestBody.model, expected.upstreamModel, 'provider request body should use upstream model');
  }
  if (expected.providerFormat === 'gemini_chat') {
    assert(!Object.hasOwn(providerRequestBody, 'model'), 'Gemini provider request body should not include model field');
  }
  assertIncludes(candidate.provider_request_body, expected.marker, 'candidate provider request body should contain marker text');
  return {
    requestId,
    record,
    candidate: omitLargePayloads(candidate),
    providerRequestBody,
  };
}

function seedFixtures() {
  insertMenuSections();
  seedGlobalModels();
  seedGroup();
  seedProviders();
  seedEndpoints();
  seedKeys();
  seedModelBindings();
  seedGroupBindings();
  applyFullRecordingSettings();
}

function insertMenuSections() {
  db.exec(`
insert into menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000101', 'overview', '概览', -10, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000102', 'operations', '运营管理', -5, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000103', 'system_management', '系统管理', 0, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00')
on conflict (id) do update set
  code = excluded.code,
  subheader = excluded.subheader,
  sort_order = excluded.sort_order,
  enabled = excluded.enabled,
  updated_at = excluded.updated_at;`);
}

function seedGlobalModels() {
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }));
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(ids.modelOpenAi)}, ${q(models.openai)}, ${q(models.openai)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(ids.modelClaude)}, ${q(models.claude)}, ${q(models.claude)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(ids.modelGemini)}, ${q(models.gemini)}, ${q(models.gemini)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(ids.modelMapped)}, ${q(models.mapped)}, ${q(models.mapped)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (id) do update set
  name = excluded.name,
  display_name = excluded.display_name,
  default_tiered_pricing = excluded.default_tiered_pricing,
  supported_capabilities = excluded.supported_capabilities,
  is_active = true,
  updated_at = now();`);
}

function seedGroup() {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(ids.group)}, ${q(groupCode)}, 'Req Real Test', null, 1, true, false, 0, now(), now())
on conflict (code) do update set
  name = excluded.name,
  billing_multiplier = excluded.billing_multiplier,
  is_active = true,
  updated_at = now();`);
}

function seedProviders() {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerHook)}, 'Req Real Hook.rs', 'custom', 0, 60, 60, 0, false, false, true, now(), now()),
  (${q(ids.providerEkan8)}, 'Req Real Ekan8', 'custom', 0, 60, 60, 10, false, false, true, now(), now())
on conflict (id) do update set
  name = excluded.name,
  provider_type = excluded.provider_type,
  max_retries = excluded.max_retries,
  request_timeout_seconds = excluded.request_timeout_seconds,
  stream_first_byte_timeout_seconds = excluded.stream_first_byte_timeout_seconds,
  priority = excluded.priority,
  keep_priority_on_conversion = excluded.keep_priority_on_conversion,
  enable_format_conversion = excluded.enable_format_conversion,
  is_active = true,
  updated_at = now();`);
}

function seedEndpoints() {
  db.exec(`
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointHookOpenAi)}, ${q(ids.providerHook)}, 'openai_chat', ${q(ctx.upstreams.hookBaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointHookClaude)}, ${q(ids.providerHook)}, 'claude_chat', ${q(ctx.upstreams.hookBaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointEkan8Gemini)}, ${q(ids.providerEkan8)}, 'gemini_chat', ${q(ctx.upstreams.ekan8BaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointEkan8OpenAi)}, ${q(ids.providerEkan8)}, 'openai_chat', ${q(ctx.upstreams.ekan8BaseUrl)}, null, 0, true, null, null, null, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id,
  api_format = excluded.api_format,
  base_url = excluded.base_url,
  custom_path = excluded.custom_path,
  max_retries = excluded.max_retries,
  is_active = true,
  format_acceptance_config = null,
  header_rules = null,
  body_rules = null,
  updated_at = now();`);
}

function seedKeys() {
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit, cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyHook)}, ${q(ids.providerHook)}, 'Req Hook key', ${q(encryptProviderKey(ctx.providerSecret, ctx.secrets.hookKey))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyEkan8)}, ${q(ids.providerEkan8)}, 'Req Ekan8 key', ${q(encryptProviderKey(ctx.providerSecret, ctx.secrets.ekan8Key))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id,
  name = excluded.name,
  encrypted_api_key = excluded.encrypted_api_key,
  internal_priority = excluded.internal_priority,
  cache_ttl_minutes = excluded.cache_ttl_minutes,
  is_active = true,
  updated_at = now();`);
}

function seedModelBindings() {
  db.exec(`
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(ids.bindingHookOpenAi)}, ${q(ids.providerHook)}, ${q(ids.modelOpenAi)}, ${q(upstream.hookOpenAi)}, null, true, null, null, null, now(), now()),
  (${q(ids.bindingHookClaude)}, ${q(ids.providerHook)}, ${q(ids.modelClaude)}, ${q(upstream.hookClaude)}, null, true, null, null, null, now(), now()),
  (${q(ids.bindingEkan8Gemini)}, ${q(ids.providerEkan8)}, ${q(ids.modelGemini)}, ${q(upstream.ekan8Gemini)}, null, true, null, null, null, now(), now()),
  (${q(ids.bindingEkan8Mapped)}, ${q(ids.providerEkan8)}, ${q(ids.modelMapped)}, 'logical-ekan8-openai', ${q(JSON.stringify({ name: upstream.ekan8Mapped }))}, true, null, null, null, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id,
  global_model_id = excluded.global_model_id,
  provider_model_name = excluded.provider_model_name,
  provider_model_mappings = excluded.provider_model_mappings,
  is_active = true,
  updated_at = now();`);
}

function seedGroupBindings() {
  db.exec(`
delete from billing_group_providers where group_code = ${q(groupCode)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  (${q(ids.groupProviderHook)}, ${q(groupCode)}, ${q(ids.providerHook)}, now(), now()),
  (${q(ids.groupProviderEkan8)}, ${q(groupCode)}, ${q(ids.providerEkan8)}, now(), now());

delete from billing_group_models where group_code = ${q(groupCode)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  (${q(ids.groupModelOpenAi)}, ${q(groupCode)}, ${q(ids.modelOpenAi)}, now(), now()),
  (${q(ids.groupModelClaude)}, ${q(groupCode)}, ${q(ids.modelClaude)}, now(), now()),
  (${q(ids.groupModelGemini)}, ${q(groupCode)}, ${q(ids.modelGemini)}, now(), now()),
  (${q(ids.groupModelMapped)}, ${q(groupCode)}, ${q(ids.modelMapped)}, now(), now());`);
}

function applyFullRecordingSettings() {
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
where id = 'global';`);
}

async function createProxyToken() {
  const created = await adminPost('/api/admin/tokens', {
    name: `req-real-${Date.now()}`,
    token_type: 'independent',
    group_code: groupCode,
    model_access_mode: 'all',
    allowed_model_ids: [],
    rate_limit_rpm: 0,
  });
  assert(created.token?.id, 'admin token create should return token id');
  assert(created.raw_token, 'admin token create should return raw token');
  proxyTokenId = created.token.id;
  return created.raw_token;
}

async function cleanup() {
  try {
    if (proxyTokenId) {
      await adminDelete(`/api/admin/tokens/${encodeURIComponent(proxyTokenId)}`).catch((error) => {
        console.warn(`failed to delete generated token: ${error.message}`);
      });
    }
    deactivateFixtures();
    if (originalSettings) restoreSystemSettings(originalSettings);
    await clearProxyCaches();
  } finally {
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

function deactivateFixtures() {
  db.exec(`
update providers set is_active = false, updated_at = now() where id in (${q(ids.providerHook)}, ${q(ids.providerEkan8)});
update global_models set is_active = false, updated_at = now() where id in (${q(ids.modelOpenAi)}, ${q(ids.modelClaude)}, ${q(ids.modelGemini)}, ${q(ids.modelMapped)});
update billing_groups set is_active = false, updated_at = now() where code = ${q(groupCode)};
update api_tokens set is_active = false, updated_at = now() where group_code = ${q(groupCode)};`);
}

function systemSettingsSnapshot() {
  const [row] = db.rows(`
select request_record_level, request_record_retention_days::text, request_record_payload_retention_days::text,
  record_request_headers::text, record_request_body::text, record_response_body::text,
  max_request_body_size_kb::text, max_response_body_size_kb::text, scheduling_mode
from system_settings where id = 'global';`);
  assert(row, 'system_settings global row should exist');
  return {
    request_record_level: row[0],
    request_record_retention_days: Number(row[1]),
    request_record_payload_retention_days: Number(row[2]),
    record_request_headers: row[3] === 't',
    record_request_body: row[4] === 't',
    record_response_body: row[5] === 't',
    max_request_body_size_kb: Number(row[6]),
    max_response_body_size_kb: Number(row[7]),
    scheduling_mode: row[8],
  };
}

function restoreSystemSettings(snapshot) {
  db.exec(`
update system_settings
set request_record_level = ${q(snapshot.request_record_level)},
    request_record_retention_days = ${snapshot.request_record_retention_days},
    request_record_payload_retention_days = ${snapshot.request_record_payload_retention_days},
    record_request_headers = ${snapshot.record_request_headers ? 'true' : 'false'},
    record_request_body = ${snapshot.record_request_body ? 'true' : 'false'},
    record_response_body = ${snapshot.record_response_body ? 'true' : 'false'},
    max_request_body_size_kb = ${snapshot.max_request_body_size_kb},
    max_response_body_size_kb = ${snapshot.max_response_body_size_kb},
    scheduling_mode = ${q(snapshot.scheduling_mode)},
    updated_at = now()
where id = 'global';`);
}

async function clearProxyCaches() {
  const keys = await redis.keys(`${ctx.redis.prefix}:llm_proxy:*`);
  await redis.del(...keys);
}

async function adminGet(path) {
  return adminRequest('GET', path);
}

async function adminPost(path, body) {
  return adminRequest('POST', path, body);
}

async function adminDelete(path) {
  return adminRequest('DELETE', path);
}

async function adminRequest(method, path, body) {
  const headers = { authorization: `Bearer ${adminToken}` };
  if (body !== undefined) headers['content-type'] = 'application/json';
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, {
    method,
    headers,
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`admin ${method} ${path} failed ${response.status}: ${text}`);
  if (!text.trim()) return null;
  const parsed = JSON.parse(text);
  assert(parsed.success, `admin ${method} ${path} should return success envelope`);
  return parsed.data;
}

async function fetchOpenAiModels(baseUrl, key) {
  const response = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/models`, {
    headers: { authorization: `Bearer ${key}` },
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`GET ${baseUrl}/v1/models failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  return (body.data ?? []).map((item) => item.id).filter(Boolean);
}

async function fetchGeminiModels(baseUrl, key) {
  const url = new URL(`${baseUrl.replace(/\/$/, '')}/v1beta/models`);
  url.searchParams.set('key', key);
  const response = await fetch(url);
  const text = await response.text();
  if (!response.ok) throw new Error(`GET ${baseUrl}/v1beta/models failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  return (body.models ?? []).map((item) => String(item.name ?? '').split('/').pop()).filter(Boolean);
}

async function waitForLatestRequestId(beforeIso, marker, timeoutMs = 30_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const requestId = db.scalar(`
select request_id
from (
  select request_id, created_at
  from request_candidates
  where created_at >= ${q(beforeIso)}
    and token_id = ${q(proxyTokenId)}
    and coalesce(provider_request_body, '') like ${q(`%${marker}%`)}
  union all
  select request_id, created_at
  from request_records
  where created_at >= ${q(beforeIso)}
    and token_id = ${q(proxyTokenId)}
    and coalesce(request_body, '') like ${q(`%${marker}%`)}
) request_matches
order by created_at desc
limit 1;`);
    if (requestId) return requestId;
    await sleep(250);
  }
  throw new Error(`request id not found for marker: ${marker}`);
}

async function waitForRecordTerminal(requestId, timeoutMs = 30_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const record = requestRecord(requestId);
    if (!['pending', 'streaming'].includes(record.status)) return record;
    await sleep(250);
  }
  throw new Error(`request record did not reach terminal status: ${requestId}`);
}

function requestRecord(requestId) {
  const [row] = db.rows(`
select request_id, status, coalesce(client_status_code::text, ''), coalesce(provider_id, ''),
  client_api_format, coalesce(provider_api_format, ''), is_stream::text, coalesce(model_name_snapshot, '')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
  assert(row, `request record should exist: ${requestId}`);
  return {
    request_id: row[0],
    status: row[1],
    client_status_code: row[2],
    provider_id: row[3],
    client_api_format: row[4],
    provider_api_format: row[5],
    is_stream: row[6],
    model_name_snapshot: row[7],
  };
}

function successCandidate(requestId) {
  const [row] = db.rows(`
select status, coalesce(status_code::text, ''), coalesce(provider_id, ''), client_api_format,
  coalesce(provider_api_format, ''), needs_conversion::text, is_stream::text,
  coalesce(provider_request_body, ''), coalesce(provider_response_body, ''),
  coalesce(latency_ms::text, ''), coalesce(first_byte_time_ms::text, '')
from request_candidates
where request_id = ${q(requestId)}
  and status = 'success'
order by candidate_index, retry_index
limit 1;`);
  assert(row, `successful candidate should exist: ${requestId}`);
  return {
    status: row[0],
    status_code: row[1],
    provider_id: row[2],
    client_api_format: row[3],
    provider_api_format: row[4],
    needs_conversion: row[5],
    is_stream: row[6],
    provider_request_body: row[7],
    provider_response_body: row[8],
    latency_ms: row[9],
    first_byte_time_ms: row[10],
  };
}

function parseNullableJson(value) {
  if (!value) return null;
  return JSON.parse(value);
}

function parseJson(value, label) {
  try {
    return JSON.parse(value);
  } catch (error) {
    throw new Error(`${label} is not JSON: ${error.message}; body=${value.slice(0, 500)}`);
  }
}

function omitLargePayloads(candidate) {
  return {
    status: candidate.status,
    status_code: candidate.status_code,
    provider_id: candidate.provider_id,
    client_api_format: candidate.client_api_format,
    provider_api_format: candidate.provider_api_format,
    needs_conversion: candidate.needs_conversion,
    is_stream: candidate.is_stream,
    latency_ms: candidate.latency_ms,
    first_byte_time_ms: candidate.first_byte_time_ms,
  };
}

function markerText(label) {
  return `req-real-${label}-${Date.now()}`;
}

function pickModels(values, preferred) {
  return preferred.filter((value) => values.includes(value));
}

function failedSummary() {
  return `failed scenarios: ${results.filter((item) => !item.ok).map((item) => item.label).join(', ')}`;
}

function writeResults() {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

function env(name, fallback) {
  return process.env[name] || fallback;
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) throw new Error(`missing required env: ${name}`);
  return value;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
