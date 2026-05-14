import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { encryptProviderKey, sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { groupCodes, ids, providerNames } from './billing_access_ids.mjs';

const PASSWORD_HASH = '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
const TEST_PRICE_PER_REQUEST = '0.01';
const TEST_PRICING = '{"tiers":[{"up_to":null,"input_price_per_1m":100,"output_price_per_1m":200}]}';

export function seedBillingAccessFixtures(ctx, db, tokenValues, runtime) {
  clearFixtureRows(db);
  seedMenuSections(db);
  seedModels(ctx, db);
  seedGroups(db);
  seedProviders(db);
  seedEndpoints(ctx, db, runtime.slowBaseUrl);
  seedKeys(ctx, db);
  seedProviderModels(ctx, db, runtime);
  seedGroupBindings(db);
  seedUsers(db);
  seedWallets(db);
  seedTokens(db, tokenValues);
}

export function deactivateBillingAccessFixtures(db) {
  db.exec(`
update api_tokens set is_active = false, updated_at = now() where id in (${tokenIds().map(q).join(',')});
update providers set is_active = false, updated_at = now() where id in (${providerIds().map(q).join(',')});
update global_models set is_active = false, updated_at = now() where id in (${modelIds().map(q).join(',')});
update billing_groups set is_active = false, updated_at = now() where code in (${Object.values(groupCodes).map(q).join(',')});
update users set is_active = false, is_deleted = true, updated_at = now() where id in (${userIds().map(q).join(',')});`);
}

export function clearFixtureRequestRows(db) {
  db.exec(`
delete from request_records
where request_id in (select distinct request_id from request_candidates where token_id in (${tokenIds().map(q).join(',')}));
delete from request_candidates where token_id in (${tokenIds().map(q).join(',')});`);
}

export function modelNames(ctx) {
  return Object.freeze({
    openai: ctx.localModelName,
    ekan8: `${ctx.localModelName}-ekan8`,
  });
}

function clearFixtureRows(db) {
  clearFixtureRequestRows(db);
  db.exec(`
delete from wallet_transactions where wallet_id in (${walletIds().map(q).join(',')});
delete from api_tokens where id in (${tokenIds().map(q).join(',')});
delete from wallets where id in (${walletIds().map(q).join(',')}) or user_id in (${userIds().map(q).join(',')});
delete from billing_group_providers where group_code in (${Object.values(groupCodes).map(q).join(',')});
delete from billing_group_models where group_code in (${Object.values(groupCodes).map(q).join(',')});
delete from provider_models where id in (${providerModelIds().map(q).join(',')});
delete from provider_api_keys where id in (${keyIds().map(q).join(',')});
delete from provider_endpoints where id in (${endpointIds().map(q).join(',')});
delete from providers where id in (${providerIds().map(q).join(',')});
delete from users where id in (${userIds().map(q).join(',')});
delete from global_models where id in (${modelIds().map(q).join(',')});
delete from billing_groups where code in (${Object.values(groupCodes).map(q).join(',')});`);
}

function seedMenuSections(db) {
  db.exec(`
insert into menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000101', 'overview', '概览', -10, true, now(), now()),
  ('00000000-0000-7000-8000-000000000102', 'operations', '运营管理', -5, true, now(), now()),
  ('00000000-0000-7000-8000-000000000103', 'system_management', '系统管理', 0, true, now(), now())
on conflict (code) do update set subheader = excluded.subheader, sort_order = excluded.sort_order,
  enabled = excluded.enabled, updated_at = now();`);
}

function seedModels(ctx, db) {
  const names = modelNames(ctx);
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(ids.modelOpenai)}, ${q(names.openai)}, ${q(names.openai)}, ${TEST_PRICE_PER_REQUEST}, ${q(TEST_PRICING)}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(ids.modelEkan8)}, ${q(names.ekan8)}, ${q(names.ekan8)}, ${TEST_PRICE_PER_REQUEST}, ${q(TEST_PRICING)}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (id) do update set name = excluded.name, display_name = excluded.display_name,
  default_price_per_request = excluded.default_price_per_request,
  default_tiered_pricing = excluded.default_tiered_pricing, is_active = true, updated_at = now();`);
}

function seedGroups(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(ids.groupHigh)}, ${q(groupCodes.high)}, 'Billing Access Real', null, 2.5, true, false, 0, now(), now()),
  (${q(ids.groupLow)}, ${q(groupCodes.low)}, 'Billing Access Real Low', null, 1, true, false, 1, now(), now());`);
}

