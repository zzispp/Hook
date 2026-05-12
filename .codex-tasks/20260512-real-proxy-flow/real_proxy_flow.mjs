import { createCipheriv, createHash, randomBytes } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import net from 'node:net';

const ids = Object.freeze({
  providerHook: '00000000-0000-7000-9000-000000000101',
  providerClaudeHook: '00000000-0000-7000-9000-000000000102',
  providerEkan8: '00000000-0000-7000-9000-000000000103',
  providerBroken: '00000000-0000-7000-9000-000000000104',
  keyHookPrimary: '00000000-0000-7000-9000-000000000201',
  keyHookSecondary: '00000000-0000-7000-9000-000000000202',
  keyClaudeHook: '00000000-0000-7000-9000-000000000203',
  keyEkan8: '00000000-0000-7000-9000-000000000204',
  keyBroken: '00000000-0000-7000-9000-000000000205',
  endpointHookChat: '00000000-0000-7000-9000-000000000301',
  endpointHookResponses: '00000000-0000-7000-9000-000000000302',
  endpointHookCompact: '00000000-0000-7000-9000-000000000303',
  endpointClaudeHook: '00000000-0000-7000-9000-000000000304',
  endpointEkan8Gemini: '00000000-0000-7000-9000-000000000305',
  endpointBrokenChat: '00000000-0000-7000-9000-000000000306',
  token: '00000000-0000-7000-9000-000000000501',
});

const modelNames = Object.freeze({
  gpt: 'gpt-5.5',
  claude: 'claude-opus-4-7',
  gemini: 'gemini-3.1-pro-preview',
});

const providerNames = Object.freeze({
  hook: 'Real Test Hook Pool',
  claudeHook: 'Real Test Claude Hook',
  ekan8: 'Real Test Ekan8',
  broken: 'Real Test Broken OpenAI',
});

const db = Object.freeze({
  host: env('HOOK_DB_HOST', 'localhost'),
  port: env('HOOK_DB_PORT', '5433'),
  user: env('HOOK_DB_USER', 'postgres'),
  password: env('HOOK_DB_PASSWORD', '123456'),
  name: env('HOOK_DB_NAME', 'postgres'),
});

const serverBaseUrl = env('HOOK_BACKEND_URL', 'http://127.0.0.1:3000');
const providerSecret = env('HOOK_PROVIDER_KEY_SECRET', 'hook-local-development-provider-key-secret-change-before-deploy');
const adminUserId = env('HOOK_ADMIN_USER_ID', '00000000-0000-7000-8000-000000000000');
const redisPrefix = env('HOOK_REDIS_PREFIX', 'hook');
const redisHost = env('HOOK_REDIS_HOST', '127.0.0.1');
const redisPort = Number(env('HOOK_REDIS_PORT', '6380'));
const psqlBin = env('PSQL_BIN', 'psql');
const claudeBaseUrl = env('CLAUDE_BASE_URL', 'https://www.hook.rs');

const secrets = Object.freeze({
  systemToken: requiredEnv('HOOK_SYSTEM_TOKEN'),
  hookPoolKey: requiredEnv('HOOK_POOL_KEY'),
  claudeKey: requiredEnv('CLAUDE_KEY'),
  ekan8Key: requiredEnv('EKAN8_KEY'),
});

let originalSchedulingMode = 'fixed_order';

async function main() {
  originalSchedulingMode = sqlScalar("select scheduling_mode from system_settings where id = 'global'") || 'fixed_order';
  const modelIds = modelIdsByName();
  seedDatabase(modelIds);
  const server = await ensureBackend();
  try {
    await runFixedOrder(modelIds);
    await runFailover(modelIds);
    await runCacheAffinity(modelIds);
    await runLoadBalance(modelIds);
    await runFormatConversion(modelIds);
    console.log('real proxy flow: all assertions passed');
  } finally {
    cleanup(modelIds);
    if (server) {
      server.kill('SIGTERM');
    }
  }
}

