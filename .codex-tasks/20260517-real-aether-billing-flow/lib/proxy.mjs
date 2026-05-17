import { sha256 } from './crypto.mjs';
import { assert, assertEqual } from './assertions.mjs';
import { q } from './db.mjs';

export async function proxyChat(ctx, db, token, model, marker) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await fetch(`${ctx.serverBaseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify({
      model,
      messages: [{ role: 'user', content: `reply with ok: ${marker}` }],
      max_tokens: 16,
      temperature: 0,
    }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`proxy chat failed ${response.status}: ${text.slice(0, 800)}`);
  }
  const requestId = await waitForLatestRequestId(db, token, model, before);
  const record = await waitForRecordTerminal(db, requestId);
  const candidate = successCandidate(db, requestId);
  assertEqual(record.status, 'success', 'request record status should be success');
  assertEqual(record.billing_status, 'settled', 'request billing status should be settled');
  assert(candidate, `success candidate should exist: ${requestId}`);
  return {
    requestId,
    response: safeJson(text),
    record,
    candidate,
    trace: requestTrace(db, requestId),
  };
}

export function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.status_code::text, ''),
  coalesce(p.name, ''), coalesce(k.name, ''), coalesce(rc.provider_api_format, ''),
  coalesce(rc.base_cost::text, ''), coalesce(rc.total_cost::text, ''), coalesce(rc.billing_multiplier::text, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(([candidateIndex, retryIndex, status, statusCode, provider, key, format, baseCost, totalCost, multiplier]) => ({
    candidate_index: candidateIndex,
    retry_index: retryIndex,
    status,
    status_code: statusCode,
    provider,
    key,
    provider_api_format: format,
    base_cost: baseCost,
    total_cost: totalCost,
    billing_multiplier: multiplier,
  }));
}

async function waitForLatestRequestId(db, token, model, beforeIso) {
  const tokenId = tokenIdForValue(db, token);
  const started = Date.now();
  while (Date.now() - started < 12000) {
    const requestId = db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and token_id = ${q(tokenId)}
  and global_model_id = (select id from global_models where name = ${q(model)})
order by created_at desc
limit 1;`);
    if (requestId) {
      return requestId;
    }
    await sleep(250);
  }
  throw new Error(`request candidate row was not created for ${model}`);
}

async function waitForRecordTerminal(db, requestId) {
  const started = Date.now();
  while (Date.now() - started < 20000) {
    const record = requestRecord(db, requestId);
    if (!['pending', 'streaming'].includes(record.status)) {
      return record;
    }
    await sleep(250);
  }
  throw new Error(`request record did not reach terminal state: ${requestId}`);
}

function requestRecord(db, requestId) {
  const [row] = db.rows(`
select request_id, status, billing_status, coalesce(prompt_tokens::text, ''), coalesce(completion_tokens::text, ''),
  coalesce(total_tokens::text, ''), coalesce(base_cost::text, ''), coalesce(total_cost::text, ''),
  coalesce(billing_multiplier::text, ''), coalesce(billing_snapshot::text, '{}'),
  coalesce(client_response_body::text, '{}')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
  assert(row, `request record should exist: ${requestId}`);
  return {
    request_id: row[0],
    status: row[1],
    billing_status: row[2],
    prompt_tokens: row[3],
    completion_tokens: row[4],
    total_tokens: row[5],
    base_cost: row[6],
    total_cost: row[7],
    billing_multiplier: row[8],
    billing_snapshot: safeJson(row[9]),
    client_response_body: safeJson(row[10]),
  };
}

function successCandidate(db, requestId) {
  const [row] = db.rows(`
select id, coalesce(prompt_tokens::text, ''), coalesce(completion_tokens::text, ''), coalesce(total_tokens::text, ''),
  coalesce(base_cost::text, ''), coalesce(total_cost::text, ''), coalesce(billing_multiplier::text, ''),
  coalesce(billing_snapshot::text, '{}'), coalesce(provider_request_body::text, '{}'), coalesce(provider_response_body::text, '{}')
from request_candidates
where request_id = ${q(requestId)} and status = 'success'
order by candidate_index, retry_index
limit 1;`);
  if (!row) {
    return null;
  }
  return {
    id: row[0],
    prompt_tokens: row[1],
    completion_tokens: row[2],
    total_tokens: row[3],
    base_cost: row[4],
    total_cost: row[5],
    billing_multiplier: row[6],
    billing_snapshot: safeJson(row[7]),
    provider_request_body: safeJson(row[8]),
    provider_response_body: safeJson(row[9]),
  };
}

function tokenIdForValue(db, token) {
  const tokenId = db.scalar(`select id from api_tokens where token_hash = ${q(sha256(token))} limit 1;`);
  assert(tokenId, 'test token should exist in api_tokens');
  return tokenId;
}

function safeJson(text) {
  try {
    return JSON.parse(text);
  } catch {
    return {};
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
