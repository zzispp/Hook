import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { ensureBackend, stopBackend } from './lib/backend.mjs';
import { randomToken } from './lib/crypto.mjs';
import { DockerDb } from './lib/db.mjs';
import { cleanupFixtures, clearTestRows, seedFixtures } from './lib/fixtures.mjs';
import { loadContext } from './lib/context.mjs';
import { proxyChat } from './lib/proxy.mjs';
import { RedisClient } from './lib/redis.mjs';
import { resolveUpstreams } from './lib/upstream.mjs';
import {
  assertSchemaReady,
  baselineSnapshot,
  clearCaches,
  validateFinal,
  validateScenario,
  waitForUsageFlush,
} from './lib/validation.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const ctx = loadContext();
const db = new DockerDb(ctx.db);
const redis = new RedisClient(ctx.redis);
const tokenValue = randomToken('sk-aether-real');
const results = [];

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

async function main() {
  assertSchemaReady(db);
  const upstream = await resolveUpstreams(ctx);
  seedFixtures(ctx, db, tokenValue, upstream);
  clearTestRows(db);
  await clearCaches(redis, ctx.redis.prefix);
  const backend = await ensureBackend(ctx.serverBaseUrl);
  try {
    const before = baselineSnapshot(db);
    const game86 = await runScenario('86gamestore', 'aether-real-86gamestore-chat', upstream.game86Model);
    const ekan8 = await runScenario('ekan8 mapped', 'aether-real-ekan8-mapped-chat', upstream.ekan8OpenAiModel);
    const after = await waitForUsageFlush(db, before, [game86, ekan8]);
    validateFinal(before, after, [game86, ekan8]);
    const evidence = buildEvidence(upstream, before, after, [game86, ekan8]);
    writeResults(evidence);
    console.log(JSON.stringify(evidence, null, 2));
  } finally {
    cleanupFixtures(db);
    await clearCaches(redis, ctx.redis.prefix);
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

async function runScenario(label, modelName, providerModel) {
  const marker = `${label.replaceAll(' ', '-')}-${Date.now()}`;
  const result = await proxyChat(ctx, db, tokenValue, modelName, marker);
  validateScenario(label, result, providerModel);
  results.push({
    label,
    requestId: result.requestId,
    modelName,
    providerModel,
  });
  return result;
}

function buildEvidence(upstream, before, after, scenarioResults) {
  return {
    executed_at: new Date().toISOString(),
    upstream: {
      game86_model: upstream.game86Model,
      ekan8_openai_model: upstream.ekan8OpenAiModel,
      game86_model_sample: upstream.game86Models,
      ekan8_model_sample: upstream.ekan8Models,
    },
    before,
    after,
    scenarios: scenarioResults.map((item) => ({
      request_id: item.requestId,
      status: item.record.status,
      billing_status: item.record.billing_status,
      prompt_tokens: item.record.prompt_tokens,
      completion_tokens: item.record.completion_tokens,
      base_cost: item.record.base_cost,
      total_cost: item.record.total_cost,
      multiplier: item.record.billing_multiplier,
      snapshot_status: item.record.billing_snapshot.status,
      snapshot_scope: item.record.billing_snapshot.scope,
      snapshot_rule_id: item.record.billing_snapshot.rule_id,
      provider_request_model: item.candidate.provider_request_body.model,
      client_response_model: item.record.client_response_body.model ?? null,
      trace: item.trace,
    })),
    generated_token_prefix: tokenValue.slice(0, 10),
  };
}

function writeResults(evidence) {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(evidence, null, 2)}\n`);
}