async function runFixedOrder(modelIds) {
  setSchedulingMode('fixed_order');
  setBrokenProviderActive(false);
  setHookKeyPriorities(0, 1);
  await deleteAffinity(modelIds.gpt, 'openai_chat');
  const result = await proxyCall('fixed order openai exact', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.gpt,
    body: openAiBody(modelNames.gpt, 'Reply with exactly: hook-fixed'),
  });
  const success = successRow(result.trace);
  assertEqual(success.provider_name, providerNames.hook, 'fixed order should use hook provider');
  assertEqual(success.key_name, 'Hook primary', 'fixed order should use primary key first');
  assertEqual(success.provider_api_format, 'openai_chat', 'fixed order should use exact OpenAI chat endpoint');
  assertEqual(success.needs_conversion, 'false', 'fixed order exact request must not require conversion');
  assertHasConvertedCandidateAfterExact(result.trace);

  const stream = await proxyCall('fixed order openai stream exact', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.gpt,
    body: { ...openAiBody(modelNames.gpt, 'Reply with exactly: hook-stream'), stream: true },
  });
  const streamSuccess = successRow(stream.trace);
  assertEqual(streamSuccess.provider_name, providerNames.hook, 'stream fixed order should use hook provider');
  assertEqual(streamSuccess.needs_conversion, 'false', 'stream exact request must not require conversion');
}

async function runFailover(modelIds) {
  setSchedulingMode('fixed_order');
  setBrokenProviderActive(true);
  await deleteAffinity(modelIds.gpt, 'openai_chat');
  const result = await proxyCall('retry and failover', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.gpt,
    body: openAiBody(modelNames.gpt, 'Reply with exactly: hook-failover'),
  });
  const brokenFailures = result.trace.filter((row) => row.provider_name === providerNames.broken && row.status === 'failed');
  assert(brokenFailures.length >= 1, 'failover should record failed broken provider attempts');
  assertEqual(successRow(result.trace).provider_name, providerNames.hook, 'failover should continue to hook provider');
  setBrokenProviderActive(false);
}

async function runCacheAffinity(modelIds) {
  setSchedulingMode('cache_affinity');
  setBrokenProviderActive(false);
  setHookKeyPriorities(0, 1);
  await setAffinity(modelIds.gpt, 'openai_chat', ids.keyHookSecondary);
  const result = await proxyCall('cache affinity', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.gpt,
    body: openAiBody(modelNames.gpt, 'Reply with exactly: hook-affinity'),
  });
  assertEqual(successRow(result.trace).key_name, 'Hook secondary', 'cache affinity should promote cached key');
  await deleteAffinity(modelIds.gpt, 'openai_chat');
}

async function runLoadBalance(modelIds) {
  setSchedulingMode('load_balance');
  setBrokenProviderActive(false);
  setHookKeyPriorities(0, 0);
  await deleteAffinity(modelIds.gpt, 'openai_chat');
  const keys = new Set();
  for (let index = 0; index < 12; index += 1) {
    const result = await proxyCall(`load balance ${index + 1}`, {
      path: '/v1/chat/completions',
      clientFormat: 'openai_chat',
      model: modelNames.gpt,
      body: openAiBody(modelNames.gpt, `Reply with exactly: hook-lb-${index}`),
    });
    const success = successRow(result.trace);
    keys.add(success.key_name);
    assertEqual(success.provider_api_format, 'openai_chat', 'load balance must keep exact endpoint ahead of converted endpoints');
  }
  assert(keys.has('Hook primary') && keys.has('Hook secondary'), 'load balance should distribute over both hook keys');
  setHookKeyPriorities(0, 1);
}

