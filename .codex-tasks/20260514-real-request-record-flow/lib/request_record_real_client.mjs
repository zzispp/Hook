import { assert, assertEqual, assertIncludes } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';

export async function proxyCall(ctx, db, token, label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, token, request, options.clientTimeoutMs ?? 120_000);
  const text = await response.text();
  const requestId = await waitForLatestRequestId(db, token, request.clientFormat, request.model, before);
  const trace = requestTrace(db, requestId);
  if (options.printTrace !== false) logTrace(label, response.status, requestId, trace);
  if (!response.ok && options.expectOk !== false) {
    throw new Error(`${label} failed with HTTP ${response.status}: ${text.slice(0, 800)}`);
  }
  return { ok: response.ok, status: response.status, text, trace, requestId };
}

export async function proxyStatus(ctx, token, request, timeoutMs = 180_000) {
  const response = await sendProxyRequest(ctx, token, request, timeoutMs);
  const text = await response.text();
  return { ok: response.ok, status: response.status, text: redactedBody(text) };
}

export async function cancelProxyStream(ctx, db, token, label, request, timeoutMs = 180_000) {
  const before = new Date(Date.now() - 1000).toISOString();
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(`${ctx.serverBaseUrl}${request.path}`, {
      method: 'POST',
      headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
      body: JSON.stringify(request.body),
      signal: controller.signal,
    });
    assert(response.ok, `${label} should start successfully`);
    assert(response.body, `${label} should expose a readable body`);
    const reader = response.body.getReader();
    const first = await reader.read();
    assert(!first.done, `${label} should yield at least one stream chunk`);
    await reader.cancel('intentional client disconnect');
    controller.abort();
    const requestId = await waitForLatestRequestId(db, token, request.clientFormat, request.model, before);
    const trace = requestTrace(db, requestId);
    logTrace(label, response.status, requestId, trace);
    return { requestId, status: response.status, trace };
  } finally {
    clearTimeout(timer);
  }
}

export async function expectProxyFailure(ctx, db, token, label, request, expectedStatus, messagePart) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, token, request, 120_000);
  const text = await response.text();
  const traces = tracesSinceByTokenIds(db, before, [tokenIdForValue(db, token)]);
  assertEqual(response.status, expectedStatus, `${label} HTTP status should match`);
  assertIncludes(text, messagePart, `${label} error body should expose reason`);
  assertEqual(traces.length, 0, `${label} should fail before upstream attempts`);
  return { status: response.status, text: redactedBody(text) };
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

export function openAiChatRequest(ctx, model, text, stream = false) {
  return {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model,
    matchText: text,
    body: { model, messages: [{ role: 'user', content: text }], max_tokens: 12, temperature: 0, stream },
  };
}

export function openAiResponsesRequest(model, text, compact = false) {
  const input = compact ? [{ role: 'user', content: text }] : text;
  const body = compact ? { model, input } : { model, input, max_output_tokens: 12, temperature: 0 };
  return { path: compact ? '/v1/responses/compact' : '/v1/responses', clientFormat: compact ? 'openai_compact' : 'openai_cli', model, matchText: text, body };
}

export function claudeMessagesRequest(model, text, stream = false) {
  return {
    path: '/v1/messages',
    clientFormat: 'claude_chat',
    model,
    matchText: text,
    body: { model, messages: [{ role: 'user', content: text }], max_tokens: 12, temperature: 0, stream },
  };
}

export function geminiRequest(model, text, stream = false) {
  const action = stream ? 'streamGenerateContent' : 'generateContent';
  return {
    path: `/v1beta/models/${encodeURIComponent(model)}:${action}`,
    clientFormat: 'gemini_chat',
    model,
    matchText: text,
    body: { contents: [{ role: 'user', parts: [{ text }] }], generationConfig: { maxOutputTokens: 12, temperature: 0 } },
  };
}

