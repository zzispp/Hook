import { encryptProviderKey, sha256 } from './crypto.mjs';
import { q } from './db.mjs';

export const IDS = Object.freeze({
  user: '00000000-0000-7000-9500-000000000001',
  wallet: '00000000-0000-7000-9500-000000000002',
  group: '00000000-0000-7000-9500-000000000003',
  providerGame86: '00000000-0000-7000-9500-000000000101',
  providerEkan8: '00000000-0000-7000-9500-000000000102',
  endpointGame86: '00000000-0000-7000-9500-000000000201',
  endpointEkan8: '00000000-0000-7000-9500-000000000202',
  keyGame86: '00000000-0000-7000-9500-000000000301',
  keyEkan8: '00000000-0000-7000-9500-000000000302',
  modelGame86: '00000000-0000-7000-9500-000000000401',
  modelEkan8: '00000000-0000-7000-9500-000000000402',
  bindingGame86: '00000000-0000-7000-9500-000000000501',
  bindingEkan8: '00000000-0000-7000-9500-000000000502',
  ruleGame86: '00000000-0000-7000-9500-000000000601',
  ruleEkan8: '00000000-0000-7000-9500-000000000602',
  token: '00000000-0000-7000-9500-000000000701',
});

export const GROUP_CODE = 'aether_real_20260517';
export const GROUP_MULTIPLIER = '1.25000000';
export const INITIAL_WALLET_BALANCE = '10.00000000';

export function seedFixtures(ctx, db, tokenValue, upstream) {
  seedMenuSections(db);
  seedRole(db);
  seedUserAndWallet(db);
  seedModels(db);
  seedBillingGroup(db);
  seedProviders(ctx, db, upstream);
  seedRules(db);
  seedToken(db, tokenValue);
}

export function cleanupFixtures(db) {
  db.exec(`
update api_tokens set is_active = false, updated_at = now() where id = ${q(IDS.token)};
update providers set is_active = false, updated_at = now() where id in (${q(IDS.providerGame86)}, ${q(IDS.providerEkan8)});
update global_models set is_active = false, updated_at = now() where id in (${q(IDS.modelGame86)}, ${q(IDS.modelEkan8)});
update billing_groups set is_active = false, updated_at = now() where code = ${q(GROUP_CODE)};
update users set is_active = false, is_deleted = true, updated_at = now() where id = ${q(IDS.user)};`);
}

export function clearTestRows(db) {
  db.exec(`
delete from request_records where request_id in (select distinct request_id from request_candidates where token_id = ${q(IDS.token)});
delete from request_candidates where token_id = ${q(IDS.token)};
delete from wallet_transactions where wallet_id = ${q(IDS.wallet)};`);
}

function seedMenuSections(db) {
  db.exec(`
insert into menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
values
  ('00000000-0000-7000-8000-000000000101', 'overview', '概览', -10, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000102', 'operations', '运营管理', -5, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00'),
  ('00000000-0000-7000-8000-000000000103', 'system_management', '系统管理', 0, true, '2026-05-14 07:25:12.573576+00', '2026-05-14 07:25:12.573576+00')
on conflict (id) do update set
  code = excluded.code,
  subheader = excluded.subheader,
  sort_order = excluded.sort_order,
  enabled = excluded.enabled,
  updated_at = excluded.updated_at;`);
}

function seedRole(db) {
  db.exec(`
insert into roles (code, name, description, enabled, system, sort_order, created_at, updated_at)
values ('admin', 'Admin', 'Test admin role', true, true, 0, now(), now())
on conflict (code) do update set enabled = true, updated_at = now();`);
}

function seedUserAndWallet(db) {
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids, allowed_provider_ids,
   created_at, updated_at, last_login_at, auth_source, email_verified, rate_limit_rpm, quota_mode)
values
  (${q(IDS.user)}, 'aether_real_user', ${q(passwordHash())}, 'aether-real@example.com', 'admin',
   true, false, '[]', '[]', now(), now(), null, 'local', true, 0, 'wallet')
on conflict (id) do update set
  username = excluded.username,
  email = excluded.email,
  role = excluded.role,
  is_active = true,
  is_deleted = false,
  quota_mode = 'wallet',
  updated_at = now();
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
   total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
  (${q(IDS.wallet)}, ${q(IDS.user)}, ${INITIAL_WALLET_BALANCE}, 0, 'CNY', 'active', 'finite',
   ${INITIAL_WALLET_BALANCE}, 0, 0, 0, now(), now())
on conflict (user_id) do update set
  recharge_balance = ${INITIAL_WALLET_BALANCE},
  gift_balance = 0,
  currency = 'CNY',
  status = 'active',
  limit_mode = 'finite',
  total_recharged = ${INITIAL_WALLET_BALANCE},
  total_consumed = 0,
  total_refunded = 0,
  total_adjusted = 0,
  updated_at = now();`);
}

function seedModels(db) {
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }));
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(IDS.modelGame86)}, 'aether-real-86gamestore-chat', 'Aether Real 86GameStore Chat', null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  (${q(IDS.modelEkan8)}, 'aether-real-ekan8-mapped-chat', 'Aether Real Ekan8 Mapped Chat', null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (id) do update set
  name = excluded.name,
  display_name = excluded.display_name,
  is_active = true,
  usage_count = 0,
  updated_at = now();`);
}

function seedBillingGroup(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(IDS.group)}, ${q(GROUP_CODE)}, 'Aether Real Billing Test', null, ${GROUP_MULTIPLIER}, true, false, 0, now(), now())
on conflict (code) do update set
  billing_multiplier = ${GROUP_MULTIPLIER},
  is_active = true,
  updated_at = now();