async function runFormatConversion(modelIds) {
  setSchedulingMode('fixed_order');
  setBrokenProviderActive(false);
  await deleteAffinity(modelIds.claude, 'openai_chat');
  await deleteAffinity(modelIds.gemini, 'openai_chat');
  await deleteAffinity(modelIds.gpt, 'claude_chat');

  const openAiToClaude = await proxyCall('openai to claude conversion', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.claude,
    body: openAiBody(modelNames.claude, 'Reply with exactly: hook-claude-convert'),
  });
  assertConversion(openAiToClaude.trace, providerNames.claudeHook, 'claude_chat');

  const openAiStreamToClaude = await proxyCall('openai stream to claude conversion', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.claude,
    body: { ...openAiBody(modelNames.claude, 'Reply with exactly: hook-claude-stream'), stream: true },
  });
  assertConversion(openAiStreamToClaude.trace, providerNames.claudeHook, 'claude_chat');
  assert(openAiStreamToClaude.text.includes('data:'), 'stream conversion should return SSE data');

  const openAiToGemini = await proxyCall('openai to gemini conversion', {
    path: '/v1/chat/completions',
    clientFormat: 'openai_chat',
    model: modelNames.gemini,
    body: openAiBody(modelNames.gemini, 'Reply with exactly: hook-gemini-convert'),
  });
  assertConversion(openAiToGemini.trace, providerNames.ekan8, 'gemini_chat');

  const claudeToOpenAi = await proxyCall('claude to openai conversion', {
    path: '/v1/messages',
    clientFormat: 'claude_chat',
    model: modelNames.gpt,
    body: claudeBody(modelNames.gpt, 'Reply with exactly: hook-openai-convert'),
  });
  assertConversion(claudeToOpenAi.trace, providerNames.hook, 'openai_chat');

  const geminiExact = await proxyCall('gemini exact', {
    path: `/v1beta/models/${encodeURIComponent(modelNames.gemini)}:generateContent`,
    clientFormat: 'gemini_chat',
    model: modelNames.gemini,
    body: geminiBody('Reply with exactly: hook-gemini-exact'),
  });
  const geminiSuccess = successRow(geminiExact.trace);
  assertEqual(geminiSuccess.provider_name, providerNames.ekan8, 'Gemini exact should use Ekan8');
  assertEqual(geminiSuccess.needs_conversion, 'false', 'Gemini exact should not require conversion');
}

