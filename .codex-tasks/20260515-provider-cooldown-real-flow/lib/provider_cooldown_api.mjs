import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

export async function adminJson(ctx, adminToken, method, path, payload) {
  const headers = { authorization: `Bearer ${adminToken}` };
  const options = { method, headers };
  if (payload !== undefined) {
    headers['content-type'] = 'application/json';
    options.body = JSON.stringify(payload);
  }
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, options);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`admin ${method} ${path} failed ${response.status}: ${text.slice(0, 1000)}`);
  }
  const body = text.trim() ? JSON.parse(text) : null;
  assert(body?.success, `admin ${method} ${path} should return success envelope`);
  return body.data;
}

export async function proxyCall(input) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(input);
  const text = await response.text();
  const requestId = await waitForLatestRequestId(input.db, input.customerToken.token.id, input.modelId, before);
  const record = await waitForTerminalRecord(input.db, requestId);
  const trace = requestTrace(input.db, requestId);
  logTrace(input.marker, response.status, requestId, trace);
  const result = { ok: response.ok, status: response.status, body: responseSummary(text), requestId, record, trace };
  if (input.expectOk && !response.ok) {
    throw new Error(`proxy request failed ${response.status}: ${text.slice(0, 1000)} trace=${JSON.stringify(trace)}`);
  }
  return result;
}

async function sendProxyRequest(input) {
  const request = {
    model: input.modelName,
    messages: [{ role: 'user', content: `provider cooldown real flow ${input.marker}` }],
    max_tokens: 8,
    temperature: 0,
  };
  return fetch(`${input.ctx.serverBaseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${input.customerToken.raw_token}`,
      'content-type': 'application/json',
    },
    body: JSON.stringify(request),
    signal: AbortSignal.timeout(input.timeoutMs),
  });
}

async function waitForLatestRequestId(db, tokenId, modelId, beforeIso) {
  const started = Date.now();
  while (Date.now() - started < 10_000) {
    const requestId = latestRequestId(db, tokenId, modelId, beforeIso);
    if (requestId) return requestId;
    await sleep(200);
  }
  throw new Error('request candidate row was not created');
}

function latestRequestId(db, tokenId, modelId, beforeIso) {
  return db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and token_id = ${q(tokenId)}
  and client_api_format = 'openai_chat'
  and global_model_id = ${q(modelId)}
order by created_at desc
limit 1;`);
}

async function waitForTerminalRecord(db, requestId) {
  const started = Date.now();
  while (Date.now() - started < 30_000) {
    const record = requestRecord(db, requestId);
    if (record && !['pending', 'streaming'].includes(record.status)) return record;
    await sleep(250);
  }
  throw new Error(`request record did not reach terminal state: ${requestId}`);
}

function requestRecord(db, requestId) {
  const [row] = db.rows(`
select request_id, status, billing_status, coalesce(client_status_code::text, ''),
  coalesce(client_error_type, ''), coalesce(client_error_message, '')
from request_records
where request_id = ${q(requestId)};`);
  if (!row) return null;
  return {
    request_id: row[0],
    status: row[1],
    billing_status: row[2],
    client_status_code: row[3],
    client_error_type: row[4],
    client_error_message: row[5],
  };
}

export function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.skip_reason, ''),
  coalesce(rc.status_code::text, ''), coalesce(rc.provider_id, ''), coalesce(p.name, ''),
  coalesce(rc.endpoint_id, ''), coalesce(e.api_format, ''), coalesce(rc.key_id, ''),
  coalesce(k.name, ''), rc.client_api_format, coalesce(rc.provider_api_format, ''),
  rc.needs_conversion::text, coalesce(rc.error_type, ''), coalesce(rc.error_message, ''),
  coalesce(rc.error_code, ''), coalesce(rc.error_param, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_endpoints e on e.id = rc.endpoint_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(traceRow);
}

function responseSummary(text) {
  const value = parseJson(text);
  if (!value) return { text: text.slice(0, 200) };
  return {
    id: value.id,
    model: value.model,
    object: value.object,
    finishReasons: choiceFinishReasons(value),
    usage: value.usage,
    error: value.error,
  };
}

function parseJson(text) {
  try {
    return text.trim() ? JSON.parse(text) : null;
  } catch {
    return null;
  }
}

function choiceFinishReasons(value) {
  return Array.isArray(value?.choices) ? value.choices.map((choice) => choice.finish_reason ?? null) : undefined;
}

function traceRow(row) {
  return {
    candidate_index: row[0],
    retry_index: row[1],
    status: row[2],
    skip_reason: row[3],
    status_code: row[4],
    provider_id: row[5],
    provider_name: row[6],
    endpoint_id: row[7],
    endpoint_api_format: row[8],
    key_id: row[9],
    key_name: row[10],
    client_api_format: row[11],
    provider_api_format: row[12],
    needs_conversion: row[13],
    error_type: row[14],
    error_message: row[15],
    error_code: row[16],
    error_param: row[17],
  };
}

function logTrace(label, status, requestId, trace) {
  console.log(`${label}: status=${status} request_id=${requestId}`);
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} ${row.client_api_format}->${row.provider_api_format} ` +
        `conversion=${row.needs_conversion} skip=${row.skip_reason || '-'} code=${row.error_code || '-'} param=${row.error_param || '-'}`,
    );
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
