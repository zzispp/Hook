import { assert, assertEqual, assertIncludes } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';

export async function proxyCall(ctx, db, token, label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, token, request, options.clientTimeoutMs ?? 120_000);
  const text = await response.text();
  const requestId = latestRequestId(db, token, request.clientFormat, request.model, before);
  const trace = requestId ? requestTrace(db, requestId) : [];
  if (options.printTrace !== false) logTrace(label, response.status, requestId, trace);
  assert(requestId, `${label} should create request candidate rows`);
  if (!response.ok && options.expectOk !== false) {
    throw new Error(`${label} failed with HTTP ${response.status}: ${text.slice(0, 800)}`);
  }
  return { ok: response.ok, status: response.status, text, trace, requestId };
}

export async function expectProxyFailure(ctx, db, token, label, request, expectedStatus, messagePart) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, token, request, 120_000);
  const text = await response.text();
  const traces = tracesSinceByTokenIds(db, before, [tokenIdForValue(db, token)]);
  console.log(`${label}: status=${response.status} traces=${traces.length}`);
  assertEqual(response.status, expectedStatus, `${label} HTTP status should match`);
  assertIncludes(text, messagePart, `${label} error body should expose reason`);
  assertEqual(traces.length, 0, `${label} should fail before upstream attempts`);
  return { status: response.status, text: redactedBody(text) };
}

export async function proxyStatus(ctx, token, request, timeoutMs = 180_000) {
  const response = await sendProxyRequest(ctx, token, request, timeoutMs);
  const text = await response.text();
  return { ok: response.ok, status: response.status, text: redactedBody(text) };
}

export async function adminSignIn(ctx) {
  const response = await fetch(`${ctx.serverBaseUrl}/api/auth/sign-in`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ identifier: ctx.adminIdentifier, password: ctx.adminPassword }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`admin sign-in failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  assert(body.success, 'admin sign-in should return success envelope');
  return body.data.access_token;
}

export async function replaceUserViaApi(ctx, accessToken, profile, access) {
  const response = await fetch(`${ctx.serverBaseUrl}/api/users/${profile.id}`, {
    method: 'PUT',
    headers: { authorization: `Bearer ${accessToken}`, 'content-type': 'application/json' },
    body: JSON.stringify({
      username: profile.username,
      password: profile.password,
      email: profile.email,
      role: profile.role,
      is_active: profile.is_active,
      allowed_model_ids: access.allowedModelIds,
      allowed_provider_ids: access.allowedProviderIds,
      rate_limit_rpm: profile.rate_limit_rpm,
      quota_mode: profile.quota_mode,
    }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`replace user failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  assertEqual(JSON.stringify(body.data.allowed_model_ids), JSON.stringify(access.allowedModelIds), 'API response should include allowed models');
  assertEqual(JSON.stringify(body.data.allowed_provider_ids), JSON.stringify(access.allowedProviderIds), 'API response should include allowed providers');
  return body.data;
}

export function openAiChatRequest(ctx, model, text, stream = false) {
  return {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model,
    matchText: text,
    body: {
      model,
      messages: [{ role: 'user', content: text }],
      max_tokens: 8,
      temperature: 0,
      stream,
    },
  };
}

export function openAiResponsesRequest(model, text, compact = false) {
  const input = compact ? [{ role: 'user', content: text }] : text;
  const body = compact ? { model, input } : { model, input, max_output_tokens: 8, temperature: 0 };
  return {
    path: compact ? '/v1/responses/compact' : '/v1/responses',
    clientFormat: compact ? 'openai_compact' : 'openai_cli',
    model,
    matchText: text,
    body,
  };
}

export function claudeMessagesRequest(model, text, stream = false) {
  return {
    path: '/v1/messages',
    clientFormat: 'claude_chat',
    model,
    matchText: text,
    body: {
      model,
      messages: [{ role: 'user', content: text }],
      max_tokens: 8,
      temperature: 0,
      stream,
    },
  };
}

export function geminiRequest(model, text, stream = false) {
  const action = stream ? 'streamGenerateContent' : 'generateContent';
  return {
    path: `/v1beta/models/${encodeURIComponent(model)}:${action}`,
    clientFormat: 'gemini_chat',
    model,
    matchText: text,
    body: {
      contents: [{ role: 'user', parts: [{ text }] }],
      generationConfig: { maxOutputTokens: 8, temperature: 0 },
    },
  };
}