export function invalidRoleOpenAiChatRequest(ctx, model, text) {
  const request = openAiChatRequest(ctx, model, text);
  request.body.messages = [{ role: 'banana', content: text }];
  return request;
}

export function successRow(trace) {
  const row = trace.find((item) => item.status === 'success');
  assert(row, 'expected successful request candidate');
  return row;
}

export function assertSingleSuccessAttempt(result, providerName, keyName) {
  const terminal = result.trace.filter((row) => ['success', 'failed', 'cancelled'].includes(row.status));
  assertEqual(terminal.length, 1, 'single-route success should record one terminal attempt');
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerName, 'provider should match');
  if (keyName) assertEqual(success.key_name, keyName, 'key should match');
  assertNoOpenRows(result.trace);
}

export function assertStreamSuccess(result, shouldConvert) {
  const success = successRow(result.trace);
  assertEqual(success.is_stream, 'true', 'stream request should be recorded as stream');
  assertEqual(success.needs_conversion, String(shouldConvert), 'stream conversion flag should match');
  assert(success.first_byte_time_ms !== '', 'stream should record first byte time');
  assertIncludes(result.text, 'data:', 'stream response should contain SSE data');
}

export function assertNoOpenRows(trace) {
  assertEqual(trace.filter((row) => ['scheduled', 'pending', 'streaming'].includes(row.status)).length, 0, 'trace should not leave open rows');
  assertEqual(trace.filter((row) => ['available', 'unused'].includes(row.status)).length, 0, 'trace should not use legacy statuses');
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

async function waitForLatestRequestId(db, token, clientFormat, model, beforeIso) {
  const started = Date.now();
  while (Date.now() - started < 8_000) {
    const requestId = latestRequestId(db, token, clientFormat, model, beforeIso);
    if (requestId) return requestId;
    await sleep(200);
  }
  throw new Error(`request candidate row was not created for ${clientFormat}/${model}`);
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
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.skip_reason, ''), coalesce(rc.status_code::text, ''),
  coalesce(p.name, ''), coalesce(k.name, ''), coalesce(e.api_format, ''), rc.client_api_format,
  coalesce(rc.provider_api_format, ''), rc.needs_conversion::text, rc.is_stream::text,
  coalesce(rc.latency_ms::text, ''), coalesce(rc.first_byte_time_ms::text, ''), coalesce(rc.error_type, ''),
  coalesce(rc.error_message, ''), coalesce(rc.error_code, ''), coalesce(rc.error_param, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
left join provider_endpoints e on e.id = rc.endpoint_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(([candidateIndex, retryIndex, status, skipReason, statusCode, providerName, keyName, endpointApiFormat, clientApiFormat, providerApiFormat, needsConversion, isStream, latencyMs, firstByteTimeMs, errorType, errorMessage, errorCode, errorParam]) => ({
    candidate_index: candidateIndex,
    retry_index: retryIndex,
    status,
    skip_reason: skipReason,
    status_code: statusCode,
    provider_name: providerName,
    key_name: keyName,
    endpoint_api_format: endpointApiFormat,
    client_api_format: clientApiFormat,
    provider_api_format: providerApiFormat,
    needs_conversion: needsConversion,
    is_stream: isStream,
    latency_ms: latencyMs,
    first_byte_time_ms: firstByteTimeMs,
    error_type: errorType,
    error_message: errorMessage,
    error_code: errorCode,
    error_param: errorParam,
  }));
}

function logTrace(label, status, requestId, trace) {
  console.log(`${label}: status=${status} request_id=${requestId}`);
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} endpoint=${row.endpoint_api_format} ` +
        `${row.client_api_format}->${row.provider_api_format} stream=${row.is_stream} ` +
        `conversion=${row.needs_conversion} skip=${row.skip_reason || '-'} code=${row.error_code || '-'} param=${row.error_param || '-'}`,
    );
  }
}

function redactedBody(text) {
  return text.length > 500 ? `${text.slice(0, 500)}...` : text;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
