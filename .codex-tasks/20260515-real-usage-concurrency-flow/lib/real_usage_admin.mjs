import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';

export async function signInAdmin(ctx) {
  const response = await fetch(`${ctx.serverBaseUrl}/api/auth/sign-in`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ identifier: ctx.adminIdentifier, password: ctx.adminPassword }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`admin sign-in failed ${response.status}: ${text}`);
  }
  const body = JSON.parse(text);
  assert(body.success, 'admin sign-in should return success envelope');
  assert(body.data.access_token, 'admin sign-in should return access token');
  return body.data.access_token;
}

export async function createAdminToken(ctx, adminToken, payload) {
  const body = await adminJson(ctx, adminToken, 'POST', '/api/admin/tokens', payload);
  assert(body.token?.id, 'admin token create should return token id');
  assert(body.raw_token, 'admin token create should return raw token');
  return body;
}

export async function deleteAdminToken(ctx, adminToken, tokenId) {
  await adminJson(ctx, adminToken, 'DELETE', `/api/admin/tokens/${encodeURIComponent(tokenId)}`);
}

export async function fetchProviderUpstreamModels(ctx, adminToken, providerId) {
  return adminJson(ctx, adminToken, 'GET', `/api/admin/providers/${encodeURIComponent(providerId)}/upstream-models`);
}

export async function adminJson(ctx, adminToken, method, path, payload) {
  const headers = { authorization: `Bearer ${adminToken}` };
  const options = { method, headers };
  if (payload !== undefined) {
    headers['content-type'] = 'application/json';
    options.body = JSON.stringify(payload);
  }
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, options);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`admin ${method} ${path} failed ${response.status}: ${text.slice(0, 1000)}`);
  }
  if (!text.trim()) {
    return null;
  }
  const body = JSON.parse(text);
  assert(body.success, `admin ${method} ${path} should return success envelope`);
  return body.data;
}
