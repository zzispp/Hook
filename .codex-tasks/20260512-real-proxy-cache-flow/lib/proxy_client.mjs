import { assert, assertEqual } from './assertions.mjs';
import { q } from './db.mjs';

export async function proxyCall(ctx, db, label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, request, options.clientTimeoutMs ?? 90_000);
  const text = await response.text();
  const requestId = latestRequestId(db, request.clientFormat, request.model, before, request.matchText);
  const trace = requestId ? requestTrace(db, requestId) : [];
  console.log(`${label}: status=${response.status} request_id=${requestId ?? 'none'}`);
  printTrace(trace);
  assertOwner(db, requestId, ctx.adminUserId);
  if (!response.ok && options.expectOk !== false) {
    throw new Error(`${label} failed with HTTP ${response.status}: ${text.slice(0, 600)}`);
  }
  return { ok: response.ok, status: response.status, text, trace, requestId };
}

export function openAiBody(model, text, stream = false) {
  return {
    model,
    messages: [{ role: 'user', content: text }],
    max_tokens: 8,
    temperature: 0,
    stream,
  };
}

export function claudeBody(model, text, stream = false) {
  return {
    model,
    max_tokens: 8,
    temperature: 0,
    stream,
    messages: [{ role: 'user', content: text }],
  };
}

export function geminiBody(text) {
  return {
    contents: [{ role: 'user', parts: [{ text }] }],
    generationConfig: { maxOutputTokens: 8, temperature: 0 },
  };
}

export function successRow(trace) {
  const row = trace.find((item) => item.status === 'success');
  assert(row, 'expected a successful request candidate');
  return row;
}

export function assertConversion(trace, providerName, providerFormat) {
  const success = successRow(trace);
  assertEqual(success.provider_name, providerName, `conversion should use ${providerName}`);
  assertEqual(success.provider_api_format, providerFormat, `conversion should target ${providerFormat}`);
  assertEqual(success.needs_conversion, 'true', 'conversion trace should be marked');
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
  const markerFilter = marker ? `and request_body like ${q(`%${marker}%`)}` : '';
  return db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and client_api_format = ${q(clientFormat)}
  and global_model_id = (select id from global_models where name = ${q(model)})
  ${markerFilter}
order by created_at desc
limit 1;`);
}

function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.status_code::text, ''),
  coalesce(p.name, ''), coalesce(k.name, ''), rc.client_api_format, coalesce(rc.provider_api_format, ''),
  rc.needs_conversion::text, rc.is_stream::text, coalesce(rc.latency_ms::text, ''),
  coalesce(rc.first_byte_time_ms::text, ''), coalesce(rc.error_type, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
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
    client_api_format: row[6],
    provider_api_format: row[7],
    needs_conversion: row[8],
    is_stream: row[9],
    latency_ms: row[10],
    first_byte_time_ms: row[11],
    error_type: row[12] ?? '',
  };
}

function assertOwner(db, requestId, expectedOwner) {
  assert(requestId, 'request should have request_candidates rows');
  const ownerId = db.scalar(`
select coalesce(t.user_id, '')
from request_candidates rc
join api_tokens t on t.id = rc.token_id
where rc.request_id = ${q(requestId)}
limit 1;`);
  assertEqual(ownerId, expectedOwner, 'independent token should belong to admin user');
}

function printTrace(trace) {
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} ${row.client_api_format}->${row.provider_api_format} ` +
        `stream=${row.is_stream} conversion=${row.needs_conversion} fb=${row.first_byte_time_ms || '-'} ${row.error_type}`,
    );
  }
}

export const paths = Object.freeze({
  openaiChat: '/v1/chat/completions',
  claudeMessages: '/v1/messages',
  gemini(model, stream = false) {
    const action = stream ? 'streamGenerateContent' : 'generateContent';
    return `/v1beta/models/${encodeURIComponent(model)}:${action}`;
  },
});

export function requestForOpenAiGpt(ctx, text, stream = false) {
  return {
    path: paths.openaiChat,
    clientFormat: 'openai_chat',
    model: ctx.models.openai,
    matchText: text,
    body: openAiBody(ctx.models.openai, text, stream),
  };
}
