import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { encryptProviderKey, sha256 } from '../../20260512-real-proxy-cache-flow/lib/crypto.mjs';
import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';

export const ids = Object.freeze({
  providerOpenAI: '00000000-0000-7000-9100-000000000101',
  providerClaude: '00000000-0000-7000-9100-000000000102',
  providerGemini: '00000000-0000-7000-9100-000000000103',
  providerBroken: '00000000-0000-7000-9100-000000000104',
  keyOpenAIPrimary: '00000000-0000-7000-9100-000000000201',
  keyOpenAISecondary: '00000000-0000-7000-9100-000000000202',
  keyClaude: '00000000-0000-7000-9100-000000000203',
  keyGemini: '00000000-0000-7000-9100-000000000204',
  keyBroken: '00000000-0000-7000-9100-000000000205',
  keyClaudeSecondary: '00000000-0000-7000-9100-000000000206',
  endpointOpenAIChat: '00000000-0000-7000-9100-000000000301',
  endpointOpenAIResponses: '00000000-0000-7000-9100-000000000302',
  endpointOpenAICompact: '00000000-0000-7000-9100-000000000303',
  endpointClaude: '00000000-0000-7000-9100-000000000304',
  endpointGemini: '00000000-0000-7000-9100-000000000305',
  endpointBrokenChat: '00000000-0000-7000-9100-000000000306',
  group: '00000000-0000-7000-9100-000000000801',
  token: '00000000-0000-7000-9100-000000000501',
});

export const groupCode = 'route_real';

export const providerNames = Object.freeze({
  openai: 'Route Test Hook Pool',
  claude: 'Route Test AIPAI Claude',
  gemini: 'Route Test Ekan8 Gemini',
  broken: 'Route Test Broken OpenAI',
});

export function seedRouteDatabase(ctx, db) {
  seedModels(ctx, db);
  const modelIds = modelIdsByName(ctx, db);
  seedGroup(db);
  seedProviders(db);
  seedEndpoints(ctx, db);
  seedKeys(ctx, db);
  seedModelBindings(ctx, db, modelIds);
  seedGroupBindings(db, modelIds);
  seedSystemToken(ctx, db);
  return modelIds;
}

export function modelIdsByName(ctx, db) {
  const names = modelNames(ctx);
  const rows = db.rows(`select name, id from global_models where name in (${Object.values(names).map(q).join(',')});`);
  const byName = Object.fromEntries(rows.map(([name, id]) => [name, id]));
  for (const name of Object.values(names)) {
    assert(byName[name], `missing global model after seed: ${name}`);
  }
  return { openai: byName[names.openai], claude: byName[names.claude], gemini: byName[names.gemini] };
}

