import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assert } from '../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { ensureBackend } from '../20260512-real-proxy-cache-flow/lib/backend.mjs';
import { stopBackend } from '../20260514-real-request-record-flow/lib/backend_session.mjs';
import { DockerDb } from '../20260514-model-usage-real-flow/lib/docker_db.mjs';
import { createAdminToken, deleteAdminToken, signInAdmin, adminJson } from '../20260515-real-usage-concurrency-flow/lib/real_usage_admin.mjs';
import { cleanupRealUsageFixtures } from '../20260515-real-usage-concurrency-flow/lib/real_usage_cleanup.mjs';
import { openAiChatRequest, runConcurrentProxyCalls, waitForTerminalRecords } from '../20260515-real-usage-concurrency-flow/lib/real_usage_client.mjs';
import { assertFixtureModel, fixtureIds, fixtureUserIds, groupCode, seedRealUsageFixtures } from '../20260515-real-usage-concurrency-flow/lib/real_usage_fixtures.mjs';
import { applyRealUsageSettings, restoreSystemSettings, systemSettingsSnapshot } from '../20260515-real-usage-concurrency-flow/lib/real_usage_settings.mjs';
import { resolveUpstreamModels } from '../20260515-real-usage-concurrency-flow/lib/real_usage_upstreams.mjs';
import { loadPerfMonitoringContext } from './lib/perf_monitoring_env.mjs';
import { assertRealtimeSnapshot, assertSnapshotMetrics, expectedMetrics, requestBucket, waitForSnapshot } from './lib/perf_monitoring_metrics.mjs';

const taskDir = dirname(fileURLToPath(import.meta.url));
const results = [];
const createdTokenIds = [];
let ctx;
let db;
let backend;
let adminToken = '';
let originalSettings;

async function main() {
  await step('load context and check schema', loadContext);
  const upstream = await step('fetch real upstream models', () => resolveUpstreamModels(ctx));
  await step('seed DB fixtures', () => seedFixtures(upstream));
  await step('start or reuse backend', startHarnessBackend);
  adminToken = await step('sign in admin', () => signInAdmin(ctx));
  try {
    const token = await step('create customer token', createCustomerToken);
    const load = await step('real proxy requests in one minute bucket', () => runTraffic(token));
    const snapshot = await step('wait worker minute snapshot', () => waitForSnapshot(db, load.bucket.startedAt, ctx.perfLoad.snapshotTimeoutMs));
    await step('verify DB aggregate equals snapshot', () => verifySnapshot(load, snapshot));
    await step('verify realtime API returns snapshot', () => verifyRealtime(snapshot));
    assert(results.every((item) => item.ok), 'all scenarios should pass');
    console.log('real performance monitoring flow: all scenarios passed');
  } finally {
    await cleanup();
    writeResults();
  }
}

async function startHarnessBackend() {
  backend = await ensureBackend(ctx.serverBaseUrl);
  return { mode: backend ? 'started_by_harness' : 'existing_process' };
}

function loadContext() {
  ctx = loadPerfMonitoringContext();
  db = new DockerDb({
    container: process.env.HOOK_PG_CONTAINER || 'hook-postgres',
    user: ctx.db.user,
    name: ctx.db.name,
  });
  prepareLocalPerformanceSchema();
  originalSettings = systemSettingsSnapshot(db);
  ensureSchema();
  return {
    serverBaseUrl: ctx.serverBaseUrl,
    db: { host: ctx.db.host, port: ctx.db.port, name: ctx.db.name },
    requestPlan: ctx.perfLoad,
  };
}

function ensureSchema() {
  const missing = ['request_records', 'performance_monitoring_snapshots'].filter((table) => !tableExists(table));
  if (missing.length > 0) {
    throw new Error(`local DB schema is missing tables: ${missing.join(', ')}`);
  }
}

