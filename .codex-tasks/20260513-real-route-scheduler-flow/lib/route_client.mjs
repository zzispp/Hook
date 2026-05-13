import { assert, assertEqual, assertIncludes } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

export async function proxyCall(ctx, db, label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, request, options.clientTimeoutMs ?? 120_000);
  const text = await response.text();
  const requestId = latestRequestId(db, request.clientFormat, request.model, before, request.matchText);
  const trace = requestId ? requestTrace(db, requestId) : [];
  console.log(`${label}: status=${response.status} request_id=${requestId ?? 'none'}`);
  printTrace(trace);
  assert(requestId, `${label} should create request candidate rows`);
  if (!response.ok && options.expectOk !== false) {
    throw new Error(`${label} failed with HTTP ${response.status}: ${text.slice(0, 800)}`);
  }
  return { ok: response.ok, status: response.status, text, trace, requestId };
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
      max_tokens: 12,
      temperature: 0,
      stream,
    },
  };
}

export function openAiResponsesRequest(ctx, model, text, compact = false) {
  const input = compact ? [{ role: 'user', content: text }] : text;
  const body = compact ? { model, input } : { model, input, max_output_tokens: 12, temperature: 0 };
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
      max_tokens: 12,
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
      generationConfig: { maxOutputTokens: 12, temperature: 0 },
    },
  };
}

export function successRow(trace) {
  const row = trace.find((item) => item.status === 'success');
  assert(row, 'expected successful request candidate');
  return row;
}

export function assertSingleSuccessAttempt(result, providerName, keyName) {
  assertEqual(result.trace.length, 1, 'single successful route should record only one real attempt');
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerName, 'provider should match');
  if (keyName) assertEqual(success.key_name, keyName, 'key should match');
  assertEqual(success.status, 'success', 'attempt status should be success');
  assertNoAvailableRows(result.trace);
}

export function assertStreamSuccess(result, shouldConvert) {
  const success = successRow(result.trace);
  assertEqual(success.is_stream, 'true', 'stream request should be recorded as stream');
  assertEqual(success.needs_conversion, String(shouldConvert), 'stream conversion flag should match');
  assert(success.first_byte_time_ms !== '', 'stream should record first byte time');
  assert(success.latency_ms !== '', 'stream should record total latency');
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

async function sendProxyRequest(ctx, request, timeoutMs) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(`${ctx.serverBaseUrl}${request.path}`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${ctx.secrets.systemToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify(request.body),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timer);
  }
}

function latestRequestId(db, clientFormat, model, beforeIso, marker) {
  return db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and client_api_format = ${q(clientFormat)}
  and global_model_id = (select id from global_models where name = ${q(model)})
  and request_body like ${q(`%${marker}%`)}
order by created_at desc
limit 1;`);
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

function printTrace(trace) {
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} endpoint=${row.endpoint_api_format} ` +
        `${row.client_api_format}->${row.provider_api_format} stream=${row.is_stream} ` +
        `conversion=${row.needs_conversion} fb=${row.first_byte_time_ms || '-'} ${row.error_type}`,
    );
  }
}