function seedProviders(db) {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerPrimaryA)}, ${q(providerNames.primaryA)}, 'openai', 0, 45, 45, 10, false, true, true, now(), now()),
  (${q(ids.providerPrimaryB)}, ${q(providerNames.primaryB)}, 'openai', 0, 45, 45, 10, false, true, true, now(), now()),
  (${q(ids.providerEkan8)}, ${q(providerNames.ekan8)}, 'gemini', 0, 60, 60, 20, false, true, true, now(), now()),
  (${q(ids.providerBroken)}, ${q(providerNames.broken)}, 'openai', 1, 5, 5, 0, false, true, false, now(), now()),
  (${q(ids.providerSlow)}, ${q(providerNames.slow)}, 'openai', 0, 0.2, 0.2, 0, false, true, false, now(), now());`);
}

function seedEndpoints(ctx, db, slowBaseUrl) {
  db.exec(`
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointPrimaryA)}, ${q(ids.providerPrimaryA)}, 'openai_chat', ${q(ctx.upstreams.primaryBaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointPrimaryB)}, ${q(ids.providerPrimaryB)}, 'openai_chat', ${q(ctx.upstreams.primaryBaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointEkan8)}, ${q(ids.providerEkan8)}, 'gemini_chat', ${q(ctx.upstreams.ekan8BaseUrl)}, null, 0, true, null, null, null, now(), now()),
  (${q(ids.endpointBroken)}, ${q(ids.providerBroken)}, 'openai_chat', 'http://127.0.0.1:9', null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointSlow)}, ${q(ids.providerSlow)}, 'openai_chat', ${q(slowBaseUrl)}, null, 0, true, null, null, null, now(), now());`);
}

function seedKeys(ctx, db) {
  const encrypted = encryptedKeys(ctx);
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit, cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyPrimaryA)}, ${q(ids.providerPrimaryA)}, 'Billing Access Hook A key', ${q(encrypted.primary)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyPrimaryB)}, ${q(ids.providerPrimaryB)}, 'Billing Access Hook B key', ${q(encrypted.primary)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyEkan8)}, ${q(ids.providerEkan8)}, 'Billing Access Ekan8 key', ${q(encrypted.ekan8)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyBroken)}, ${q(ids.providerBroken)}, 'Billing Access Broken key', ${q(encrypted.broken)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keySlow)}, ${q(ids.providerSlow)}, 'Billing Access Slow key', ${q(encrypted.primary)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now());`);
}

function seedProviderModels(ctx, db, runtime) {
  const names = modelNames(ctx);
  db.exec(`
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(ids.providerModelPrimaryA)}, ${q(ids.providerPrimaryA)}, ${q(ids.modelOpenai)}, ${q(runtime.primaryProviderModel)}, null, true, null, null, null, now(), now()),
  (${q(ids.providerModelPrimaryB)}, ${q(ids.providerPrimaryB)}, ${q(ids.modelOpenai)}, ${q(runtime.primaryProviderModel)}, null, true, null, null, null, now(), now()),
  (${q(ids.providerModelEkan8)}, ${q(ids.providerEkan8)}, ${q(ids.modelEkan8)}, ${q(names.ekan8)}, ${q(JSON.stringify({ name: runtime.ekan8ProviderModel }))}, true, null, null, null, now(), now()),
  (${q(ids.providerModelBroken)}, ${q(ids.providerBroken)}, ${q(ids.modelOpenai)}, ${q(runtime.primaryProviderModel)}, null, true, null, null, null, now(), now()),
  (${q(ids.providerModelSlow)}, ${q(ids.providerSlow)}, ${q(ids.modelOpenai)}, ${q(runtime.primaryProviderModel)}, null, true, null, null, null, now(), now());`);
}

function seedGroupBindings(db) {
  db.exec(`
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9300-000000000811', ${q(groupCodes.high)}, ${q(ids.providerPrimaryA)}, now(), now()),
  ('00000000-0000-7000-9300-000000000812', ${q(groupCodes.high)}, ${q(ids.providerPrimaryB)}, now(), now()),
  ('00000000-0000-7000-9300-000000000813', ${q(groupCodes.high)}, ${q(ids.providerEkan8)}, now(), now()),
  ('00000000-0000-7000-9300-000000000814', ${q(groupCodes.high)}, ${q(ids.providerBroken)}, now(), now()),
  ('00000000-0000-7000-9300-000000000815', ${q(groupCodes.high)}, ${q(ids.providerSlow)}, now(), now()),
  ('00000000-0000-7000-9300-000000000816', ${q(groupCodes.low)}, ${q(ids.providerPrimaryA)}, now(), now()),
  ('00000000-0000-7000-9300-000000000817', ${q(groupCodes.low)}, ${q(ids.providerPrimaryB)}, now(), now());
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9300-000000000821', ${q(groupCodes.high)}, ${q(ids.modelOpenai)}, now(), now()),
  ('00000000-0000-7000-9300-000000000822', ${q(groupCodes.high)}, ${q(ids.modelEkan8)}, now(), now()),
  ('00000000-0000-7000-9300-000000000823', ${q(groupCodes.low)}, ${q(ids.modelOpenai)}, now(), now());`);
}

