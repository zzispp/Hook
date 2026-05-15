import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { encryptProviderKey, sha256 } from '../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { Db, q } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { startBackend, stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { clearAuth, clearScheduling } from '../20260514-real-request-record-flow/lib/request_record_real_support.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const ids = Object.freeze({
  providerMsutools: '00000000-0000-7000-9400-000000000101',
  providerEkan8: '00000000-0000-7000-9400-000000000102',
  keyMsutools: '00000000-0000-7000-9400-000000000201',
  keyEkan8: '00000000-0000-7000-9400-000000000202',
  endpointMsutools: '00000000-0000-7000-9400-000000000301',
  endpointEkan8: '00000000-0000-7000-9400-000000000302',
  model: '00000000-0000-7000-9400-000000000701',
  group: '00000000-0000-7000-9400-000000000801',
  user: '00000000-0000-7000-9400-000000000901',
  wallet: '00000000-0000-7000-9400-000000000951',
  token: '00000000-0000-7000-9400-000000000501',
});

const groupCode = 'real_billing_details';
const modelName = process.env.HOOK_REAL_BILLING_MODEL || 'hook-real-billing-details';
const customerToken = process.env.HOOK_REAL_BILLING_TOKEN || 'sk-real-billing-details-local-token';
const serviceTier = process.env.HOOK_REAL_BILLING_SERVICE_TIER || 'standard';
const provider1Base = process.env.HOOK_REAL_PROVIDER1_BASE || 'https://www.msutools.cn';
const provider2Base = process.env.HOOK_REAL_PROVIDER2_BASE || 'https://www.ekan8.com';
const results = [];

let ctx;
let db;
let redis;
let backend;

async function main() {
  ctx = loadContextWithSyntheticSecrets();
  db = new Db(ctx.db);
  redis = new RedisClient(ctx.redis);

  const upstream = await step('resolve upstream models', resolveUpstreamModels);
  try {
    await step('seed local DB fixtures', () => seedFixtures(upstream));
    await clearScheduling(redis, ctx.redis.prefix);
    await clearAuth(redis, ctx.redis.prefix);
    backend = await startBackend(ctx.serverBaseUrl);
    const requestId = await step('send real proxy request', sendRealProxyRequest);
    await step('verify billing detail records', () => verifyBillingDetails(requestId));
    console.log('real billing details flow: passed');
  } finally {
    await cleanup();
    writeResults();
  }
}

async function resolveUpstreamModels() {
  const provider1Key = requiredEnv('HOOK_REAL_PROVIDER1_KEY');
  const provider2Key = requiredEnv('HOOK_REAL_PROVIDER2_KEY');
  const msutoolsModels = await fetchOpenAiModels(provider1Base, provider1Key);
  const ekan8Models = await fetchGeminiModels(provider2Base, provider2Key);
  const msutoolsModel = chooseModel(msutoolsModels, [process.env.HOOK_REAL_PROVIDER1_MODEL, 'gpt-5.4-mini', 'gpt-5.4', 'gpt-5.5']);
  const ekan8Model = chooseModel(ekan8Models, [process.env.HOOK_REAL_EKAN8_MODEL, 'gemini-3.1-pro-preview', '[满血]gemini-3.1-pro-preview']);
  assert(msutoolsModel, `msutools model list has no usable model: ${msutoolsModels.slice(0, 12).join(', ')}`);
  assert(ekan8Model, `ekan8 model list has no usable model: ${ekan8Models.slice(0, 12).join(', ')}`);
  return {
    msutoolsModel,
    ekan8Model,
    msutoolsSample: sampleModels(msutoolsModels, msutoolsModel),
    ekan8Sample: sampleModels(ekan8Models, ekan8Model),
  };
}

