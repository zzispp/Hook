import { sha256 } from './crypto.mjs';
import { ids } from './fixtures.mjs';

export function schedulingSnapshotKey(ctx) {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:snapshot:v1`;
}

export function schedulingLockKey(ctx) {
  return `${ctx.redis.prefix}:llm_proxy:scheduling:rebuild_lock`;
}

export function authVersionKey(ctx) {
  return `${ctx.redis.prefix}:llm_proxy:auth:version`;
}

export function authTokenKey(ctx, version) {
  return `${ctx.redis.prefix}:llm_proxy:auth:v${version}:${sha256(ctx.secrets.systemToken)}`;
}

export function authTokenPattern(ctx) {
  return `${ctx.redis.prefix}:llm_proxy:auth:v*`;
}

export function affinityKey(ctx, modelId, format) {
  return `${ctx.redis.prefix}:llm_proxy:affinity:${ids.token}:${modelId}:${format}`;
}

export async function clearSchedulingSnapshot(ctx, redis) {
  await redis.del(schedulingSnapshotKey(ctx), schedulingLockKey(ctx));
}

export async function clearAuthCache(ctx, redis) {
  const keys = await redis.keys(authTokenPattern(ctx));
  await redis.del(...keys, authVersionKey(ctx));
}

export async function clearAffinity(ctx, redis, modelIds) {
  await redis.del(
    affinityKey(ctx, modelIds.gpt, 'openai_chat'),
    affinityKey(ctx, modelIds.gpt, 'claude_chat'),
    affinityKey(ctx, modelIds.claude, 'openai_chat'),
    affinityKey(ctx, modelIds.gemini, 'openai_chat'),
    affinityKey(ctx, modelIds.gemini, 'gemini_chat'),
  );
}

export async function schedulingSnapshot(ctx, redis) {
  const value = await redis.get(schedulingSnapshotKey(ctx));
  return value ? JSON.parse(value) : null;
}

export async function authVersion(ctx, redis) {
  return Number((await redis.get(authVersionKey(ctx))) ?? 0);
}