delete from billing_group_models where group_code = ${q(GROUP_CODE)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9500-000000000801', ${q(GROUP_CODE)}, ${q(IDS.modelGame86)}, now(), now()),
  ('00000000-0000-7000-9500-000000000802', ${q(GROUP_CODE)}, ${q(IDS.modelEkan8)}, now(), now());`);
}

function seedProviders(ctx, db, upstream) {
  const game86Key = encryptProviderKey(ctx.providerSecret, ctx.keys.game86);
  const ekan8Key = encryptProviderKey(ctx.providerSecret, ctx.keys.ekan8);
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds,
   priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(IDS.providerGame86)}, 'Aether Real 86GameStore', 'openai', 0, 90, 90, 10, false, true, true, now(), now()),
  (${q(IDS.providerEkan8)}, 'Aether Real Ekan8', 'openai', 0, 90, 90, 20, false, true, true, now(), now())
on conflict (id) do update set
  priority = excluded.priority,
  is_active = true,
  updated_at = now();
delete from provider_endpoints where id in (${q(IDS.endpointGame86)}, ${q(IDS.endpointEkan8)});
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active,
   format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(IDS.endpointGame86)}, ${q(IDS.providerGame86)}, 'openai_chat', ${q(ctx.baseUrls.game86)}, null, 0, true, null, null, null, now(), now()),
  (${q(IDS.endpointEkan8)}, ${q(IDS.providerEkan8)}, 'openai_chat', ${q(ctx.baseUrls.ekan8)}, null, 0, true, null, null, null, now(), now());
delete from provider_api_keys where id in (${q(IDS.keyGame86)}, ${q(IDS.keyEkan8)});
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end,
   health_by_format, circuit_breaker_by_format, is_active, api_formats, allowed_model_ids, created_at, updated_at)
values
  (${q(IDS.keyGame86)}, ${q(IDS.providerGame86)}, 'Aether 86GameStore key', ${q(game86Key)}, null, 0, null, null,
   10, 0, false, null, null, null, null, true, '["openai_chat"]', '[]', now(), now()),
  (${q(IDS.keyEkan8)}, ${q(IDS.providerEkan8)}, 'Aether Ekan8 key', ${q(ekan8Key)}, null, 0, null, null,
   10, 0, false, null, null, null, null, true, '["openai_chat"]', '[]', now(), now());
delete from provider_models where id in (${q(IDS.bindingGame86)}, ${q(IDS.bindingEkan8)});
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active,
   price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(IDS.bindingGame86)}, ${q(IDS.providerGame86)}, ${q(IDS.modelGame86)}, ${q(upstream.game86Model)}, null, true, null, null, null, now(), now()),
  (${q(IDS.bindingEkan8)}, ${q(IDS.providerEkan8)}, ${q(IDS.modelEkan8)}, 'mapped-ekan8-placeholder',
   ${q(JSON.stringify({ name: upstream.ekan8OpenAiModel }))}, true, null, null, null, now(), now());
delete from billing_group_providers where group_code = ${q(GROUP_CODE)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9500-000000000811', ${q(GROUP_CODE)}, ${q(IDS.providerGame86)}, now(), now()),
  ('00000000-0000-7000-9500-000000000812', ${q(GROUP_CODE)}, ${q(IDS.providerEkan8)}, now(), now());`);
}

function seedRules(db) {
  const variables = q(JSON.stringify({ input_price_per_1m: 2, output_price_per_1m: 4, request_fee: 0.0003 }));
  const mappings = q(JSON.stringify({
    input_tokens: { source: 'dimension', key: 'input_tokens', required: true, allow_zero: false },
    output_tokens: { source: 'dimension', key: 'output_tokens', required: true, allow_zero: false },
    input_cost: { source: 'computed', expression: 'input_tokens * input_price_per_1m / 1000000', required: true },
    output_cost: { source: 'computed', expression: 'output_tokens * output_price_per_1m / 1000000', required: true },
    request_cost: { source: 'computed', expression: 'request_fee', required: true },
    cache_creation_cost: { source: 'constant', default: 0 },
    cache_read_cost: { source: 'constant', default: 0 },
  }));
  db.exec(`
delete from billing_rules where id in (${q(IDS.ruleGame86)}, ${q(IDS.ruleEkan8)});
insert into billing_rules
  (id, global_model_id, model_id, name, task_type, expression, variables, dimension_mappings, is_enabled, created_at, updated_at)
values
  (${q(IDS.ruleGame86)}, ${q(IDS.modelGame86)}, null, 'Aether real 86GameStore rule', 'chat',
   'input_cost + output_cost + cache_creation_cost + cache_read_cost + request_cost', ${variables}, ${mappings}, true, now(), now()),
  (${q(IDS.ruleEkan8)}, ${q(IDS.modelEkan8)}, null, 'Aether real Ekan8 rule', 'chat',
   'input_cost + output_cost + cache_creation_cost + cache_read_cost + request_cost', ${variables}, ${mappings}, true, now(), now());`);
}

function seedToken(db, tokenValue) {
  db.exec(`
delete from api_tokens where id = ${q(IDS.token)};
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
  (${q(IDS.token)}, ${q(IDS.user)}, 'user', 'Aether real billing token', ${q(tokenValue)}, ${q(sha256(tokenValue))},
   ${q(tokenValue.slice(0, 10))}, ${q(GROUP_CODE)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());`);
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}
