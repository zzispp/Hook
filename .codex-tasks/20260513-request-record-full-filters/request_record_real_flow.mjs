import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { Db, q } from '../20260512-real-proxy-cache-flow/lib/db.mjs';
import { RedisClient } from '../20260512-real-proxy-cache-flow/lib/redis.mjs';
import { loadContext } from '../20260512-real-proxy-cache-flow/lib/env.mjs';
import { ensureBackend } from '../20260512-real-proxy-cache-flow/lib/backend.mjs';
import { assert, assertEqual } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import {
  groupCode,
  routeIds,
  tokenIds,
  providerNames,
  makeTokenValues,
  seedRouteDatabase,
  setClaudePrimaryKey,
  setOpenAIChatBaseUrl,
  setBrokenProviderActive,
  setOpenAIKeyPriorities,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
  seedUserAccessFixtures,
  resetUserAccessFixtures,
  deactivateUserAccessFixtures,
} from '../20260513-user-access-real-flow/lib/user_access_fixtures.mjs';
import {
  proxyCall,
  proxyStatus,
  successRow,
  adminSignIn,
  geminiRequest,
  assertNoAvailableRows,
  openAiChatRequest,
  assertStreamSuccess,
  expectProxyFailure,
  claudeMessagesRequest,
  openAiResponsesRequest,
  tracesSinceByTokenIds,
  assertSingleSuccessAttempt,
} from '../20260513-user-access-real-flow/lib/user_access_client.mjs';

const baseCtx = loadContext();
const db = new Db(baseCtx.db);
const ctx = contextWithExistingModels(baseCtx, db);
const redis = new RedisClient(ctx.redis);
const taskDir = dirname(fileURLToPath(import.meta.url));
const tokenValues = makeTokenValues();
const results = [];
const originalMode = db.scalar("select scheduling_mode from system_settings where id = 'global'") || 'fixed_order';

async function main() {
  const modelIds = seedRouteDatabase(ctx, db);
  seedUserAccessFixtures(db, tokenValues, modelIds);
  await resetAll(modelIds);
  clearTestRequestRows();
  const server = await ensureBackend(ctx.serverBaseUrl);
  try {
    await runScenarios(modelIds);
    assert(results.every((item) => item.ok), failedSummary());
    console.log('request record real flow: all scenarios passed');
  } finally {
    await cleanup(modelIds, server);
    writeResults();
  }
}

async function runScenarios(modelIds) {
  await step('fixed order non-stream and stream routes', () => fixedOrderRoutes(modelIds));
  await step('user provider and model allow matrix', () => userAllowMatrix(modelIds));
  await step('user provider and model deny matrix', () => userDenyMatrix(modelIds));
  await step('route key failover', () => routeKeyFailover(modelIds));
  await step('route endpoint fallback conversion', () => routeEndpointFallback(modelIds));
  await step('provider failover', () => providerFailover(modelIds));
  await step('cache affinity', () => cacheAffinity(modelIds));
  await step('load balance', () => loadBalance(modelIds));
  await step('format conversion matrix', () => formatConversionMatrix(modelIds));
  await step('active streaming request record', () => activeStreamingRequestRecord(modelIds));
  await step('100 concurrent mixed requests', () => highConcurrency(modelIds));
  await step('request record filters and details', () => requestRecordFiltersAndDetails(modelIds));
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence });
    console.log(`scenario passed: ${label}`);
  } catch (error) {
    results.push({ label, ok: false, error: error.stack || error.message });
    console.error(`scenario failed: ${label}: ${error.stack || error.message}`);
  }
}