async function fetchOpenAiModels(baseUrl, key) {
  const response = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/models`, {
    headers: { authorization: `Bearer ${key}` },
  });
  return parseModelResponse(response, 'openai_chat');
}

async function fetchGeminiModels(baseUrl, key) {
  const url = new URL(`${baseUrl.replace(/\/$/, '')}/v1beta/models`);
  url.searchParams.set('key', key);
  const response = await fetch(url);
  return parseModelResponse(response, 'gemini_chat');
}

async function parseModelResponse(response, apiFormat) {
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`upstream model fetch failed ${response.status}: ${text.slice(0, 500)}`);
  }
  return extractModelNames(text.trim() ? JSON.parse(text) : null, apiFormat);
}

function seedFixtures(upstream) {
  ensureBillingColumns();
  cleanupRows();
  seedRequestedMenuSections();
  seedUserWalletToken();
  seedModelGroupProviders(upstream);
  db.exec(`update system_settings set scheduling_mode = 'fixed_order', request_record_level = 'full', record_request_body = true, record_response_body = true, updated_at = now() where id = 'global';`);
  return { modelName, groupCode, upstream };
}

function ensureBillingColumns() {
  for (const table of ['request_records', 'request_candidates']) {
    const count = Number(db.scalar(`
select count(*)::text from information_schema.columns
where table_schema = 'public' and table_name = ${q(table)}
  and column_name in ('service_tier','input_cost','output_cost','cache_creation_cost','cache_read_cost','request_cost','input_price_per_million','output_price_per_million','cache_creation_price_per_million','cache_read_price_per_million');`));
    assert(count === 10, `${table} should have all billing detail columns before real test`);
  }
}

function seedRequestedMenuSections() {
  db.exec(`
insert into menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000101', 'overview', '概览', -10, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000102', 'operations', '运营管理', -5, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000103', 'system_management', '系统管理', 0, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00')
on conflict (id) do update set code = excluded.code, subheader = excluded.subheader, sort_order = excluded.sort_order, enabled = excluded.enabled, updated_at = excluded.updated_at;`);
}

function seedUserWalletToken() {
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids,
   created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
  (${q(ids.user)}, 'real_billing_user', ${q(passwordHash())}, 'real_billing_user@example.com', 'user', true, false, '[]', '[]',
   now(), now(), null, 'local', true, 0, 'wallet')
on conflict (id) do update set is_active = true, is_deleted = false, quota_mode = 'wallet', updated_at = now();
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
   total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
  (${q(ids.wallet)}, ${q(ids.user)}, 0, 25, 'USD', 'active', 'finite', 0, 0, 0, 25, now(), now())
on conflict (user_id) do update set gift_balance = 25, status = 'active', limit_mode = 'finite', updated_at = now();
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
  (${q(ids.token)}, ${q(ids.user)}, 'user', 'Real billing details token', ${q(customerToken)}, ${q(sha256(customerToken))},
   ${q(customerToken.slice(0, 10))}, ${q(groupCode)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now())
on conflict (id) do update set token_value = excluded.token_value, token_hash = excluded.token_hash, group_code = excluded.group_code, is_active = true, updated_at = now();`);
}

function seedModelGroupProviders(upstream) {
  const provider1Key = encryptProviderKey(ctx.providerSecret, requiredEnv('HOOK_REAL_PROVIDER1_KEY'));
  const provider2Key = encryptProviderKey(ctx.providerSecret, requiredEnv('HOOK_REAL_PROVIDER2_KEY'));
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 2.5, output_price_per_1m: 15, cache_creation_price_per_1m: 1.25, cache_read_price_per_1m: 0.25 }] }));
  const msutoolsMapping = q(JSON.stringify({ name: upstream.msutoolsModel }));
  const ekan8Mapping = q(JSON.stringify({ name: upstream.ekan8Model }));
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(ids.model)}, ${q(modelName)}, ${q(modelName)}, 0, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (id) do update set name = excluded.name, display_name = excluded.display_name, default_price_per_request = excluded.default_price_per_request,
  default_tiered_pricing = excluded.default_tiered_pricing, supported_capabilities = excluded.supported_capabilities, is_active = true, updated_at = now();
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(ids.group)}, ${q(groupCode)}, 'Real Billing Details', null, 0.15, true, false, 0, now(), now())
on conflict (code) do update set billing_multiplier = excluded.billing_multiplier, is_active = true, updated_at = now();
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds,
   priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerMsutools)}, 'Real Billing MSUTools', 'custom', 0, 120, 120, 0, false, true, true, now(), now()),
  (${q(ids.providerEkan8)}, 'Real Billing Ekan8', 'custom', 0, 120, 120, 10, true, true, true, now(), now())
