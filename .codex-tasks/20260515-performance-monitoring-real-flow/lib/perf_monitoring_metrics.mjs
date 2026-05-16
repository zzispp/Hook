import { assert, assertEqual } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

const BUCKET_SECONDS = 60;
const POLL_MS = 1000;

export function requestBucket(db, requestIds) {
  const rows = db.rows(`
select date_trunc('minute', created_at)::text, count(*)::text
from request_records
where request_id in (${requestIds.map(q).join(',')})
group by 1
order by 1;`);
  assertEqual(rows.length, 1, `test requests must land in one minute bucket: ${JSON.stringify(rows)}`);
  return { startedAt: rows[0][0], requestCount: Number(rows[0][1]) };
}

export async function waitForSnapshot(db, bucketStartedAt, timeoutMs) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const snapshot = snapshotForBucket(db, bucketStartedAt);
    if (snapshot) {
      return snapshot;
    }
    await sleep(POLL_MS);
  }
  throw new Error(`performance monitoring minute snapshot not found for ${bucketStartedAt}`);
}

export function expectedMetrics(db, bucketStartedAt) {
  const summary = summaryRow(db, bucketStartedAt);
  return {
    core: coreMetrics(summary),
    llm: llmMetrics(summary, dimensionRows(db, bucketStartedAt)),
  };
}

export function assertSnapshotMetrics(snapshot, expected) {
  const metrics = snapshot.metrics;
  assertCore(metrics.core, expected.core);
  assertLlm(metrics.llm, expected.llm);
}

export function assertRealtimeSnapshot(realtime, snapshot) {
  assert(realtime.snapshot, 'realtime API should return latest minute snapshot');
  assertEqual(Date.parse(realtime.snapshot.bucket_started_at), Date.parse(snapshot.bucket_started_at), 'realtime bucket_started_at');
  assertEqual(realtime.snapshot.metrics.core.request_count, snapshot.metrics.core.request_count, 'realtime request_count');
  assertEqual(realtime.snapshot.metrics.llm.total_tokens, snapshot.metrics.llm.total_tokens, 'realtime total_tokens');
}

function snapshotForBucket(db, bucketStartedAt) {
  const rows = db.rows(`
select bucket_started_at::text, bucket_ended_at::text, metrics
from performance_monitoring_snapshots
where bucket_granularity = 'minute'
  and bucket_started_at = ${q(bucketStartedAt)}::timestamptz
limit 1;`);
  if (!rows.length) {
    return null;
  }
  return {
    bucket_started_at: new Date(rows[0][0]).toISOString(),
    bucket_ended_at: new Date(rows[0][1]).toISOString(),
    metrics: JSON.parse(rows[0][2]),
  };
}

function summaryRow(db, bucketStartedAt) {
  const [row] = db.rows(`
select count(*)::text,
  count(*) filter (where status = 'success')::text,
  count(*) filter (where status in ('failed', 'cancelled'))::text,
  count(*) filter (where status in ('pending', 'streaming'))::text,
  count(*) filter (where termination_reason like '%timeout%' or client_error_type like '%timeout%')::text,
  count(*) filter (where client_status_code = 429 or client_error_type = 'rate_limit_error')::text,
  count(*) filter (where client_status_code >= 500)::text,
  percentile_disc(0.50) within group (order by total_latency_ms)::text,
  percentile_disc(0.95) within group (order by total_latency_ms)::text,
  percentile_disc(0.99) within group (order by total_latency_ms)::text,
  percentile_disc(0.50) within group (order by first_byte_time_ms)::text,
  percentile_disc(0.95) within group (order by first_byte_time_ms)::text,
  percentile_disc(0.99) within group (order by first_byte_time_ms)::text,
  count(*) filter (where has_retry)::text,
  count(*) filter (where is_stream)::text,
  coalesce(sum(prompt_tokens), 0)::text,
  coalesce(sum(completion_tokens), 0)::text,
  coalesce(sum(total_tokens), 0)::text,
  count(*) filter (where has_failover)::text,
  count(*) filter (where coalesce(cache_read_input_tokens, 0) > 0)::text,
  coalesce(sum(total_cost), 0)::text,
  count(*) filter (where client_error_type = 'new_api_error' or client_error_message like '%quota%')::text
from request_records
where created_at >= ${q(bucketStartedAt)}::timestamptz
  and created_at < ${q(bucketStartedAt)}::timestamptz + interval '1 minute';`);
  assert(row, 'summary row should exist');
  return row;
}