function seedDatabase(modelIds) {
  const tokenHash = sha256(secrets.systemToken);
  const tokenPrefix = secrets.systemToken.slice(0, 10);
  const sql = `
begin;
insert into providers
  (id, name, provider_type, max_retries, request_timeout_seconds, stream_first_byte_timeout_seconds, priority,
   keep_priority_on_conversion, enable_format_conversion, is_active, created_at, updated_at)
values
  (${q(ids.providerHook)}, ${q(providerNames.hook)}, 'openai', 1, 20, 20, 10, false, true, true, now(), now()),
  (${q(ids.providerClaudeHook)}, ${q(providerNames.claudeHook)}, 'claude', 1, 20, 20, 20, false, true, true, now(), now()),
  (${q(ids.providerEkan8)}, ${q(providerNames.ekan8)}, 'gemini', 1, 20, 20, 30, false, true, true, now(), now()),
  (${q(ids.providerBroken)}, ${q(providerNames.broken)}, 'openai', 1, 3, 3, 0, false, true, false, now(), now())
on conflict (id) do update set
  name = excluded.name,
  provider_type = excluded.provider_type,
  max_retries = excluded.max_retries,
  request_timeout_seconds = excluded.request_timeout_seconds,
  stream_first_byte_timeout_seconds = excluded.stream_first_byte_timeout_seconds,
  priority = excluded.priority,
  keep_priority_on_conversion = excluded.keep_priority_on_conversion,
  enable_format_conversion = excluded.enable_format_conversion,
  is_active = excluded.is_active,
  updated_at = now();

insert into provider_endpoints
  (id, provider_id, api_format, base_url, custom_path, max_retries, is_active, format_acceptance_config,
   header_rules, body_rules, created_at, updated_at)
values
  (${q(ids.endpointHookChat)}, ${q(ids.providerHook)}, 'openai_chat', 'https://pool.hook.rs', null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointHookResponses)}, ${q(ids.providerHook)}, 'openai_cli', 'https://pool.hook.rs', null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointHookCompact)}, ${q(ids.providerHook)}, 'openai_compact', 'https://pool.hook.rs', null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointClaudeHook)}, ${q(ids.providerClaudeHook)}, 'claude_chat', ${q(claudeBaseUrl)}, null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointEkan8Gemini)}, ${q(ids.providerEkan8)}, 'gemini_chat', 'https://www.ekan8.com', null, 1, true, null, null, null, now(), now()),
  (${q(ids.endpointBrokenChat)}, ${q(ids.providerBroken)}, 'openai_chat', 'https://pool.hook.rs', null, 1, true, null, null, null, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id,
  api_format = excluded.api_format,
  base_url = excluded.base_url,
  custom_path = excluded.custom_path,
  max_retries = excluded.max_retries,
  is_active = excluded.is_active,
  updated_at = now();

insert into provider_api_keys
  (id, provider_id, name, encrypted_api_key, note, internal_priority, rpm_limit, learned_rpm_limit,
   cache_ttl_minutes, max_probe_interval_minutes, time_range_enabled, time_range_start, time_range_end,
   health_by_format, circuit_breaker_by_format, is_active, created_at, updated_at)
values
  (${q(ids.keyHookPrimary)}, ${q(ids.providerHook)}, 'Hook primary', ${q(encryptProviderKey(secrets.hookPoolKey))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyHookSecondary)}, ${q(ids.providerHook)}, 'Hook secondary', ${q(encryptProviderKey(secrets.hookPoolKey))}, null, 1, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyClaudeHook)}, ${q(ids.providerClaudeHook)}, 'Claude Hook primary', ${q(encryptProviderKey(secrets.claudeKey))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyEkan8)}, ${q(ids.providerEkan8)}, 'Ekan8 primary', ${q(encryptProviderKey(secrets.ekan8Key))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now()),
  (${q(ids.keyBroken)}, ${q(ids.providerBroken)}, 'Broken invalid key', ${q(encryptProviderKey('sk-real-proxy-test-invalid'))}, null, 0, null, null, 10, 0, false, null, null, null, null, true, now(), now())
on conflict (id) do update set
  provider_id = excluded.provider_id,
  name = excluded.name,
  encrypted_api_key = excluded.encrypted_api_key,
  internal_priority = excluded.internal_priority,
  cache_ttl_minutes = excluded.cache_ttl_minutes,
  max_probe_interval_minutes = excluded.max_probe_interval_minutes,
  is_active = excluded.is_active,
  updated_at = now();

delete from provider_models where id in (
  '00000000-0000-7000-9000-000000000401',
  '00000000-0000-7000-9000-000000000402',
  '00000000-0000-7000-9000-000000000403',
  '00000000-0000-7000-9000-000000000404'
);
insert into provider_models
  (id, provider_id, global_model_id, provider_model_name, provider_model_mappings, is_active,
   price_per_request, tiered_pricing, config, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000401', ${q(ids.providerHook)}, ${q(modelIds.gpt)}, ${q(modelNames.gpt)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000402', ${q(ids.providerClaudeHook)}, ${q(modelIds.claude)}, ${q(modelNames.claude)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000403', ${q(ids.providerEkan8)}, ${q(modelIds.gemini)}, ${q(modelNames.gemini)}, null, true, null, null, null, now(), now()),
  ('00000000-0000-7000-9000-000000000404', ${q(ids.providerBroken)}, ${q(modelIds.gpt)}, ${q(modelNames.gpt)}, null, true, null, null, null, now(), now());

delete from billing_group_providers where group_code = 'default' and provider_id in (
  ${q(ids.providerHook)}, ${q(ids.providerClaudeHook)}, ${q(ids.providerEkan8)}, ${q(ids.providerBroken)}
);
insert into billing_group_providers (id, group_code, provider_id, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000601', 'default', ${q(ids.providerHook)}, now(), now()),
  ('00000000-0000-7000-9000-000000000602', 'default', ${q(ids.providerClaudeHook)}, now(), now()),
  ('00000000-0000-7000-9000-000000000603', 'default', ${q(ids.providerEkan8)}, now(), now()),
  ('00000000-0000-7000-9000-000000000604', 'default', ${q(ids.providerBroken)}, now(), now());

delete from billing_group_models where group_code = 'default' and global_model_id in (
  ${q(modelIds.gpt)}, ${q(modelIds.claude)}, ${q(modelIds.gemini)}
);
insert into billing_group_models (id, group_code, global_model_id, created_at, updated_at)
values
  ('00000000-0000-7000-9000-000000000611', 'default', ${q(modelIds.gpt)}, now(), now()),
  ('00000000-0000-7000-9000-000000000612', 'default', ${q(modelIds.claude)}, now(), now()),
  ('00000000-0000-7000-9000-000000000613', 'default', ${q(modelIds.gemini)}, now(), now());

delete from api_tokens where id = ${q(ids.token)} or token_hash = ${q(tokenHash)};
insert into api_tokens
  (id, user_id, token_type, name, token_value, token_hash, token_prefix, group_code, expires_at,
   model_access_mode, allowed_model_ids, rate_limit_rpm, quota_limit, used_quota, request_count,
   is_active, last_used_at, created_at, updated_at)
values
  (${q(ids.token)}, ${q(adminUserId)}, 'independent', 'Real proxy flow token', ${q(secrets.systemToken)}, ${q(tokenHash)}, ${q(tokenPrefix)},
   'default', null, 'all', '[]', 0, null, 0, 0, true, null, now(), now());
commit;`;
  runPsql(sql);
}