on conflict (id) do update set provider_type = excluded.provider_type, priority = excluded.priority, is_active = true, updated_at = now();
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointMsutools)}, ${q(ids.providerMsutools)}, 'openai_chat', ${q(provider1Base)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointEkan8)}, ${q(ids.providerEkan8)}, 'gemini_chat', ${q(provider2Base)}, null, 0, true, null, null, null, now(), now())
on conflict (id) do update set base_url = excluded.base_url, is_active = true, updated_at = now();
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end,
   health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyMsutools)}, ${q(ids.providerMsutools)}, 'Real Billing MSUTools key', ${q(provider1Key)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyEkan8)}, ${q(ids.providerEkan8)}, 'Real Billing Ekan8 key', ${q(provider2Key)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set encrypted_api_key = excluded.encrypted_api_key, is_active = true, updated_at = now();
delete from provider_models where id in ('00000000-0000-7000-9400-000000000401', '00000000-0000-7000-9400-000000000402');
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  ('00000000-0000-7000-9400-000000000401', ${q(ids.providerMsutools)}, ${q(ids.model)}, ${q(modelName)}, ${msutoolsMapping}, true, 0, ${pricing}, null, now(), now()),
  ('00000000-0000-7000-9400-000000000402', ${q(ids.providerEkan8)}, ${q(ids.model)}, ${q(modelName)}, ${ekan8Mapping}, true, 0, ${pricing}, null, now(), now());
delete from billing_group_providers where group_code = ${q(groupCode)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9400-000000000601', ${q(groupCode)}, ${q(ids.providerMsutools)}, now(), now()),
  ('00000000-0000-7000-9400-000000000602', ${q(groupCode)}, ${q(ids.providerEkan8)}, now(), now());
delete from billing_group_models where group_code = ${q(groupCode)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values ('00000000-0000-7000-9400-000000000611', ${q(groupCode)}, ${q(ids.model)}, now(), now());`);
}

async function sendRealProxyRequest() {
  const marker = `real-billing-details-${Date.now()}`;
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await fetch(`${ctx.serverBaseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { authorization: `Bearer ${customerToken}`, 'content-type': 'application/json' },
    body: JSON.stringify({
      model: modelName,
      service_tier: serviceTier,
      messages: [{ role: 'user', content: `Return one short sentence. marker=${marker}` }],
      max_tokens: 16,
      temperature: 0,
    }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`proxy request failed ${response.status}: ${redact(text).slice(0, 1000)}`);
  }
  const requestId = await waitForLatestRequestId(before, marker);
  return { requestId, status: response.status, body: summarizeBody(text) };
}

async function waitForLatestRequestId(beforeIso, marker) {
  const started = Date.now();
  while (Date.now() - started < 15_000) {
    const requestId = db.scalar(`
select request_id
from request_records
where created_at >= ${q(beforeIso)}
  and token_id = ${q(ids.token)}
  and request_body like ${q(`%${marker}%`)}
order by created_at desc
limit 1;`);
    if (requestId) {
      await waitForTerminalRecord(requestId);
      return requestId;
    }
    await sleep(250);
  }
  throw new Error('request record was not created');
}

async function waitForTerminalRecord(requestId) {
  const started = Date.now();
  while (Date.now() - started < 30_000) {
    const status = db.scalar(`select status from request_records where request_id = ${q(requestId)};`);
    if (status && !['pending', 'streaming'].includes(status)) return;
    await sleep(250);
  }
  throw new Error(`request record did not reach terminal state: ${requestId}`);
}

function verifyBillingDetails(input) {
  const requestId = typeof input === 'string' ? input : input.requestId;
  const summary = billingRow('request_records', requestId);
  const candidate = billingRow('request_candidates', requestId);
  assert(summary.status === 'success', `request_records status should be success, got ${summary.status}`);
  assert(summary.billing_status === 'settled', `request_records billing_status should be settled, got ${summary.billing_status}`);
  assert(candidate.status === 'success', `request_candidates status should be success, got ${candidate.status}`);
  assert(summary.service_tier === serviceTier, `summary service_tier should be ${serviceTier}, got ${summary.service_tier}`);
  assert(candidate.service_tier === serviceTier, `candidate service_tier should be ${serviceTier}, got ${candidate.service_tier}`);
  for (const row of [summary, candidate]) {
    assert(Number(row.prompt_tokens) > 0, `${row.source} prompt_tokens should be recorded`);
    assert(Number(row.completion_tokens) > 0, `${row.source} completion_tokens should be recorded`);
    assertEqualMoney(row.input_price_per_million, '2.50000000', `${row.source} input price`);
    assertEqualMoney(row.output_price_per_million, '15.00000000', `${row.source} output price`);
    assertEqualMoney(row.cache_creation_price_per_million, '1.25000000', `${row.source} cache creation price`);
    assertEqualMoney(row.cache_read_price_per_million, '0.25000000', `${row.source} cache read price`);
    assert(row.cost_currency === 'USD', `${row.source} cost_currency should use project accounting currency USD, got ${row.cost_currency}`);
    assert(Number(row.input_cost) > 0, `${row.source} input_cost should be positive`);
    assert(Number(row.output_cost) > 0, `${row.source} output_cost should be positive`);
    assertEqualMoney(row.request_cost, '0.00000000', `${row.source} request_cost`);
    assertEqualMoney(row.billing_multiplier, '0.15000000', `${row.source} billing_multiplier`);
    assertEqualMoney(row.total_cost, multiply(row.base_cost, '0.15'), `${row.source} billed cost`);
  }
  return { requestId, summary, candidate };
}

function billingRow(table, requestId) {
  const orderBy = table === 'request_candidates' ? 'candidate_index nulls first, retry_index nulls first' : 'created_at desc';
  const billingStatusSql = table === 'request_candidates' ? "''" : 'billing_status';
  const rows = db.rows(`
select ${q(table)}, request_id, status, ${billingStatusSql}, coalesce(service_tier, ''),
  coalesce(prompt_tokens::text, ''), coalesce(completion_tokens::text, ''), coalesce(cache_read_input_tokens::text, ''),
  coalesce(input_cost::text, ''), coalesce(output_cost::text, ''), coalesce(cache_creation_cost::text, ''),
  coalesce(cache_read_cost::text, ''), coalesce(request_cost::text, ''), coalesce(base_cost::text, ''),
  coalesce(total_cost::text, ''), coalesce(billing_multiplier::text, ''), coalesce(cost_currency, ''),
  coalesce(input_price_per_million::text, ''), coalesce(output_price_per_million::text, ''),
  coalesce(cache_creation_price_per_million::text, ''), coalesce(cache_read_price_per_million::text, '')
from ${table}
where request_id = ${q(requestId)} ${table === 'request_candidates' ? "and status = 'success'" : ''}
order by ${orderBy}
limit 1;`);
  assert(rows.length === 1, `${table} billing row should exist for ${requestId}`);
  const [
    source,
    rowRequestId,
    status,
    billingStatus,
    rowServiceTier,
    promptTokens,
    completionTokens,
    cacheReadTokens,
    inputCost,
    outputCost,
    cacheCreationCost,
    cacheReadCost,
    requestCost,
    baseCost,
    totalCost,
    multiplier,
    costCurrency,
    inputPrice,
    outputPrice,
    cacheCreationPrice,
    cacheReadPrice,
  ] = rows[0];
  return {
    source,
    request_id: rowRequestId,
    status,
    billing_status: billingStatus,
    service_tier: rowServiceTier,
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
    cache_read_input_tokens: cacheReadTokens,
    input_cost: inputCost,
    output_cost: outputCost,
    cache_creation_cost: cacheCreationCost,
    cache_read_cost: cacheReadCost,
    request_cost: requestCost,
    base_cost: baseCost,
    total_cost: totalCost,
    billing_multiplier: multiplier,
    cost_currency: costCurrency,
    input_price_per_million: inputPrice,
    output_price_per_million: outputPrice,
    cache_creation_price_per_million: cacheCreationPrice,
    cache_read_price_per_million: cacheReadPrice,
  };
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence: redactEvidence(evidence) });
    console.log(`scenario passed: ${label}`);
    return evidence;
  } catch (error) {
    results.push({ label, ok: false, error: redact(error.stack || error.message) });
    console.error(`scenario failed: ${label}: ${redact(error.stack || error.message)}`);
    throw error;
  }
}

