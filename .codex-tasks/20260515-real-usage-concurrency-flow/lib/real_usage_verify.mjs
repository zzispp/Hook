import { assert, assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { usageKeys } from './real_usage_keys.mjs';

export async function waitForUsageFlush(db, redis, prefix, tokenIds, expectedRequests, modelId, timeoutMs = 30_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const tokenCount = tokenRequestCountDelta(db, tokenIds);
    const modelCount = modelUsageCount(db, modelId);
    const processingEmpty = await usageProcessingEmpty(redis, prefix);
    const pendingEmpty = await usagePendingEmpty(redis, prefix);
    if (tokenCount >= expectedRequests && modelCount >= expectedRequests && processingEmpty && pendingEmpty) {
      return usageSummary(db, tokenIds, modelId);
    }
    await sleep(500);
  }
  const summary = usageSummary(db, tokenIds, modelId);
  const redisState = await usageRedisState(redis, prefix);
  throw new Error(`usage flush did not complete: ${JSON.stringify({ summary, redisState, expectedRequests }, null, 2)}`);
}

export function usageSummary(db, tokenIds, modelId) {
  const tokenRows = db.rows(`
select id, used_quota::text, request_count::text, coalesce(last_used_at::text, '')
from api_tokens
where id in (${tokenIds.map(q).join(',')})
order by id;`);
  const [modelRow] = db.rows(`select usage_count::text from global_models where id = ${q(modelId)};`);
  return {
    tokens: tokenRows.map(([id, usedQuota, requestCount, lastUsedAt]) => ({
      id,
      usedQuota,
      requestCount: Number(requestCount),
      lastUsedAt,
    })),
    modelUsageCount: modelRow ? Number(modelRow[0]) : 0,
  };
}

export function walletSummary(db, userIds) {
  const rows = db.rows(`
select w.user_id, w.gift_balance::text, w.recharge_balance::text, w.total_consumed::text,
  count(t.id)::text, coalesce(sum(-t.amount), 0)::text
from wallets w
left join wallet_transactions t on t.wallet_id = w.id and t.reason_code = 'llm_model_usage'
where w.user_id in (${userIds.map(q).join(',')})
group by w.user_id, w.gift_balance, w.recharge_balance, w.total_consumed
order by w.user_id;`);
  return rows.map(([userId, giftBalance, rechargeBalance, totalConsumed, txCount, txAmount]) => ({
    userId,
    giftBalance,
    rechargeBalance,
    totalConsumed,
    transactionCount: Number(txCount),
    transactionAmount: txAmount,
  }));
}

export function assertWalletBilling(db, userIds, expectedRequests) {
  const summary = walletSummary(db, userIds);
  const totalTransactions = summary.reduce((sum, item) => sum + item.transactionCount, 0);
  assertEqual(totalTransactions, expectedRequests, 'wallet_transactions should match successful request count');
  for (const item of summary) {
    assert(Number(item.totalConsumed) > 0, `wallet should consume balance for ${item.userId}`);
  }
  return summary;
}

export function requestCandidateAggregate(db, requestIds) {
  const rows = db.rows(`
select coalesce(p.name, ''), coalesce(k.name, ''), rc.provider_api_format, rc.status, count(*)::text
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id in (${requestIds.map(q).join(',')})
group by p.name, k.name, rc.provider_api_format, rc.status
order by p.name, k.name, rc.provider_api_format, rc.status;`);
  return rows.map(([providerName, keyName, providerApiFormat, status, count]) => ({
    providerName,
    keyName,
    providerApiFormat,
    status,
    count: Number(count),
  }));
}

export function assertCandidateCoverage(aggregate) {
  const successRows = aggregate.filter((row) => row.status === 'success');
  const providers = new Set(successRows.map((row) => row.providerName));
  const keys = new Set(successRows.map((row) => row.keyName));
  assert(providers.size >= 2, `expected at least two providers in successful traffic, got ${providers.size}`);
  assert(keys.size >= 4, `expected at least four provider key records in successful traffic, got ${keys.size}`);
}

export async function usageRedisState(redis, prefix) {
  const keys = usageKeys(prefix);
  return {
    pendingTokenCost: await redis.command('HLEN', keys.pendingTokenCost),
    pendingTokenCount: await redis.command('HLEN', keys.pendingTokenCount),
    pendingTokenLastUsedAt: await redis.command('HLEN', keys.pendingTokenLastUsedAt),
    processingTokenCost: await redis.command('HLEN', keys.processingTokenCost),
    processingTokenCount: await redis.command('HLEN', keys.processingTokenCount),
    processingTokenLastUsedAt: await redis.command('HLEN', keys.processingTokenLastUsedAt),
    processingTokenBatchId: await redis.get(keys.processingTokenBatchId),
    pendingModelCount: await redis.command('HLEN', keys.pendingModelCount),
    processingModelCount: await redis.command('HLEN', keys.processingModelCount),
    processingModelBatchId: await redis.get(keys.processingModelBatchId),
  };
}

function tokenRequestCountDelta(db, tokenIds) {
  return Number(db.scalar(`select coalesce(sum(request_count), 0)::text from api_tokens where id in (${tokenIds.map(q).join(',')});`));
}

function modelUsageCount(db, modelId) {
  return Number(db.scalar(`select usage_count::text from global_models where id = ${q(modelId)};`) || 0);
}

async function usageProcessingEmpty(redis, prefix) {
  const keys = usageKeys(prefix);
  const token = await redis.command('HLEN', keys.processingTokenCount);
  const model = await redis.command('HLEN', keys.processingModelCount);
  const tokenBatch = await redis.get(keys.processingTokenBatchId);
  const modelBatch = await redis.get(keys.processingModelBatchId);
  return token === 0 && model === 0 && tokenBatch === null && modelBatch === null;
}

async function usagePendingEmpty(redis, prefix) {
  const keys = usageKeys(prefix);
  const token = await redis.command('HLEN', keys.pendingTokenCount);
  const model = await redis.command('HLEN', keys.pendingModelCount);
  return token === 0 && model === 0;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