async function fixedOrderRoutes(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const token = tokenValues.unrestricted;
  const chat = await proxyCall(ctx, db, token, 'fixed openai chat', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-chat')));
  assertSingleSuccessAttempt(chat, providerNames.openai, 'Route Hook primary');
  const stream = await proxyCall(ctx, db, token, 'fixed openai stream', openAiChatRequest(ctx, ctx.models.openai, marker('fixed-stream'), true));
  assertSingleSuccessAttempt(stream, providerNames.openai, 'Route Hook primary');
  assertStreamSuccess(stream, false);
  const responses = await proxyCall(ctx, db, token, 'fixed openai responses', openAiResponsesRequest(ctx.models.openai, marker('fixed-responses')));
  assertSingleSuccessAttempt(responses, providerNames.openai, 'Route Hook primary');
  const compact = await proxyCall(ctx, db, token, 'fixed openai compact', openAiResponsesRequest(ctx.models.openai, marker('fixed-compact'), true));
  assertSingleSuccessAttempt(compact, providerNames.openai, 'Route Hook primary');
  await assertSummaries([chat, stream, responses, compact]);
  return requestIds(chat, stream, responses, compact);
}

async function userAllowMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const openai = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai user allows openai',
    openAiChatRequest(ctx, ctx.models.openai, marker('allow-openai')),
  );
  assertSingleSuccessAttempt(openai, providerNames.openai, 'Route Hook primary');
  const claude = await proxyCall(
    ctx,
    db,
    tokenValues.claudeOnly,
    'claude user allows claude conversion',
    openAiChatRequest(ctx, ctx.models.claude, marker('allow-claude')),
  );
  assertEqual(successRow(claude.trace).provider_name, providerNames.claude, 'claude user should use Claude provider');
  assertEqual(successRow(claude.trace).needs_conversion, 'true', 'claude user should convert OpenAI request');
  const gemini = await proxyCall(
    ctx,
    db,
    tokenValues.geminiOnly,
    'gemini user allows gemini conversion',
    openAiChatRequest(ctx, ctx.models.gemini, marker('allow-gemini')),
  );
  assertEqual(successRow(gemini.trace).provider_name, providerNames.gemini, 'gemini user should use Gemini provider');
  assertEqual(successRow(gemini.trace).needs_conversion, 'true', 'gemini user should convert OpenAI request');
  await assertSummaries([openai, claude, gemini]);
  return requestIds(openai, claude, gemini);
}

async function userDenyMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const modelDenied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai user denied claude model',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-model')),
    403,
    'model is not allowed by user',
  );
  const modelOnlyDenied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.modelOpenaiOnly,
    'model-only user denied claude model',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-model-only')),
    403,
    'model is not allowed by user',
  );
  const providerDenied = await expectProxyFailure(
    ctx,
    db,
    tokenValues.providerMismatch,
    'provider mismatch user has no allowed provider route',
    openAiChatRequest(ctx, ctx.models.claude, marker('deny-provider')),
    404,
    'no active provider candidate',
  );
  return { modelDenied, modelOnlyDenied, providerDenied };
}

async function routeKeyFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setClaudePrimaryKey(db, ctx, 'sk-request-record-invalid');
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.claudeOnly,
    'claude key failover',
    openAiChatRequest(ctx, ctx.models.claude, marker('key-failover')),
  );
  const success = successRow(result.trace);
  assertEqual(result.trace[0].key_name, 'Route Claude primary', 'first attempt should use primary key');
  assertEqual(result.trace[0].status, 'failed', 'invalid primary key should fail visibly');
  assertEqual(success.key_name, 'Route Claude secondary', 'secondary key should succeed');
  assertEqual(success.needs_conversion, 'true', 'key failover should still convert OpenAI request');
  assertNoAvailableRows(result.trace);
  setClaudePrimaryKey(db, ctx, ctx.secrets.claudeKey);
  await assertSummaries([result]);
  return { requestId: result.requestId, attempts: result.trace.length };
}

