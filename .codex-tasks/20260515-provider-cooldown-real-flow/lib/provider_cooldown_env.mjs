import { loadContext } from '../../20260512-real-proxy-cache-flow/lib/env.mjs';

export function loadProviderCooldownContext() {
  const base = loadContextWithSyntheticSecrets();
  return Object.freeze({
    ...base,
    upstreams: Object.freeze({
      ...base.upstreams,
      msutoolsBaseUrl: env('HOOK_COOLDOWN_MSUTOOLS_BASE_URL', 'https://www.msutools.cn'),
      ekan8BaseUrl: env('HOOK_COOLDOWN_EKAN8_BASE_URL', 'https://www.ekan8.com'),
    }),
    cooldownSecrets: Object.freeze({
      msutoolsKey: requiredEnv('HOOK_COOLDOWN_MSUTOOLS_KEY'),
      ekan8Key: requiredEnv('HOOK_COOLDOWN_EKAN8_KEY'),
    }),
    cooldown: Object.freeze({
      model: env('HOOK_COOLDOWN_MODEL', 'hook-provider-cooldown-real-chat'),
      statusCode: positiveInt('HOOK_COOLDOWN_STATUS_CODE', 404),
      windowSeconds: positiveInt('HOOK_COOLDOWN_WINDOW_SECONDS', 60),
      thresholdCount: positiveInt('HOOK_COOLDOWN_THRESHOLD_COUNT', 1),
      cooldownSeconds: positiveInt('HOOK_COOLDOWN_SECONDS', 120),
      timeoutMs: positiveInt('HOOK_COOLDOWN_REQUEST_TIMEOUT_MS', 180_000),
    }),
  });
}

function loadContextWithSyntheticSecrets() {
  const names = ['HOOK_SYSTEM_TOKEN', 'HOOK_POOL_KEY', 'EKAN8_KEY', 'CLAUDE_KEY'];
  const snapshot = Object.fromEntries(names.map((name) => [name, process.env[name]]));
  process.env.HOOK_SYSTEM_TOKEN ||= 'sk-provider-cooldown-system-placeholder';
  process.env.HOOK_POOL_KEY ||= 'sk-provider-cooldown-msutools-placeholder';
  process.env.EKAN8_KEY ||= 'sk-provider-cooldown-ekan8-placeholder';
  process.env.CLAUDE_KEY ||= 'sk-provider-cooldown-ekan8-placeholder';
  try {
    return loadContext();
  } finally {
    for (const [name, value] of Object.entries(snapshot)) {
      if (value === undefined) {
        delete process.env[name];
      } else {
        process.env[name] = value;
      }
    }
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

function positiveInt(name, fallback) {
  const value = Number(env(name, String(fallback)));
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

