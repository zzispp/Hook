import { assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { usageKeys } from './real_usage_keys.mjs';

export async function runUsageRecoveryScenarios(state) {
  const { db, redis, ctx, tokenIds, modelId, stopBackend, startBackend } = state;
  const keys = usageKeys(ctx.redis.prefix);
  const unapplied = await applyUnappliedRecovery({ db, redis, keys, tokenId: tokenIds[0], modelId, stopBackend, startBackend });
  const applied = await applyAlreadyMarkedRecovery({ db, redis, keys, tokenId: tokenIds[1], modelId, stopBackend, startBackend });
  return { unapplied, applied };
}

async function applyUnappliedRecovery(args) {
  const before = recoveryBaseline(args.db, args.tokenId, args.modelId);
  await args.stopBackend();
  await writeProcessingBatch(args.redis, args.keys, {
    tokenBatchId: 'real-usage-token-recovery-new',
    modelBatchId: 'real-usage-model-recovery-new',
    tokenId: args.tokenId,
    modelId: args.modelId,
    costUnits: 12345,
    count: 2,
    usedAt: '2026-05-15T08:00:00Z',
  });
  await args.startBackend();
  await waitForProcessingClear(args.redis, args.keys);
  const after = recoveryBaseline(args.db, args.tokenId, args.modelId);
  assertEqual(after.tokenRequestCount - before.tokenRequestCount, 2, 'unapplied token processing batch should flush once');
  assertEqual(after.modelUsageCount - before.modelUsageCount, 2, 'unapplied model processing batch should flush once');
  return { before, after };
}

async function applyAlreadyMarkedRecovery(args) {
  const before = recoveryBaseline(args.db, args.tokenId, args.modelId);
  await args.stopBackend();
  args.db.exec(`
insert into usage_flush_batches (id, usage_kind, record_count, created_at)
values
  ('real-usage-token-recovery-applied', 'token', 1, now()),
  ('real-usage-model-recovery-applied', 'model', 1, now())
on conflict (id) do nothing;`);
  await writeProcessingBatch(args.redis, args.keys, {
    tokenBatchId: 'real-usage-token-recovery-applied',
    modelBatchId: 'real-usage-model-recovery-applied',
    tokenId: args.tokenId,
    modelId: args.modelId,
    costUnits: 99999,
    count: 3,
    usedAt: '2026-05-15T08:01:00Z',
  });
  await args.startBackend();
  await waitForProcessingClear(args.redis, args.keys);
  const after = recoveryBaseline(args.db, args.tokenId, args.modelId);
  assertEqual(after.tokenRequestCount, before.tokenRequestCount, 'already marked token batch should not double count');
  assertEqual(after.modelUsageCount, before.modelUsageCount, 'already marked model batch should not double count');
  return { before, after };
}

async function writeProcessingBatch(redis, keys, input) {
  await redis.del(
    keys.processingTokenCost,
    keys.processingTokenCount,
    keys.processingTokenLastUsedAt,
    keys.processingTokenBatchId,
    keys.processingModelCount,
    keys.processingModelBatchId,
    keys.flushLock,
  );
  await redis.command('HSET', keys.processingTokenCost, input.tokenId, String(input.costUnits));
  await redis.command('HSET', keys.processingTokenCount, input.tokenId, String(input.count));
  await redis.command('HSET', keys.processingTokenLastUsedAt, input.tokenId, input.usedAt);
  await redis.command('SET', keys.processingTokenBatchId, input.tokenBatchId);
  await redis.command('HSET', keys.processingModelCount, input.modelId, String(input.count));
  await redis.command('SET', keys.processingModelBatchId, input.modelBatchId);
}

function recoveryBaseline(db, tokenId, modelId) {
  const [token] = db.rows(`
select used_quota::text, request_count::text, coalesce(last_used_at::text, '')
from api_tokens
where id = ${q(tokenId)};`);
  const modelUsageCount = Number(db.scalar(`select usage_count::text from global_models where id = ${q(modelId)};`));
  return {
    tokenUsedQuota: token?.[0] ?? '0',
    tokenRequestCount: Number(token?.[1] ?? 0),
    tokenLastUsedAt: token?.[2] ?? '',
    modelUsageCount,
  };
}

async function waitForProcessingClear(redis, keys, timeoutMs = 20_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const token = await redis.command('HLEN', keys.processingTokenCount);
    const model = await redis.command('HLEN', keys.processingModelCount);
    const tokenBatch = await redis.get(keys.processingTokenBatchId);
    const modelBatch = await redis.get(keys.processingModelBatchId);
    if (token === 0 && model === 0 && tokenBatch === null && modelBatch === null) {
      return;
    }
    await sleep(500);
  }
  throw new Error('usage processing keys were not cleared by recovery flush');
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
