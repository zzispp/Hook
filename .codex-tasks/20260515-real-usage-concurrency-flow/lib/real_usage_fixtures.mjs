import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { encryptProviderKey } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

export const fixtureIds = Object.freeze({
  providerHook: '00000000-0000-7000-9300-000000000101',
  providerEkan8: '00000000-0000-7000-9300-000000000102',
  keyHookA: '00000000-0000-7000-9300-000000000201',
  keyHookB: '00000000-0000-7000-9300-000000000202',
  keyEkan8A: '00000000-0000-7000-9300-000000000203',
  keyEkan8B: '00000000-0000-7000-9300-000000000204',
  endpointHook: '00000000-0000-7000-9300-000000000301',
  endpointEkan8: '00000000-0000-7000-9300-000000000302',
  group: '00000000-0000-7000-9300-000000000801',
  model: '00000000-0000-7000-9300-000000000701',
});

export const fixtureUserIds = Object.freeze(['00000000-0000-7000-9300-000000000901', '00000000-0000-7000-9300-000000000902', '00000000-0000-7000-9300-000000000903']);

export const fixtureWalletIds = Object.freeze(['00000000-0000-7000-9300-000000000951', '00000000-0000-7000-9300-000000000952', '00000000-0000-7000-9300-000000000953']);

export const groupCode = 'real_usage_concurrency';

export function seedRealUsageFixtures(ctx, db, upstream) {
  ensureSchema(db);
  seedRequestedMenuSections(db);
  seedUsersAndWallets(db);
  seedModel(db, ctx.realModels.chat);
  seedGroup(db);
  seedProviders(db);
  seedEndpoints(db, ctx);
  seedProviderKeys(db, ctx, upstream);
  seedProviderModels(db, ctx.realModels.chat, upstream.hookModel, upstream.ekan8Model);
  seedGroupBindings(db);
  setSchedulingMode(db, 'load_balance');
}

export function setSchedulingMode(db, mode) {
  db.exec(`update system_settings set scheduling_mode = ${q(mode)}, updated_at = now() where id = 'global';`);
}

function ensureSchema(db) {
  const missing = [];
  for (const table of ['usage_flush_batches', 'request_records', 'request_candidates', 'wallets', 'wallet_transactions']) {
    if (!tableExists(db, table)) {
      missing.push(table);
    }
  }
  if (missing.length > 0) {
    throw new Error(`local DB schema is missing tables: ${missing.join(', ')}; run cargo run -p backend -- migration up first`);
  }
}

function seedRequestedMenuSections(db) {
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

function seedUsersAndWallets(db) {
  const users = fixtureUserIds.map((id, index) => {
    const username = `real_usage_user_${index + 1}`;
    return `(${q(id)}, ${q(username)}, ${q(passwordHash())}, ${q(`${username}@example.com`)}, 'user',
      true, false, '[]', '[]', now(), now(), null, 'local', true, 0, 'wallet')`;
  });
  const wallets = fixtureWalletIds.map((id, index) => {
    const userId = fixtureUserIds[index];
    return `(${q(id)}, ${q(userId)}, 0, 25, 'USD', 'active', 'finite', 0, 0, 0, 25, now(), now())`;
  });
  db.exec(`
insert into users
  (id, username, password_hash, email, role, is_active, is_deleted, allowed_model_ids,
   allowed_provider_ids, created_at, updated_at, last_login_at, auth_source,
   email_verified, rate_limit_rpm, quota_mode)
values
${users.join(',\n')}
on conflict (id) do update set
  username = excluded.username,
  email = excluded.email,
  role = excluded.role,
  is_active = true,
  is_deleted = false,
  allowed_model_ids = excluded.allowed_model_ids,
  allowed_provider_ids = excluded.allowed_provider_ids,
  rate_limit_rpm = excluded.rate_limit_rpm,
  quota_mode = excluded.quota_mode,
  updated_at = now();
insert into wallets
  (id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
   total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at)
values
${wallets.join(',\n')}
on conflict (user_id) do update set
  recharge_balance = 0,
  gift_balance = 25,
  currency = 'USD',
  status = 'active',
  limit_mode = 'finite',
  total_recharged = 0,
  total_consumed = 0,
  total_refunded = 0,
  total_adjusted = 25,
  updated_at = now();`);
}

function seedModel(db, modelName) {
  const conflictId = db.scalar(`select id from global_models where name = ${q(modelName)} and id <> ${q(fixtureIds.model)} limit 1;`);
  if (conflictId) {
    throw new Error(`fixture global model name conflicts with existing model id ${conflictId}; set HOOK_REAL_CHAT_MODEL to a test-only name`);
  }
  const pricing = q(
    JSON.stringify({
      tiers: [
        {
          up_to: null,
          input_price_per_1m: 0,
          output_price_per_1m: 0,
        },
      ],
    }),
  );
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing,
   supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  (${q(fixtureIds.model)}, ${q(modelName)}, ${q(modelName)}, 0.00010000, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
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

function seedGroup(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(fixtureIds.group)}, ${q(groupCode)}, 'Real Usage Concurrency', null, 1, true, false, 0, now(), now())
on conflict (code) do update set
  name = excluded.name,
  billing_multiplier = excluded.billing_multiplier,
  is_active = true,
  updated_at = now();`);
}

function seedProviders(db) {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds,
   priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(fixtureIds.providerHook)}, 'Real Usage OpenAI Compatible', 'openai', 1, 90, 90, 10, false, true, true, now(), now()),
  (${q(fixtureIds.providerEkan8)}, 'Real Usage Ekan8 Gemini', 'gemini', 1, 90, 90, 10, true, true, true, now(), now())
on conflict (id) do update set
  name = excluded.name,
  provider_type = excluded.provider_type,
  max_retries = excluded.max_retries,
  request_timeout_seconds = excluded.request_timeout_seconds,
  stream_first_byte_timeout_seconds = excluded.stream_first_byte_timeout_seconds,
  priority = excluded.priority,
  keep_priority_on_conversion = excluded.keep_priority_on_conversion,
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
  (${q(fixtureIds.endpointHook)}, ${q(fixtureIds.providerHook)}, 'openai_chat', ${q(ctx.upstreams.provider1BaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(fixtureIds.endpointEkan8)}, ${q(fixtureIds.providerEkan8)}, 'gemini_chat', ${q(ctx.upstreams.provider2BaseUrl)}, null, 1, true, null, null, null, now(), now())
on conflict (id) do update set
  base_url = excluded.base_url,
  max_retries = excluded.max_retries,
  is_active = true,
  updated_at = now();`);
}