function modelIdsByName() {
  const rows = sqlRows(`
select name, id
from global_models
where name in (${q(modelNames.gpt)}, ${q(modelNames.claude)}, ${q(modelNames.gemini)})
order by name;`);
  const byName = Object.fromEntries(rows.map(([name, id]) => [name, id]));
  for (const name of Object.values(modelNames)) {
    assert(byName[name], `missing active global model: ${name}`);
  }
  return {
    gpt: byName[modelNames.gpt],
    claude: byName[modelNames.claude],
    gemini: byName[modelNames.gemini],
  };
}

async function proxyCall(label, request, options = {}) {
  const before = new Date(Date.now() - 1000).toISOString();
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), options.clientTimeoutMs ?? 60_000);
  let response;
  try {
    response = await fetch(`${serverBaseUrl}${request.path}`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${secrets.systemToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify(request.body),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timer);
  }
  const text = await response.text();
  const requestId = latestRequestId(request.clientFormat, request.model, before);
  const trace = requestId ? requestTrace(requestId) : [];
  console.log(`${label}: status=${response.status} request_id=${requestId ?? 'none'}`);
  printTrace(trace);
  if (requestId) {
    assertEqual(requestOwnerId(requestId), adminUserId, 'independent token should belong to admin user');
  }
  if (!response.ok && options.expectOk !== false) {
    throw new Error(`${label} failed with HTTP ${response.status}: ${text.slice(0, 600)}`);
  }
  return { ok: response.ok, status: response.status, text, trace, requestId };
}

function latestRequestId(clientFormat, model, beforeIso) {
  return sqlScalar(`
select request_id
from request_candidates
where created_at >= ${q(beforeIso)}
  and client_api_format = ${q(clientFormat)}
  and global_model_id = (select id from global_models where name = ${q(model)})
order by created_at desc
limit 1;`);
}

