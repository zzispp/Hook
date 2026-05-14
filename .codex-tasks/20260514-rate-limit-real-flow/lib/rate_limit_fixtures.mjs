import { randomBytes } from 'node:crypto';

import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { clearAuth, clearScheduling } from '../../20260514-real-request-record-flow/lib/request_record_real_support.mjs';
import { groupCode, ids as routeIds } from '../../20260513-real-route-scheduler-flow/lib/route_fixtures.mjs';

export const rateLimitUserIds = Object.freeze({
  userLimited: '00000000-0000-7000-9300-000000000101',
  tokenLimited: '00000000-0000-7000-9300-000000000102',
  combined: '00000000-0000-7000-9300-000000000103',
  shared: '00000000-0000-7000-9300-000000000104',
});

export const rateLimitTokenIds = Object.freeze({
  userLimited: '00000000-0000-7000-9300-000000000201',
  tokenLimited: '00000000-0000-7000-9300-000000000202',
  combined: '00000000-0000-7000-9300-000000000203',
  sharedA: '00000000-0000-7000-9300-000000000204',
  sharedB: '00000000-0000-7000-9300-000000000205',
});

export function makeTokenValues() {
  return Object.fromEntries(Object.keys(rateLimitTokenIds).map((key) => [key, randomToken(`sk-rate-${key}`)]));
}

export function systemSettingsSnapshot(db) {
  const [row] = db.rows(`
select default_rate_limit_rpm::text, scheduling_mode
from system_settings
where id = 'global';`);
  return {
    default_rate_limit_rpm: Number(row[0]),
    scheduling_mode: row[1],
  };
}

export function seedRateLimitFixtures(db, tokenValues) {
  seedUsers(db);
  seedTokens(db, tokenValues);
}

export function resetRateLimitFixtures(db, tokenValues) {
  seedRateLimitFixtures(db, tokenValues);
  db.exec(`
update users
set rate_limit_rpm = 0, allowed_model_ids = '[]', allowed_provider_ids = '[]', is_active = true,
    is_deleted = false, quota_mode = 'unlimited', updated_at = now()
where id in (${Object.values(rateLimitUserIds).map(q).join(',')});
update api_tokens
set rate_limit_rpm = 0, request_count = 0, used_quota = 0, last_used_at = null, is_active = true, updated_at = now()
where id in (${Object.values(rateLimitTokenIds).map(q).join(',')});
update provider_api_keys
set rpm_limit = 0, updated_at = now()
where id in (${q(routeIds.keyOpenAIPrimary)}, ${q(routeIds.keyOpenAISecondary)});
update system_settings
set default_rate_limit_rpm = 0, scheduling_mode = 'fixed_order', updated_at = now()
where id = 'global';`);
}

export function deactivateRateLimitFixtures(db) {
  db.exec(`
update api_tokens set is_active = false, updated_at = now()
where id in (${Object.values(rateLimitTokenIds).map(q).join(',')});
update users set is_active = false, is_deleted = true, updated_at = now()
where id in (${Object.values(rateLimitUserIds).map(q).join(',')});
update provider_api_keys set rpm_limit = 0, updated_at = now()
where id in (${q(routeIds.keyOpenAIPrimary)}, ${q(routeIds.keyOpenAISecondary)});`);
}

export function setUserRateLimit(db, userId, limit) {
  db.exec(`
update users
set rate_limit_rpm = ${Number(limit)},
    updated_at = now()
where id = ${q(userId)};`);
}

export function setTokenRateLimit(db, tokenId, limit) {
  db.exec(`
update api_tokens
set rate_limit_rpm = ${Number(limit)},
    updated_at = now()
where id = ${q(tokenId)};`);
}

export function setProviderKeyRateLimit(db, keyId, limit) {
  db.exec(`
update provider_api_keys
set rpm_limit = ${Number(limit)},
    updated_at = now()
where id = ${q(keyId)};`);
}

export function restoreSystemSettings(db, snapshot) {
  db.exec(`
update system_settings
set default_rate_limit_rpm = ${snapshot.default_rate_limit_rpm},
    scheduling_mode = ${q(snapshot.scheduling_mode)},
    updated_at = now()
where id = 'global';`);
}

export async function clearRateLimitCounters(redis, prefix) {
  const keys = await redis.keys(`${prefix}:llm_proxy:rate_limit:*`);
  await redis.del(...keys);
}

export async function clearProxyState(redis, prefix) {
  await clearScheduling(redis, prefix);
  await clearAuth(redis, prefix);
  await clearRateLimitCounters(redis, prefix);
}

export function openAiKeyIds() {
  return {
    primary: routeIds.keyOpenAIPrimary,
    secondary: routeIds.keyOpenAISecondary,
  };
}

function seedUsers(db) {
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids,
   created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
  ${userRow('userLimited')},
  ${userRow('tokenLimited')},
  ${userRow('combined')},
  ${userRow('shared')}
on conflict (id) do update set
  username = excluded.username,
  email = excluded.email,
  role = excluded.role,
  is_active = true,
  is_deleted = false,
  allowed_model_ids = '[]',
  allowed_provider_ids = '[]',
  rate_limit_rpm = 0,
  quota_mode = 'unlimited',
  updated_at = now();`);
}

function seedTokens(db, tokenValues) {
  db.exec(`
delete from api_tokens where id in (${Object.values(rateLimitTokenIds).map(q).join(',')});
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
  ${tokenRow('userLimited', rateLimitTokenIds.userLimited, rateLimitUserIds.userLimited, tokenValues.userLimited)},
  ${tokenRow('tokenLimited', rateLimitTokenIds.tokenLimited, rateLimitUserIds.tokenLimited, tokenValues.tokenLimited)},
  ${tokenRow('combined', rateLimitTokenIds.combined, rateLimitUserIds.combined, tokenValues.combined)},
  ${tokenRow('sharedA', rateLimitTokenIds.sharedA, rateLimitUserIds.shared, tokenValues.sharedA)},
  ${tokenRow('sharedB', rateLimitTokenIds.sharedB, rateLimitUserIds.shared, tokenValues.sharedB)};`);
}

function userRow(key) {
  const username = `rate_limit_${key}`;
  return `(
    ${q(rateLimitUserIds[key])},
    ${q(username)},
    ${q(passwordHash())},
    ${q(`${username}@example.com`)},
    'admin',
    true,
    false,
    '[]',
    '[]',
    now(),
    now(),
    null,
    'local',
    true,
    0,
    'unlimited'
  )`;
}

function tokenRow(key, tokenId, userId, value) {
  return `(
    ${q(tokenId)},
    ${q(userId)},
    'user',
    ${q(`Rate limit ${key}`)},
    ${q(value)},
    ${q(sha256(value))},
    ${q(value.slice(0, 10))},
    ${q(groupCode)},
    null,
    'all',
    '[]',
    0,
    null,
    0,
    0,
    true,
    null,
    now(),
    now()
  )`;
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}

function randomToken(prefix) {
  return `${prefix}-${randomBytes(18).toString('hex')}`;
}