export function modelNames(ctx) {
  return Object.freeze({
    openai: ctx.models.openai,
    claude: ctx.models.claude,
    gemini: ctx.models.gemini,
  });
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

export function setOpenAIPrimaryKey(db, ctx, plaintext) {
  const encrypted = encryptProviderKey(ctx.providerSecret, plaintext);
  db.exec(`update provider_api_keys set encrypted_api_key = ${q(encrypted)}, updated_at = now() where id = ${q(ids.keyOpenAIPrimary)};`);
}

export function setClaudePrimaryKey(db, ctx, plaintext) {
  const encrypted = encryptProviderKey(ctx.providerSecret, plaintext);
  db.exec(`update provider_api_keys set encrypted_api_key = ${q(encrypted)}, updated_at = now() where id = ${q(ids.keyClaude)};`);
}

export function setOpenAIChatBaseUrl(db, baseUrl) {
  db.exec(`update provider_endpoints set base_url = ${q(baseUrl)}, updated_at = now() where id = ${q(ids.endpointOpenAIChat)};`);
}

export function restoreRouteFixtures(ctx, db, originalMode) {
  setSchedulingModeDb(db, originalMode);
  setBrokenProviderActive(db, false);
  setOpenAIKeyPriorities(db, 0, 1);
  setOpenAIPrimaryKey(db, ctx, ctx.secrets.hookPoolKey);
  setClaudePrimaryKey(db, ctx, ctx.secrets.claudeKey);
  setOpenAIChatBaseUrl(db, ctx.upstreams.openaiBaseUrl);
}

export function deactivateRouteFixtures(db) {
  db.exec(`
update providers set is_active = false, updated_at = now() where id in (${providerIds().map(q).join(',')});
update global_models set is_active = false, updated_at = now() where name like 'hook-real-%';
update api_tokens set is_active = false, updated_at = now() where id = ${q(ids.token)};
update billing_groups set is_active = false, updated_at = now() where code = ${q(groupCode)};`);
}

function seedModels(ctx, db) {
  const names = modelNames(ctx);
  const pricing = q(JSON.stringify({ tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0 }] }));
  db.exec(`
insert into global_models
  (id, name, display_name, default_price_per_request, default_tiered_pricing, supported_capabilities, config, is_active, usage_count, created_at, updated_at)
values
  ('00000000-0000-7000-9100-000000000701', ${q(names.openai)}, ${q(names.openai)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  ('00000000-0000-7000-9100-000000000702', ${q(names.claude)}, ${q(names.claude)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now()),
  ('00000000-0000-7000-9100-000000000703', ${q(names.gemini)}, ${q(names.gemini)}, null, ${pricing}, '["chat","stream"]', null, true, 0, now(), now())
on conflict (name) do update set is_active = true, updated_at = now();`);
}

function seedGroup(db) {
  db.exec(`
insert into billing_groups
  (id, code, name, description, billing_multiplier, is_active, is_system, sort_order, created_at, updated_at)
values
  (${q(ids.group)}, ${q(groupCode)}, 'Route Real Test', null, 1, true, false, 0, now(), now())
on conflict (code) do update set is_active = true, updated_at = now();`);
}

function seedProviders(db) {
  db.exec(`
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority, keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerOpenAI)}, ${q(providerNames.openai)}, 'openai', 1, 45, 45, 10, false, true, true, now(), now()),
  (${q(ids.providerClaude)}, ${q(providerNames.claude)}, 'claude', 1, 60, 60, 20, false, true, true, now(), now()),
  (${q(ids.providerGemini)}, ${q(providerNames.gemini)}, 'gemini', 1, 60, 60, 30, false, true, true, now(), now()),
  (${q(ids.providerBroken)}, ${q(providerNames.broken)}, 'openai', 1, 8, 8, 0, false, true, false, now(), now())
on conflict (id) do update set is_active = excluded.is_active, priority = excluded.priority, updated_at = now();`);
}

function seedEndpoints(ctx, db) {
  db.exec(`
insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config, header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointOpenAIChat)}, ${q(ids.providerOpenAI)}, 'openai_chat', ${q(ctx.upstreams.openaiBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointOpenAIResponses)}, ${q(ids.providerOpenAI)}, 'openai_cli', ${q(ctx.upstreams.openaiBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointOpenAICompact)}, ${q(ids.providerOpenAI)}, 'openai_compact', ${q(ctx.upstreams.openaiBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointClaude)}, ${q(ids.providerClaude)}, 'claude_chat', ${q(ctx.upstreams.claudeBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointGemini)}, ${q(ids.providerGemini)}, 'gemini_chat', ${q(ctx.upstreams.geminiBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointBrokenChat)}, ${q(ids.providerBroken)}, 'openai_chat', 'http://127.0.0.1:9', null, 1, true, null, null, null, now(), now())
on conflict (id) do update set base_url = excluded.base_url, is_active = true, updated_at = now();`);
}