function prepareLocalPerformanceSchema() {
  db.exec(`
alter table system_settings
  add column if not exists performance_monitoring_retention_days bigint not null default 30;

create table if not exists performance_monitoring_snapshots (
  id varchar(36) primary key,
  bucket_granularity varchar(16) not null,
  bucket_started_at timestamptz not null,
  bucket_ended_at timestamptz not null,
  metrics text not null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);

create unique index if not exists index_performance_monitoring_snapshots_unique_bucket
  on performance_monitoring_snapshots (bucket_granularity, bucket_started_at);
create index if not exists index_performance_monitoring_snapshots_by_bucket
  on performance_monitoring_snapshots (bucket_granularity, bucket_started_at);

create table if not exists usage_flush_batches (
  id varchar(36) primary key,
  usage_kind varchar(20) not null,
  record_count bigint not null,
  created_at timestamptz not null
);

insert into api_permissions (id, code, method, path_pattern, name, enabled, system, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000461', 'performance_monitoring_overview_read', 'GET', '/api/admin/performance-monitoring/overview', '读取性能监控概览', true, true, now(), now()),
  ('00000000-0000-7000-8000-000000000462', 'performance_monitoring_realtime_read', 'GET', '/api/admin/performance-monitoring/realtime', '读取性能监控实时指标', true, true, now(), now())
on conflict (code) do update set
  method = excluded.method,
  path_pattern = excluded.path_pattern,
  name = excluded.name,
  enabled = true,
  system = true,
  updated_at = now();

insert into menu_items (id, section_id, parent_id, code, title, route_path, icon, caption, deep_match, sort_order, enabled, created_at, updated_at)
values ('00000000-0000-7000-8000-000000000223', '00000000-0000-7000-8000-000000000101', null, 'admin_performance_monitoring',
  '性能监控', '/dashboard/admin/performance-monitoring', 'solar:monitor-bold', null, true, 1, true, now(), now())
on conflict (code) do update set
  section_id = excluded.section_id,
  title = excluded.title,
  route_path = excluded.route_path,
  icon = excluded.icon,
  deep_match = excluded.deep_match,
  sort_order = excluded.sort_order,
  enabled = true,
  updated_at = now();

insert into menu_api_permissions (menu_item_id, api_permission_id, created_at, updated_at)
select '00000000-0000-7000-8000-000000000223', id, now(), now()
from api_permissions
where code in ('performance_monitoring_overview_read', 'performance_monitoring_realtime_read')
on conflict (menu_item_id, api_permission_id) do update set updated_at = now();

insert into role_menu_permissions (role_code, menu_item_id, created_at, updated_at)
values ('admin', '00000000-0000-7000-8000-000000000223', now(), now())
on conflict (role_code, menu_item_id) do update set updated_at = now();

insert into role_api_permissions (role_code, api_permission_id, created_at, updated_at)
select 'admin', id, now(), now()
from api_permissions
where code in ('performance_monitoring_overview_read', 'performance_monitoring_realtime_read')
on conflict (role_code, api_permission_id) do update set updated_at = now();`);
}

function seedFixtures(upstream) {
  cleanupRealUsageFixtures(db, []);
  deleteFixtureSnapshots();
  seedRealUsageFixtures(ctx, db, upstream);
  applyRealUsageSettings(db);
  assertFixtureModel(db, ctx.realModels.chat);
  return { model: ctx.realModels.chat, groupCode, upstream };
}

async function createCustomerToken() {
  const data = await createAdminToken(ctx, adminToken, {
    name: 'Performance monitoring real token',
    token_type: 'user',
    user_id: fixtureUserIds[0],
    group_code: groupCode,
    model_access_mode: 'all',
    allowed_model_ids: [],
    rate_limit_rpm: 0,
  });
  createdTokenIds.push(data.token.id);
  return { id: data.token.id, value: data.raw_token, prefix: data.token.token_prefix };
}

async function runTraffic(token) {
  await waitForSafeMinuteWindow();
  const requests = trafficRequests(token);
  const responses = await runConcurrentProxyCalls(ctx, db, requests, ctx.perfLoad.requestTimeoutMs);
  const requestIds = responses.map((item) => item.requestId);
  await waitForTerminalRecords(db, requestIds);
  const bucket = requestBucket(db, requestIds);
  assert(bucket.requestCount === requestIds.length, 'all test requests should be in the selected bucket');
  return { requestIds, bucket, sample: responseSample(responses) };
}

function trafficRequests(token) {
  const total = ctx.perfLoad.nonStreamRequests + ctx.perfLoad.streamRequests;
  return Array.from({ length: total }, (_, index) => {
    const stream = index >= ctx.perfLoad.nonStreamRequests;
    const marker = `perf-monitoring-real-${Date.now()}-${index}`;
    return { token: token.value, request: openAiChatRequest(ctx.realModels.chat, marker, stream) };
  });
}

async function verifySnapshot(load, snapshot) {
  const expected = expectedMetrics(db, load.bucket.startedAt);
  assertSnapshotMetrics(snapshot, expected);
  assert(snapshot.metrics.core.request_count >= load.requestIds.length, 'snapshot should include test requests');
  return {
    bucket: load.bucket,
    snapshot: snapshotSummary(snapshot),
    expected: evidenceMetrics(expected),
    trafficSample: load.sample,
  };
}