function coreMetrics(row) {
  const requestCount = num(row[0]);
  return {
    request_count: requestCount,
    qps: requestCount / BUCKET_SECONDS,
    concurrent_requests: num(row[3]),
    success_rate: ratio(num(row[1]), requestCount),
    error_rate: ratio(num(row[2]), requestCount),
    timeout_rate: ratio(num(row[4]), requestCount),
    rate_limited_count: num(row[5]),
    server_error_count: num(row[6]),
    p50_latency_ms: nullableNum(row[7]),
    p95_latency_ms: nullableNum(row[8]),
    p99_latency_ms: nullableNum(row[9]),
    p50_ttft_ms: nullableNum(row[10]),
    p95_ttft_ms: nullableNum(row[11]),
    p99_ttft_ms: nullableNum(row[12]),
    retry_count: num(row[13]),
    circuit_breaker_count: 0,
    stream_request_count: num(row[14]),
  };
}

function llmMetrics(row, dimensions) {
  const requestCount = num(row[0]);
  const totalTokens = num(row[17]);
  return {
    prompt_tokens: num(row[15]),
    completion_tokens: num(row[16]),
    total_tokens: totalTokens,
    tokens_per_request: ratio(totalTokens, requestCount),
    tokens_per_second: totalTokens / BUCKET_SECONDS,
    model_distribution: dimensions.models,
    provider_distribution: dimensions.providers,
    failover_count: num(row[18]),
    cache_hit_rate: ratio(num(row[19]), requestCount),
    cost: Number(row[20]),
    quota_limited_count: num(row[21]),
  };
}

function dimensionRows(db, bucketStartedAt) {
  return {
    models: dimension(db, bucketStartedAt, "coalesce(model_name_snapshot, global_model_id, 'unknown')"),
    providers: dimension(db, bucketStartedAt, "coalesce(provider_name_snapshot, provider_id, 'unknown')"),
  };
}

function dimension(db, bucketStartedAt, expression) {
  return db
    .rows(`
select ${expression} as name, count(*)::text
from request_records
where created_at >= ${q(bucketStartedAt)}::timestamptz
  and created_at < ${q(bucketStartedAt)}::timestamptz + interval '1 minute'
group by name
order by count(*) desc, name asc
limit 12;`)
    .map(([name, count]) => ({ name, count: Number(count) }));
}

function assertCore(actual, expected) {
  for (const key of Object.keys(expected)) {
    assertMetric(actual[key], expected[key], `core.${key}`);
  }
}

function assertLlm(actual, expected) {
  for (const key of Object.keys(expected)) {
    if (Array.isArray(expected[key])) {
      assertEqual(JSON.stringify(actual[key]), JSON.stringify(expected[key]), `llm.${key}`);
    } else {
      assertMetric(actual[key], expected[key], `llm.${key}`);
    }
  }
}

function assertMetric(actual, expected, label) {
  if (expected === null) {
    assert(actual === null || actual === undefined, `${label}: expected null, got ${actual}`);
    return;
  }
  assertApprox(Number(actual), Number(expected), label);
}

function assertApprox(actual, expected, label) {
  const tolerance = Math.max(0.000001, Math.abs(expected) * 0.000001);
  assert(Math.abs(actual - expected) <= tolerance, `${label}: expected ${expected}, got ${actual}`);
}

function nullableNum(value) {
  return value === '' ? null : Number(value);
}

function num(value) {
  return Number(value || 0);
}

function ratio(numerator, denominator) {
  return denominator <= 0 ? 0 : numerator / denominator;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
