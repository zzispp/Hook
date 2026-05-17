import { encryptProviderKey } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

import { fixtureIds, groupCode } from './provider_cooldown_constants.mjs';

export function seedProviderCooldownFixtures(ctx, db, upstream) {
  seedMenuSections(db);
  seedUser(db);
  seedWallet(db);
  seedBillingGroup(db);
  seedGlobalModel(db, ctx);
  seedProviders(db);
  seedEndpoints(db, ctx);
  seedProviderKeys(db, ctx);
  seedProviderModels(db, upstream);
  seedGroupBindings(db);
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

function seedUser(db) {
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids,
   allowed_provider_ids, created_at, updated_at, last_login_at, auth_source,
   email_verified, rate_limit_rpm, quota_mode)
values
  (${q(fixtureIds.user)}, 'provider_cooldown_user', ${q(passwordHash())}, 'provider-cooldown@example.com', 'user',
   true, false, '[]', '[]', now(), now(), null, 'local', true, 0, 'wallet')
on conflict (id) do update set
  is_active = true,
  is_deleted = false,
  allowed_model_ids = '[]',
  allowed_provider_ids = '[]',
  rate_limit_rpm = 0,
  quota_mode = 'wallet',
  updated_at = now();`);
}

function seedWallet(db) {
  db.exec(`
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
   total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
  (${q(fixtureIds.wallet)}, ${q(fixtureIds.user)}, 0, 10, 'USD', 'active', 'finite', 0, 0, 0, 10, now(), now())
on conflict (user_id) do update set
  recharge_balance = 0,
  gift_balance = 10,
  currency = 'USD',
  status = 'active',
  limit_mode = 'finite',
  updated_at = now();`);
}

function seedBillingGroup(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(fixtureIds.group)}, ${q(groupCode)}, 'Provider Cooldown Real', null, 1, true, false, 0, now(), now())
on conflict (code) do update set
  name = excluded.name,
  is_active = true,
  billing_multiplier = 1,
  updated_at = now();`);
}

function seedGlobalModel(db, ctx) {
  const pricing = JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] });
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing,
   supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(fixtureIds.model)}, ${q(ctx.cooldown.model)}, ${q(ctx.cooldown.model)}, 0.00010000,
   ${q(pricing)}, '["chat"]', null, true, 0, now(), now())
on conflict (id) do update set
  name = excluded.name,
  display_name = excluded.display_name,
  default_price_per_request = excluded.default_price_per_request,
  default_tiered_pricing = excluded.default_tiered_pricing,
  supported_capabilities = excluded.supported_capabilities,
  is_active = true,
  usage_count = 0,
  updated_at = now();`);
}

function seedProviders(db) {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds,
   priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(fixtureIds.providerMsutools)}, 'Cooldown Real Msutools', 'openai', 0, 90, 90, 0, false, true, true, now(), now()),
  (${q(fixtureIds.providerEkan8)}, 'Cooldown Real Ekan8', 'gemini', 0, 90, 90, 10, true, true, true, now(), now())
on conflict (id) do update set
  name = excluded.name,
  provider_type = excluded.provider_type,
  max_retries = excluded.max_retries,
  priority = excluded.priority,
  enable_format_conversion = excluded.enable_format_conversion,
  is_active = true,
  updated_at = now();`);
}

function seedEndpoints(db, ctx) {
  db.exec(`
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active,
   format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(fixtureIds.endpointMsutools)}, ${q(fixtureIds.providerMsutools)}, 'openai_chat', ${q(ctx.upstreams.msutoolsBaseUrl)},
   '/v1/provider-cooldown-real-not-found', 0, true, null, null, null, now(), now()),
  (${q(fixtureIds.endpointEkan8)}, ${q(fixtureIds.providerEkan8)}, 'gemini_chat', ${q(ctx.upstreams.ekan8BaseUrl)},
   null, 0, true, null, null, null, now(), now())
on conflict (id) do update set
  base_url = excluded.base_url,
  custom_path = excluded.custom_path,
  max_retries = excluded.max_retries,
  is_active = true,
  updated_at = now();`);
}

function seedProviderKeys(db, ctx) {
  const msutoolsKey = encryptProviderKey(ctx.providerSecret, ctx.cooldownSecrets.msutoolsKey);
  const ekan8Key = encryptProviderKey(ctx.providerSecret, ctx.cooldownSecrets.ekan8Key);
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start,
   time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(fixtureIds.keyMsutools)}, ${q(fixtureIds.providerMsutools)}, 'Cooldown Real Msutools key', ${q(msutoolsKey)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(fixtureIds.keyEkan8)}, ${q(fixtureIds.providerEkan8)}, 'Cooldown Real Ekan8 key', ${q(ekan8Key)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set
  encrypted_api_key = excluded.encrypted_api_key,
  internal_priority = excluded.internal_priority,
  is_active = true,
  updated_at = now();`);
}

function seedProviderModels(db, upstream) {
  db.exec(`
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings,
   is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  (${q(fixtureIds.modelBindingMsutools)}, ${q(fixtureIds.providerMsutools)}, ${q(fixtureIds.model)}, ${q(upstream.msutoolsModel)}, ${q(JSON.stringify({ name: upstream.msutoolsModel }))}, true, null, null, null, now(), now()),
  (${q(fixtureIds.modelBindingEkan8)}, ${q(fixtureIds.providerEkan8)}, ${q(fixtureIds.model)}, ${q(upstream.ekan8Model)}, ${q(JSON.stringify({ name: upstream.ekan8Model }))}, true, null, null, null, now(), now())
on conflict (id) do update set
  provider_model_name = excluded.provider_model_name,
  provider_model_mappings = excluded.provider_model_mappings,
  is_active = true,
  updated_at = now();`);
}

function seedGroupBindings(db) {
  db.exec(`
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  (${q(fixtureIds.groupBindingMsutools)}, ${q(groupCode)}, ${q(fixtureIds.providerMsutools)}, now(), now()),
  (${q(fixtureIds.groupBindingEkan8)}, ${q(groupCode)}, ${q(fixtureIds.providerEkan8)}, now(), now())
on conflict do nothing;

insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  (${q(fixtureIds.groupModel)}, ${q(groupCode)}, ${q(fixtureIds.model)}, now(), now())
on conflict do nothing;`);
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}
