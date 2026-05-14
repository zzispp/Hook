import { ensureBackend } from '../../20260512-real-proxy-cache-flow/lib/backend.mjs';

export async function startBackend(serverBaseUrl) {
  const child = await ensureBackend(serverBaseUrl);
  if (!child) {
    throw new Error(`expected harness-owned backend on ${serverBaseUrl}, but an external process is already serving it`);
  }
  return child;
}

export async function restartBackend(child, serverBaseUrl) {
  await stopBackend(child, serverBaseUrl);
  return startBackend(serverBaseUrl);
}

export async function stopBackend(child, serverBaseUrl) {
  if (!child) return;
  child.kill('SIGTERM');
  await waitForDown(serverBaseUrl);
}

async function waitForDown(serverBaseUrl) {
  for (let index = 0; index < 50; index += 1) {
    await sleep(200);
    if (!(await healthOk(serverBaseUrl))) return;
  }
  throw new Error(`backend did not stop: ${serverBaseUrl}`);
}

async function healthOk(serverBaseUrl) {
  try {
    const response = await fetch(`${serverBaseUrl}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