async function cleanup() {
  try {
    cleanupRows();
    if (redis && ctx) {
      await clearScheduling(redis, ctx.redis.prefix);
      await clearAuth(redis, ctx.redis.prefix);
    }
  } finally {
    if (ctx) {
      await stopBackend(backend, ctx.serverBaseUrl);
    }
  }
}

function cleanupRows() {
  if (!db) return;
  const requestIds = db.rows(`select request_id from request_records where token_id = ${q(ids.token)};`).map(([id]) => id);
  if (requestIds.length > 0) {
    db.exec(`
delete from request_candidates where request_id in (${requestIds.map(q).join(',')});
delete from request_records where request_id in (${requestIds.map(q).join(',')});
delete from wallet_transactions where link_type = 'llm_request_record' and link_id in (${requestIds.map(q).join(',')});`);
  }
  db.exec(`
delete from api_tokens where id = ${q(ids.token)};
delete from billing_group_models where group_code = ${q(groupCode)};
delete from billing_group_providers where group_code = ${q(groupCode)};
delete from provider_models where provider_id in (${q(ids.providerMsutools)}, ${q(ids.providerEkan8)});
delete from provider_api_keys where provider_id in (${q(ids.providerMsutools)}, ${q(ids.providerEkan8)});
delete from provider_endpoints where provider_id in (${q(ids.providerMsutools)}, ${q(ids.providerEkan8)});
delete from providers where id in (${q(ids.providerMsutools)}, ${q(ids.providerEkan8)});
delete from global_models where id = ${q(ids.model)};
delete from billing_groups where code = ${q(groupCode)};
delete from wallet_transactions where wallet_id = ${q(ids.wallet)};
delete from wallets where user_id = ${q(ids.user)};
update users set is_active = false, is_deleted = true, updated_at = now() where id = ${q(ids.user)};`);
}