function seedKeys(ctx, db) {
  const encrypted = keyValues(ctx);
  db.exec(`
insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit, cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end, health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyOpenAIPrimary)}, ${q(ids.providerOpenAI)}, 'Route Hook primary', ${q(encrypted.openai)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyOpenAISecondary)}, ${q(ids.providerOpenAI)}, 'Route Hook secondary', ${q(encrypted.openai)}, null, 1, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyClaude)}, ${q(ids.providerClaude)}, 'Route Claude primary', ${q(encrypted.claude)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyGemini)}, ${q(ids.providerGemini)}, 'Route Gemini primary', ${q(encrypted.gemini)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyBroken)}, ${q(ids.providerBroken)}, 'Route Broken invalid', ${q(encrypted.broken)}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyClaudeSecondary)}, ${q(ids.providerClaude)}, 'Route Claude secondary', ${q(encrypted.claude)}, null, 1, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set encrypted_api_key = excluded.encrypted_api_key, internal_priority = excluded.internal_priority, is_active = true, updated_at = now();`);
}

function seedModelBindings(ctx, db, modelIds) {
  db.exec(`
delete from provider_models where id like '00000000-0000-7000-9100-%';
insert into provider_models (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active, price_per_request, tiered_pricing, config, created_at, updated_at)
values
  ('00000000-0000-7000-9100-000000000401', ${q(ids.providerOpenAI)}, ${q(modelIds.openai)}, ${q(ctx.models.openaiProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9100-000000000402', ${q(ids.providerClaude)}, ${q(modelIds.claude)}, ${q(ctx.models.claudeProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9100-000000000403', ${q(ids.providerGemini)}, ${q(modelIds.gemini)}, ${q(ctx.models.geminiProvider)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9100-000000000404', ${q(ids.providerBroken)}, ${q(modelIds.openai)}, ${q(ctx.models.openaiProvider)}, null, true, null, null, null, now(), now());`);
}

function seedGroupBindings(db, modelIds) {
  db.exec(`
delete from billing_group_providers where group_code = ${q(groupCode)};
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9100-000000000601', ${q(groupCode)}, ${q(ids.providerOpenAI)}, now(), now()),
  ('00000000-0000-7000-9100-000000000602', ${q(groupCode)}, ${q(ids.providerClaude)}, now(), now()),
  ('00000000-0000-7000-9100-000000000603', ${q(groupCode)}, ${q(ids.providerGemini)}, now(), now()),
  ('00000000-0000-7000-9100-000000000604', ${q(groupCode)}, ${q(ids.providerBroken)}, now(), now());
delete from billing_group_models where group_code = ${q(groupCode)};
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9100-000000000611', ${q(groupCode)}, ${q(modelIds.openai)}, now(), now()),
  ('00000000-0000-7000-9100-000000000612', ${q(groupCode)}, ${q(modelIds.claude)}, now(), now()),
  ('00000000-0000-7000-9100-000000000613', ${q(groupCode)}, ${q(modelIds.gemini)}, now(), now());`);
}

function seedSystemToken(ctx, db) {
  const tokenHash = sha256(ctx.secrets.systemToken);
  db.exec(`
delete from api_tokens where id = ${q(ids.token)};
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at, model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count, is_active, last_used_at, created_at, updated_at)
values
  (${q(ids.token)}, ${q(ctx.adminUserId)}, 'independent', 'Route real scheduler token', ${q(ctx.secrets.systemToken)}, ${q(tokenHash)}, ${q(ctx.secrets.systemToken.slice(0, 10))}, ${q(groupCode)}, null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());`);
}

function keyValues(ctx) {
  return {
    openai: encryptProviderKey(ctx.providerSecret, ctx.secrets.hookPoolKey),
    claude: encryptProviderKey(ctx.providerSecret, ctx.secrets.claudeKey),
    gemini: encryptProviderKey(ctx.providerSecret, ctx.secrets.geminiKey),
    broken: encryptProviderKey(ctx.providerSecret, 'sk-route-real-invalid'),
  };
}

function providerIds() {
  return [ids.providerOpenAI, ids.providerClaude, ids.providerGemini, ids.providerBroken];
}