export function successRow(trace) {
  const row = trace.find((item) => item.status === 'success');
  assert(row, 'expected successful request candidate');
  return row;
}

export function assertSingleSuccessAttempt(result, providerName, keyName) {
  assertEqual(result.trace.length, 1, 'single successful route should record one real attempt');
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerName, 'provider should match');
  if (keyName) assertEqual(success.key_name, keyName, 'key should match');
  assertNoAvailableRows(result.trace);
}

export function assertStreamSuccess(result, shouldConvert) {
  const success = successRow(result.trace);
  assertEqual(success.is_stream, 'true', 'stream request should be recorded as stream');
  assertEqual(success.needs_conversion, String(shouldConvert), 'stream conversion flag should match');
  assert(success.first_byte_time_ms !== '', 'stream should record first byte time');
  assertIncludes(result.text, 'data:', 'stream response should contain SSE data');
}

export function assertNoAvailableRows(trace) {
  assertEqual(trace.filter((row) => row.status === 'available').length, 0, 'trace should not leave available rows');
}

export function tracesSince(db, beforeIso, marker) {
  const ids = db.rows(`
select distinct request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and request_body like ${q(`%${marker}%`)}
order by request_id;`);
  return ids.map(([requestId]) => ({ requestId, trace: requestTrace(db, requestId) }));
}

export function tracesSinceByTokenIds(db, beforeIso, tokenIds) {
  const ids = db.rows(`
select distinct request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and token_id in (${tokenIds.map(q).join(',')})
order by request_id;`);
  return ids.map(([requestId]) => ({ requestId, trace: requestTrace(db, requestId) }));
}

async function sendProxyRequest(ctx, token, request, timeoutMs) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(`${ctx.serverBaseUrl}${request.path}`, {
      method: 'POST',
      headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
      body: JSON.stringify(request.body),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timer);
  }
}

function latestRequestId(db, token, clientFormat, model, beforeIso) {
  const tokenId = tokenIdForValue(db, token);
  return db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and token_id = ${q(tokenId)}
  and client_api_format = ${q(clientFormat)}
  and global_model_id = (select id from global_models where name = ${q(model)})
order by created_at desc
limit 1;`);
}

function tokenIdForValue(db, token) {
  const tokenId = db.scalar(`select id from api_tokens where token_hash = ${q(sha256(token))} limit 1;`);
  assert(tokenId, 'test token should exist in api_tokens');
  return tokenId;
}

function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.status_code::text, ''),
  coalesce(p.name, ''), coalesce(k.name, ''), coalesce(e.api_format, ''), rc.client_api_format,
  coalesce(rc.provider_api_format, ''), rc.needs_conversion::text, rc.is_stream::text,
  coalesce(rc.latency_ms::text, ''), coalesce(rc.first_byte_time_ms::text, ''),
  coalesce(rc.error_type, ''), coalesce(rc.error_message, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
left join provider_endpoints e on e.id = rc.endpoint_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(traceRow);
}

function traceRow(row) {
  return {
    candidate_index: row[0],
    retry_index: row[1],
    status: row[2],
    status_code: row[3],
    provider_name: row[4],
    key_name: row[5],
    endpoint_api_format: row[6],
    client_api_format: row[7],
    provider_api_format: row[8],
    needs_conversion: row[9],
    is_stream: row[10],
    latency_ms: row[11],
    first_byte_time_ms: row[12],
    error_type: row[13],
    error_message: row[14],
  };
}

function logTrace(label, status, requestId, trace) {
  console.log(`${label}: status=${status} request_id=${requestId ?? 'none'}`);
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} endpoint=${row.endpoint_api_format} ` +
        `${row.client_api_format}->${row.provider_api_format} stream=${row.is_stream} ` +
        `conversion=${row.needs_conversion} fb=${row.first_byte_time_ms || '-'} ${row.error_type}`,
    );
  }
}

function redactedBody(text) {
  return text.length > 500 ? `${text.slice(0, 500)}...` : text;
}