function loadContextWithSyntheticSecrets() {
  const original = snapshotEnv(['HOOK_SYSTEM_TOKEN', 'HOOK_POOL_KEY', 'EKAN8_KEY', 'CLAUDE_KEY']);
  process.env.HOOK_SYSTEM_TOKEN ||= 'sk-real-billing-system-placeholder';
  process.env.HOOK_POOL_KEY ||= requiredEnv('HOOK_REAL_PROVIDER1_KEY');
  process.env.EKAN8_KEY ||= requiredEnv('HOOK_REAL_PROVIDER2_KEY');
  process.env.CLAUDE_KEY ||= requiredEnv('HOOK_REAL_PROVIDER2_KEY');
  try {
    return loadContext();
  } finally {
    restoreEnv(original);
  }
}

function extractModelNames(value, apiFormat) {
  const names = new Set();
  for (const item of modelItems(value)) {
    const raw = typeof item === 'string' ? item : item?.id || item?.name;
    if (!raw || typeof raw !== 'string') continue;
    const name = apiFormat === 'gemini_chat' ? raw.trim().split('/').pop() : raw.trim();
    if (name) names.add(name);
  }
  return [...names].sort();
}

function modelItems(value) {
  if (Array.isArray(value)) return value;
  if (value && typeof value === 'object') return Array.isArray(value.data) ? value.data : Array.isArray(value.models) ? value.models : [];
  return [];
}

