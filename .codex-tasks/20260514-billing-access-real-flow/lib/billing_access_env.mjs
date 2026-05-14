export function loadBillingAccessContext() {
  const primaryKey = firstEnv(['HOOK_BILLING_PRIMARY_KEY', 'HOOK_RS_KEY', 'HOOK_POOL_KEY']);
  const ekan8Key = firstEnv(['HOOK_BILLING_EKAN8_KEY', 'EKAN8_KEY']);
  return Object.freeze({
    serverBaseUrl: env('HOOK_BACKEND_URL', 'http://127.0.0.1:5555'),
    providerSecret: env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy'),
    redis: Object.freeze({
      prefix: env('HOOK_REDIS_PREFIX', 'hook'),
      host: env('HOOK_REDIS_HOST', '127.0.0.1'),
      port: Number(env('HOOK_REDIS_PORT', '6380')),
    }),
    db: Object.freeze({
      host: env('HOOK_DB_HOST', 'localhost'),
      port: env('HOOK_DB_PORT', '5433'),
      user: env('HOOK_DB_USER', 'postgres'),
      password: env('HOOK_DB_PASSWORD', '123456'),
      name: env('HOOK_DB_NAME', 'postgres'),
      psqlBin: env('PSQL_BIN', 'psql'),
    }),
    upstreams: Object.freeze({
      primaryBaseUrl: env('HOOK_BILLING_PRIMARY_BASE_URL', 'https://www.hook.rs'),
      primaryModel: env('HOOK_BILLING_PRIMARY_MODEL', ''),
      primaryKey,
      ekan8BaseUrl: env('HOOK_BILLING_EKAN8_BASE_URL', 'https://www.ekan8.com'),
      ekan8Model: env('HOOK_BILLING_EKAN8_MODEL', ''),
      ekan8Key,
    }),
    localModelName: env('HOOK_BILLING_LOCAL_MODEL', 'hook-billing-access-real'),
  });
}

function env(name, fallback) {
  return process.env[name] || fallback;
}

function firstEnv(names) {
  for (const name of names) {
    const value = process.env[name];
    if (value?.trim()) return value;
  }
  throw new Error(`missing required env: ${names.join(' or ')}`);
}
