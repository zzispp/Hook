import { q } from './db.mjs';
import { encryptProviderKey, sha256 } from './crypto.mjs';
import { assert } from './assertions.mjs';

export const ids = Object.freeze({
  providerOpenAI: '00000000-0000-7000-9000-000000000101',
  providerClaude: '00000000-0000-7000-9000-000000000102',
  providerGemini: '00000000-0000-7000-9000-000000000103',
  providerBroken: '00000000-0000-7000-9000-000000000104',
  keyOpenAIPrimary: '00000000-0000-7000-9000-000000000201',
  keyOpenAISecondary: '00000000-0000-7000-9000-000000000202',
  keyClaude: '00000000-0000-7000-9000-000000000203',
  keyGemini: '00000000-0000-7000-9000-000000000204',
  keyBroken: '00000000-0000-7000-9000-000000000205',
  endpointOpenAIChat: '00000000-0000-7000-9000-000000000301',
  endpointOpenAIResponses: '00000000-0000-7000-9000-000000000302',
  endpointOpenAICompact: '00000000-0000-7000-9000-000000000303',
  endpointClaude: '00000000-0000-7000-9000-000000000304',
  endpointGemini: '00000000-0000-7000-9000-000000000305',
  endpointBrokenChat: '00000000-0000-7000-9000-000000000306',
  token: '00000000-0000-7000-9000-000000000501',
});

export function modelNames(ctx) {
  return Object.freeze({
    gpt: ctx.models.openai,
    claude: ctx.models.claude,
    gemini: ctx.models.gemini,
  });
}

export const providerNames = Object.freeze({
  openai: 'Real Test Hook Pool',
  claude: 'Real Test Claude',
  gemini: 'Real Test Ekan8',
  broken: 'Real Test Broken OpenAI',
});

export function seedDatabase(ctx, db) {
  seedGlobalModels(ctx, db);
  const modelIds = modelIdsByName(ctx, db);
  seedDefaultGroup(db);
  seedProviders(ctx, db);
  seedEndpoints(ctx, db);
  seedKeys(ctx, db);
  seedModelBindings(ctx, db, modelIds);
  seedGroupBindings(db, modelIds);
  seedSystemToken(ctx, db);
  return modelIds;
}

export function modelIdsByName(ctx, db) {
  const current = modelNames(ctx);
  const names = Object.values(current);
  const rows = db.rows(`select name, id from global_models where name in (${names.map(q).join(',')});`);
  const byName = Object.fromEntries(rows.map(([name, id]) => [name, id]));
  for (const name of names) {
    assert(byName[name], `missing global model after seed: ${name}`);
  }
  return {
    gpt: byName[current.gpt],
    claude: byName[current.claude],
    gemini: byName[current.gemini],
  };
}

export function seedGlobalModels(ctx, db) {
  const names = modelNames(ctx);
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }));
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000701', ${q(names.gpt)}, ${q(names.gpt)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  ('00000000-0000-7000-9000-000000000702', ${q(names.claude)}, ${q(names.claude)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  ('00000000-0000-7000-9000-000000000703', ${q(names.gemini)}, ${q(names.gemini)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (name) do update set
  display_name = excluded.display_name,
  default_tiered_pricing = excluded.default_tiered_pricing,
  supported_capabilities = excluded.supported_capabilities,
  is_active = true,
  updated_at = now();`);
}

export function setSchedulingModeDb(db, mode) {
  db.exec(`update system_settings set scheduling_mode = ${q(mode)}, updated_at = now() where id = 'global';`);
}

export function setBrokenProviderActive(db, active) {
  db.exec(`update providers set is_active = ${active ? 'true' : 'false'}, updated_at = now() where id = ${q(ids.providerBroken)};`);
}

export function setOpenAIKeyPriorities(db, primary, secondary) {
  db.exec(`
update provider_api_keys set internal_priority = ${Number(primary)}, updated_at = now() where id = ${q(ids.keyOpenAIPrimary)};
update provider_api_keys set internal_priority = ${Number(secondary)}, updated_at = now() where id = ${q(ids.keyOpenAISecondary)};`);
}

function seedDefaultGroup(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000801', 'default', 'Default', null, 1, true, true, 0, now(), now())
on conflict (code) do update set is_active = true, updated_at = now();`);
}

function seedProviders(ctx, db) {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerOpenAI)}, ${q(providerNames.openai)}, 'openai', 1, 20, 20, 10, false, true, true, now(), now()),
  (${q(ids.providerClaude)}, ${q(providerNames.claude)}, 'claude', 1, 20, 20, 20, false, true, true, now(), now()),
  (${q(ids.providerGemini)}, ${q(providerNames.gemini)}, 'gemini', 1, 20, 20, 30, false, true, true, now(), now()),
  (${q(ids.providerBroken)}, ${q(providerNames.broken)}, 'openai', 1, 4, 4, 0, false, true, false, now(), now())