async function routeEndpointFallback(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setOpenAIChatBaseUrl(db, 'http://127.0.0.1:9');
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'openai endpoint fallback',
    openAiChatRequest(ctx, ctx.models.openai, marker('endpoint-fallback')),
  );
  const success = successRow(result.trace);
  assert(Number(success.retry_index) >= 2, 'converted endpoint should be reached after exact endpoint key attempts');
  assertEqual(success.needs_conversion, 'true', 'endpoint fallback should convert OpenAI chat to Responses');
  assert(['openai_cli', 'openai_compact'].includes(success.provider_api_format), 'fallback should use an OpenAI Responses endpoint');
  assertNoAvailableRows(result.trace);
  setOpenAIChatBaseUrl(db, ctx.upstreams.openaiBaseUrl);
  await assertSummaries([result]);
  return { requestId: result.requestId, attempts: result.trace.length, endpoint: success.provider_api_format };
}

async function providerFailover(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  setBrokenProviderActive(db, true);
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.unrestricted,
    'provider failover',
    openAiChatRequest(ctx, ctx.models.openai, marker('provider-failover')),
  );
  const failures = result.trace.filter((row) => row.provider_name === providerNames.broken && row.status === 'failed');
  assert(failures.length >= 1, 'broken provider should fail before provider failover');
  assertEqual(successRow(result.trace).provider_name, providerNames.openai, 'provider failover should reach Hook Pool');
  assertNoAvailableRows(result.trace);
  setBrokenProviderActive(db, false);
  await assertSummaries([result]);
  return { requestId: result.requestId, attempts: result.trace.length };
}