function seedProviderKeys(db, ctx, upstream) {
  const hookA = encryptProviderKey(ctx.providerSecret, ctx.realSecrets.provider1Keys[0]);
  const hookB = encryptProviderKey(ctx.providerSecret, ctx.realSecrets.provider1Keys[1] || ctx.realSecrets.provider1Keys[0]);
  const ekan8A = encryptProviderKey(ctx.providerSecret, ctx.realSecrets.provider2Keys[0]);
  const ekan8B = encryptProviderKey(ctx.providerSecret, ctx.realSecrets.provider2Keys[1] || ctx.realSecrets.provider2Keys[0]);
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start,
   time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(fixtureIds.keyHookA)}, ${q(fixtureIds.providerHook)}, 'Real Usage OpenAI key A', ${q(hookA)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(fixtureIds.keyHookB)}, ${q(fixtureIds.providerHook)}, 'Real Usage OpenAI key B', ${q(hookB)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(fixtureIds.keyEkan8A)}, ${q(fixtureIds.providerEkan8)}, 'Real Usage Ekan8 key A', ${q(ekan8A)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(fixtureIds.keyEkan8B)}, ${q(fixtureIds.providerEkan8)}, 'Real Usage Ekan8 key B', ${q(ekan8B)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set
  encrypted_api_key = excluded.encrypted_api_key,
  internal_priority = excluded.internal_priority,
  is_active = true,
  updated_at = now();`);
}

function seedProviderModels(db, modelName, hookModel, ekan8Model) {
  const hookMapping = q(JSON.stringify({ name: hookModel }));
  const mapping = q(JSON.stringify({ name: ekan8Model }));
  db.exec(`
delete from provider_models where id in ('00000000-0000-7000-9300-000000000401', '00000000-0000-7000-9300-000000000402');
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings,
   is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  ('00000000-0000-7000-9300-000000000401', ${q(fixtureIds.providerHook)}, ${q(fixtureIds.model)}, ${q(modelName)}, ${hookMapping}, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9300-000000000402', ${q(fixtureIds.providerEkan8)}, ${q(fixtureIds.model)}, ${q(modelName)}, ${mapping}, true, null, null, null, now(), now());`);
}

function seedGroupBindings(db) {
  db.exec(`
delete from billing_group_providers where group_code = ${q(groupCode)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9300-000000000601', ${q(groupCode)}, ${q(fixtureIds.providerHook)}, now(), now()),
  ('00000000-0000-7000-9300-000000000602', ${q(groupCode)}, ${q(fixtureIds.providerEkan8)}, now(), now());
delete from billing_group_models where group_code = ${q(groupCode)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9300-000000000611', ${q(groupCode)}, ${q(fixtureIds.model)}, now(), now());`);
}

function tableExists(db, table) {
  return db.scalar(`select to_regclass(${q(`public.${table}`)});`) === table;
}

function passwordHash() {
  return '$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa';
}

export function assertFixtureModel(db, name) {
  const id = db.scalar(`select id from global_models where id = ${q(fixtureIds.model)} and name = ${q(name)} and is_active = true;`);
  assert(id === fixtureIds.model, 'real usage fixture model should be active');
}
