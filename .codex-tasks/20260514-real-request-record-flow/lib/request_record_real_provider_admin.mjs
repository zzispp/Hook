import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';

export async function listProviderModelBindings(ctx, adminToken, providerId) {
  return getAdminData(ctx, adminToken, `/api/admin/providers/${encodeURIComponent(providerId)}/models`);
}

export async function listProviderEndpoints(ctx, adminToken, providerId) {
  return getAdminData(ctx, adminToken, `/api/admin/providers/${encodeURIComponent(providerId)}/endpoints`);
}

export async function listProviderKeys(ctx, adminToken, providerId) {
  return getAdminData(ctx, adminToken, `/api/admin/providers/${encodeURIComponent(providerId)}/keys`);
}

export async function updateProviderModelBinding(ctx, adminToken, providerId, modelId, payload) {
  return sendAdminJson(ctx, adminToken, 'PATCH', `/api/admin/providers/${encodeURIComponent(providerId)}/models/${encodeURIComponent(modelId)}`, payload);
}

export async function updateProviderEndpoint(ctx, adminToken, providerId, endpointId, payload) {
  return sendAdminJson(
    ctx,
    adminToken,
    'PATCH',
    `/api/admin/providers/${encodeURIComponent(providerId)}/endpoints/${encodeURIComponent(endpointId)}`,
    payload,
  );
}

export async function updateProviderKey(ctx, adminToken, providerId, keyId, payload) {
  return sendAdminJson(ctx, adminToken, 'PATCH', `/api/admin/providers/${encodeURIComponent(providerId)}/keys/${encodeURIComponent(keyId)}`, payload);
}

export async function fetchProviderUpstreamModels(ctx, adminToken, providerId) {
  return getAdminData(ctx, adminToken, `/api/admin/providers/${encodeURIComponent(providerId)}/upstream-models`);
}

async function getAdminData(ctx, adminToken, path) {
  return sendAdmin(ctx, adminToken, 'GET', path);
}

async function sendAdminJson(ctx, adminToken, method, path, payload) {
  return sendAdmin(ctx, adminToken, method, path, payload);
}

async function sendAdmin(ctx, adminToken, method, path, payload) {
  const headers = { authorization: `Bearer ${adminToken}` };
  if (payload !== undefined) headers['content-type'] = 'application/json';
  const response = await fetch(`${ctx.serverBaseUrl}${path}`, {
    method,
    headers,
    ...(payload === undefined ? {} : { body: JSON.stringify(payload) }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`admin ${method} ${path} failed ${response.status}: ${text}`);
  }
  const body = JSON.parse(text);
  assert(body.success, `admin ${method} ${path} should return success envelope`);
  return body.data;
}