async function cacheAffinity(modelIds) {
  await directSchedulingChange(modelIds, 'cache_affinity');
  setOpenAIKeyPriorities(db, 0, 1);
  await redis.setex(affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_chat'), 300, routeIds.keyOpenAISecondary);
  await clearScheduling();
  const result = await proxyCall(
    ctx,
    db,
    tokenValues.openaiOnly,
    'cache affinity',
    openAiChatRequest(ctx, ctx.models.openai, marker('affinity')),
  );
  assertEqual(successRow(result.trace).key_name, 'Route Hook secondary', 'affinity key should be attempted first');
  await assertSummaries([result]);
  return { requestId: result.requestId, key: successRow(result.trace).key_name };
}

async function loadBalance(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearScheduling();
  const keys = new Set();
  const requestIdsSeen = [];
  for (let index = 0; index < 20; index += 1) {
    const result = await proxyCall(
      ctx,
      db,
      tokenValues.openaiOnly,
      `load balance ${index}`,
      openAiChatRequest(ctx, ctx.models.openai, marker(`lb-${index}`)),
      { printTrace: index < 4 },
    );
    keys.add(successRow(result.trace).key_name);
    requestIdsSeen.push(result.requestId);
    assertSingleSuccessAttempt(result, providerNames.openai);
  }
  assert(keys.has('Route Hook primary') && keys.has('Route Hook secondary'), 'load balance should use both OpenAI keys');
  await waitForSummaries(requestIdsSeen);
  return { keys: [...keys].sort(), requests: requestIdsSeen.length };
}

async function formatConversionMatrix(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const cases = [
    ['openai to claude', tokenValues.claudeOnly, openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude')), providerNames.claude, true],
    [
      'openai stream to claude',
      tokenValues.claudeOnly,
      openAiChatRequest(ctx, ctx.models.claude, marker('openai-claude-stream'), true),
      providerNames.claude,
      true,
    ],
    ['openai to gemini', tokenValues.geminiOnly, openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini')), providerNames.gemini, true],
    [
      'openai stream to gemini',
      tokenValues.geminiOnly,
      openAiChatRequest(ctx, ctx.models.gemini, marker('openai-gemini-stream'), true),
      providerNames.gemini,
      true,
    ],
    ['claude to openai', tokenValues.openaiOnly, claudeMessagesRequest(ctx.models.openai, marker('claude-openai')), providerNames.openai, true],
    ['gemini exact', tokenValues.geminiOnly, geminiRequest(ctx.models.gemini, marker('gemini-exact')), providerNames.gemini, false],
    ['gemini stream exact', tokenValues.geminiOnly, geminiRequest(ctx.models.gemini, marker('gemini-stream'), true), providerNames.gemini, false],
  ];
  const evidence = [];
  for (const item of cases) evidence.push(await runConversionCase(...item));
  return evidence;
}

async function runConversionCase(label, token, request, providerName, converts) {
  const result = await proxyCall(ctx, db, token, label, request);
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerName, `${label} provider should match`);
  assertEqual(success.needs_conversion, String(converts), `${label} conversion flag should match`);
  if (request.body.stream) assertStreamSuccess(result, converts);
  assertNoAvailableRows(result.trace);
  await assertSummaries([result]);
  return { label, requestId: result.requestId, provider: success.provider_name, conversion: success.needs_conversion };
}

async function activeStreamingRequestRecord(modelIds) {
  await directSchedulingChange(modelIds, 'fixed_order');
  const adminToken = await adminSignIn(ctx);
  const text = marker(`active-stream-${Date.now()}`);
  const request = openAiChatRequest(ctx, ctx.models.openai, text, true);
  request.body.max_tokens = 180;
  request.body.messages[0].content = `${text} Write 160 short numbered words.`;
  const before = new Date(Date.now() - 1000).toISOString();
  const response = await sendProxyRequest(tokenValues.openaiOnly, request, 180_000);
  assert(response.ok, `active stream should return ok status, got ${response.status}`);
  const requestId = await waitForRequestIdByMarker(tokenIds.openaiOnly, before, text);
  const activeRecord = await waitForActiveRecord(adminToken, requestId);
  assert(['pending', 'streaming'].includes(activeRecord.status), 'active endpoint should expose pending or streaming record');
  assert(activeRecord.is_stream === true, 'active record should be marked stream');
  const streamText = await response.text();
  assert(streamText.includes('data:'), 'active stream response should contain SSE data');
  await waitForFinalSummary(requestId);
  return { requestId, activeStatus: activeRecord.status };
}

async function highConcurrency(modelIds) {
  await directSchedulingChange(modelIds, 'load_balance');
  setOpenAIKeyPriorities(db, 0, 0);
  await clearScheduling();
  const markerRoot = marker(`concurrency-${Date.now()}`);
  const before = new Date().toISOString();
  const requests = concurrencyRequests(markerRoot);
  const responses = await Promise.all(requests.map((item) => proxyStatus(ctx, item.token, item.request)));
  assert(
    responses.every((item) => item.ok),
    `all 100 high concurrency requests should succeed: ${JSON.stringify(statusCounts(responses))}`,
  );
  const traces = tracesSinceByTokenIds(db, before, [tokenIds.openaiOnly]);
  assertEqual(traces.length, requests.length, 'high concurrency should record every request');
  const successes = traces.map(({ trace }) => successRow(trace));
  const openaiKeys = new Set(successes.filter((row) => row.provider_name === providerNames.openai).map((row) => row.key_name));
  assert(openaiKeys.has('Route Hook primary') && openaiKeys.has('Route Hook secondary'), 'concurrency load balance should use both OpenAI keys');
  assertEqual(traces.flatMap(({ trace }) => trace.filter((row) => row.status === 'failed')).length, 0, 'concurrency should not fail attempts');
  await waitForSummaries(traces.map((item) => item.requestId));
  return {
    requests: requests.length,
    stream: 70,
    nonStream: 30,
    openaiKeys: [...openaiKeys].sort(),
  };
}

function concurrencyRequests(markerRoot) {
  const requests = [];
  for (let index = 0; index < 30; index += 1) {
    requests.push({
      token: tokenValues.openaiOnly,
      request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|nonstream|${String(index).padStart(3, '0')}|`),
    });
  }
  for (let index = 0; index < 70; index += 1) {
    requests.push({
      token: tokenValues.openaiOnly,
      request: openAiChatRequest(ctx, ctx.models.openai, `${markerRoot}|stream|${String(index).padStart(3, '0')}|`, true),
    });
  }
  return requests;
}

async function requestRecordFiltersAndDetails(modelIds) {
  const adminToken = await adminSignIn(ctx);
  const openaiSearch = 'User access openaiOnly';
  const base = { search: openaiSearch, status: 'success' };
  const all = await listRequestRecords(adminToken, { ...base, limit: 100 });
  assert(all.total >= 100, 'openaiOnly token should have request record summaries from real flow');
  assert(all.records.every((record) => record.token_name === openaiSearch), 'search should constrain records by token name');
  await assertRecordFilter(adminToken, { ...base, model_id: modelIds.openai }, (record) => record.global_model_id === modelIds.openai, 'model_id');
  await assertRecordFilter(adminToken, { ...base, provider_id: routeIds.providerOpenAI }, (record) => record.provider_id === routeIds.providerOpenAI, 'provider_id');
  await assertRecordFilter(
    adminToken,
    { ...base, api_format: 'openai_chat' },
    (record) => record.client_api_format === 'openai_chat' || record.provider_api_format === 'openai_chat',
    'api_format',
  );
  await assertRecordFilter(adminToken, { ...base, type: 'stream' }, (record) => record.is_stream === true, 'stream type');
  await assertRecordFilter(adminToken, { ...base, type: 'non_stream' }, (record) => record.is_stream === false, 'non_stream type');
  const detail = await getRequestRecord(adminToken, all.records[0].request_id);
  assertEqual(detail.record.request_id, all.records[0].request_id, 'detail should return the selected record');
  assertEqual(detail.candidates.length, detail.record.candidate_count, 'detail candidate count should match summary');
  const streamRow = await firstRecord(adminToken, { ...base, type: 'stream' });
  assert(streamRow.first_byte_time_ms !== null && streamRow.first_byte_time_ms !== undefined, 'stream summary should include first byte time');
  assert(streamRow.total_latency_ms !== null && streamRow.total_latency_ms !== undefined, 'stream summary should include total latency');
  return {
    total: all.total,
    sample: all.records[0].request_id,
    streamSample: streamRow.request_id,
  };
}

async function assertRecordFilter(adminToken, params, predicate, label) {
  const response = await listRequestRecords(adminToken, { ...params, limit: 50 });
  assert(response.total > 0, `${label} filter should return records`);
  assert(response.records.every(predicate), `${label} filter should only return matching records`);
}

async function firstRecord(adminToken, params) {
  const response = await listRequestRecords(adminToken, { ...params, limit: 1 });
  assert(response.records.length === 1, 'expected one request record');
  return response.records[0];
}

async function listRequestRecords(adminToken, params) {
  const url = new URL(`${ctx.serverBaseUrl}/api/admin/request-records`);
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') url.searchParams.set(key, String(value));
  }
  const body = await fetchJson(url, adminToken);
  return body.data;
}

async function getRequestRecord(adminToken, requestId) {
  const body = await fetchJson(`${ctx.serverBaseUrl}/api/admin/request-records/${encodeURIComponent(requestId)}`, adminToken);
  return body.data;
}

async function fetchJson(url, adminToken) {
  const response = await fetch(url, { headers: { authorization: `Bearer ${adminToken}` } });
  const text = await response.text();
  if (!response.ok) throw new Error(`request record API failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  assert(body.success, 'request record API should return success envelope');
  return body;
}

async function activeRequestRecords(adminToken, ids) {
  const response = await fetch(`${ctx.serverBaseUrl}/api/admin/request-records/active`, {
    method: 'POST',
    headers: { authorization: `Bearer ${adminToken}`, 'content-type': 'application/json' },
    body: JSON.stringify({ ids }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`active request record API failed ${response.status}: ${text}`);
  const body = JSON.parse(text);
  assert(body.success, 'active request record API should return success envelope');
  return body.data.records;
}

async function waitForActiveRecord(adminToken, requestId) {
  const started = Date.now();
  while (Date.now() - started < 8_000) {
    const records = await activeRequestRecords(adminToken, [requestId]);
    const record = records.find((item) => item.request_id === requestId);
    if (record && ['pending', 'streaming'].includes(record.status)) return record;
    await sleep(250);
  }
  throw new Error(`active record was not visible while streaming: ${requestId}`);
}

async function sendProxyRequest(token, request, timeoutMs) {
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

async function assertSummaries(resultsToCheck) {
  await waitForSummaries(resultsToCheck.map((item) => item.requestId));
  for (const result of resultsToCheck) {
    const summary = summaryRow(result.requestId);
    assertEqual(summary.status, 'success', 'summary status should be success');
    assertEqual(summary.candidate_count, String(result.trace.length), 'summary candidate count should match trace');
    assertEqual(summary.has_failover, String(hasFailover(result.trace)), 'summary failover flag should match trace');
    assertEqual(summary.has_retry, String(hasRetry(result.trace)), 'summary retry flag should match trace');
    if (successRow(result.trace).is_stream === 'true') {
      assert(summary.first_byte_time_ms !== '', 'stream summary should record first byte');
      assert(summary.total_latency_ms !== '', 'stream summary should record total latency');
    }
  }
}

async function waitForSummaries(requestIds) {
  const expected = [...new Set(requestIds)].filter(Boolean);
  const started = Date.now();
  while (Date.now() - started < 10_000) {
    const count = Number(db.scalar(`select count(*) from request_records where request_id in (${expected.map(q).join(',')});`));
    if (count === expected.length) return;
    await sleep(200);
  }
  throw new Error(`missing request_records summaries for ${expected.length} request ids`);
}

async function waitForFinalSummary(requestId) {
  const started = Date.now();
  while (Date.now() - started < 20_000) {
    const row = summaryRow(requestId);
    if (row.status === 'success' && row.total_latency_ms !== '') return row;
    await sleep(300);
  }
  throw new Error(`request record summary did not finish: ${requestId}`);
}

async function waitForRequestIdByMarker(tokenId, before, text) {
  const started = Date.now();
  while (Date.now() - started < 8_000) {
    const requestId = db.scalar(`
select request_id
from request_candidates
where created_at >= ${q(before)}
  and token_id = ${q(tokenId)}
  and request_body like ${q(`%${text}%`)}
order by created_at desc
limit 1;`);
    if (requestId) return requestId;
    await sleep(200);
  }
  throw new Error(`request candidate was not created for marker: ${text}`);
}

function summaryRow(requestId) {
  const rows = db.rows(`
select request_id, status, candidate_count::text, has_failover::text, has_retry::text,
  coalesce(first_byte_time_ms::text, ''), coalesce(total_latency_ms::text, ''),
  is_stream::text, coalesce(provider_id, ''), coalesce(global_model_id, '')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
  assert(rows.length === 1, `summary row should exist: ${requestId}`);
  const row = rows[0];
  return {
    request_id: row[0],
    status: row[1],
    candidate_count: row[2],
    has_failover: row[3],
    has_retry: row[4],
    first_byte_time_ms: row[5],
    total_latency_ms: row[6],
    is_stream: row[7],
    provider_id: row[8],
    global_model_id: row[9],
  };
}

function hasFailover(trace) {
  const indexes = trace.filter((row) => row.status === 'success' || row.status === 'failed').map((row) => row.candidate_index);
  return new Set(indexes).size > 1;
}

function hasRetry(trace) {
  return trace.some((row) => (row.status === 'success' || row.status === 'failed') && Number(row.retry_index) > 0);
}

function statusCounts(responses) {
  const counts = {};
  for (const response of responses) counts[String(response.status)] = (counts[String(response.status)] ?? 0) + 1;
  return counts;
}

async function directSchedulingChange(modelIds, mode) {
  setSchedulingModeDb(db, mode);
  await clearScheduling();
  await clearAffinity(modelIds);
}

async function resetAll(modelIds) {
  restoreRouteFixtures(ctx, db, originalMode);
  resetUserAccessFixtures(db, tokenValues, modelIds);
  await clearScheduling();
  await clearAuth();
  await clearAffinity(modelIds);
}

async function cleanup(modelIds, server) {
  try {
    restoreRouteFixtures(ctx, db, originalMode);
    deactivateUserAccessFixtures(db);
    deactivateRouteFixtures(db);
    await clearScheduling();
    await clearAuth();
    await clearAffinity(modelIds);
  } finally {
    if (server) server.kill('SIGTERM');
  }
}

function clearTestRequestRows() {
  db.exec(`
delete from request_records
where request_id in (select distinct request_id from request_candidates where token_id in (${Object.values(tokenIds).map(q).join(',')}));
delete from request_candidates where token_id in (${Object.values(tokenIds).map(q).join(',')});`);
}

async function clearScheduling() {
  await redis.del(schedulingSnapshotKey(), schedulingLockKey());
}

async function clearAuth() {
  const keys = await redis.keys(`${ctx.redis.prefix}:llm_proxy:auth:v*`);
  await redis.del(...keys, `${ctx.redis.prefix}:llm_proxy:auth:version`);
}

async function clearAffinity(modelIds) {
  await redis.del(
    affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_chat'),
    affinityKey(tokenIds.openaiOnly, modelIds.openai, 'openai_cli'),
    affinityKey(tokenIds.claudeOnly, modelIds.claude, 'openai_chat'),
    affinityKey(tokenIds.geminiOnly, modelIds.gemini, 'openai_chat'),
    affinityKey(tokenIds.geminiOnly, modelIds.gemini, 'gemini_chat'),
    affinityKey(tokenIds.unrestricted, modelIds.openai, 'openai_chat'),
  );
}

function affinityKey(tokenId, modelId, format) {
  return `${ctx.redis.prefix}:llm_proxy:affinity:${tokenId}:${modelId}:${format}`;
}

function schedulingSnapshotKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:snapshot:v2`;
}

function schedulingLockKey() {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:rebuild_lock`;
}

function contextWithExistingModels(ctx, database) {
  const openai = selectedModel(database, process.env.HOOK_OPENAI_MODEL, (name) => name.startsWith('gpt-'), 'OpenAI');
  const claude = selectedModel(database, process.env.HOOK_CLAUDE_MODEL, (name) => name.startsWith('claude-'), 'Claude');
  const gemini = selectedModel(database, process.env.HOOK_GEMINI_MODEL, (name) => name.includes('gemini'), 'Gemini');
  const models = Object.freeze({
    openai,
    claude,
    gemini,
    openaiProvider: process.env.HOOK_OPENAI_PROVIDER_MODEL || openai,
    claudeProvider: process.env.HOOK_CLAUDE_PROVIDER_MODEL || claude,
    geminiProvider: process.env.HOOK_GEMINI_PROVIDER_MODEL || geminiProviderName(gemini),
  });
  console.log(`models: ${JSON.stringify(models)}`);
  return Object.freeze({ ...ctx, models });
}

function selectedModel(database, configured, matches, label) {
  if (configured && modelExists(database, configured)) return configured;
  if (configured) throw new Error(`${label} model does not exist in global_models: ${configured}`);
  const candidates = database.rows("select name from global_models where is_active = true order by created_at desc, name;").map(([value]) => value);
  const matched = candidates.find(matches);
  if (!matched) throw new Error(`${label} model not found in active global_models; available: ${candidates.join(', ') || 'none'}`);
  return matched;
}

function modelExists(database, name) {
  return database.scalar(`select id from global_models where name = ${q(name)} and is_active = true limit 1;`) !== '';
}

function geminiProviderName(globalName) {
  return globalName.startsWith('gemini-') ? `[满血]${globalName}` : globalName;
}

function marker(value) {
  return `request-record-real-${value}`;
}

function requestIds(...items) {
  return items.map((item) => item.requestId);
}

function failedSummary() {
  return `failed scenarios: ${results.filter((item) => !item.ok).map((item) => item.label).join(', ')}`;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function writeResults() {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'request_record_real_flow_results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
