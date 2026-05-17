import { GROUP_CODE, GROUP_MULTIPLIER, IDS, INITIAL_WALLET_BALANCE } from './fixtures.mjs';
import { assert, assertDecimalEqual, assertEqual, assertGreaterThan } from './assertions.mjs';
import { q } from './db.mjs';

export function assertSchemaReady(db) {
  const requiredTables = ['billing_rules', 'dimension_collectors', 'request_records', 'request_candidates'];
  for (const table of requiredTables) {
    const exists = db.scalar(`select to_regclass(${q(`public.${table}`)})::text;`);
    assertEqual(exists, table, `${table} table should exist`);
  }
  const columns = db.rows(`
select table_name, column_name
from information_schema.columns
where table_schema = 'public'
  and (table_name, column_name) in (('request_records', 'billing_snapshot'), ('request_candidates', 'billing_snapshot'));`);
  assertEqual(columns.length, 2, 'billing_snapshot columns should exist on request and candidate records');
}

export function clearCaches(redis, prefix) {
  return Promise.all([
    clearPattern(redis, `${prefix}:llm_proxy:auth:v*`),
    clearPattern(redis, `${prefix}:llm_proxy:affinity:${IDS.token}:*`),
    clearUsageFields(redis, prefix),
    clearEmptyProcessingBatch(redis, prefix, 'token'),
    clearEmptyProcessingBatch(redis, prefix, 'model'),
    redis.del(
      `${prefix}:llm_proxy:auth:version`,
      `${prefix}:llm_proxy:scheduling:snapshot:v2`,
      `${prefix}:llm_proxy:scheduling:rebuild_lock`,
      `${prefix}:llm_proxy:usage:flush_lock`,
    ),
  ]);
}

export function baselineSnapshot(db) {
  return {
    wallet: walletSnapshot(db),
    token: tokenSnapshot(db),
    game86Usage: Number(db.scalar(`select usage_count::text from global_models where id = ${q(IDS.modelGame86)};`)),
    ekan8Usage: Number(db.scalar(`select usage_count::text from global_models where id = ${q(IDS.modelEkan8)};`)),
  };
}

export function validateScenario(name, result, expectedProviderModel) {
  const snapshot = result.record.billing_snapshot;
  const candidateSnapshot = result.candidate.billing_snapshot;
  assertEqual(snapshot.status, 'complete', `${name} record billing snapshot should be complete`);
  assertEqual(candidateSnapshot.status, 'complete', `${name} candidate billing snapshot should be complete`);
  assertEqual(snapshot.group_code, GROUP_CODE, `${name} snapshot group code should match`);
  assertDecimalEqual(snapshot.billing_multiplier, GROUP_MULTIPLIER, `${name} snapshot multiplier should match`);
  assertDecimalEqual(result.record.base_cost, snapshot.base_total_cost, `${name} record base cost should come from snapshot`);
  assertDecimalEqual(result.record.total_cost, snapshot.total_cost, `${name} record total cost should come from snapshot`);
  assertDecimalEqual(result.candidate.total_cost, result.record.total_cost, `${name} candidate total should match record total`);
  assertGreaterThan(result.record.prompt_tokens, 0, `${name} prompt tokens should be recorded`);
  assertGreaterThan(result.record.completion_tokens, 0, `${name} completion tokens should be recorded`);
  assertGreaterThan(result.record.total_cost, 0, `${name} total cost should be positive`);
  assertEqual(result.candidate.provider_request_body.model, expectedProviderModel, `${name} provider request model should be mapped`);
  assertEqual(snapshot.scope, 'global', `${name} billing rule scope should be global`);
  assert(snapshot.rule_id, `${name} billing rule id should be present`);
  assert(snapshot.resolved_dimensions.input_tokens, `${name} input_tokens dimension should be present`);
  assert(snapshot.resolved_dimensions.output_tokens, `${name} output_tokens dimension should be present`);
  assert(snapshot.cost_breakdown.input_cost, `${name} input_cost breakdown should be present`);
  assert(snapshot.cost_breakdown.output_cost, `${name} output_cost breakdown should be present`);
  assert(snapshot.cost_breakdown.request_cost, `${name} request_cost breakdown should be present`);
}

