import { assert, assertEqual } from './assertions.mjs';

export async function signIn(ctx) {
  const body = {
    identifier: ctx.adminIdentifier,
    password: ctx.adminPassword,
  };
  const data = await apiJson(ctx, '/api/auth/sign-in', 'POST', null, body);
  assert(data.access_token, 'admin sign-in should return an access token');
  return data.access_token;
}

export async function patchSchedulingMode(ctx, accessToken, mode) {
  const data = await apiJson(ctx, '/api/admin/settings/system', 'PATCH', accessToken, {
    scheduling_mode: mode,
  });
  assertEqual(data.scheduling_mode, mode, 'settings API should persist scheduling mode');
  return data;
}

export async function createTransientAdminToken(ctx, accessToken) {
  const data = await apiJson(ctx, '/api/admin/tokens', 'POST', accessToken, {
    name: `proxy-cache-hook-${Date.now()}`,
    token_type: 'independent',
    group_code: 'default',
    model_access_mode: 'all',
    allowed_model_ids: [],
    rate_limit_rpm: 0,
  });
  assert(data.token?.id, 'admin token create should return token id');
  assert(data.raw_token, 'admin token create should return raw token');
  return data.token.id;
}

export async function deleteAdminToken(ctx, accessToken, id) {
  await apiJson(ctx, `/api/admin/tokens/${encodeURIComponent(id)}`, 'DELETE', accessToken, null);
}

async function apiJson(ctx, path, method, accessToken, body) {
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, {
    method,
    headers: headers(accessToken, body),
    body: body === null ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${method} ${path} failed with HTTP ${response.status}: ${text.slice(0, 600)}`);
  }
  if (!text.trim()) {
    return null;
  }
  const parsed = JSON.parse(text);
  return parsed.data;
}

function headers(accessToken, body) {
  const value = {};
  if (accessToken) {
    value.Authorization = `Bearer ${accessToken}`;
  }
  if (body !== null) {
    value['content-type'] = 'application/json';
  }
  return value;
}

