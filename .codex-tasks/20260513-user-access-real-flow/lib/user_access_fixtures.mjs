import { randomBytes } from 'node:crypto';

import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import {
  ids as routeIds,
  groupCode,
  providerNames,
  seedRouteDatabase,
  setClaudePrimaryKey,
  setOpenAIChatBaseUrl,
  setBrokenProviderActive,
  setOpenAIKeyPriorities,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
} from '../../20260513-real-route-scheduler-flow/lib/route_fixtures.mjs';

export {
  groupCode,
  routeIds,
  providerNames,
  seedRouteDatabase,
  setClaudePrimaryKey,
  setOpenAIChatBaseUrl,
  setBrokenProviderActive,
  setOpenAIKeyPriorities,
  setSchedulingModeDb,
  restoreRouteFixtures,
  deactivateRouteFixtures,
};

export const userIds = Object.freeze({
  unrestricted: '00000000-0000-7000-9200-000000000101',
  openaiOnly: '00000000-0000-7000-9200-000000000102',
  claudeOnly: '00000000-0000-7000-9200-000000000103',
  geminiOnly: '00000000-0000-7000-9200-000000000104',
  modelOpenaiOnly: '00000000-0000-7000-9200-000000000105',
  providerMismatch: '00000000-0000-7000-9200-000000000106',
  apiUpdated: '00000000-0000-7000-9200-000000000107',
});

export const tokenIds = Object.freeze({
  unrestricted: '00000000-0000-7000-9200-000000000201',
  openaiOnly: '00000000-0000-7000-9200-000000000202',
  claudeOnly: '00000000-0000-7000-9200-000000000203',
  geminiOnly: '00000000-0000-7000-9200-000000000204',
  modelOpenaiOnly: '00000000-0000-7000-9200-000000000205',
  providerMismatch: '00000000-0000-7000-9200-000000000206',
  apiUpdated: '00000000-0000-7000-9200-000000000207',
});

export function makeTokenValues() {
  return Object.fromEntries(Object.keys(tokenIds).map((key) => [key, randomToken()]));
}

export function ensureUserAccessColumns(db) {
  db.exec(`
alter table users add column if not exists allowed_model_ids text not null default '[]';
alter table users add column if not exists allowed_provider_ids text not null default '[]';`);
}

export function seedUserAccessFixtures(db, tokenValues, modelIds) {
  seedUsers(db, modelIds);
  seedTokens(db, tokenValues);
}

export function resetUserAccessFixtures(db, tokenValues, modelIds) {
  seedUserAccessFixtures(db, tokenValues, modelIds);
  db.exec(`
update api_tokens set request_count = 0, used_quota = 0, last_used_at = null, is_active = true, updated_at = now()
where id in (${Object.values(tokenIds).map(q).join(',')});`);
}

export function deactivateUserAccessFixtures(db) {
  db.exec(`
update api_tokens set is_active = false, updated_at = now() where id in (${Object.values(tokenIds).map(q).join(',')});
update users set is_active = false, is_deleted = true, updated_at = now() where id in (${Object.values(userIds).map(q).join(',')});`);
}

export function userProfile(key) {
  const username = `real_access_${key}`;
  return {
    id: userIds[key],
    username,
    email: `${username}@example.com`,
    password: '12345678',
    role: 'admin',
    is_active: true,
    quota_mode: 'unlimited',
    rate_limit_rpm: 0,
  };
}

function seedUsers(db, modelIds) {
  const rows = [
    userRow('unrestricted', [], []),
    userRow('openaiOnly', [modelIds.openai], [routeIds.providerOpenAI]),
    userRow('claudeOnly', [modelIds.claude], [routeIds.providerClaude]),
    userRow('geminiOnly', [modelIds.gemini], [routeIds.providerGemini]),
    userRow('modelOpenaiOnly', [modelIds.openai], []),
    userRow('providerMismatch', [modelIds.claude], [routeIds.providerOpenAI]),
    userRow('apiUpdated', [], []),
  ];
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids,
   created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
${rows.join(',\n')}
on conflict (id) do update set
  username = excluded.username,
  email = excluded.email,
  role = excluded.role,
  is_active = excluded.is_active,
  is_deleted = false,
  allowed_model_ids = excluded.allowed_model_ids,
  allowed_provider_ids = excluded.allowed_provider_ids,
  rate_limit_rpm = excluded.rate_limit_rpm,
  quota_mode = excluded.quota_mode,
  updated_at = now();`);
}

function seedTokens(db, tokenValues) {
  const rows = Object.entries(tokenIds).map(([key, id]) => tokenRow(key, id, tokenValues[key]));
  db.exec(`
delete from api_tokens where id in (${Object.values(tokenIds).map(q).join(',')});
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
${rows.join(',\n')};`);
}

function userRow(key, allowedModelIds, allowedProviderIds) {
  const user = userProfile(key);
  return `(${q(user.id)}, ${q(user.username)}, ${q(passwordHash())}, ${q(user.email)}, ${q(user.role)},
    true, false, ${jsonText(allowedModelIds)}, ${jsonText(allowedProviderIds)},
    now(), now(), null, 'local', true, 0, 'unlimited')`;
}

function tokenRow(key, id, value) {
  return `(${q(id)}, ${q(userIds[key])}, 'user', ${q(`User access ${key}`)}, ${q(value)}, ${q(sha256(value))},
    ${q(value.slice(0, 10))}, ${q(groupCode)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now())`;
}

function jsonText(value) {
  return q(JSON.stringify(value));
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}

function randomToken() {
  return `sk-user-access-${randomBytes(24).toString('hex')}`;
}
