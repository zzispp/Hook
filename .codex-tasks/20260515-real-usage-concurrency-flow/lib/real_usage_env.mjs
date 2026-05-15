import { loadContext } from '../../20260512-real-proxy-cache-flow/lib/env.mjs';

export function loadRealUsageContext() {
  const base = loadContextWithSyntheticSecrets();
  const provider1Keys = requiredKeyList('HOOK_REAL_PROVIDER1_KEYS', 'HOOK_REAL_PROVIDER1_KEY');
  const provider2Keys = requiredKeyList('HOOK_REAL_PROVIDER2_KEYS', 'HOOK_REAL_PROVIDER2_KEY');
  return Object.freeze({
    ...base,
    upstreams: Object.freeze({
      ...base.upstreams,
      provider1BaseUrl: env('HOOK_REAL_PROVIDER1_BASE', 'https://www.hook.rs'),
      provider2BaseUrl: env('HOOK_REAL_PROVIDER2_BASE', 'https://www.ekan8.com'),
    }),
    realSecrets: Object.freeze({
      provider1Key: provider1Keys[0],
      provider2Key: provider2Keys[0],
      provider1Keys,
      provider2Keys,
    }),
    realLoad: Object.freeze({
      requestCount: positiveInt('HOOK_REAL_CONCURRENCY_REQUESTS', 36),
      requestTimeoutMs: positiveInt('HOOK_REAL_REQUEST_TIMEOUT_MS', 180_000),
    }),
    realModels: Object.freeze({
      chat: env('HOOK_REAL_CHAT_MODEL', 'hook-real-usage-chat'),
      ekan8: process.env.HOOK_REAL_EKAN8_MODEL || '',
    }),
  });
}

function loadContextWithSyntheticSecrets() {
  const original = snapshotEnv(['HOOK_SYSTEM_TOKEN', 'HOOK_POOL_KEY', 'EKAN8_KEY', 'CLAUDE_KEY']);
  process.env.HOOK_SYSTEM_TOKEN ||= 'sk-real-usage-system-token-placeholder';
  process.env.HOOK_POOL_KEY ||= 'sk-real-usage-provider1-placeholder';
  process.env.EKAN8_KEY ||= 'sk-real-usage-provider2-placeholder';
  process.env.CLAUDE_KEY ||= 'sk-real-usage-provider2-placeholder';
  try {
    return loadContext();
  } finally {
    restoreEnv(original);
  }
}

function env(name, fallback) {
  return process.env[name] || fallback;
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) {
    throw new Error(`missing required env: ${name}`);
  }
  return value;
}

function requiredKeyList(listName, singleName) {
  const raw = process.env[listName] || process.env[singleName];
  if (!raw || !raw.trim()) {
    throw new Error(`missing required env: ${singleName} or ${listName}`);
  }
  const values = raw.split(',').map((item) => item.trim()).filter(Boolean);
  if (values.length === 0) {
    throw new Error(`${singleName} or ${listName} must contain at least one provider key`);
  }
  return Object.freeze(values);
}

function positiveInt(name, fallback) {
  const raw = env(name, String(fallback));
  const value = Number(raw);
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

function snapshotEnv(names) {
  return Object.fromEntries(names.map((name) => [name, process.env[name]]));
}

function restoreEnv(snapshot) {
  for (const [name, value] of Object.entries(snapshot)) {
    if (value === undefined) {
      delete process.env[name];
    } else {
      process.env[name] = value;
    }
  }
}
