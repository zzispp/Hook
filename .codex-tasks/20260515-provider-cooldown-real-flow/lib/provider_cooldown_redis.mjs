export async function clearProviderCooldownRedis(redis, prefix, providerIds = []) {
  const keys = [
    `${prefix}:llm_proxy:scheduling:snapshot:v2`,
    `${prefix}:llm_proxy:scheduling:rebuild_lock`,
    `${prefix}:llm_proxy:auth:version`,
  ];
  const authKeys = await redis.keys(`${prefix}:llm_proxy:auth:v*`);
  const cooldownKeys = await redis.keys(`${prefix}:llm_proxy:provider_cooldown*`);
  const authUsageKeys = providerIds.map((id) => `${prefix}:llm_proxy:auth:usage:${id}`);
  await redis.del(...keys, ...authKeys, ...cooldownKeys, ...authUsageKeys);
}

export function providerCooldownKey(prefix, providerId) {
  return `${prefix}:llm_proxy:provider_cooldown:${providerId}`;
}

export function providerFailureKey(prefix, providerId, statusCode) {
  return `${prefix}:llm_proxy:provider_cooldown_failures:${providerId}:${statusCode}`;
}