function requestTrace(requestId) {
  const rows = sqlRows(`
select
  rc.candidate_index::text,
  rc.retry_index::text,
  rc.status,
  coalesce(rc.status_code::text, ''),
  coalesce(p.name, ''),
  coalesce(k.name, ''),
  rc.client_api_format,
  coalesce(rc.provider_api_format, ''),
  rc.needs_conversion::text,
  coalesce(rc.error_type, '')
from request_candidates rc
left join providers p on p.id = rc.provider_id
left join provider_api_keys k on k.id = rc.key_id
where rc.request_id = ${q(requestId)}
order by rc.candidate_index, rc.retry_index;`);
  return rows.map((row) => ({
    candidate_index: row[0],
    retry_index: row[1],
    status: row[2],
    status_code: row[3],
    provider_name: row[4],
    key_name: row[5],
    client_api_format: row[6],
    provider_api_format: row[7],
    needs_conversion: row[8],
    error_type: row[9] ?? '',
  }));
}

function requestOwnerId(requestId) {
  return sqlScalar(`
select coalesce(t.user_id, '')
from request_candidates rc
join api_tokens t on t.id = rc.token_id
where rc.request_id = ${q(requestId)}
limit 1;`);
}

function printTrace(trace) {
  for (const row of trace) {
    console.log(
      `  #${row.candidate_index}.${row.retry_index} ${row.status} ${row.status_code || '-'} ` +
        `${row.provider_name}/${row.key_name} ${row.client_api_format}->${row.provider_api_format} conversion=${row.needs_conversion} ${row.error_type}`,
    );
  }
}

function openAiBody(model, text) {
  return {
    model,
    messages: [{ role: 'user', content: text }],
    max_tokens: 16,
    temperature: 0,
    stream: false,
  };
}

function claudeBody(model, text) {
  return {
    model,
    max_tokens: 16,
    temperature: 0,
    messages: [{ role: 'user', content: text }],
  };
}

function geminiBody(text) {
  return {
    contents: [{ role: 'user', parts: [{ text }] }],
    generationConfig: { maxOutputTokens: 16, temperature: 0 },
  };
}

function assertConversion(trace, providerName, providerFormat) {
  const success = successRow(trace);
  assertEqual(success.provider_name, providerName, `conversion should use ${providerName}`);
  assertEqual(success.provider_api_format, providerFormat, `conversion should target ${providerFormat}`);
  assertEqual(success.needs_conversion, 'true', 'conversion trace should be marked');
}

function assertHasConvertedCandidateAfterExact(trace) {
  const exact = trace.find((row) => row.needs_conversion === 'false');
  const converted = trace.find((row) => row.needs_conversion === 'true');
  assert(exact && converted, 'candidate list should contain exact and converted candidates');
  assert(Number(exact.candidate_index) < Number(converted.candidate_index), 'converted candidates should be demoted after exact candidates');
}

function successRow(trace) {
  const row = trace.find((item) => item.status === 'success');
  assert(row, 'expected a successful request candidate');
  return row;
}

function setSchedulingMode(mode) {
  runPsql(`update system_settings set scheduling_mode = ${q(mode)}, updated_at = now() where id = 'global';`);
}

function setBrokenProviderActive(active) {
  runPsql(`update providers set is_active = ${active ? 'true' : 'false'}, updated_at = now() where id = ${q(ids.providerBroken)};`);
}

function setHookKeyPriorities(primary, secondary) {
  runPsql(`
update provider_api_keys set internal_priority = ${Number(primary)}, updated_at = now() where id = ${q(ids.keyHookPrimary)};
update provider_api_keys set internal_priority = ${Number(secondary)}, updated_at = now() where id = ${q(ids.keyHookSecondary)};`);
}

async function setAffinity(modelId, format, keyId) {
  await redisCommand('SETEX', affinityKey(modelId, format), '300', keyId);
}

async function deleteAffinity(modelId, format) {
  await redisCommand('DEL', affinityKey(modelId, format));
}

function affinityKey(modelId, format) {
  return `${redisPrefix}:llm_proxy:affinity:${ids.token}:${modelId}:${format}`;
}

