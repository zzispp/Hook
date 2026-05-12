export function loadContext() {
  return Object.freeze({
    serverBaseUrl: env('HOOK_BACKEND_URL', 'http://127.0.0.1:3000'),
    providerSecret: env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy'),
    adminUserId: env('HOOK_ADMIN_USER_ID', '00000000-0000-7000-8000-000000000000'),
    adminIdentifier: env('HOOK_ADMIN_IDENTIFIER', 'admin'),
    adminPassword: env('HOOK_ADMIN_PASSWORD', '12345678'),
    db: Object.freeze({
      host: env('HOOK_DB_HOST', 'localhost'),
      port: env('HOOK_DB_PORT', '5433'),
      user: env('HOOK_DB_USER', 'postgres'),
      password: env('HOOK_DB_PASSWORD', '123456'),
      name: env('HOOK_DB_NAME', 'postgres'),
      psqlBin: env('PSQL_BIN', 'psql'),
    }),
    redis: Object.freeze({
      prefix: env('HOOK_REDIS_PREFIX', 'hook'),
      host: env('HOOK_REDIS_HOST', '127.0.0.1'),
      port: Number(env('HOOK_REDIS_PORT', '6380')),
    }),
    upstreams: Object.freeze({
      openaiBaseUrl: env('HOOK_POOL_BASE_URL', 'https://pool.hook.rs'),
      claudeBaseUrl: env('CLAUDE_BASE_URL', env('HOOK_AIPAI_BASE_URL', 'https://api.aipaibox.com')),
      geminiBaseUrl: env('EKAN8_BASE_URL', 'https://www.ekan8.com'),
    }),
    models: Object.freeze({
      openai: env('HOOK_OPENAI_MODEL', 'hook-real-openai'),
      openaiProvider: env('HOOK_OPENAI_PROVIDER_MODEL', 'gpt-5.4-mini'),
      claude: env('HOOK_CLAUDE_MODEL', 'hook-real-claude'),
      claudeProvider: env('HOOK_CLAUDE_PROVIDER_MODEL', 'claude-haiku-4-5-20251001'),
      gemini: env('HOOK_GEMINI_MODEL', 'hook-real-gemini'),
      geminiProvider: env('HOOK_GEMINI_PROVIDER_MODEL', '[满血]gemini-3.1-pro-preview'),
    }),
    secrets: Object.freeze({
      systemToken: requiredEnv('HOOK_SYSTEM_TOKEN'),
      hookPoolKey: requiredEnv('HOOK_POOL_KEY'),
      claudeKey: firstEnv(['CLAUDE_KEY', 'HOOK_AIPAI_KEY']),
      geminiKey: requiredEnv('EKAN8_KEY'),
    }),
    stress: Object.freeze({
      nonStream: Number(env('HOOK_STRESS_NON_STREAM', '24')),
      stream: Number(env('HOOK_STRESS_STREAM', '8')),
      rebuildPatches: Number(env('HOOK_STRESS_REBUILD_PATCHES', '14')),
    }),
  });
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

function firstEnv(names) {
  for (const name of names) {
    const value = process.env[name];
    if (value && value.trim()) {
      return value;
    }
  }
  throw new Error(`missing required env: ${names.join(' or ')}`);
}