async function verifyRealtime(snapshot) {
  const realtime = await adminJson(ctx, adminToken, 'GET', '/api/admin/performance-monitoring/realtime');
  assertRealtimeSnapshot(realtime, snapshot);
  return {
    bucket_started_at: realtime.snapshot.bucket_started_at,
    request_count: realtime.snapshot.metrics.core.request_count,
    total_tokens: realtime.snapshot.metrics.llm.total_tokens,
    host_status: realtime.host.metrics.status,
  };
}

async function step(label, action) {
  console.log(`scenario: ${label}`);
  try {
    const evidence = await action();
    results.push({ label, ok: true, evidence: redactEvidence(evidence) });
    console.log(`scenario passed: ${label}`);
    return evidence;
  } catch (error) {
    results.push({ label, ok: false, error: redactText(error.stack || error.message) });
    console.error(`scenario failed: ${label}: ${redactText(error.message)}`);
    throw error;
  }
}

async function cleanup() {
  for (const tokenId of createdTokenIds) {
    await cleanupToken(tokenId);
  }
  if (db) {
    cleanupRealUsageFixtures(db, createdTokenIds);
    restoreSystemSettings(db, originalSettings);
  }
  if (backend) {
    await stopBackend(backend, ctx.serverBaseUrl);
  }
}

async function cleanupToken(tokenId) {
  try {
    if (adminToken) {
      await deleteAdminToken(ctx, adminToken, tokenId);
    }
  } catch (error) {
    results.push({ label: `cleanup token ${tokenId}`, ok: false, error: redactText(error.message) });
  }
}

async function waitForSafeMinuteWindow() {
  while (new Date().getUTCSeconds() > 20) {
    await sleep(500);
  }
}

function deleteFixtureSnapshots() {
  db.exec("delete from performance_monitoring_snapshots where metrics like '%hook-real-usage-chat%' or metrics like '%Performance monitoring real token%';");
}

function snapshotSummary(snapshot) {
  return {
    bucket_started_at: snapshot.bucket_started_at,
    request_count: snapshot.metrics.core.request_count,
    success_rate: snapshot.metrics.core.success_rate,
    stream_request_count: snapshot.metrics.core.stream_request_count,
    prompt_tokens: snapshot.metrics.llm.prompt_tokens,
    completion_tokens: snapshot.metrics.llm.completion_tokens,
    total_tokens: snapshot.metrics.llm.total_tokens,
    model_distribution: snapshot.metrics.llm.model_distribution,
    provider_distribution: snapshot.metrics.llm.provider_distribution,
  };
}

function evidenceMetrics(expected) {
  return {
    core: {
      request_count: expected.core.request_count,
      success_rate: expected.core.success_rate,
      stream_request_count: expected.core.stream_request_count,
      p95_latency_ms: expected.core.p95_latency_ms,
      p95_ttft_ms: expected.core.p95_ttft_ms,
    },
    llm: {
      prompt_tokens: expected.llm.prompt_tokens,
      completion_tokens: expected.llm.completion_tokens,
      total_tokens: expected.llm.total_tokens,
      tokens_per_request: expected.llm.tokens_per_request,
      tokens_per_second: expected.llm.tokens_per_second,
      model_distribution: expected.llm.model_distribution,
      provider_distribution: expected.llm.provider_distribution,
    },
  };
}

function responseSample(responses) {
  return responses.slice(0, 4).map((item) => ({
    requestId: item.requestId,
    status: item.status,
    providerNames: item.trace.filter((row) => row.status === 'success').map((row) => row.providerName),
  }));
}

function tableExists(table) {
  return db.scalar(`select to_regclass('public.${table}');`) === table;
}

function redactEvidence(value) {
  return JSON.parse(JSON.stringify(value, (key, item) => (isSecretField(key, item) ? '[redacted]' : item)));
}

function isSecretField(key, item) {
  return typeof item === 'string' && (key === 'value' || isJwt(item) || secretValues().includes(item));
}

function isJwt(value) {
  return /^eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(value);
}

function redactText(value) {
  return secretValues().reduce((text, secret) => text.replaceAll(secret, '[redacted]'), String(value));
}

function secretValues() {
  if (!ctx?.realSecrets) {
    return [];
  }
  return [ctx.realSecrets.provider1Key, ctx.realSecrets.provider2Key, ...ctx.realSecrets.provider1Keys, ...ctx.realSecrets.provider2Keys].filter(Boolean);
}

function writeResults() {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main().catch(() => {
  process.exitCode = 1;
});
