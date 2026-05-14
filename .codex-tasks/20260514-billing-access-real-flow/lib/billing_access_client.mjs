import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { ids } from './billing_access_ids.mjs';

const BODY_PREVIEW_LIMIT = 800;
const REQUEST_TIMEOUT_MS = 120_000;

export function openAiChatRequest(model, text) {
  return Object.freeze({
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model,
    body: {
      model,
      messages: [{ role: 'user', content: text }],
      max_tokens: 12,
    },
  });
}

export async function proxyExchange(ctx, db, tokenId, token, label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(ctx, token, request, options.timeoutMs ?? REQUEST_TIMEOUT_MS);
  const text = await response.text();
  await sleep(options.recordDelayMs ?? 300);
  const traces = tracesSinceToken(db, before, tokenId);
  const latest = traces[0] ?? null;
  return {
    label,
    ok: response.ok,
    status: response.status,
    body: bodyPreview(text),
    requestId: latest?.requestId ?? '',
    trace: latest?.trace ?? [],
    record: latest?.requestId ? requestRecord(db, latest.requestId) : null,
  };
}

export async function fetchUpstreamModels(baseUrl, apiKey) {
  const response = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/models`, {
    headers: { authorization: `Bearer ${apiKey}` },
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`upstream models failed ${response.status}: ${bodyPreview(text)}`);
  const body = JSON.parse(text);
  const names = Array.isArray(body.data) ? body.data.map((item) => item.id).filter(Boolean) : [];
  return names.sort();
}

export function tracesSinceToken(db, beforeIso, tokenId) {
  const ids = db.rows(`
select request_id, max(created_at)
from request_candidates
where created_at >= ${q(beforeIso)} and token_id = ${q(tokenId)}
group by request_id
order by max(created_at) desc;`);
  return ids.map(([requestId]) => ({ requestId, trace: requestTrace(db, requestId) }));
}

export function requestTrace(db, requestId) {
  const rows = db.rows(`
select rc.candidate_index::text, rc.retry_index::text, rc.status, coalesce(rc.skip_reason, ''),
  coalesce(rc.status_code::text, ''), coalesce(p.name, ''), coalesce(k.name, ''),
  coalesce(rc.key_id, ''), coalesce(rc.error_type, ''), coalesce(rc.error_message, ''), coalesce(rc.error_code, ''),
  coalesce(rc.prompt_tokens::text, ''), coalesce(rc.completion_tokens::text, ''),
  coalesce(rc.token_cost::text, ''), coalesce(rc.base_cost::text, ''),
  coalesce(rc.total_cost::text, ''), coalesce(rc.billing_multiplier::text, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map(traceRow);
}

export function requestRecord(db, requestId) {
  const [row] = db.rows(`
select status, billing_status, coalesce(client_status_code::text, ''), coalesce(client_error_type, ''),
  coalesce(provider_id, ''), coalesce(key_id, ''), has_failover::text, has_retry::text,
  coalesce(token_cost::text, ''), coalesce(base_cost::text, ''), coalesce(total_cost::text, ''),
  coalesce(billing_multiplier::text, ''), candidate_count::text
from request_records
where request_id = ${q(requestId)};`);
  if (!row) return null;
  return {
    status: row[0],
    billing_status: row[1],
    client_status_code: row[2],
    client_error_type: row[3],
    provider_id: row[4],
    key_id: row[5],
    has_failover: row[6],
    has_retry: row[7],
    token_cost: row[8],
    base_cost: row[9],
    total_cost: row[10],
    billing_multiplier: row[11],
    candidate_count: row[12],
  };
}

export function successCandidate(trace) {
  return trace.find((row) => row.status === 'success') ?? null;
}

export function tokenSnapshot(db, tokenId) {
  const [row] = db.rows(`
select used_quota::text, request_count::text, is_active::text
from api_tokens where id = ${q(tokenId)};`);
  if (!row) throw new Error(`missing api token: ${tokenId}`);
  return { used_quota: row[0], request_count: Number(row[1]), is_active: row[2] === 'true' };
}

export function walletSnapshot(db, walletId) {
  const [row] = db.rows(`
select recharge_balance::text, gift_balance::text, total_consumed::text, status, limit_mode
from wallets where id = ${q(walletId)};`);
  if (!row) throw new Error(`missing wallet: ${walletId}`);
  return {
    recharge_balance: row[0],
    gift_balance: row[1],
    total_consumed: row[2],
    status: row[3],
    limit_mode: row[4],
  };
}

export function walletTransactions(db, walletId) {
  const rows = db.rows(`
select category, reason_code, amount::text, balance_before::text, balance_after::text,
  coalesce(link_type, ''), coalesce(link_id, ''), coalesce(description, '')
from wallet_transactions
where wallet_id = ${q(walletId)}
order by created_at desc;`);
  return rows.map(([category, reasonCode, amount, before, after, linkType, linkId, description]) => ({
    category,
    reasonCode,
    amount,
    before,
    after,
    linkType,
    linkId,
    description,
    snapshot: parseJson(description),
  }));
}

export async function clearProxyCaches(redis, prefix) {
  const authKeys = await redis.keys(`${prefix}:llm_proxy:auth:v*`);
  const affinityKeys = await redis.keys(`${prefix}:llm_proxy:affinity:*`);
  await redis.del(
    `${prefix}:llm_proxy:scheduling:snapshot:v2`,
    `${prefix}:llm_proxy:scheduling:rebuild_lock`,
    `${prefix}:llm_proxy:auth:version`,
    ...authKeys,
    ...affinityKeys,
  );
}

export async function affinityKeyValue(redis, prefix) {
  return redis.get(`${prefix}:llm_proxy:affinity:${ids.tokenRouting}:${ids.modelOpenai}:openai_chat`);
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

function traceRow(row) {
  return {
    candidate_index: row[0],
    retry_index: row[1],
    status: row[2],
    skip_reason: row[3],
    status_code: row[4],
    provider_name: row[5],
    key_name: row[6],
    key_id: row[7],
    error_type: row[8],
    error_message: row[9],
    error_code: row[10],
    prompt_tokens: row[11],
    completion_tokens: row[12],
    token_cost: row[13],
    base_cost: row[14],
    total_cost: row[15],
    billing_multiplier: row[16],
  };
}

function bodyPreview(text) {
  return text.length > BODY_PREVIEW_LIMIT ? `${text.slice(0, BODY_PREVIEW_LIMIT)}...` : text;
}

function parseJson(text) {
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