async function ensureBackend() {
  if (await healthOk()) {
    console.log(`backend: using existing ${serverBaseUrl}`);
    return null;
  }
  console.log('backend: starting cargo run -p backend');
  const child = spawn('cargo', ['run', '-p', 'backend'], {
    cwd: process.cwd(),
    env: process.env,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let logs = '';
  child.stdout.on('data', (chunk) => {
    logs = keepTail(logs + chunk.toString());
  });
  child.stderr.on('data', (chunk) => {
    logs = keepTail(logs + chunk.toString());
  });
  for (let i = 0; i < 90; i += 1) {
    await sleep(1000);
    if (await healthOk()) {
      console.log('backend: started');
      return child;
    }
    if (child.exitCode !== null) {
      throw new Error(`backend exited before health check passed:\n${logs}`);
    }
  }
  child.kill('SIGTERM');
  throw new Error(`backend did not become healthy:\n${logs}`);
}

async function healthOk() {
  try {
    const response = await fetch(`${serverBaseUrl}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

function cleanup(modelIds) {
  try {
    setBrokenProviderActive(false);
    setHookKeyPriorities(0, 1);
    setSchedulingMode(originalSchedulingMode);
    runPsql(`delete from provider_models where id = '00000000-0000-7000-9000-000000000499';`);
    for (const [modelId, format] of [
      [modelIds.gpt, 'openai_chat'],
      [modelIds.gpt, 'claude_chat'],
      [modelIds.claude, 'openai_chat'],
      [modelIds.gemini, 'openai_chat'],
    ]) {
      redisCommand('DEL', affinityKey(modelId, format)).catch(() => {});
    }
  } catch (error) {
    console.error(`cleanup failed: ${error.message}`);
  }
}

function encryptProviderKey(plaintext) {
  const key = createHash('sha256').update(providerSecret).digest();
  const nonce = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', key, nonce);
  const encrypted = Buffer.concat([cipher.update(plaintext, 'utf8'), cipher.final(), cipher.getAuthTag()]);
  return `v1:${nonce.toString('hex')}:${encrypted.toString('hex')}`;
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function sqlScalar(sql) {
  const rows = sqlRows(sql);
  return rows[0]?.[0] ?? '';
}

function sqlRows(sql) {
  const output = runPsql(sql, ['-At', '-F', '\t']);
  if (!output.trim()) {
    return [];
  }
  return output
    .trimEnd()
    .split('\n')
    .map((line) => line.split('\t'));
}

function runPsql(sql, extraArgs = []) {
  const result = spawnSync(psqlBin, ['-X', '-q', '-v', 'ON_ERROR_STOP=1', ...extraArgs, '-h', db.host, '-p', db.port, '-U', db.user, '-d', db.name], {
    input: sql,
    encoding: 'utf8',
    env: { ...process.env, PGPASSWORD: db.password },
    maxBuffer: 1024 * 1024 * 10,
  });
  if (result.status !== 0) {
    throw new Error(`psql failed: ${result.stderr || result.stdout}`);
  }
  return result.stdout;
}

function redisCommand(...args) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: redisHost, port: redisPort });
    let response = '';
    socket.setTimeout(5000);
    socket.on('connect', () => socket.write(resp(args)));
    socket.on('data', (chunk) => {
      response += chunk.toString();
      if (response.includes('\r\n')) {
        socket.end();
      }
    });
    socket.on('end', () => {
      if (response.startsWith('-')) {
        reject(new Error(response.trim()));
        return;
      }
      resolve(response);
    });
    socket.on('timeout', () => {
      socket.destroy();
      reject(new Error('redis command timed out'));
    });
    socket.on('error', reject);
  });
}

function resp(args) {
  return `*${args.length}\r\n${args.map((arg) => `$${Buffer.byteLength(String(arg))}\r\n${arg}\r\n`).join('')}`;
}

function q(value) {
  if (value === null || value === undefined) {
    return 'null';
  }
  return `'${String(value).replaceAll("'", "''")}'`;
}

function env(name, fallback) {
  return process.env[name] || fallback;
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) {
    throw new Error(`missing required env: ${name}`);
  }
  return value;
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${expected}, got ${actual}`);
  }
}

function keepTail(value) {
  return value.length > 6000 ? value.slice(value.length - 6000) : value;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
