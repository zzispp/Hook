import http from 'node:http';

import { ensureBackend } from '../../20260512-real-proxy-cache-flow/lib/backend.mjs';

const SLOW_RESPONSE_DELAY_MS = 5_000;

export async function ensureBackendForTest(serverBaseUrl) {
  const child = await ensureBackend(serverBaseUrl);
  return {
    owned: Boolean(child),
    stop: async () => {
      if (!child) return;
      child.kill('SIGTERM');
      await waitForDown(serverBaseUrl);
    },
  };
}

export async function startSlowUpstream() {
  const server = http.createServer((request, response) => {
    request.resume();
    setTimeout(() => {
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end(JSON.stringify(slowChatCompletion()));
    }, SLOW_RESPONSE_DELAY_MS);
  });
  await listen(server);
  const address = server.address();
  return {
    baseUrl: `http://127.0.0.1:${address.port}`,
    stop: () => close(server),
  };
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

function listen(server) {
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

function slowChatCompletion() {
  return {
    id: 'chatcmpl-slow-test',
    object: 'chat.completion',
    created: Math.floor(Date.now() / 1000),
    model: 'slow-test',
    choices: [{ index: 0, message: { role: 'assistant', content: 'slow ok' }, finish_reason: 'stop' }],
    usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
  };
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