function seedUsers(db) {
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids, created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
${userRows().join(',\n')};`);
}

function seedWallets(db) {
  db.exec(`
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode, total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
  (${q(ids.walletActive)}, ${q(ids.userActive)}, 10, 0, 'CNY', 'active', 'finite', 10, 0, 0, 0, now(), now()),
  (${q(ids.walletDisabled)}, ${q(ids.userDisabled)}, 10, 0, 'CNY', 'active', 'finite', 10, 0, 0, 0, now(), now()),
  (${q(ids.walletTokenQuota)}, ${q(ids.userTokenQuota)}, 10, 0, 'CNY', 'active', 'finite', 10, 0, 0, 0, now(), now()),
  (${q(ids.walletQuota)}, ${q(ids.userWalletQuota)}, 0, 0, 'CNY', 'active', 'finite', 0, 0, 0, 0, now(), now()),
  (${q(ids.walletRouting)}, ${q(ids.userRouting)}, 10, 0, 'CNY', 'active', 'finite', 10, 0, 0, 0, now(), now());`);
}

function seedTokens(db, tokenValues) {
  db.exec(`
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at, model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count, is_active, last_used_at, created_at, updated_at)
values
${tokenRows(tokenValues).join(',\n')};`);
}

function userRows() {
  return [
    userRow(ids.userActive, 'active', true, 'wallet'),
    userRow(ids.userDisabled, 'disabled_user', false, 'wallet'),
    userRow(ids.userTokenQuota, 'token_quota', true, 'wallet'),
    userRow(ids.userWalletQuota, 'wallet_quota', true, 'wallet'),
    userRow(ids.userRouting, 'routing', true, 'unlimited'),
  ];
}

function userRow(id, name, active, quotaMode) {
  return `(${q(id)}, ${q(`billing_access_${name}`)}, ${q(PASSWORD_HASH)}, ${q(`billing_access_${name}@example.com`)},
    'admin', ${active ? 'true' : 'false'}, false, '[]', '[]', now(), now(), null, 'local', true, 0, ${q(quotaMode)})`;
}

function tokenRows(tokenValues) {
  return [
    tokenRow(ids.tokenActive, ids.userActive, 'active', tokenValues.active, groupCodes.high, 'null', 'true'),
    tokenRow(ids.tokenDisabled, ids.userActive, 'disabled_token', tokenValues.disabledToken, groupCodes.high, 'null', 'false'),
    tokenRow(ids.tokenDisabledUser, ids.userDisabled, 'disabled_user', tokenValues.disabledUser, groupCodes.high, 'null', 'true'),
    tokenRow(ids.tokenQuota, ids.userTokenQuota, 'token_quota', tokenValues.tokenQuota, groupCodes.high, '0', 'true'),
    tokenRow(ids.tokenWallet, ids.userWalletQuota, 'wallet_quota', tokenValues.walletQuota, groupCodes.high, 'null', 'true'),
    tokenRow(ids.tokenRouting, ids.userRouting, 'routing', tokenValues.routing, groupCodes.high, 'null', 'true'),
  ];
}

function tokenRow(id, userId, name, value, groupCode, quotaLimit, active) {
  return `(${q(id)}, ${q(userId)}, 'user', ${q(`Billing Access ${name}`)}, ${q(value)}, ${q(sha256(value))},
    ${q(value.slice(0, 10))}, ${q(groupCode)}, null, 'all', '[]', 0, ${quotaLimit}, 0, 0, ${active}, null, now(), now())`;
}

function encryptedKeys(ctx) {
  return {
    primary: encryptProviderKey(ctx.providerSecret, ctx.upstreams.primaryKey),
    ekan8: encryptProviderKey(ctx.providerSecret, ctx.upstreams.ekan8Key),
    broken: encryptProviderKey(ctx.providerSecret, 'sk-billing-access-invalid'),
  };
}

function modelIds() {
  return [ids.modelOpenai, ids.modelEkan8];
}

function providerIds() {
  return [ids.providerPrimaryA, ids.providerPrimaryB, ids.providerEkan8, ids.providerBroken, ids.providerSlow];
}

function keyIds() {
  return [ids.keyPrimaryA, ids.keyPrimaryB, ids.keyEkan8, ids.keyBroken, ids.keySlow];
}

function endpointIds() {
  return [ids.endpointPrimaryA, ids.endpointPrimaryB, ids.endpointEkan8, ids.endpointBroken, ids.endpointSlow];
}

function providerModelIds() {
  return [ids.providerModelPrimaryA, ids.providerModelPrimaryB, ids.providerModelEkan8, ids.providerModelBroken, ids.providerModelSlow];
}

function userIds() {
  return [ids.userActive, ids.userDisabled, ids.userTokenQuota, ids.userWalletQuota, ids.userRouting];
}

function tokenIds() {
  return [ids.tokenActive, ids.tokenDisabled, ids.tokenDisabledUser, ids.tokenQuota, ids.tokenWallet, ids.tokenRouting];
}

function walletIds() {
  return [ids.walletActive, ids.walletDisabled, ids.walletTokenQuota, ids.walletQuota, ids.walletRouting];
}
