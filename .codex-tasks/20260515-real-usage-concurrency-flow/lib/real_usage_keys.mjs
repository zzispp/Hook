export function usageKeys(prefix) {
  return Object.freeze({
    authVersion: `${prefix}:llm_proxy:auth:version`,
    schedulingSnapshot: `${prefix}:llm_proxy:scheduling:snapshot:v2`,
    schedulingLock: `${prefix}:llm_proxy:scheduling:rebuild_lock`,
    flushLock: `${prefix}:llm_proxy:usage:flush_lock`,
    pendingTokenCost: `${prefix}:llm_proxy:usage:pending:token:cost`,
    pendingTokenCount: `${prefix}:llm_proxy:usage:pending:token:count`,
    pendingTokenLastUsedAt: `${prefix}:llm_proxy:usage:pending:token:last_used_at`,
    processingTokenCost: `${prefix}:llm_proxy:usage:processing:token:cost`,
    processingTokenCount: `${prefix}:llm_proxy:usage:processing:token:count`,
    processingTokenLastUsedAt: `${prefix}:llm_proxy:usage:processing:token:last_used_at`,
    processingTokenBatchId: `${prefix}:llm_proxy:usage:processing:token:batch_id`,
    pendingModelCount: `${prefix}:llm_proxy:usage:pending:model:count`,
    processingModelCount: `${prefix}:llm_proxy:usage:processing:model:count`,
    processingModelBatchId: `${prefix}:llm_proxy:usage:processing:model:batch_id`,
  });
}

export async function clearUsageRedis(redis, prefix, tokenIds = []) {
  const keys = usageKeys(prefix);
  const authKeys = await redis.keys(`${prefix}:llm_proxy:auth:v*`);
  const authUsageKeys = tokenIds.map((id) => `${prefix}:llm_proxy:auth:usage:${id}`);
  await redis.del(
    ...authKeys,
    keys.authVersion,
    keys.flushLock,
    keys.pendingTokenCost,
    keys.pendingTokenCount,
    keys.pendingTokenLastUsedAt,
    keys.processingTokenCost,
    keys.processingTokenCount,
    keys.processingTokenLastUsedAt,
    keys.processingTokenBatchId,
    keys.pendingModelCount,
    keys.processingModelCount,
    keys.processingModelBatchId,
    ...authUsageKeys,
  );
}

export async function clearSchedulingRedis(redis, prefix) {
  await redis.del(`${prefix}:llm_proxy:scheduling:snapshot:v2`, `${prefix}:llm_proxy:scheduling:rebuild_lock`);
}