export function finalSnapshot(db) {
  return {
    wallet: walletSnapshot(db),
    token: tokenSnapshot(db),
    game86Usage: Number(db.scalar(`select usage_count::text from global_models where id = ${q(IDS.modelGame86)};`)),
    ekan8Usage: Number(db.scalar(`select usage_count::text from global_models where id = ${q(IDS.modelEkan8)};`)),
    walletTransactions: Number(db.scalar(`select count(*)::text from wallet_transactions where wallet_id = ${q(IDS.wallet)};`)),
  };
}

export async function waitForUsageFlush(db, before, results) {
  const started = Date.now();
  while (Date.now() - started < 20000) {
    const current = finalSnapshot(db);
    try {
      validateUsageFlush(before, current, results);
      return current;
    } catch {
      await sleep(500);
    }
  }
  const current = finalSnapshot(db);
  validateUsageFlush(before, current, results);
  return current;
}

export function validateFinal(before, after, results) {
  const totalCost = results.reduce((sum, item) => sum + Number(item.record.total_cost), 0);
  assertDecimalEqual(after.wallet.totalConsumed, totalCost, 'wallet total_consumed should equal request total costs');
  assertDecimalEqual(after.wallet.rechargeBalance, Number(INITIAL_WALLET_BALANCE) - totalCost, 'wallet recharge balance should be reduced');
  validateUsageFlush(before, after, results);
  assertEqual(after.walletTransactions, results.length, 'wallet should have one consume transaction per request');
}

function validateUsageFlush(before, after, results) {
  const totalCost = results.reduce((sum, item) => sum + Number(item.record.total_cost), 0);
  assertDecimalEqual(after.token.usedQuota, totalCost, 'token used_quota should equal request total costs');
  assertEqual(after.token.requestCount - before.token.requestCount, results.length, 'token request_count should increment');
  assertEqual(after.game86Usage - before.game86Usage, 1, '86gamestore global model usage_count should increment');
  assertEqual(after.ekan8Usage - before.ekan8Usage, 1, 'Ekan8 global model usage_count should increment');
}

function walletSnapshot(db) {
  const [row] = db.rows(`
select recharge_balance::text, gift_balance::text, total_consumed::text
from wallets where id = ${q(IDS.wallet)} limit 1;`);
  assert(row, 'test wallet should exist');
  return {
    rechargeBalance: Number(row[0]),
    giftBalance: Number(row[1]),
    totalConsumed: Number(row[2]),
  };
}

function tokenSnapshot(db) {
  const [row] = db.rows(`
select used_quota::text, request_count::text
from api_tokens where id = ${q(IDS.token)} limit 1;`);
  assert(row, 'test api token should exist');
  return {
    usedQuota: Number(row[0]),
    requestCount: Number(row[1]),
  };
}

async function clearPattern(redis, pattern) {
  const keys = await redis.keys(pattern);
  await redis.del(...keys);
}

async function clearUsageFields(redis, prefix) {
  await Promise.all([
    redis.hdel(`${prefix}:llm_proxy:usage:pending:token:cost`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:pending:token:count`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:pending:token:last_used_at`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:processing:token:cost`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:processing:token:count`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:processing:token:last_used_at`, IDS.token),
    redis.hdel(`${prefix}:llm_proxy:usage:pending:model:count`, IDS.modelGame86, IDS.modelEkan8),
    redis.hdel(`${prefix}:llm_proxy:usage:processing:model:count`, IDS.modelGame86, IDS.modelEkan8),
  ]);
}

async function clearEmptyProcessingBatch(redis, prefix, kind) {
  const countKey = kind === 'token' ? `${prefix}:llm_proxy:usage:processing:token:count` : `${prefix}:llm_proxy:usage:processing:model:count`;
  const batchKey = `${prefix}:llm_proxy:usage:processing:${kind}:batch_id`;
  const count = await redis.hlen(countKey);
  if (count === 0) {
    await redis.del(batchKey);
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
