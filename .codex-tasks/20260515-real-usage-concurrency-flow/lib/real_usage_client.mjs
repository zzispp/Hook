import { assert, assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';

export function openAiChatRequest(model, text, stream = false) {
  return Object.freeze({
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model,
    marker: text,
    body: {
      model,
      messages: [{ role: 'user', content: text }],
      max_tokens: 12,
      temperature: 0,
      stream,
    },
  });
}

export async function proxyCall(ctx, db, tokenValue, label, request, timeoutMs) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, tokenValue, request, timeoutMs);
  const text = await response.text();
  const tokenId = tokenIdForValue(db, tokenValue);
  const requestId = await waitForLatestRequestId(db, tokenId, request, before);
  const trace = requestTrace(db, requestId);
  if (!response.ok) {
    throw new Error(`${label} failed HTTP ${response.status}: ${text.slice(0, 1000)}`);
  }
  return { label, status: response.status, requestId, trace, tokenId, body: summarizeBody(text) };
}

export async function runConcurrentProxyCalls(ctx, db, requests, timeoutMs) {
  const settled = await Promise.allSettled(
    requests.map((item, index) => proxyCall(ctx, db, item.token, `real-concurrency-${index}`, item.request, timeoutMs)),
  );
  const failures = settled
    .map((result, index) => ({ result, index }))
    .filter(({ result }) => result.status === 'rejected')
    .map(({ result, index }) => ({ index, error: result.reason.stack || result.reason.message }));
  if (failures.length > 0) {
    const summary = summarizeConcurrentFailure(settled, failures);
    throw new Error(`concurrent proxy calls failed: ${JSON.stringify(summary, null, 2)}`);
  }
  return settled.map((result) => result.value);
}

function summarizeConcurrentFailure(settled, failures) {
  return {
    successCount: settled.length - failures.length,
    failureCount: failures.length,
    sampleFailures: failures.slice(0, 8).map((failure) => ({
      index: failure.index,
      error: failure.error.split('\n').slice(0, 6).join('\n'),
    })),
  };
}

export async function waitForTerminalRecords(db, requestIds, timeoutMs = 30_000) {
  const expected = [...new Set(requestIds)];
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const rows = db.rows(`
select request_id, status, billing_status
from request_records
where request_id in (${expected.map(q).join(',')});`);
    const terminal = rows.filter(([, status]) => !['pending', 'streaming'].includes(status));
    if (terminal.length === expected.length) {
      return rows.map(([requestId, status, billingStatus]) => ({ requestId, status, billingStatus }));
    }
    await sleep(250);
  }
  throw new Error(`request records did not reach terminal states: ${expected.length}`);
}

export function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.status_code::text, ''),
  coalesce(rc.provider_id, ''), coalesce(p.name, ''), coalesce(rc.key_id, ''), coalesce(k.name, ''),
  rc.client_api_format, coalesce(rc.provider_api_format, ''), rc.needs_conversion::text,
  coalesce(rc.token_id, ''), coalesce(rc.global_model_id, ''), coalesce(rc.total_cost::text, ''),
  coalesce(rc.prompt_tokens::text, ''), coalesce(rc.completion_tokens::text, ''), coalesce(rc.total_tokens::text, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(traceRow);
}

function traceRow(row) {
  const [
    candidateIndex,
    retryIndex,
    status,
    statusCode,
    providerId,
    providerName,
    keyId,
    keyName,
    clientApiFormat,
    providerApiFormat,
    needsConversion,
    tokenId,
    globalModelId,
    totalCost,
    promptTokens,
    completionTokens,
    totalTokens,
  ] = row;
  return {
    candidateIndex,
    retryIndex,
    status,
    statusCode,
    providerId,
    providerName,
    keyId,
    keyName,
    clientApiFormat,
    providerApiFormat,
    needsConversion,
    tokenId,
    globalModelId,
    totalCost,
    promptTokens,
    completionTokens,
    totalTokens,
  };
}

export function successTraceRows(results) {
  return results.map((result) => {
    const success = result.trace.find((row) => row.status === 'success');
    assert(success, `request should have success candidate: ${result.requestId}`);
    return success;
  });
}

export function assertMultiAccountAndKeyCoverage(results, minimumTokens, minimumKeys) {
  const successes = successTraceRows(results);
  const tokenIds = new Set(successes.map((row) => row.tokenId));
  const keyIds = new Set(successes.map((row) => row.keyId));
  assert(tokenIds.size >= minimumTokens, `expected at least ${minimumTokens} customer tokens, got ${tokenIds.size}`);
  assert(keyIds.size >= minimumKeys, `expected at least ${minimumKeys} provider keys, got ${keyIds.size}`);
}

export function assertRequestRecordSummary(db, requestIds) {
  const rows = db.rows(`
select status, billing_status, count(*)::text
from request_records
where request_id in (${requestIds.map(q).join(',')})
group by status, billing_status
order by status, billing_status;`);
  const successSettled = rows.find(([status, billingStatus]) => status === 'success' && billingStatus === 'settled');
  assert(successSettled, 'request_records should include success/settled rows');
  assertEqual(Number(successSettled[2]), requestIds.length, 'all real requests should be settled successes');
  return rows.map(([status, billingStatus, count]) => ({ status, billingStatus, count: Number(count) }));
}

async function sendProxyRequest(ctx, tokenValue, request, timeoutMs) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(`${ctx.serverBaseUrl}${request.path}`, {
      method: 'POST',
      headers: { authorization: `Bearer ${tokenValue}`, 'content-type': 'application/json' },
      body: JSON.stringify(request.body),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timer);
  }
}

async function waitForLatestRequestId(db, tokenId, request, beforeIso) {
  const started = Date.now();
  while (Date.now() - started < 10_000) {
    const requestId = db.scalar(`
select request_id
from request_records
where created_at >= ${q(beforeIso)}
  and token_id = ${q(tokenId)}
  and client_api_format = ${q(request.clientFormat)}
  and global_model_id = (select id from global_models where name = ${q(request.model)})
  and request_body like ${q(`%${request.marker}%`)}
order by created_at desc
limit 1;`);
    if (requestId) {
      return requestId;
    }
    await sleep(200);
  }
  throw new Error(`request candidate row was not created for ${request.model}`);
}

function tokenIdForValue(db, tokenValue) {
  const tokenId = db.scalar(`select id from api_tokens where token_hash = ${q(sha256(tokenValue))} limit 1;`);
  assert(tokenId, 'created customer token should exist in api_tokens');
  return tokenId;
}

function summarizeBody(text) {
  try {
    const body = JSON.parse(text);
    return { id: body.id, model: body.model, usage: body.usage };
  } catch {
    return { text: text.slice(0, 200) };
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