on conflict (id) do update set
  name = excluded.name, provider_type = excluded.provider_type, max_retries = excluded.max_retries,
  request_timeout_seconds = excluded.request_timeout_seconds,
  stream_first_byte_timeout_seconds = excluded.stream_first_byte_timeout_seconds,
  priority = excluded.priority, keep_priority_on_conversion = excluded.keep_priority_on_conversion,
  enable_format_conversion = excluded.enable_format_conversion, is_active = excluded.is_active, updated_at = now();`);
}

function seedEndpoints(ctx, db) {
  const openai = ctx.upstreams.openaiBaseUrl;
  const claude = ctx.upstreams.claudeBaseUrl;
  const gemini = ctx.upstreams.geminiBaseUrl;
  db.exec(`
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointOpenAIChat)}, ${q(ids.providerOpenAI)}, 'openai_chat', ${q(openai)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointOpenAIResponses)}, ${q(ids.providerOpenAI)}, 'openai_cli', ${q(openai)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointOpenAICompact)}, ${q(ids.providerOpenAI)}, 'openai_compact', ${q(openai)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointClaude)}, ${q(ids.providerClaude)}, 'claude_chat', ${q(claude)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointGemini)}, ${q(ids.providerGemini)}, 'gemini_chat', ${q(gemini)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointBrokenChat)}, ${q(ids.providerBroken)}, 'openai_chat', ${q(openai)}, null, 1, true, null, null, null, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id, api_format = excluded.api_format, base_url = excluded.base_url,
  custom_path = excluded.custom_path, max_retries = excluded.max_retries, is_active = excluded.is_active,
  updated_at = now();`);
}

function seedKeys(ctx, db) {
  const encrypted = keyValues(ctx);
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit, cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyOpenAIPrimary)}, ${q(ids.providerOpenAI)}, 'Hook primary', ${q(encrypted.openai)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyOpenAISecondary)}, ${q(ids.providerOpenAI)}, 'Hook secondary', ${q(encrypted.openai)}, null, 1, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyClaude)}, ${q(ids.providerClaude)}, 'Claude primary', ${q(encrypted.claude)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyGemini)}, ${q(ids.providerGemini)}, 'Ekan8 primary', ${q(encrypted.gemini)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyBroken)}, ${q(ids.providerBroken)}, 'Broken invalid key', ${q(encrypted.broken)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id, name = excluded.name, encrypted_api_key = excluded.encrypted_api_key,
  internal_priority = excluded.internal_priority, cache_ttl_minutes = excluded.cache_ttl_minutes,
  max_probe_interval_minutes = excluded.max_probe_interval_minutes, is_active = excluded.is_active, updated_at = now();`);
}

function keyValues(ctx) {
  return {
    openai: encryptProviderKey(ctx.providerSecret, ctx.secrets.hookPoolKey),
    claude: encryptProviderKey(ctx.providerSecret, ctx.secrets.claudeKey),
    gemini: encryptProviderKey(ctx.providerSecret, ctx.secrets.geminiKey),
    broken: encryptProviderKey(ctx.providerSecret, 'sk-real-proxy-test-invalid'),
  };
}

function seedModelBindings(ctx, db, modelIds) {
  db.exec(`
delete from provider_models where id in ('00000000-0000-7000-9000-000000000401','00000000-0000-7000-9000-000000000402','00000000-0000-7000-9000-000000000403','00000000-0000-7000-9000-000000000404');
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000401', ${q(ids.providerOpenAI)}, ${q(modelIds.gpt)}, ${q(ctx.models.openaiProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000402', ${q(ids.providerClaude)}, ${q(modelIds.claude)}, ${q(ctx.models.claudeProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000403', ${q(ids.providerGemini)}, ${q(modelIds.gemini)}, ${q(ctx.models.geminiProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000404', ${q(ids.providerBroken)}, ${q(modelIds.gpt)}, ${q(ctx.models.openaiProvider)}, null, true, null, null, null, now(), now());`);
}

function seedGroupBindings(db, modelIds) {
  db.exec(`
delete from billing_group_providers where group_code = 'default' and provider_id in (${q(ids.providerOpenAI)}, ${q(ids.providerClaude)}, ${q(ids.providerGemini)}, ${q(ids.providerBroken)});
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000601', 'default', ${q(ids.providerOpenAI)}, now(), now()),
  ('00000000-0000-7000-9000-000000000602', 'default', ${q(ids.providerClaude)}, now(), now()),
  ('00000000-0000-7000-9000-000000000603', 'default', ${q(ids.providerGemini)}, now(), now()),
  ('00000000-0000-7000-9000-000000000604', 'default', ${q(ids.providerBroken)}, now(), now());
delete from billing_group_models where group_code = 'default' and global_model_id in (${q(modelIds.gpt)}, ${q(modelIds.claude)}, ${q(modelIds.gemini)});
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000611', 'default', ${q(modelIds.gpt)}, now(), now()),
  ('00000000-0000-7000-9000-000000000612', 'default', ${q(modelIds.claude)}, now(), now()),
  ('00000000-0000-7000-9000-000000000613', 'default', ${q(modelIds.gemini)}, now(), now());`);
}

function seedSystemToken(ctx, db) {
  const tokenHash = sha256(ctx.secrets.systemToken);
  const tokenPrefix = ctx.secrets.systemToken.slice(0, 10);
  db.exec(`
delete from api_tokens where id = ${q(ids.token)} or token_hash = ${q(tokenHash)};
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at, model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count, is_active, last_used_at, created_at, updated_at)
values
  (${q(ids.token)}, ${q(ctx.adminUserId)}, 'independent', 'Real proxy cache-flow token', ${q(ctx.secrets.systemToken)}, ${q(tokenHash)}, ${q(tokenPrefix)}, 'default', null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());`);
}