function chooseModel(models, preferred) {
  for (const name of preferred.filter(Boolean)) {
    if (models.includes(name)) return name;
  }
  return models.find((name) => /gpt|gemini|claude/i.test(name)) || models[0] || '';
}

function sampleModels(models, selected) {
  return [...new Set([selected, ...models.slice(0, 8)].filter(Boolean))];
}

function assertEqualMoney(actual, expected, label) {
  const normalized = decimalToScale(actual, 8);
  assert(normalized === expected, `${label}: expected ${expected}, got ${actual}`);
}

function multiply(amount, multiplier) {
  return multiplyDecimals(amount, multiplier, 8);
}

function multiplyDecimals(left, right, scale) {
  const parsedLeft = parseDecimal(left);
  const parsedRight = parseDecimal(right);
  const value = parsedLeft.value * parsedRight.value;
  const sourceScale = parsedLeft.scale + parsedRight.scale;
  return formatScaled(roundToScale(value, sourceScale, scale), scale);
}

function decimalToScale(value, scale) {
  const parsed = parseDecimal(value);
  return formatScaled(roundToScale(parsed.value, parsed.scale, scale), scale);
}

function parseDecimal(value) {
  const text = String(value);
  const match = text.match(/^(-?)(\d+)(?:\.(\d+))?$/);
  assert(match, `invalid decimal: ${text}`);
  const [, sign, whole, fraction = ''] = match;
  const numeric = BigInt(`${whole}${fraction}`);
  return { value: sign ? -numeric : numeric, scale: fraction.length };
}

function roundToScale(value, sourceScale, targetScale) {
  if (sourceScale <= targetScale) {
    return value * 10n ** BigInt(targetScale - sourceScale);
  }
  const factor = 10n ** BigInt(sourceScale - targetScale);
  const quotient = value / factor;
  const remainder = value % factor;
  const half = factor / 2n;
  if (remainder >= half) return quotient + 1n;
  if (remainder <= -half) return quotient - 1n;
  return quotient;
}

function formatScaled(value, scale) {
  const negative = value < 0n;
  const absolute = negative ? -value : value;
  const factor = 10n ** BigInt(scale);
  const whole = absolute / factor;
  const fraction = String(absolute % factor).padStart(scale, '0');
  return `${negative ? '-' : ''}${whole}.${fraction}`;
}

function summarizeBody(text) {
  try {
    const body = JSON.parse(text);
    return { id: body.id, model: body.model, usage: body.usage };
  } catch {
    return { text: text.slice(0, 200) };
  }
}

function redactEvidence(value) {
  return JSON.parse(JSON.stringify(value, (key, item) => (secretKey(key) || secretValue(item) ? '[redacted]' : item)));
}

function secretKey(key) {
  return ['api_key', 'authorization', 'token_value', 'token_hash', 'token_prefix'].some((part) => key.toLowerCase().includes(part));
}

function secretValue(value) {
  return typeof value === 'string' && secretValues().includes(value);
}

function redact(value) {
  return secretValues().reduce((text, secret) => String(text).replaceAll(secret, '[redacted]'), String(value));
}

function secretValues() {
  return [process.env.HOOK_REAL_PROVIDER1_KEY, process.env.HOOK_REAL_PROVIDER2_KEY, customerToken].filter(Boolean);
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) throw new Error(`missing required env: ${name}`);
  return value;
}

function snapshotEnv(names) {
  return Object.fromEntries(names.map((name) => [name, process.env[name]]));
}

function restoreEnv(snapshot) {
  for (const [name, value] of Object.entries(snapshot)) {
    if (value === undefined) delete process.env[name];
    else process.env[name] = value;
  }
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
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
  console.error(redact(error.stack || error.message));
  process.exit(1);
});
