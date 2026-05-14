import { assert, assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
export function contextWithExistingModels(ctx, database) {
  const openai = selectedModel(database, process.env.HOOK_OPENAI_MODEL, (name) => name.startsWith('gpt-'), 'OpenAI');
  const claude = selectedModel(database, process.env.HOOK_CLAUDE_MODEL, (name) => name.startsWith('claude-'), 'Claude');
  const gemini = selectedModel(database, process.env.HOOK_GEMINI_MODEL, (name) => name.includes('gemini'), 'Gemini');
  return Object.freeze({
    ...ctx,
    models: Object.freeze({
      openai,
      claude,
      gemini,
      openaiProvider: process.env.HOOK_OPENAI_PROVIDER_MODEL || openai,
      claudeProvider: process.env.HOOK_CLAUDE_PROVIDER_MODEL || claude,
      geminiProvider: process.env.HOOK_GEMINI_PROVIDER_MODEL || geminiProviderName(gemini),
    }),
  });
}
export function systemSettingsSnapshot(db) {
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
export function applyFullRecordingSettings(db) {
  db.exec(`
update system_settings
set request_record_level = 'full',
    record_request_headers = true,
    record_request_body = true,
    record_response_body = true,
    max_request_body_size_kb = 5120,
    max_response_body_size_kb = 5120,
    updated_at = now()
where id = 'global';`);
}

export function restoreSystemSettings(db, snapshot) {
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

export function clearTestRequestRows(db, tokenIds) {
  const ids = Object.values(tokenIds).map(q).join(',');
  db.exec(`
delete from request_records
where request_id in (select distinct request_id from request_candidates where token_id in (${ids}));
delete from request_candidates where token_id in (${ids});`);
}

export async function clearScheduling(redis, prefix) {
  await redis.del(`${prefix}:llm_proxy:scheduling:snapshot:v2`, `${prefix}:llm_proxy:scheduling:rebuild_lock`);
}

export async function clearAuth(redis, prefix) {
  const keys = await redis.keys(`${prefix}:llm_proxy:auth:v*`);
  await redis.del(...keys, `${prefix}:llm_proxy:auth:version`);
}

export async function clearAffinity(redis, prefix, tokenIds, modelIds) {
  const keys = [
    affinityKey(prefix, tokenIds.openaiOnly, modelIds.openai, 'openai_chat'),
    affinityKey(prefix, tokenIds.openaiOnly, modelIds.openai, 'openai_cli'),
    affinityKey(prefix, tokenIds.claudeOnly, modelIds.claude, 'openai_chat'),
    affinityKey(prefix, tokenIds.geminiOnly, modelIds.gemini, 'openai_chat'),
    affinityKey(prefix, tokenIds.geminiOnly, modelIds.gemini, 'gemini_chat'),
    affinityKey(prefix, tokenIds.unrestricted, modelIds.openai, 'openai_chat'),
  ];
  await redis.del(...keys);
}

export async function waitForRequestRecords(db, requestIds, timeoutMs = 10_000) {
  const expected = [...new Set(requestIds)].filter(Boolean);
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const count = Number(db.scalar(`select count(*) from request_records where request_id in (${expected.map(q).join(',')});`));
    if (count === expected.length) return;
    await sleep(200);
  }
  throw new Error(`missing request_records rows for ${expected.length} request ids`);
}

export async function waitForRecordTerminal(db, requestId, timeoutMs = 20_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const row = requestRecordRow(db, requestId);
    if (!['pending', 'streaming'].includes(row.status)) return row;
    await sleep(250);
  }
  throw new Error(`request record did not reach terminal state: ${requestId}`);
}

export async function waitForRecordStatus(db, requestId, expectedStatus, timeoutMs = 20_000) {
  const row = await waitForRecordTerminal(db, requestId, timeoutMs);
  assertEqual(row.status, expectedStatus, `request record status should become ${expectedStatus}`);
  return row;
}

export function requestRecordRow(db, requestId) {
  const [row] = db.rows(`
select request_id, status, billing_status, coalesce(client_status_code::text, ''), coalesce(client_error_type, ''),
  coalesce(client_error_message, ''), coalesce(termination_origin, ''), coalesce(termination_reason, ''),
  coalesce(stream_end_reason, ''), candidate_count::text, has_failover::text, has_retry::text,
  is_stream::text, coalesce(first_byte_time_ms::text, ''), coalesce(total_latency_ms::text, '')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
  assert(row, `request record should exist: ${requestId}`);
  return {
    request_id: row[0],
    status: row[1],
    billing_status: row[2],
    client_status_code: row[3],
    client_error_type: row[4],
    client_error_message: row[5],
    termination_origin: row[6],
    termination_reason: row[7],
    stream_end_reason: row[8],
    candidate_count: row[9],
    has_failover: row[10],
    has_retry: row[11],
    is_stream: row[12],
    first_byte_time_ms: row[13],
    total_latency_ms: row[14],
  };
}

export function rawSummaryPayloads(db, requestId) {
  const [row] = db.rows(`
select coalesce(request_headers, ''), coalesce(request_body, ''), coalesce(client_response_headers, ''), coalesce(client_response_body, '')
from request_records where request_id = ${q(requestId)} limit 1;`);
  assert(row, `summary payloads should exist: ${requestId}`);
  return { request_headers: row[0], request_body: row[1], client_response_headers: row[2], client_response_body: row[3] };
}

export function rawCandidatePayloads(db, requestId) {
  const rows = db.rows(`
select status, coalesce(provider_request_headers, ''), coalesce(provider_request_body, ''),
  coalesce(provider_response_headers, ''), coalesce(provider_response_body, '')
from request_candidates where request_id = ${q(requestId)} order by candidate_index, retry_index;`);
  return rows.map(([status, providerRequestHeaders, providerRequestBody, providerResponseHeaders, providerResponseBody]) => ({
    status,
    provider_request_headers: providerRequestHeaders,
    provider_request_body: providerRequestBody,
    provider_response_headers: providerResponseHeaders,
    provider_response_body: providerResponseBody,
  }));
}

export async function getRequestRecord(ctx, adminToken, requestId) {
  const body = await fetchJson(`${ctx.serverBaseUrl}/api/admin/request-records/${encodeURIComponent(requestId)}`, adminToken);
  return body.data;
}

export async function listRequestRecords(ctx, adminToken, params) {
  const url = new URL(`${ctx.serverBaseUrl}/api/admin/request-records`);
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') url.searchParams.set(key, String(value));
  }
  const body = await fetchJson(url, adminToken);
  return body.data;
}

export function markForCompression(db, requestId) {
  db.exec(`
update system_settings set request_record_payload_retention_days = 0, updated_at = now() where id = 'global';
update request_records set created_at = now() - interval '2 days', updated_at = now() where request_id = ${q(requestId)};
update request_candidates set created_at = now() - interval '2 days' where request_id = ${q(requestId)};`);
}

export function prepareStalePendingRecord(db, requestId) {
  const activeId = primaryCandidateId(db, requestId);
  const shadowId = insertScheduledShadowCandidate(db, requestId, 90);
  db.exec(`
update request_records
set status = 'pending', billing_status = 'pending', client_status_code = null, client_error_type = null,
    client_error_message = null, termination_origin = null, termination_reason = null, stream_end_reason = null,
    started_at = now() - interval '20 minutes', finished_at = null, updated_at = now()
where request_id = ${q(requestId)};
update request_candidates
set status = 'pending', skip_reason = null, status_code = null, prompt_tokens = null, completion_tokens = null,
    total_tokens = null, cache_creation_input_tokens = null, cache_read_input_tokens = null, cost_currency = null,
    token_cost = null, base_cost = null, total_cost = null, billing_multiplier = null, latency_ms = null,
    first_byte_time_ms = null, error_type = null, error_message = null, error_code = null, error_param = null,
    started_at = now() - interval '20 minutes', finished_at = null
where id = ${q(activeId)};
update request_candidates
set status = 'scheduled', skip_reason = null, status_code = null, latency_ms = null, first_byte_time_ms = null,
    error_type = null, error_message = null, error_code = null, error_param = null, started_at = null, finished_at = null
where id = ${q(shadowId)};`);
}

export function prepareStaleStreamingRecord(db, requestId) {
  const activeId = primaryCandidateId(db, requestId);
  const shadowId = insertScheduledShadowCandidate(db, requestId, 91);
  db.exec(`
update request_records
set status = 'streaming', billing_status = 'pending', client_status_code = null, client_error_type = null,
    client_error_message = null, termination_origin = null, termination_reason = null, stream_end_reason = null,
    started_at = now() - interval '3 hours', finished_at = null, updated_at = now()
where request_id = ${q(requestId)};
update request_candidates
set status = 'streaming', skip_reason = null, status_code = 200, prompt_tokens = null, completion_tokens = null,
    total_tokens = null, cache_creation_input_tokens = null, cache_read_input_tokens = null, cost_currency = null,
    token_cost = null, base_cost = null, total_cost = null, billing_multiplier = null, latency_ms = null,
    first_byte_time_ms = null, error_type = null, error_message = null, error_code = null, error_param = null,
    started_at = now() - interval '3 hours', finished_at = null
where id = ${q(activeId)};
update request_candidates
set status = 'scheduled', skip_reason = null, status_code = null, latency_ms = null, first_byte_time_ms = null,
    error_type = null, error_message = null, error_code = null, error_param = null, started_at = null, finished_at = null
where id = ${q(shadowId)};`);
}

async function fetchJson(url, adminToken) {
  const response = await fetch(url, { headers: { authorization: `Bearer ${adminToken}` } });
  const text = await response.text();
  if (!response.ok) throw new Error(`request record API failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  assert(body.success, 'request record API should return success envelope');
  return body;
}

function affinityKey(prefix, tokenId, modelId, format) {
  return `${prefix}:llm_proxy:affinity:${tokenId}:${modelId}:${format}`;
}

function selectedModel(database, configured, matches, label) {
  if (configured && modelExists(database, configured)) return configured;
  if (configured) throw new Error(`${label} model does not exist in global_models: ${configured}`);
  const candidates = database.rows("select name from global_models where is_active = true order by created_at desc, name;").map(([value]) => value);
  const matched = candidates.find(matches);
  if (!matched) throw new Error(`${label} model not found in active global_models; available: ${candidates.join(', ') || 'none'}`);
  return matched;
}

function modelExists(database, name) {
  return database.scalar(`select id from global_models where name = ${q(name)} and is_active = true limit 1;`) !== '';
}

function geminiProviderName(globalName) {
  return globalName.startsWith('gemini-') ? `[满血]${globalName}` : globalName;
}

function primaryCandidateId(db, requestId) {
  const id = db.scalar(`select id from request_candidates where request_id = ${q(requestId)} order by candidate_index, retry_index limit 1;`);
  assert(id, `primary request candidate should exist: ${requestId}`);
  return id;
}

function insertScheduledShadowCandidate(db, requestId, candidateIndex) {
  const shadowId = `00000000-0000-7000-${String(candidateIndex).padStart(4, '0')}-${requestId.slice(-12)}`;
  db.exec(`
delete from request_candidates where id = ${q(shadowId)};
insert into request_candidates
  (id, request_id, token_id, group_code, global_model_id, provider_id, endpoint_id, key_id, client_api_format, provider_api_format,
   needs_conversion, is_stream, provider_request_headers, provider_request_body, provider_response_headers, provider_response_body,
   candidate_index, retry_index, status, skip_reason, status_code, prompt_tokens, completion_tokens, total_tokens,
   cache_creation_input_tokens, cache_read_input_tokens, cost_currency, token_cost, base_cost, total_cost, billing_multiplier,
   latency_ms, first_byte_time_ms, error_type, error_message, error_code, error_param, created_at, started_at, finished_at)
select
  ${q(shadowId)}, request_id, token_id, group_code, global_model_id, provider_id, endpoint_id, key_id, client_api_format, provider_api_format,
  needs_conversion, is_stream, provider_request_headers, provider_request_body, provider_response_headers, provider_response_body,
  ${candidateIndex}, 0, 'scheduled', null, null, null, null, null, null, null, null, null, null, null, null,
  null, null, null, null, null, null, created_at, null, null
from request_candidates
where request_id = ${q(requestId)}
order by candidate_index, retry_index
limit 1;`);
  return shadowId;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
