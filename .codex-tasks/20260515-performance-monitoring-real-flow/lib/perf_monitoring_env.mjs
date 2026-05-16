import { loadRealUsageContext } from '../../20260515-real-usage-concurrency-flow/lib/real_usage_env.mjs';

export function loadPerfMonitoringContext() {
  const base = loadRealUsageContext();
  return Object.freeze({
    ...base,
    perfLoad: Object.freeze({
      nonStreamRequests: positiveInt('HOOK_PERF_MONITORING_REAL_REQUESTS', 8),
      streamRequests: nonNegativeInt('HOOK_PERF_MONITORING_REAL_STREAMS', 0),
      requestTimeoutMs: positiveInt('HOOK_PERF_MONITORING_REQUEST_TIMEOUT_MS', base.realLoad.requestTimeoutMs),
      snapshotTimeoutMs: positiveInt('HOOK_PERF_MONITORING_SNAPSHOT_TIMEOUT_MS', 150_000),
    }),
  });
}

function positiveInt(name, fallback) {
  const value = Number(process.env[name] || fallback);
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

function nonNegativeInt(name, fallback) {
  const value = Number(process.env[name] || fallback);
  if (!Number.isInteger(value) || value < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return value;
}
