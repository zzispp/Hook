export function loadContext() {
  return Object.freeze({
    serverBaseUrl: env('HOOK_BACKEND_URL', 'http://127.0.0.1:5555'),
    providerSecret: env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy'),
    baseUrls: Object.freeze({
      game86: env('HOOK_86GAMESTORE_BASE_URL', 'https://api.86gamestore.com'),
      ekan8: env('EKAN8_BASE_URL', 'https://www.ekan8.com'),
    }),
    keys: Object.freeze({
      game86: requiredEnv('HOOK_86GAMESTORE_KEY'),
      ekan8: requiredEnv('EKAN8_KEY'),
    }),
    db: Object.freeze({
      container: env('HOOK_PG_CONTAINER', 'hook-postgres'),
      user: env('HOOK_PG_USER', 'postgres'),
      name: env('HOOK_PG_DB', 'postgres'),
    }),
    redis: Object.freeze({
      host: env('HOOK_REDIS_HOST', '127.0.0.1'),
      port: Number(env('HOOK_REDIS_PORT', '6380')),
      prefix: env('HOOK_REDIS_PREFIX', 'hook'),
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
